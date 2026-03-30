use async_trait::async_trait;
use super::{KvStore, StoreError};

/// Wraps an AntidoteDB connection and implements KvStore.
/// NOTE: AntidoteDB Rust client support is limited — verify connectivity early.
pub struct AntidoteStore {
    // TODO: determine best client approach (HTTP API or native protocol)
}

impl AntidoteStore {
    pub async fn connect(endpoint: String) -> Result<Self, StoreError> {
        // TODO: implement once client approach is confirmed
        todo!("AntidoteDB connection not yet implemented")
    }
}

#[async_trait]
impl KvStore for AntidoteStore {
    async fn put(&self, key: &str, value: &str) -> Result<(), StoreError> {
        todo!()
    }

    async fn get(&self, key: &str) -> Result<Option<String>, StoreError> {
        todo!()
    }
}