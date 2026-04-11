use std::fs::create_dir_all;
use std::fs::OpenOptions;
use std::io::{BufWriter, Write};
use chrono::Local;

/// One recorded data point per request
pub struct MetricEntry {
    pub system: String,
    pub workload: String,
    pub operation: String, // "GET" or "PUT"
    pub latency_ms: f64,
    pub success: bool,
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
        writeln!(writer, "timestamp,elapsed_seconds,system,workload,operation,latency_ms,success")?;
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
            "{},{:.3},{},{},{},{:.3},{}",
            timestamp,
            elapsed,
            entry.system,
            entry.workload,
            entry.operation,
            entry.latency_ms,
            entry.success,
        )?;
        self.writer.flush()?;
        Ok(())
    }
}