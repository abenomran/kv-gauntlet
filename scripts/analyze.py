import pandas as pd
from pathlib import Path
import argparse
import os

"""
Outputs two CSV files (aggregated trial_results and summary)

EXAMPLE USAGE

python ./scripts/analyze.py \
  --system cassandra \
  --workload balanced \
  --fault none \
  --inputs results/cassandra_balanced_20260416_210137.csv \
           results/cassandra_balanced_20260416_210243.csv \
           results/cassandra_balanced_20260416_210348.csv \
  --trial-results-out results/trials/cassandra_balanced_nofaults.csv \
  --scenario-summary-out results/trials/cassandra_balanced_nofaults_summary.csv
"""

def analyze_trial(path, system, workload, fault, trial_label=None):
    # load trial csv
    df = pd.read_csv(path).copy()

    # make sure timestamps are usable for sorting (ignore parse failures)
    if "timestamp" in df.columns:
        df["timestamp"] = pd.to_datetime(df["timestamp"], errors="coerce")

    # sort (process operations by time)
    sort_cols = [c for c in ["run_index", "timestamp", "elapsed_seconds"] if c in df.columns]
    if sort_cols:
        df = df.sort_values(sort_cols)

    # keep only successful ops for latency + stale-read analysis
    # (failures still matter for availability, so we keep full df too)
    success_df = df[df["success"] == True].copy()

    # track the highest version we've successfully written per (run, key)
    # "ground truth"
    max_written = {}

    stale = 0
    total_gets = 0

    # walk through operations in order
    for _, row in success_df.iterrows():
        key = row["key"]
        ver = row["version"]

        # Skip rows that don't have a version (e.g., failed parses)
        if pd.isna(ver):
            continue

        ver = int(ver)

        run_index = row["run_index"] if "run_index" in row else 0
        map_key = (run_index, key)

        if row["operation"] == "PUT":
            # update max version seen so far for this key
            max_written[map_key] = max(max_written.get(map_key, 0), ver)

        elif row["operation"] == "GET":
            total_gets += 1

            expected = max_written.get(map_key, 0)

            # if we read an older version, count it as stale
            if ver < expected:
                stale += 1

    stale_rate = stale / total_gets if total_gets > 0 else 0.0

    # split metrics into baseline vs fault window
    baseline_all = df[df["fault_active"] == 0]
    fault_all = df[df["fault_active"] == 1]

    baseline_success = success_df[success_df["fault_active"] == 0]
    fault_success = success_df[success_df["fault_active"] == 1]

    return {
        "system": system,
        "workload": workload,
        "fault": fault,
        "trial": trial_label if trial_label is not None else Path(path).stem,
        "source_file": Path(path).name,

        # Consistency
        "stale_read_rate": stale_rate,

        # Availability (computed from ALL ops, not just successes)
        "availability_baseline": baseline_all["success"].mean() if len(baseline_all) else None,
        "availability_fault": fault_all["success"].mean() if len(fault_all) else None,

        # latency (only successful ops)
        "p50_baseline": baseline_success["latency_ms"].quantile(0.50) if len(baseline_success) else None,
        "p99_baseline": baseline_success["latency_ms"].quantile(0.99) if len(baseline_success) else None,
        "p50_fault": fault_success["latency_ms"].quantile(0.50) if len(fault_success) else None,
        "p99_fault": fault_success["latency_ms"].quantile(0.99) if len(fault_success) else None,

        # some extra data
        "total_ops": len(df),
        "total_gets": total_gets,
        "stale_gets": stale,
    }


def append_csv(row_df, output_path):
    # Make sure the directory exists (e.g., results/trials/)
    os.makedirs(os.path.dirname(output_path), exist_ok=True)
    # append to csv
    exists = os.path.exists(output_path)
    row_df.to_csv(output_path, mode="a", header=not exists, index=False)


def summarize_trials(trials_df):
    # metrics to aggregate across trials
    metric_cols = [
        "stale_read_rate",
        "availability_baseline",
        "availability_fault",
        "p50_baseline",
        "p99_baseline",
        "p50_fault",
        "p99_fault",
        "total_ops",
        "total_gets",
        "stale_gets",
    ]

    # assume all rows belong to the same scenario
    summary = {
        "system": trials_df["system"].iloc[0],
        "workload": trials_df["workload"].iloc[0],
        "fault": trials_df["fault"].iloc[0],
        "num_trials": len(trials_df),
    }

    # compute avg + std for each metric
    for col in metric_cols:
        summary[f"{col}_avg"] = trials_df[col].mean()
        summary[f"{col}_std"] = trials_df[col].std()

    return pd.DataFrame([summary])


def main():
    parser = argparse.ArgumentParser(description="Analyze distributed system experiment results")

    parser.add_argument("--system", required=True, help="System name (e.g. cassandra)")
    parser.add_argument("--workload", required=True, help="Workload name (e.g. 80_20)")
    parser.add_argument("--fault", required=True, help="Fault type (e.g. partition)")
    parser.add_argument("--inputs", nargs="+", required=True, help="CSV files (one per trial)")

    parser.add_argument("--trial-results-out", default="trial_results.csv")
    parser.add_argument("--scenario-summary-out", default="scenario_summary.csv")

    args = parser.parse_args()

    trial_rows = []

    # analyze each trial
    for idx, path in enumerate(args.inputs, start=1):
        row = analyze_trial(
            path=path,
            system=args.system,
            workload=args.workload,
            fault=args.fault,
            trial_label=idx,
        )
        trial_rows.append(row)

    trials_df = pd.DataFrame(trial_rows)

    # aggregate across trials
    summary_df = summarize_trials(trials_df)

    # save outputs
    append_csv(trials_df, args.trial_results_out)
    append_csv(summary_df, args.scenario_summary_out)

    # print a quick summary to the console
    print("\nPer-trial results:")
    print(trials_df.to_string(index=False))

    print("\nScenario summary:")
    print(summary_df.to_string(index=False))

    print(f"\nSaved to:")
    print(f"  {args.trial_results_out}")
    print(f"  {args.scenario_summary_out}")


if __name__ == "__main__":
    main()