use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::time::sleep;
use std::collections::HashMap;

use crate::config::Config;
use crate::metrics::{MetricEntry, MetricsWriter};
use crate::systems::KvStore;
use crate::workload::{next_operation, Operation, WorkloadType};
use crate::dataset::Dataset;

/// Runs the full experiment:
/// 1. Sends workload requests for the configured duration
/// 2. Injects a fault at the configured time (if any)
/// 3. Records all metrics in real time
pub async fn run(config: &Config, store: Arc<dyn KvStore>, dataset: Dataset, run_index: u64) -> Result<(), Box<dyn std::error::Error>> {
    let mut metrics = MetricsWriter::new(&config.system, &config.workload)?;

    let workload_kind = WorkloadType::from_str(&config.workload)
        .ok_or("Unknown workload type in config")?;

    let duration = Duration::from_secs(config.duration_seconds);
    let start = Instant::now();
    let mut i: u64 = 0;

    let version_map: Arc<Mutex<HashMap<String, u64>>> = Arc::new(Mutex::new(HashMap::new()));
    let fault_fired = Arc::new(std::sync::atomic::AtomicBool::new(false));

    println!("Running experiment for {}s...", config.duration_seconds);

    // Fault injection: spawn a background task that fires at the right time
    if let Some(fault) = &config.fault {
        let script = fault.script.clone();
        let trigger_at = fault.trigger_at_seconds;
        let fault_fired_clone = Arc::clone(&fault_fired);

        tokio::spawn(async move {
            sleep(Duration::from_secs(trigger_at)).await;
            fault_fired_clone.store(true, std::sync::atomic::Ordering::SeqCst);
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
        let op = next_operation(&workload_kind, &dataset, i, run_index);

        let op_start = Instant::now();

        let fault_active = fault_fired.load(std::sync::atomic::Ordering::SeqCst);

        // Run the operation and record success/failure
        let (op_name, op_key, result, version) = match &op {
            Operation::Put { key, value } => {
                // Embed version as prefix before sending
                let ver = {
                    let mut map = version_map.lock().unwrap();
                    let v = map.entry(key.clone()).or_insert(0);
                    *v += 1;
                    *v
                };
                let versioned_value = format!("{:08}|{}", ver, value);
                let res = store.put(key, &versioned_value).await.map(|_| ());
                ("PUT", key.clone(), res, Some(ver))
            }
            Operation::Get { key } => {
                let res = store.get(key).await;
                let version = res.as_ref().ok()
                    .and_then(|v| v.as_ref())
                    .and_then(|s| s.split('|').next())
                    .and_then(|p| p.parse::<u64>().ok());
                ("GET", key.clone(), res.map(|_| ()), version)
            }
        };

        let latency_ms = op_start.elapsed().as_secs_f64() * 1000.0;
        let success = result.is_ok();

        metrics.record(&MetricEntry {
            run_index,
            key: op_key,
            system: config.system.clone(),
            workload: config.workload.clone(),
            operation: op_name.to_string(),
            latency_ms,
            success,
            version,
            fault_active,
        })?;

        i += 1;
    }

    println!("Experiment complete. {} operations ran.", i);

    // restore containers the experiment finishes
    if let Some(fault) = &config.fault {
        if let Some(restore) = &fault.restore_script {
            println!(">>> Restoring cluster...");
            let status = std::process::Command::new("bash")
                .arg(restore)
                .status();
            match status {
                Ok(s) => println!(">>> Restore script exited with: {}", s),
                Err(e) => println!(">>> Failed to run restore script: {}", e),
            }
        }
    }

    Ok(())
}