#!/usr/bin/env python3
"""Runs the comparison benchmarks of `biguint` and writes `crates/biguint/BENCHMARKS.md`.

    python3 scripts/benchmarks.py time [--build NAME] [--cpu N]
    python3 scripts/benchmarks.py count [--build NAME] [--jobs N]
    python3 scripts/benchmarks.py report

`time` runs the criterion bench of each build on one logical CPU and samples the load average
during the run. `count` runs every case of each build once under callgrind. Both write their
results to target/benchmarks/<build>/. `report` reads criterion's JSON and the callgrind output
and writes BENCHMARKS.md and one SVG chart per operation to crates/biguint/benchmarks/.

Uses the standard library only.
"""

import argparse
import concurrent.futures
import hashlib
import json
import os
import re
import shutil
import subprocess
import sys
import threading
import time
from datetime import datetime, timezone
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import benchmarks_document  # noqa: E402

ROOT = Path(__file__).resolve().parent.parent
RESULTS = ROOT / "target" / "benchmarks"
# Build name: RUSTFLAGS.
BUILDS = {"x86-64": "", "native": "-C target-cpu=native"}
# Callgrind counts the instructions from the entry to the exit of this function.
COUNTED = "compare::suite::run_batch*"
# The files that the measured executables are built from, as git pathspecs.
MEASURED = [
    "Cargo.toml",
    "Cargo.lock",
    "rust-toolchain.toml",
    "crates/biguint",
    ":(exclude)crates/biguint/BENCHMARKS.md",
    ":(exclude)crates/biguint/benchmarks",
]
LOAD_INTERVAL_SECONDS = 5.0


def main():
    parser = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    commands = parser.add_subparsers(dest="command", required=True)
    timing = commands.add_parser("time", help="time every case with criterion")
    timing.add_argument("--build", choices=BUILDS, action="append", help="default: every build")
    timing.add_argument("--cpu", type=int, help="logical CPU (default: the idlest core)")
    counting = commands.add_parser("count", help="count instructions with callgrind")
    counting.add_argument("--build", choices=BUILDS, action="append", help="default: every build")
    counting.add_argument("--jobs", type=int, default=4, help="parallel callgrind runs")
    commands.add_parser("report", help="write BENCHMARKS.md and the charts")
    args = parser.parse_args()
    if args.command == "report":
        report()
        return
    for build in args.build or BUILDS:
        if args.command == "time":
            time_build(build, args.cpu)
        else:
            count_build(build, args.jobs)


def run(command, **kwargs):
    """The standard output of `command`; raises when it fails."""
    return subprocess.run(command, check=True, capture_output=True, text=True, **kwargs).stdout


def git(*args):
    return run(["git", "-C", str(ROOT), *args]).strip()


def now():
    return datetime.now(timezone.utc).isoformat(timespec="seconds")


def uptime():
    """The output of `uptime`, with `.` as the decimal separator."""
    return run(["uptime"], env=dict(os.environ, LC_ALL="C")).strip()


def write_json(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=1) + "\n")


def read_json(path):
    if not path.exists():
        raise SystemExit(f"{path.relative_to(ROOT)} is missing: run the step that writes it")
    return json.loads(path.read_text())


def compile_bench(build):
    """Builds the bench of `build` in its own target directory and returns the executable."""
    env = dict(os.environ, CARGO_TARGET_DIR=str(RESULTS / build / "cargo"), RUSTFLAGS=BUILDS[build])
    env.pop("CARGO_ENCODED_RUSTFLAGS", None)
    command = ["cargo", "bench", "--locked", "-p", "biguint", "--bench", "compare", "--no-run"]
    output = run([*command, "--message-format=json-render-diagnostics"], cwd=ROOT, env=env)
    for line in output.splitlines():
        message = json.loads(line)
        if message.get("reason") == "compiler-artifact" and message.get("executable"):
            if message["target"]["name"] == "compare":
                return message["executable"]
    raise SystemExit(f"cargo built no `compare` bench for {build}")


def source_state():
    """The commit, the last commit that changed the library, whether the measured files differ
    from the commit, and a digest of their contents."""
    digest = hashlib.sha256()
    files = git("ls-files", "--cached", "--others", "--exclude-standard", "--", *MEASURED)
    for name in sorted(set(files.splitlines())):
        path = ROOT / name
        if path.is_file():
            digest.update(name.encode() + b"\0" + path.read_bytes() + b"\0")
    return {
        "head": git("rev-parse", "HEAD"),
        "library_commit": git("log", "-1", "--format=%H", "--", "crates/biguint/src"),
        "dirty": git("status", "--porcelain", "--", *MEASURED) != "",
        "digest": digest.hexdigest(),
    }


def search(pattern, text, what):
    """The first group of `pattern` in `text`; raises when `text` does not hold `what`."""
    found = re.search(pattern, text, re.MULTILINE)
    if found is None:
        raise SystemExit(f"cannot read {what}")
    return found.group(1)


def environment():
    """The compiler, the machine and the crate versions, as recorded with each run."""
    rustc = run(["rustc", "-vV"], cwd=ROOT)
    cpus = run(["rustc", "--print", "target-cpus"], cwd=ROOT)
    lock = (ROOT / "Cargo.lock").read_text()
    versions = dict(re.findall(r'name = "([^"]+)"\nversion = "([^"]+)"', lock))
    return {
        "rustc": rustc.splitlines()[0],
        "llvm": search(r"^LLVM version: (.+)$", rustc, "the LLVM version from rustc -vV"),
        "native_cpu": search(r"currently (\S+?)\)?\.?$", cpus, "the native CPU from rustc"),
        "cpu": search(r"^model name\s*: (.+)$", Path("/proc/cpuinfo").read_text(),
                      "the CPU model from /proc/cpuinfo"),
        "logical_cpus": os.cpu_count(),
        "kernel": run(["uname", "-sr"]).strip(),
        "crates": {name: versions[name] for name in benchmarks_document.CRATE_VERSIONS},
    }


def describe(build, executable):
    """What a run measures: the build, the sources, the environment and the cells."""
    cases = [json.loads(line) for line in run([executable, "--cases"]).splitlines()]
    return {
        "build": build,
        "rustflags": BUILDS[build],
        "source": source_state(),
        "environment": environment(),
        "cases": cases,
    }


class LoadSampler:
    """Reads the 1, 5 and 15-minute load averages from /proc/loadavg when it starts, every few
    seconds while in use, and when it stops. Each sample is `[seconds, load1, load5, load15]`."""

    def __init__(self):
        self.samples = []
        self.start = time.monotonic()
        self.stopped = threading.Event()
        self.thread = threading.Thread(target=self.sample_until_stopped)

    def sample(self):
        loads = Path("/proc/loadavg").read_text().split()[:3]
        self.samples.append([round(time.monotonic() - self.start, 1), *map(float, loads)])

    def sample_until_stopped(self):
        while not self.stopped.wait(LOAD_INTERVAL_SECONDS):
            self.sample()

    def __enter__(self):
        self.sample()
        self.thread.start()
        return self

    def __exit__(self, *exception):
        self.stopped.set()
        self.thread.join()
        self.sample()


def idlest_cpu():
    """The logical CPU of the physical core that was idlest over one second."""

    def idle_times():
        times = {}
        for line in Path("/proc/stat").read_text().splitlines():
            fields = line.split()
            if re.fullmatch(r"cpu\d+", fields[0]):
                values = list(map(int, fields[1:]))
                times[int(fields[0][3:])] = (values[3] + values[4], sum(values))
        return times

    before = idle_times()
    time.sleep(1)
    after = idle_times()
    idle = {
        cpu: (after[cpu][0] - before[cpu][0]) / max(1, after[cpu][1] - before[cpu][1])
        for cpu in after
    }

    def core_idle(cpu):
        siblings = Path(f"/sys/devices/system/cpu/cpu{cpu}/topology/thread_siblings_list")
        members = re.findall(r"\d+", siblings.read_text()) if siblings.exists() else [cpu]
        return sum(idle[int(member)] for member in members), idle[cpu]

    return max(idle, key=core_idle)


def time_build(build, cpu):
    executable = compile_bench(build)
    record = describe(build, executable)
    criterion = RESULTS / build / "criterion"
    shutil.rmtree(criterion, ignore_errors=True)
    record["cpu"] = idlest_cpu() if cpu is None else cpu
    record["uptime_before"] = uptime()
    record["started"] = now()
    print(f"{build}: timing on CPU {record['cpu']}; {record['uptime_before']}", flush=True)
    log = RESULTS / build / "criterion.log"
    with LoadSampler() as sampler, log.open("w") as output:
        subprocess.run(
            ["taskset", "--cpu-list", str(record["cpu"]), executable, "--bench"],
            env=dict(os.environ, CRITERION_HOME=str(criterion)),
            stdout=output,
            stderr=subprocess.STDOUT,
            check=True,
        )
    record["finished"] = now()
    record["uptime_after"] = uptime()
    record["load"] = sampler.samples
    write_json(RESULTS / build / "timing.json", record)
    print(f"{build}: timed; {record['uptime_after']}", flush=True)


def count_case(executable, directory, case):
    stem = case.replace("/", "_")
    output = directory / f"{stem}.out"
    command = [
        "valgrind",
        "--tool=callgrind",
        f"--callgrind-out-file={output}",
        "--collect-atstart=no",
        f"--toggle-collect={COUNTED}",
        executable,
        "--count",
        case,
    ]
    process = subprocess.run(command, capture_output=True, text=True)
    if process.returncode != 0:
        raise SystemExit(f"{case}: callgrind failed\n{process.stderr}")
    (directory / f"{stem}.json").write_text(process.stdout.strip().splitlines()[-1] + "\n")


def count_build(build, jobs):
    executable = compile_bench(build)
    record = describe(build, executable)
    record["valgrind"] = run(["valgrind", "--version"]).strip()
    directory = RESULTS / build / "callgrind"
    shutil.rmtree(directory, ignore_errors=True)
    directory.mkdir(parents=True)
    cases = [cell["case"] for cell in record["cases"] if cell["kind"] != "absent"]
    record["started"] = now()
    with concurrent.futures.ThreadPoolExecutor(jobs) as pool:
        for future in [pool.submit(count_case, executable, directory, case) for case in cases]:
            future.result()
    record["finished"] = now()
    write_json(RESULTS / build / "count.json", record)
    print(f"{build}: counted {len(cases)} cases", flush=True)


def criterion_times(directory, cells):
    """Nanoseconds per operation of every case: criterion's fastest sample per iteration,
    divided by the batch."""
    fastest = {}
    for benchmark in directory.glob("*/new/benchmark.json"):
        sample = read_json(benchmark.parent / "sample.json")
        per_iteration = [t / n for t, n in zip(sample["times"], sample["iters"])]
        fastest[read_json(benchmark)["full_id"]] = min(per_iteration)
    return {
        cell["case"]: fastest[cell["case"]] / cell["batch"]
        for cell in present(cells, fastest, f"{directory.relative_to(ROOT)}")
    }


def callgrind_counts(directory, cells):
    """Instructions per operation of every case: the callgrind total of one batch, divided by
    the operations that the batch ran."""
    counts = {}
    for output in directory.glob("*.out"):
        totals = re.search(r"^totals: (\d+)", output.read_text(), re.MULTILINE)
        operations = read_json(output.with_suffix(".json"))
        if totals is None or int(totals.group(1)) == 0:
            raise SystemExit(f"{output}: callgrind counted nothing inside {COUNTED}")
        counts[operations["case"]] = int(totals.group(1)) / operations["operations"]
    return {cell["case"]: counts[cell["case"]] for cell in present(cells, counts, str(directory))}


def present(cells, results, source):
    """The cells with a case; raises unless `results` holds every one of them."""
    cases = [cell for cell in cells if cell["kind"] != "absent"]
    missing = [cell["case"] for cell in cases if cell["case"] not in results]
    if missing:
        raise SystemExit(f"{source} has no result for {', '.join(missing)}")
    return cases


def report():
    runs = {
        build: {step: read_json(RESULTS / build / f"{step}.json") for step in ("timing", "count")}
        for build in BUILDS
    }
    records = [record for steps in runs.values() for record in steps.values()]
    for key in ("source", "environment", "cases"):
        if any(record[key] != records[0][key] for record in records):
            raise SystemExit(f"the runs differ in their {key}: rerun `time` and `count`")
    cells = records[0]["cases"]
    results = {
        build: {
            "time": criterion_times(RESULTS / build / "criterion", cells),
            "instructions": callgrind_counts(RESULTS / build / "callgrind", cells),
        }
        for build in BUILDS
    }
    benchmarks_document.write(runs, cells, results)


if __name__ == "__main__":
    main()
