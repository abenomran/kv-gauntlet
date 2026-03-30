/// The type of operation to perform
#[derive(Debug, Clone)]
pub enum Operation {
    Put { key: String, value: String },
    Get { key: String },
}

/// Workload types matching the config
#[derive(Debug, Clone)]
pub enum WorkloadType {
    Balanced,    // 50% reads, 50% writes
    ReadHeavy,   // 95% reads, 5% writes
    WriteHeavy,  // 5% reads, 95% writes
    Contention,  // many writes to the same key
}

impl WorkloadType {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "balanced"    => Some(Self::Balanced),
            "read-heavy"  => Some(Self::ReadHeavy),
            "write-heavy" => Some(Self::WriteHeavy),
            "contention"  => Some(Self::Contention),
            _             => None,
        }
    }
}

/// Generates the next operation based on the workload type.
/// `i` is the operation index, used to generate unique keys.
pub fn next_operation(kind: &WorkloadType, i: u64) -> Operation {
    let key = format!("key_{:05}", i % 1000); // cycle through 1000 keys
    let value = format!("value_{}", i);

    // Decide read vs write based on workload type
    let write_percent: u64 = match kind {
        WorkloadType::Balanced   => 50,
        WorkloadType::ReadHeavy  => 5,
        WorkloadType::WriteHeavy => 95,
        WorkloadType::Contention => 95,
    };

    // Use modulo to alternate deterministically
    if i % 100 < write_percent {
        // For contention, all writes go to the same key
        let key = match kind {
            WorkloadType::Contention => "key_contention".to_string(),
            _ => key,
        };
        Operation::Put { key, value }
    } else {
        Operation::Get { key }
    }
}