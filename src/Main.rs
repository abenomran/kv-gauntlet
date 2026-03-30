mod config;
mod metrics;
mod runner;
mod workload;
mod systems;

use config::Config;

#[tokio::main]
async fn main() {
    // Load config from file
    let config = Config::load("config.toml").expect("Failed to load config.toml");

    println!("Starting kv-gauntlet");
    println!("System:   {}", config.system);
    println!("Workload: {}", config.workload);
    println!("Duration: {}s", config.duration_seconds);

    // TODO: hand off to runner
}