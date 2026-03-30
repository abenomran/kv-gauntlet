use serde::Deserialize;
use std::fs;

/// Represents the contents of config.toml.
#[derive(Debug, Deserialize)]
pub struct Config {
    /// Which system to test: "etcd", "cassandra", or "antidote"
    pub system: String,

    /// Workload type: "balanced", "read-heavy", "write-heavy", "contention"
    pub workload: String,

    /// How long to run the workload in seconds
    pub duration_seconds: u64,

    /// How many concurrent workers to use
    pub concurrency: usize,

    /// Where to write metrics output
    pub output_file: String,

    /// Fault injection config (optional)
    pub fault: Option<FaultConfig>,
}

#[derive(Debug, Deserialize)]
pub struct FaultConfig {
    /// Shell script to run to inject the fault
    pub script: String,

    /// How many seconds into the run to trigger the fault
    pub trigger_at_seconds: u64,
}

impl Config {
    pub fn load(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&contents)?;
        Ok(config)
    }
}