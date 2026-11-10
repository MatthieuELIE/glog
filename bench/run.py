"""Measure the release glog binary in a pseudo-terminal on a synthetic repository."""
import argparse
import math
import statistics
import subprocess
import sys
import time
from dataclasses import dataclass
from pathlib import Path

import repos
import session

PROJECT = Path(__file__).parent.parent
BINARY = PROJECT / "target" / "release" / "glog"
COLUMNS, ROWS = 200, 50
KEYS = 200
START_TIMEOUT = 60
KEY_TIMEOUT = 2
LATENCY_BUDGET_MS = 16
MEMORY_BUDGET_MB = 250
FIRST_FRAME_BUDGET_MS = 3000


@dataclass
class Measure:
    first_frame_ms: float
    loaded_ms: float
    latencies_ms: list[float]
    timed_out: bool
    memory_mb: float


def percentile_95(values):
    return sorted(values)[math.ceil(0.95 * len(values)) - 1]


def latency_line(measure):
    if measure.timed_out:
        return f"key latency   > {KEY_TIMEOUT * 1000} ms on key {len(measure.latencies_ms) + 1}"

    median = statistics.median(measure.latencies_ms)
    worst = percentile_95(measure.latencies_ms)

    return f"key latency   median {median:.1f} ms, p95 {worst:.1f} ms over {len(measure.latencies_ms)} keys"


def report(measure):
    return [
        f"first frame   {measure.first_frame_ms:.1f} ms",
        f"loading done  {measure.loaded_ms:.1f} ms",
        latency_line(measure),
        f"memory        {measure.memory_mb:.0f} MB",
    ]


def over_budget(measure):
    failures = []
    if measure.first_frame_ms >= FIRST_FRAME_BUDGET_MS:
        failures.append(f"first frame over {FIRST_FRAME_BUDGET_MS} ms")

    if measure.timed_out or percentile_95(measure.latencies_ms) >= LATENCY_BUDGET_MS:
        failures.append(f"p95 key latency over {LATENCY_BUDGET_MS} ms")

    if measure.memory_mb >= MEMORY_BUDGET_MB:
        failures.append(f"memory over {MEMORY_BUDGET_MB} MB")

    return failures


def press_keys(master, end):
    key = b"j"
    if end:
        key = b"k"
        if session.key_latency(master, b"G", KEY_TIMEOUT) is None:
            return [], True

    latencies_ms = []
    for _ in range(KEYS):
        latency = session.key_latency(master, key, KEY_TIMEOUT)
        if latency is None:
            return latencies_ms, True

        latencies_ms.append(latency * 1000)

    return latencies_ms, False


def measure(repo, end):
    start = time.monotonic()
    process, master = session.start(BINARY, repo, COLUMNS, ROWS)
    if not session.wait_frame(master, START_TIMEOUT):
        session.stop(process, master)
        sys.exit(f"glog drew no frame within {START_TIMEOUT} s")

    first_frame_ms = (time.monotonic() - start) * 1000
    while session.has_git_child(process):
        if time.monotonic() - start > START_TIMEOUT:
            session.stop(process, master)
            sys.exit(f"glog was still loading after {START_TIMEOUT} s")

        session.drain(master, 0.01)

    loaded_ms = (time.monotonic() - start) * 1000
    session.drain(master, 0.3)
    latencies_ms, timed_out = press_keys(master, end)
    memory_mb = session.memory(process)
    session.stop(process, master)

    return Measure(first_frame_ms, loaded_ms, latencies_ms, timed_out, memory_mb)


def arguments():
    parser = argparse.ArgumentParser(description=__doc__)
    size = parser.add_mutually_exclusive_group(required=True)
    size.add_argument("size", nargs="?", choices=["s", "m", "l"], help="the repository to measure")
    size.add_argument("--smoke", action="store_true", help="measure a tiny repository")
    parser.add_argument("--check", action="store_true", help="fail when a budget of the scale spec is exceeded")
    parser.add_argument("--end", action="store_true", help="press G first and measure k from the last commit")

    return parser.parse_args()


def main():
    args = arguments()
    size = args.size
    if args.smoke:
        size = "smoke"

    repo = repos.ensure(size)
    subprocess.run(["cargo", "build", "--release", "--locked", "--quiet"], cwd=PROJECT, check=True)
    # cargo links the binary anew on every build, and macOS takes 200 ms to check a new file on its first run.
    subprocess.run([BINARY, "--version"], check=True, stdout=subprocess.DEVNULL)
    result = measure(repo, args.end)
    print("\n".join(report(result)))

    if args.check:
        failures = over_budget(result)
        if failures:
            sys.exit("\n".join(failures))


main()
