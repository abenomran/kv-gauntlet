use async_trait::async_trait;
use super::{KvStore, StoreError};

/// Wraps an etcd client and implements KvStore.
pub struct EtcdStore {
    // TODO: hold the etcd client here once connected
}

impl EtcdStore {
    /// Connect to an etcd cluster.
    /// `endpoints` is a list of node addresses, e.g. ["http://localhost:2379"]
    pub async fn connect(endpoints: Vec<String>) -> Result<Self, StoreError> {
        // TODO: use etcd_client::Client::connect()
        todo!("etcd connection not yet implemented")
    }
}

#[async_trait]
impl KvStore for EtcdStore {
    async fn put(&self, key: &str, value: &str) -> Result<(), StoreError> {
        // TODO: call etcd client put
        todo!()
    }

    async fn get(&self, key: &str) -> Result<Option<String>, StoreError> {
        // TODO: call etcd client get
        todo!()
    }
}