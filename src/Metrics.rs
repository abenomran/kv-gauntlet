use std::fs::OpenOptions;
use std::io::{BufWriter, Write};
use chrono::Utc;

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
}

impl MetricsWriter {
    /// Open (or create) the output file and write the CSV header
    pub fn new(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;

        let mut writer = BufWriter::new(file);

        // Write header row
        writeln!(writer, "timestamp,system,workload,operation,latency_ms,success")?;
        writer.flush()?;

        Ok(Self { writer })
    }

    /// Append one metrics row to the file immediately
    pub fn record(&mut self, entry: &MetricEntry) -> Result<(), Box<dyn std::error::Error>> {
        let timestamp = Utc::now().to_rfc3339();
        writeln!(
            self.writer,
            "{},{},{},{},{:.3},{}",
            timestamp,
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