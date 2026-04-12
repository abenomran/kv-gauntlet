# kv-gauntlet

A Rust-based experiment harness for evaluating replication and consistency trade-offs across three real-world distributed key-value systems:

| System | Consistency Model |
|--------|-------------------|
| [etcd](https://etcd.io) | Strong consistency (Raft) |
| [Apache Cassandra](https://cassandra.apache.org) | Tunable consistency (Quorum) |
| [AntidoteDB](https://antidotedb.eu) | Eventual consistency (CRDTs) |

This is not a simulator and not a new database. It is a controlled experiment harness that sends real workloads to real database clusters, injects faults, and records the results.

---

## How It Works

```
Rust Harness
↓
System Adapter (etcd / Cassandra / AntidoteDB)
↓
Real 3-Node Cluster (Docker)
↓
CSV Metrics Output
```

The harness runs a configurable workload (reads and writes) against a real database cluster for a set duration. At a configured point during the run, a fault is injected, a node is killed via a shell script. Latency, success rate, and operation metadata are recorded in real time to a CSV file.

---

## Dataset

Experiments use a sample of 10,000 English Wikipedia articles. Article titles serve as keys and article text (first 500 characters) serves as values. This gives realistic variable-length values rather than synthetic uniform data.

To generate the dataset locally:

```bash
pip install datasets
python dataset/fetch_dataset.py
```

This creates `dataset/wikipedia_10k.json`. It is not committed to the repo.

---

## Project Structure

```
consistency-lab/
├── src/
│   ├── main.rs           # Entry point
│   ├── config.rs         # Config file loading
│   ├── runner.rs         # Experiment orchestration
│   ├── workload.rs       # Operation generation
│   ├── dataset.rs        # Wikipedia dataset loading
│   ├── metrics.rs        # Real-time CSV metrics writer
│   └── systems/
│       ├── mod.rs        # KvStore trait
│       ├── etcd.rs       # etcd adapter
│       ├── cassandra.rs  # Cassandra adapter
│       └── antidote.rs   # AntidoteDB adapter
├── docker/
│   ├── etcd/             # Docker Compose for etcd cluster
│   ├── cassandra/        # Docker Compose for Cassandra cluster
│   └── antidote/         # Docker Compose for AntidoteDB cluster
├── scripts/
│   ├── etcd/             # start.sh, stop.sh, kill_node.sh
│   ├── cassandra/        # start.sh, stop.sh, kill_node.sh
│   └── antidote/         # start.sh, stop.sh, kill_node.sh
├── dataset/
│   └── fetch_dataset.py  # Dataset download script
├── results/              # CSV output (gitignored)
└── config.toml           # Experiment configuration
```

---

## Configuration

Edit `config.toml` to control the experiment:

```toml
system = "cassandra" # "etcd", "cassandra", or "antidote"
workload = "balanced" # "balanced", "read-heavy", "write-heavy", "contention"
duration_seconds = 60
concurrency = 4

[fault]
script = "scripts/cassandra/kill_node.sh"
trigger_at_seconds = 30
```

Remove or comment out the `[fault]` section to run without fault injection.

---

## Workload Types

| Workload | Read % | Write % | Purpose |
|----------|--------|---------|---------|
| balanced | 50 | 50 | General baseline |
| read-heavy | 95 | 5 | Read-dominant workload |
| write-heavy | 5 | 95 | Write-dominant workload |
| contention | 5 | 95 | All writes to the same key |

---

## Running an Experiment

**1. Start the cluster for your system:**
```bash
./scripts/cassandra/start.sh
```

**2. Wait for all nodes to be ready:**
```bash
docker exec cassandra1 nodetool status # all three should show UN
```

**3. Run the experiment:**
```bash
cargo run
```

**4. Results are written to:**
```
results/<system>_<workload>_<timestamp>.csv
```

**5. Stop the cluster when done:**
```bash
./scripts/cassandra/stop.sh
```

---

## Metrics Output

Each row in the CSV represents one operation:

| Column | Description |
|--------|-------------|
| timestamp | Clock time |
| elapsed_seconds | Seconds since experiment start |
| system | Database system |
| workload | Workload type |
| operation | GET or PUT |
| key | Wikipedia article title used as key |
| latency_ms | Operation latency in milliseconds |
| success | true / false |

---

## Requirements

- [Rust](https://rustup.rs) (stable)
- [Docker Desktop](https://www.docker.com/products/docker-desktop/)
- Python 3 + `pip install datasets` (for dataset generation)
- Docker memory allocation: at least 8GB (Cassandra is memory hungry)

---

## System Adapters

Each database system implements the shared `KvStore` trait:

```rust
#[async_trait]
pub trait KvStore: Send + Sync {
    async fn put(&self, key: &str, value: &str) -> Result<(), StoreError>;
    async fn get(&self, key: &str) -> Result<Option<String>, StoreError>;
}
```

The runner only ever interacts with this trait — it has no knowledge of which database is underneath.