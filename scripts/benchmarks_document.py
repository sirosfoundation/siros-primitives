"""Writes crates/biguint/BENCHMARKS.md and its charts from the results of scripts/benchmarks.py."""

from datetime import datetime
from pathlib import Path

import benchmarks_chart

ROOT = Path(__file__).resolve().parent.parent
DOCUMENT = ROOT / "crates" / "biguint" / "BENCHMARKS.md"
CHARTS = DOCUMENT.parent / "benchmarks"
# The crate under test.
SUBJECT = "biguint"
# Timings taken above this 1-minute load average are upper bounds.
QUIET_LOAD = 2.0
# The crates whose versions the setup table names.
CRATE_VERSIONS = ["ruint", "crypto-bigint", "bnum", "uint", "num-bigint", "criterion"]
# The metrics, exact instruction counts first.
METRICS = ("instructions", "time")
TITLES = {
    "add": "a + b",
    "sub": "a − b",
    "mul": "a × b",
    "div": "a / b",
    "mod": "a mod b",
    "mulmod": "a × b mod m",
}


def write(runs, cells, results):
    builds = list(runs)
    ops = unique(cell["op"] for cell in cells)
    widths = unique(cell["bits"] for cell in cells)
    crates = unique(cell["crate"] for cell in cells)
    by_id = {(cell["op"], cell["bits"], cell["crate"]): cell for cell in cells}
    provisional = any(
        max(sample[1] for sample in steps["timing"]["load"]) > QUIET_LOAD for steps in runs.values()
    )
    CHARTS.mkdir(parents=True, exist_ok=True)
    for op in ops:
        series = [
            (crate, {(build, metric, bits): results[build][metric].get(f"{op}/{bits}/{crate}")
                     for build in builds for metric in METRICS for bits in widths})
            for crate in crates
        ]
        note = ("The times are provisional upper bounds: the load average was above "
                f"{QUIET_LOAD:g}." if provisional else "")
        chart = benchmarks_chart.svg(f"{op}: {TITLES[op]}", note, builds, widths, series)
        (CHARTS / f"{op}.svg").write_text(chart)

    first = next(iter(runs.values()))["timing"]
    others = [f"`{crate}`" for crate in crates if crate != SUBJECT]
    lines = [
        "# Benchmarks",
        "",
        f"`{SUBJECT}` against {sentence_list(others)}: {sentence_list(ops)} at "
        f"{sentence_list(widths)} bits.",
        "",
        *source_lines(first["source"]),
        "",
        "```sh",
        "python3 scripts/benchmarks.py time     # criterion, about 15 minutes per build",
        "python3 scripts/benchmarks.py count    # callgrind",
        "python3 scripts/benchmarks.py report   # this file and benchmarks/*.svg",
        "```",
        "",
        "## Summary",
        "",
        f"Each cell holds two ratios: `{SUBJECT}`'s instructions per operation over the fewest "
        f"of any other crate, then `{SUBJECT}`'s time per operation over the fastest other "
        f"crate's time. A ratio below 1 favors `{SUBJECT}`.",
        "",
        *summary_table(builds, ops, widths, crates, results),
        "",
        *setup_lines(runs, provisional),
        "",
        *method_lines(cells),
        "",
        *functions_lines(ops, widths, crates, by_id),
        "",
        "## Results",
        "",
        "Each cell: instructions per operation · nanoseconds per operation. Bold marks the fewest "
        "instructions and the fastest time in each row. A dash marks a crate without a case.",
    ]
    for op in ops:
        lines += ["", f"### {op}: {TITLES[op]}", "", f"![{op}: {TITLES[op]}](benchmarks/{op}.svg)",
                  "", *result_table(op, builds, widths, crates, results)]
    DOCUMENT.write_text("\n".join(lines) + "\n")


def unique(values):
    return list(dict.fromkeys(values))


def sentence_list(values):
    values = [str(value) for value in values]
    return values[0] if len(values) == 1 else ", ".join(values[:-1]) + " and " + values[-1]


def table(header, rows, align):
    return [
        "| " + " | ".join(header) + " |",
        "|" + "|".join("---:" if side == "r" else "---" for side in align) + "|",
        *("| " + " | ".join(row) + " |" for row in rows),
    ]


def nanoseconds(value):
    """Three significant digits; whole nanoseconds from 100 up."""
    if value >= 100:
        return f"{value:,.0f}"
    return f"{value:.{3 - len(str(int(value))) if value >= 1 else 3}f}"


def instructions(value):
    return f"{value:,.0f}" if value >= 100 else f"{value:.1f}"


def source_lines(source):
    library = source["library_commit"][:7]
    head = source["head"][:7]
    state = " The measured tree had uncommitted changes." if source["dirty"] else ""
    return [
        f"The numbers measure `{SUBJECT}` at commit `{library}`, the last commit that changed "
        f"`crates/{SUBJECT}/src`. The bench ran at commit `{head}`.{state} "
        "`scripts/benchmarks.py` regenerates the results:",
    ]


def summary_table(builds, ops, widths, crates, results):
    rows = []
    for op in ops:
        for build in builds:
            row = [op, build]
            for bits in widths:
                ratios = []
                for metric in METRICS:
                    values = results[build][metric]
                    own = values.get(f"{op}/{bits}/{SUBJECT}")
                    others = [values[f"{op}/{bits}/{crate}"] for crate in crates
                              if crate != SUBJECT and f"{op}/{bits}/{crate}" in values]
                    if own is not None and others:
                        ratios.append(f"{own / min(others):.2f}")
                row.append(" · ".join(ratios) if ratios else "–")
            rows.append(row)
    header = ["Operation", "Build", *(f"{bits} bits" for bits in widths)]
    return table(header, rows, "ll" + "r" * len(widths))


def setup_lines(runs, provisional):
    first = next(iter(runs.values()))
    environment = first["timing"]["environment"]
    crates = environment["crates"]
    versions = ", ".join(f"{name} {crates[name]}" for name in CRATE_VERSIONS if name != "criterion")
    builds = []
    for build, steps in runs.items():
        flags = steps["timing"]["rustflags"]
        if flags:
            builds.append(f"`{build}`: `RUSTFLAGS=\"{flags}\"`, here "
                          f"`{environment['native_cpu']}`, with the `adx` carry intrinsics in "
                          f"`{SUBJECT}`.")
        else:
            builds.append(f"`{build}`: no `RUSTFLAGS`, the baseline x86-64 target.")
    rows = [
        ["CPU", f"{environment['cpu']}, {environment['logical_cpus']} logical CPUs"],
        ["System", environment["kernel"]],
        ["Compiler", f"{environment['rustc']}, LLVM {environment['llvm']}, `bench` profile"],
        ["Builds", " ".join(builds)],
        ["Timing", f"criterion {crates['criterion']}: 1 s warm-up, 3 s of samples, 100 samples, "
                   "on one logical CPU through `taskset`"],
        ["Instructions", f"{first['count']['valgrind']}, callgrind"],
        ["Crates", versions],
    ]
    load_rows = []
    for build, steps in runs.items():
        timing = steps["timing"]
        loads = [sample[1] for sample in timing["load"]]
        started = datetime.fromisoformat(timing["started"])
        minutes = (datetime.fromisoformat(timing["finished"]) - started).total_seconds() / 60
        load_rows.append([
            build,
            str(timing["cpu"]),
            started.strftime("%Y-%m-%d %H:%M"),
            f"{minutes:.0f} min",
            *(f"{load:.1f}" for load in (loads[0], min(loads), max(loads), loads[-1])),
        ])
    if provisional:
        verdict = (
            "**The times are provisional.** The 1-minute load average rose above "
            f"{QUIET_LOAD:g} during the timing runs, so each time is an upper bound. The times "
            "compare only within one table. The instruction counts are exact at any load."
        )
    else:
        verdict = (f"The 1-minute load average stayed at or below {QUIET_LOAD:g} during the "
                   "timing runs.")
    return [
        "## Setup",
        "",
        *table(["", ""], rows, "ll"),
        "",
        "The 1-minute load average of each timing run, read from `/proc/loadavg` at its start, "
        "every 5 seconds, and at its end:",
        "",
        *table(["Build", "CPU", "Started (UTC)", "Duration", "Start", "Lowest", "Highest", "End"],
               load_rows, "lrlrrrrr"),
        "",
        verdict,
    ]


def method_lines(cells):
    batch = cells[0]["batch"]
    return [
        "## Method",
        "",
        f"- Each case runs one crate's function over a batch of {batch} operand sets. Each "
        "operand set and each result passes through `core::hint::black_box`, so the branch "
        "predictors see varied data and the compiler computes every result.",
        f"- Instructions per operation: callgrind's count from the entry to the exit of "
        f"`run_batch`, divided by {batch}. The count includes the loop around the operation: "
        "it loads each operand set and stores each result. The counts are exact at any load.",
        f"- Time per operation: criterion's fastest sample, per iteration, divided by {batch}. "
        "The other processes on the machine disturb the fastest sample least.",
        "- Before a case is measured, its result on every operand set must equal the exact "
        "result from `num-bigint`, reduced modulo 2^N for the crates that wrap.",
        "",
        "## Operands",
        "",
        "At width N:",
        "",
        *table(["Operation", "Operands"], [
            ["add, mul", "`a` and `b` uniform below 2^N"],
            ["sub", "`a` and `b` uniform below 2^N, ordered so that `a >= b`"],
            ["div, mod", "a dividend with bit N − 1 set; a divisor below 2^(N/2) with bit "
                         "N/2 − 1 set, held in an N-bit integer"],
            ["mulmod", "a modulus `m` with bit N − 1 set; `a` and `b` uniform below `m`"],
        ], "ll"),
        "",
        f"Every crate in a row gets the same {batch} operand sets.",
    ]


def functions_lines(ops, widths, crates, by_id):
    rows = []
    for op in ops:
        row = [op]
        for crate in crates:
            groups = {}
            for bits in widths:
                cell = by_id[(op, bits, crate)]
                if cell["kind"] != "absent":
                    groups.setdefault((cell["kind"], cell["detail"]), []).append(bits)
            entries = []
            for (kind, detail), group in groups.items():
                text = f"`{detail}`" if kind == "api" else f"{detail} (composed)"
                if group != widths:
                    text += f" at {sentence_list(group)} bits"
                entries.append(text)
            row.append("; ".join(entries) or "–")
        rows.append(row)
    absent = {}
    for (op, bits, crate), cell in by_id.items():
        if cell["kind"] == "absent":
            absent.setdefault((crate, cell["detail"]), {}).setdefault(op, []).append(bits)
    gaps = []
    for (crate, reason), per_op in absent.items():
        by_widths = {}
        for op, group in per_op.items():
            by_widths.setdefault(tuple(group), []).append(op)
        for group, group_ops in by_widths.items():
            gaps.append(f"- `{crate}` has no case for {sentence_list(group_ops)} at "
                        f"{sentence_list(group)} bits: {reason}.")
    return [
        "## Functions",
        "",
        "Each crate runs its own fastest public function for an operation. A crate without one "
        "runs the obvious composition of its public functions, marked composed.",
        "",
        *table(["Operation", *crates], rows, "l" * (len(crates) + 1)),
        "",
        "- `native` is `u64` at 64 bits and `u128` at 128 bits. Its divisors are `NonZero`.",
        "- `crypto-bigint` is constant-time by design. Its div, mod and mulmod cases run its "
        "`*_vartime` functions. Its add, sub and mul have no variable-time variant.",
        "- `num-bigint` allocates every result and returns it exactly: add returns up to N + 1 "
        "bits, and mul returns the 2N-bit product. Its times include the allocation.",
        "- `uint` is the parity crate. Its types come from `construct_uint!`, as in "
        "`primitive-types`; its add, sub and mul wrap through `overflowing_*`.",
        *gaps,
    ]


def result_table(op, builds, widths, crates, results):
    formats = {"instructions": instructions, "time": lambda value: f"{nanoseconds(value)} ns"}
    rows = []
    for bits in widths:
        for build in builds:
            cases = [f"{op}/{bits}/{crate}" for crate in crates]
            present = [case for case in cases if case in results[build]["time"]]
            best = {metric: min(results[build][metric][case] for case in present)
                    for metric in METRICS}
            row = [str(bits), build]
            for case in cases:
                parts = []
                for metric in METRICS if case in present else ():
                    value = results[build][metric][case]
                    text = formats[metric](value)
                    parts.append(f"**{text}**" if value == best[metric] else text)
                row.append(" · ".join(parts) or "–")
            rows.append(row)
    return table(["Bits", "Build", *crates], rows, "rl" + "r" * len(crates))
