#!/usr/bin/env python3
import subprocess
import time
import statistics
import os
import re

CALCULATORS = {
    "omnicalc": {
        "bin": "/home/sagnik/bin/omnicalc",
        "cmd": lambda expr: ["/home/sagnik/bin/omnicalc", "-e", expr],
    },
    "qalc": {
        "bin": "/home/sagnik/bench_tools/bin/qalc",
        "cmd": lambda expr: ["/home/sagnik/bench_tools/bin/qalc", "-t", expr],
    },
    "numbat": {
        "bin": "/home/sagnik/bench_tools/bin/numbat",
        "cmd": lambda expr: ["/home/sagnik/bench_tools/bin/numbat", "-e", expr.replace(" to ", " -> ")],
    },
    "fend": {
        "bin": "/home/sagnik/bench_tools/bin/fend",
        "cmd": lambda expr: ["/home/sagnik/bench_tools/bin/fend", expr],
    },
}

WORKLOADS = [
    ("Basic Arithmetic", "2 + 2 * 3^2"),
    ("Transcendental Math", "exp(1) + sin(0.5) * cos(0.5)"),
    ("Velocity Unit Conversion", "100 km/h to m/s"),
    ("Power Unit Derivation", "100 J / 10 s"),
]

ITERATIONS = 50
WARMUP = 10

def measure_rss(cmd_list):
    time_cmd = ["/usr/bin/time", "-v"] + cmd_list
    p = subprocess.run(time_cmd, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
    for line in p.stderr.splitlines():
        if "Maximum resident set size" in line:
            parts = line.split(":")
            return int(parts[1].strip())
    return 0

def run_benchmarks():
    print("=" * 80)
    print("Cross-Calculator Execution Benchmark: omnicalc vs qalc vs numbat vs fend")
    print(f"Sampling: {WARMUP} warmups, {ITERATIONS} timed iterations per workload")
    print("=" * 80)

    # 1. Binary Sizes
    bin_sizes = {}
    for name, spec in CALCULATORS.items():
        bin_path = spec["bin"]
        if os.path.exists(bin_path):
            size_bytes = os.path.getsize(bin_path)
            bin_sizes[name] = size_bytes / (1024 * 1024)
        else:
            bin_sizes[name] = 0.0

    results = {}

    for w_name, expr in WORKLOADS:
        print(f"\n--- Benchmark Workload: {w_name} ('{expr}') ---")
        results[w_name] = {}

        for calc_name, spec in CALCULATORS.items():
            cmd = spec["cmd"](expr)

            # Warmup
            for _ in range(WARMUP):
                subprocess.run(cmd, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)

            # Timed runs
            durations = []
            for _ in range(ITERATIONS):
                t0 = time.perf_counter_ns()
                subprocess.run(cmd, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
                t1 = time.perf_counter_ns()
                durations.append((t1 - t0) / 1_000_000.0) # convert to ms

            mean_ms = statistics.mean(durations)
            median_ms = statistics.median(durations)
            p95_ms = sorted(durations)[int(0.95 * len(durations))]
            min_ms = min(durations)
            max_ms = max(durations)
            rss_kb = measure_rss(cmd)

            results[w_name][calc_name] = {
                "mean_ms": mean_ms,
                "median_ms": median_ms,
                "p95_ms": p95_ms,
                "min_ms": min_ms,
                "max_ms": max_ms,
                "rss_kb": rss_kb,
            }

            print(f"[{calc_name:8s}] Mean: {mean_ms:5.2f} ms | Median: {median_ms:5.2f} ms | p95: {p95_ms:5.2f} ms | RSS: {rss_kb:6d} KB")

    return bin_sizes, results

if __name__ == "__main__":
    bin_sizes, results = run_benchmarks()
