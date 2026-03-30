pub mod etcd;
pub mod cassandra;
pub mod antidote;

use async_trait::async_trait;

/// The error type returned by all KvStore operations.
/// We use a Box here so any kind of error can be wrapped inside it —
/// each database client will produce different error types.
pub type StoreError = Box<dyn std::error::Error + Send + Sync>;

/// The shared interface every system must implement.
/// The runner only ever talks to this trait — it never knows which
/// database is underneath.
#[async_trait]
pub trait KvStore: Send + Sync {
    /// Write a key-value pair. Returns Ok(()) on success, Err on failure.
    async fn put(&self, key: &str, value: &str) -> Result<(), StoreError>;

    /// Read a value by key. Returns Ok(Some(value)) if found,
    /// Ok(None) if the key doesn't exist, Err on failure.
    async fn get(&self, key: &str) -> Result<Option<String>, StoreError>;
}