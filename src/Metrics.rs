use std::fs::create_dir_all;
use std::fs::OpenOptions;
use std::io::{BufWriter, Write};
use chrono::Local;

/// One recorded data point per request
pub struct MetricEntry {
    pub run_index: u64,
    pub key: String,
    pub system: String,
    pub workload: String,
    pub operation: String, // "GET" or "PUT"
    pub latency_ms: f64,
    pub success: bool,
    pub version: Option<u64>, // version written (PUT) or observed (GET)
    pub fault_active: bool, // was fault injected at time of this op?
}

/// Writes metrics to a CSV file in real time.
/// Each call to `record` appends one line immediately.
pub struct MetricsWriter {
    writer: BufWriter<std::fs::File>,
    start_time: std::time::Instant,
}

impl MetricsWriter {
    /// Open (or create) the output file and write the CSV header
    pub fn new(system: &str, workload: &str) -> Result<Self, Box<dyn std::error::Error>> {
        // Create results directory if it doesn't exist
        create_dir_all("results")?;

        // Generate a timestamped filename
        let timestamp = Local::now().format("%Y%m%d_%H%M%S");
        let path = format!("results/{}_{}_{}.csv", system, workload, timestamp);

        println!("Writing metrics to: {}", path);

        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)?;

        let mut writer = BufWriter::new(file);

        // write header row
        writeln!(
            writer,
            "timestamp,elapsed_seconds,run_index,system,workload,operation,key,latency_ms,success,version,fault_active"
        )?;
        writer.flush()?;

        Ok(Self {
            writer,
            start_time: std::time::Instant::now(),
        })
    }

    /// Append one metrics row to the file immediately
    pub fn record(&mut self, entry: &MetricEntry) -> Result<(), Box<dyn std::error::Error>> {
        let elapsed = self.start_time.elapsed().as_secs_f64();
        let timestamp = Local::now().to_rfc3339();
        writeln!(
            self.writer,
            "{},{:.3},{},{},{},{},\"{}\",{:.3},{},{},{}",
            timestamp,
            elapsed,
            entry.run_index,
            entry.system,
            entry.workload,
            entry.operation,
            entry.key,
            entry.latency_ms,
            entry.success,
            entry.version.map(|v| v.to_string()).unwrap_or_default(),
            entry.fault_active as u8,
        )?;
        self.writer.flush()?;
        Ok(())
    }
}