use async_trait::async_trait;
use super::{KvStore, StoreError};

/// Wraps a Cassandra/Scylla session and implements KvStore.
pub struct CassandraStore {
    // TODO: hold the scylla Session here once connected
}

impl CassandraStore {
    /// Connect to a Cassandra cluster.
    /// `nodes` is a list of contact points, e.g. ["127.0.0.1:9042"]
    pub async fn connect(nodes: Vec<String>) -> Result<Self, StoreError> {
        // TODO: use scylla::SessionBuilder
        todo!("Cassandra connection not yet implemented")
    }
}

#[async_trait]
impl KvStore for CassandraStore {
    async fn put(&self, key: &str, value: &str) -> Result<(), StoreError> {
        // TODO: execute INSERT CQL statement
        todo!()
    }

    async fn get(&self, key: &str) -> Result<Option<String>, StoreError> {
        // TODO: execute SELECT CQL statement
        todo!()
    }
}