use async_trait::async_trait;
use scylla::{Session, SessionBuilder};
use std::sync::Arc;
use super::{KvStore, StoreError};

pub struct CassandraStore {
    session: Arc<Session>,
}

impl CassandraStore {
    pub async fn connect(nodes: Vec<String>) -> Result<Self, StoreError> {
        let session = SessionBuilder::new()
            .known_nodes(&nodes)
            .build()
            .await?;

        session.query(
            "CREATE KEYSPACE IF NOT EXISTS consistency_lab \
             WITH replication = {'class': 'SimpleStrategy', 'replication_factor': 3}",
            &[],
        ).await?;

        session.query(
            "CREATE TABLE IF NOT EXISTS consistency_lab.kv_store \
             (key text PRIMARY KEY, value text)",
            &[],
        ).await?;

        Ok(Self {
            session: Arc::new(session),
        })
    }
}

#[async_trait]
impl KvStore for CassandraStore {
    async fn put(&self, key: &str, value: &str) -> Result<(), StoreError> {
        use scylla::statement::Consistency;
        let mut query = scylla::query::Query::new(
            "INSERT INTO consistency_lab.kv_store (key, value) VALUES (?, ?)"
        );
        query.set_consistency(Consistency::Quorum);
        self.session.query(query, (key, value)).await?;
        Ok(())
    }

    async fn get(&self, key: &str) -> Result<Option<String>, StoreError> {
        use scylla::statement::Consistency;
        let mut query = scylla::query::Query::new(
            "SELECT value FROM consistency_lab.kv_store WHERE key = ?"
        );
        query.set_consistency(Consistency::Quorum);
        let result = self.session.query(query, (key,)).await?;

        if let Some(rows) = result.rows {
            if let Some(row) = rows.into_iter().next() {
                let value: Option<String> = row.columns[0]
                    .as_ref()
                    .and_then(|v| v.as_text().map(|s| s.to_owned()));
                return Ok(value);
            }
        }

        Ok(None)
    }
}