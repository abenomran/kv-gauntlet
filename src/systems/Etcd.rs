use async_trait::async_trait;
use super::{KvStore, StoreError};

pub struct EtcdStore {
    client: etcd_client::Client,
}

impl EtcdStore {
    pub async fn connect(endpoints: Vec<String>) -> Result<Self, StoreError> {
        let client = etcd_client::Client::connect(endpoints, None).await?;
        Ok(Self { client })
    }
}

#[async_trait]
impl KvStore for EtcdStore {
    async fn put(&self, key: &str, value: &str) -> Result<(), StoreError> {
        self.client
            .kv_client()
            .put(key, value, None)
            .await?;
        Ok(())
    }

    async fn get(&self, key: &str) -> Result<Option<String>, StoreError> {
        let resp = self.client
            .kv_client()
            .get(key, None)
            .await?;

        match resp.kvs().first() {
            Some(kv) => {
                let value = kv.value_str()
                    .map(|s| s.to_owned())
                    .map_err(|e| Box::new(e) as StoreError)?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_put_and_get() {
        let store = EtcdStore::connect(vec![
            "http://localhost:2379".to_string(),
        ])
        .await
        .expect("failed to connect to etcd");

        store.put("test-key", "hello-etcd").await.expect("put failed");

        let val = store.get("test-key").await.expect("get failed");
        assert_eq!(val, Some("hello-etcd".to_string()));

        println!("✓ put and get working correctly");
    }
}