use serde::Deserialize;
use std::fs;

/// One record from the Wikipedia dataset
#[derive(Deserialize, Clone)]
pub struct WikiRecord {
    pub key: String,
    pub value: String,
}

/// Holds all loaded records in memory
#[derive(Clone)]
pub struct Dataset {
    pub records: Vec<WikiRecord>,
}

impl Dataset {
    pub fn load(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = fs::read_to_string(path)?;
        let records: Vec<WikiRecord> = serde_json::from_str(&contents)?;
        println!("Loaded {} records from dataset", records.len());
        Ok(Self { records })
    }

    /// Get a record by index, cycling through the dataset
    pub fn get(&self, i: u64) -> &WikiRecord {
        let idx = (i as usize) % self.records.len();
        &self.records[idx]
    }
}