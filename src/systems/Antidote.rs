use async_trait::async_trait;
use super::{KvStore, StoreError};
use tokio::process::Command;

/// Wraps an AntidoteDB connection and implements KvStore.
/// NOTE: AntidoteDB Rust client support is limited — verify connectivity early.
pub struct AntidoteStore {
    // TODO: determine best client approach (HTTP API or native protocol)
    container: String,
    bucket: String,
}

impl AntidoteStore {
    fn erlang_binary_literal(input: &str) -> String {
        let mut escaped = String::with_capacity(input.len() + 8);
        for ch in input.chars() {
            match ch {
                '\\' => escaped.push_str("\\\\"),
                '"' => escaped.push_str("\\\""),
                '\n' => escaped.push_str("\\n"),
                '\r' => escaped.push_str("\\r"),
                '\t' => escaped.push_str("\\t"),
                _ => escaped.push(ch),
            }
        }
        format!("<<\"{}\">>", escaped)
    }

    async fn run_eval(&self, expr: &str) -> Result<String, StoreError> {
        let output = Command::new("docker")
            .arg("exec")
            .arg(&self.container)
            .arg("/antidote/bin/antidote")
            .arg("eval")
            .arg(expr)
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Antidote eval failed: {}", stderr.trim()).into());
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}

impl AntidoteStore {
    pub async fn connect(endpoint: String) -> Result<Self, StoreError> {
        // TODO: implement once client approach is confirmed
        let container = if endpoint.trim().is_empty() {
            "antidote1".to_string()
        } else {
            endpoint
        };

        let output = Command::new("docker")
            .arg("exec")
            .arg(&container)
            .arg("/antidote/bin/antidote")
            .arg("ping")
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Failed to connect to Antidote container '{}': {}", container, stderr.trim()).into());
        }

        Ok(Self {
            container,
            bucket: "kv_gauntlet".to_string(),
        })
    }
}

#[async_trait]
impl KvStore for AntidoteStore {
    async fn put(&self, key: &str, value: &str) -> Result<(), StoreError> {
        let key_bin = Self::erlang_binary_literal(key);
        let value_bin = Self::erlang_binary_literal(value);
        let bucket_bin = Self::erlang_binary_literal(&self.bucket);

        let expr = format!(
            "Obj={{{}, antidote_crdt_register_lww, {}}}, \
             {{ok,Tx}}=antidote:start_transaction(ignore,[{{update_clock,true}}]), \
             ok=antidote:update_objects([{{Obj, assign, {}}}], Tx), \
             {{ok,_}}=antidote:commit_transaction(Tx).",
            key_bin, bucket_bin, value_bin
        );

        self.run_eval(&expr).await?;
        Ok(())
    }

    async fn get(&self, key: &str) -> Result<Option<String>, StoreError> {
        let key_bin = Self::erlang_binary_literal(key);
        let bucket_bin = Self::erlang_binary_literal(&self.bucket);

        let expr = format!(
            "Obj={{{}, antidote_crdt_register_lww, {}}}, \
             {{ok,Tx}}=antidote:start_transaction(ignore,[{{update_clock,true}}]), \
             Read=antidote:read_objects([Obj], Tx), \
             {{ok,_}}=antidote:commit_transaction(Tx), \
             case Read of \
               {{ok,[<<>>]}} -> io:format(\"KVGAUNTLET_NONE~n\", []); \
               {{ok,[V]}} -> io:format(\"KVGAUNTLET_VALUE=~s~n\", [binary_to_list(base64:encode(V))]); \
               Other -> io:format(\"KVGAUNTLET_ERR=~p~n\", [Other]) \
             end.",
            key_bin, bucket_bin
        );

        let out = self.run_eval(&expr).await?;
        for line in out.lines() {
            if let Some(encoded) = line.strip_prefix("KVGAUNTLET_VALUE=") {
                let decoded = decode_base64(encoded)?;
                return Ok(Some(String::from_utf8(decoded)?));
            }
            if line.trim() == "KVGAUNTLET_NONE" {
                return Ok(None);
            }
            if line.starts_with("KVGAUNTLET_ERR=") {
                return Err(format!("Antidote read returned unexpected payload: {}", line).into());
            }
        }

        Err("Antidote read did not return a parseable value".into())
    }
}

fn decode_base64(input: &str) -> Result<Vec<u8>, StoreError> {
    const TABLE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut rev = [255u8; 256];
    for (i, c) in TABLE.iter().enumerate() {
        rev[*c as usize] = i as u8;
    }

    let bytes = input.trim().as_bytes();
    if bytes.is_empty() {
        return Ok(Vec::new());
    }
    if bytes.len() % 4 != 0 {
        return Err("Invalid base64 length".into());
    }

    let mut out = Vec::with_capacity(bytes.len() / 4 * 3);
    for chunk in bytes.chunks_exact(4) {
        let mut vals = [0u8; 4];
        let mut padding = 0usize;
        for i in 0..4 {
            let b = chunk[i];
            if b == b'=' {
                vals[i] = 0;
                padding += 1;
            } else {
                let v = rev[b as usize];
                if v == 255 {
                    return Err("Invalid base64 character".into());
                }
                vals[i] = v;
            }
        }

        let n = ((vals[0] as u32) << 18)
            | ((vals[1] as u32) << 12)
            | ((vals[2] as u32) << 6)
            | (vals[3] as u32);

        out.push(((n >> 16) & 0xFF) as u8);
        if padding < 2 {
            out.push(((n >> 8) & 0xFF) as u8);
        }
        if padding < 1 {
            out.push((n & 0xFF) as u8);
        }
    }

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_put_and_get() {
        let store = AntidoteStore::connect("antidote1".to_string())
            .await
            .expect("failed to connect to antidote");

        store
            .put("test-key", "hello-antidote")
            .await
            .expect("put failed");

        let val = store.get("test-key").await.expect("get failed");
        assert_eq!(val, Some("hello-antidote".to_string()));
    }
}