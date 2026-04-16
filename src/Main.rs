mod config;
mod metrics;
mod runner;
mod workload;
mod systems;
mod dataset;

use std::sync::Arc;
use config::Config;
use systems::cassandra::CassandraStore;
use systems::antidote::AntidoteStore;

#[tokio::main]
async fn main() {
    let config = Config::load("Config.toml").expect("Failed to load Config.toml");
    let num_runs = config.num_runs;

    println!("Starting gauntlet!");
    println!("System:   {}", config.system);
    println!("Workload: {}", config.workload);
    println!("Duration: {}s", config.duration_seconds);

    let dataset = dataset::Dataset::load("dataset/wikipedia_10k.json")
        .expect("Failed to load dataset");

    let num_runs = config.num_runs;

    match config.system.as_str() {
       "cassandra" => {
            for run_index in 0..num_runs {
                println!("\n=== Starting run {} / {} ===", run_index + 1, num_runs);

                let store = CassandraStore::connect(vec![
                    "127.0.0.1:9042".to_string(),
                ])
                .await
                .expect("Failed to connect to Cassandra");

                let store = Arc::new(store);

                runner::run(&config, store, dataset.clone(), run_index)
                    .await
                    .expect("Experiment failed");
            }
        }
        "antidote" => {
            for run_index in 0..num_runs {
                println!("\n=== Starting run {} / {} ===", run_index + 1, num_runs);

                let store = AntidoteStore::connect("antidote1".to_string())
                    .await
                    .expect("Failed to connect to Antidote");

                let store = Arc::new(store);

                runner::run(&config, store, dataset.clone(), run_index)
                    .await
                    .expect("Experiment failed");
            }
        }
        other => {
            eprintln!("Unknown system: {}. Supported: cassandra, antidote", other);
            std::process::exit(1);
        }
    }
}