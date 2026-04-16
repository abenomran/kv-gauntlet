use crate::dataset::Dataset;

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

/// Generates the next operation based on the workload type and dataset.
/// `i` is the operation index, used to cycle through dataset records deterministically.
pub fn next_operation(kind: &WorkloadType, dataset: &Dataset, i: u64, run_index: u64) -> Operation {
    // XOR with run_index to get different sequences per repetition
    let effective_i = i ^ (run_index.wrapping_mul(0xdeadbeef));
    let record = dataset.get(effective_i);

    // decision: use a hash of (i, run_index) for independence
    let write_roll = (i.wrapping_add(run_index.wrapping_mul(6364136223846793005))) % 100;
    
    // Decide read vs write based on workload type
    let write_percent: u64 = match kind {
        WorkloadType::Balanced => 50,
        WorkloadType::ReadHeavy => 5,
        WorkloadType::WriteHeavy => 95,
        WorkloadType::Contention => 95,
    };

    if write_roll < write_percent {
        // For contention, all writes go to the same key regardless of dataset record
        let key = match kind {
            WorkloadType::Contention => "contention_key".to_string(),
            _ => record.key.clone(),
        };
        Operation::Put { key, value: record.value.clone() }
    } else {
        Operation::Get { key: record.key.clone() }
    }
}