use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::sleep;

use crate::config::Config;
use crate::metrics::{MetricEntry, MetricsWriter};
use crate::systems::KvStore;
use crate::workload::{next_operation, Operation, WorkloadType};

/// Runs the full experiment:
/// 1. Sends workload requests for the configured duration
/// 2. Injects a fault at the configured time (if any)
/// 3. Records all metrics in real time
pub async fn run(config: &Config, store: Arc<dyn KvStore>) -> Result<(), Box<dyn std::error::Error>> {
    let mut metrics = MetricsWriter::new(&config.output_file)?;

    let workload_kind = WorkloadType::from_str(&config.workload)
        .ok_or("Unknown workload type in config")?;

    let duration = Duration::from_secs(config.duration_seconds);
    let start = Instant::now();
    let mut i: u64 = 0;

    println!("Running experiment for {}s...", config.duration_seconds);

    // Fault injection: spawn a background task that fires at the right time
    if let Some(fault) = &config.fault {
        let script = fault.script.clone();
        let trigger_at = fault.trigger_at_seconds;

        tokio::spawn(async move {
            sleep(Duration::from_secs(trigger_at)).await;
            println!(">>> Injecting fault: running {}", script);

            let status = std::process::Command::new("bash")
                .arg(&script)
                .status();

            match status {
                Ok(s) => println!(">>> Fault script exited with: {}", s),
                Err(e) => println!(">>> Failed to run fault script: {}", e),
            }
        });
    }

    // Main workload loop
    while start.elapsed() < duration {
        let op = next_operation(&workload_kind, i);

        let op_start = Instant::now();

        // Run the operation and record success/failure
        let (op_name, result) = match &op {
            Operation::Put { key, value } => {
                ("PUT", store.put(key, value).await.map(|_| ()))
            }
            Operation::Get { key } => {
                ("GET", store.get(key).await.map(|_| ()))
            }
        };

        let latency_ms = op_start.elapsed().as_secs_f64() * 1000.0;
        let success = result.is_ok();

        metrics.record(&MetricEntry {
            system:     config.system.clone(),
            workload:   config.workload.clone(),
            operation:  op_name.to_string(),
            latency_ms,
            success,
        })?;

        i += 1;
    }

    println!("Experiment complete. {} operations ran.", i);
    Ok(())
}