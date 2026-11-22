#!/usr/bin/env python3
"""Compare calculator startup/evaluation latency for locally installed tools.

Override executable locations with OMNICALC_BIN, QALC_BIN, NUMBAT_BIN, and
FEND_BIN. Missing calculators are skipped rather than reported as zero-size,
zero-latency successes.
"""

import os
import shutil
import statistics
import subprocess
import time
from pathlib import Path

ITERATIONS = 50
WARMUP = 10
WORKLOADS = [
    ("Basic arithmetic", "2 + 2 * 3^2"),
    ("Transcendental math", "exp(1) + sin(0.5) * cos(0.5)"),
    ("Velocity conversion", "100 km/h to m/s"),
    ("Power derivation", "100 J / 10 s"),
]


def resolve_binary(env_name: str, fallback: str) -> str | None:
    configured = os.environ.get(env_name)
    if configured:
        path = Path(configured).expanduser()
        return str(path) if path.is_file() and os.access(path, os.X_OK) else None
    return shutil.which(fallback)


def calculator_specs() -> dict[str, tuple[str, callable]]:
    specs = {}
    entries = [
        ("omnicalc", "OMNICALC_BIN", "omnicalc", lambda binary, expr: [binary, "-e", expr]),
        ("qalc", "QALC_BIN", "qalc", lambda binary, expr: [binary, "-t", expr]),
        ("numbat", "NUMBAT_BIN", "numbat", lambda binary, expr: [binary, "-e", expr.replace(" to ", " -> ")]),
        ("fend", "FEND_BIN", "fend", lambda binary, expr: [binary, expr]),
    ]
    for name, env_name, fallback, command in entries:
        binary = resolve_binary(env_name, fallback)
        if binary:
            specs[name] = (binary, command)
    return specs


def run_checked(command: list[str]) -> None:
    subprocess.run(command, check=True, stdout=subprocess.DEVNULL, stderr=subprocess.PIPE, text=True)


def measure_rss_kb(command: list[str]) -> int | None:
    """Return peak child RSS when GNU time is available; otherwise omit it."""
    time_bin = "/usr/bin/time"
    if not os.path.isfile(time_bin):
        return None
    result = subprocess.run(
        [time_bin, "-v", *command], capture_output=True, text=True, check=False
    )
    if result.returncode != 0:
        return None
    for line in result.stderr.splitlines():
        if "Maximum resident set size" in line:
            try:
                return int(line.split(":", 1)[1].strip())
            except (IndexError, ValueError):
                return None
    return None


def run_benchmarks() -> None:
    specs = calculator_specs()
    if not specs:
        raise SystemExit("No calculator executables found on PATH or via *_BIN variables.")

    print("Calculator startup/evaluation benchmark")
    print(f"Sampling: {WARMUP} warmups, {ITERATIONS} timed runs per workload")
    print("Note: timings include process startup and are machine/load dependent.")

    for workload, expression in WORKLOADS:
        print(f"\n--- {workload}: {expression} ---")
        for name, (binary, make_command) in specs.items():
            command = make_command(binary, expression)
            try:
                for _ in range(WARMUP):
                    run_checked(command)
                durations = []
                for _ in range(ITERATIONS):
                    start = time.perf_counter_ns()
                    run_checked(command)
                    durations.append((time.perf_counter_ns() - start) / 1_000_000)
            except (OSError, subprocess.CalledProcessError) as error:
                print(f"[{name:8s}] skipped: command failed ({error})")
                continue

            ordered = sorted(durations)
            p95 = ordered[min(len(ordered) - 1, int(0.95 * len(ordered)))]
            rss = measure_rss_kb(command)
            rss_text = f"{rss:6d} KB" if rss is not None else "unavailable"
            print(
                f"[{name:8s}] median {statistics.median(durations):6.2f} ms | "
                f"p95 {p95:6.2f} ms | RSS {rss_text}"
            )


if __name__ == "__main__":
    run_benchmarks()
