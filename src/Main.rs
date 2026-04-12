mod config;
mod metrics;
mod runner;
mod workload;
mod systems;
mod dataset;

use std::sync::Arc;
use config::Config;
use systems::cassandra::CassandraStore;

#[tokio::main]
async fn main() {
    let config = Config::load("config.toml").expect("Failed to load config.toml");

    println!("Starting gauntlet!");
    println!("System:   {}", config.system);
    println!("Workload: {}", config.workload);
    println!("Duration: {}s", config.duration_seconds);

    match config.system.as_str() {
        "cassandra" => {
            let store = CassandraStore::connect(vec![
                "127.0.0.1:9042".to_string(),
            ])
            .await
            .expect("Failed to connect to Cassandra");

            let store = Arc::new(store);
            let dataset = dataset::Dataset::load("dataset/wikipedia_10k.json")
                .expect("Failed to load dataset");

            runner::run(&config, store, dataset).await.expect("Experiment failed");
        }
        other => {
            eprintln!("Unknown system: {}. Supported: cassandra", other);
            std::process::exit(1);
        }
    }
}