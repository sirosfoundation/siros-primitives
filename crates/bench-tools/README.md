# bench-tools

The report tools of a comparison that [`bench-compare`](../bench-compare) measures: they build and
run its benches, natively and in Chrome and Firefox as WebAssembly, and write its `BENCHMARKS.md`.
No primitive depends on this crate.

## Features

- `execute`: the report tool.
  - `bench [--cpu <cpu>] [--machine <description>]` builds, runs and records
    `benchmarks/record.json`;
  - `bench-wasm --chrome <executable> --firefox <executable> [--changed [--dry-run]] [--runs <runs>]
    [--target-dir <directory>] [--cpu <cpu>[,<cpu>...]] [--browser-cpu <cpu>[,<cpu>...]] [--machine
    <description>]` builds the builds of `WASM` and runs them in the two browsers, on one stream per
    measuring CPU, and records `benchmarks/record-wasm.json` after each run of a bench;
  - `stamp-wasm --chrome <executable> [--target-dir <directory>]` writes the keys of the tree into
    the WebAssembly record;
  - `modules --out <directory> [--target-dir <directory>]` builds the modules of a decision;
  - `report` writes `BENCHMARKS.md` from the records.
- `bench-browser`: runs one WebAssembly bench in one browser and prints its lines. `decide` compares
  two directories of modules; `list`, `diff` and `edges` read modules.

The tests run by name, which keeps `cargo test --workspace` from compiling this crate:

```sh
cargo test -p bench-tools --test changed --test identity
```

## Usage

A report tool declares what its comparison measures and how the document names it:

```rust,no_run
use bench_tools::{Layout, Measured, NATIVE, Operation, Paths, sentence};

const MEASURED: Measured = Measured {
    package: "biguint",
    benches: &["add", "mul"],
    versions: &["ruint"],
    features: &[],
    forbidden: &["not-precomputed"],
    builds: &[NATIVE],
};

const LAYOUT: Layout<'static, u32> = Layout {
    subject: "biguint",
    tool: "biguint-bench-report",
    operations: &[
        Operation::new("add", "add", "a + b"),
        Operation::constant_time("add_ct", "add", "a + b"),
    ],
    row_header: "Bits",
    rows_phrase: |widths| format!("at {} bits", sentence(widths)),
    bits: |bits| *bits,
    method: &["Operands at width N: uniform below 2^N."],
    wasm_native: "",
};

fn main() -> std::process::ExitCode {
    bench_tools::execute(
        "biguint-bench-report",
        &Paths::new(env!("CARGO_MANIFEST_DIR")),
        &MEASURED,
        &LAYOUT,
    )
}
```

```sh
cargo run -p biguint-bench-report -- bench
cargo run -p biguint-bench-report -- bench-wasm --chrome <chrome> --firefox <firefox> \
  --cpu 12 --browser-cpu 14
cargo run -p biguint-bench-report -- report
```

The header names the last commit of the library code: the `src`, `Cargo.toml` and `build.rs` of
the package and of every workspace package it is built from, as `cargo metadata` resolves them.
The record marks the run dirty when those files, the root manifests or the package's `benches`
differ from `HEAD`.

One bench in one browser, such as the calibration bench of `bench-compare`:

```sh
cargo build --release --target wasm32-unknown-unknown -p bench-compare --bench calibration
cargo run --release -p bench-tools --bin bench-browser -- \
  target/wasm32-unknown-unknown/release/deps/calibration-<hash>.wasm --firefox <firefox> \
  --cpu 12 --browser-cpu 14 [--build baseline | wide] [--tiering optimizing | default] \
  [--threads measuring | process] [--only <op/key/crate,...>] [--runs <runs>]
```

## WebAssembly

- The driver starts the browser headless with a fresh profile and serves the page, the module and
  the workers. The page runs the module in a dedicated worker, which the driver finds by its CPU
  time while it spins, pins to the measuring CPU and counts with perf_event. The rest of the
  browser runs on `--browser-cpu`. A thread of the driver holds the measuring CPU: it yields to the
  measuring thread, and keeps other work off the CPU while that thread sleeps.
- A second worker carries each count's start and end to the driver as synchronous requests, while
  the measuring worker sleeps in `Atomics.wait`: a count costs the measuring thread 1,900
  instructions in Firefox and 3,100 in Chrome. Shared memory needs the cross-origin-isolation
  headers, which the driver sends.
- Chrome runs with `--no-sandbox`, which a system that forbids unprivileged user namespaces needs,
  and `--js-flags=--no-liftoff`; Firefox runs with `javascript.options.wasm_baselinejit` off:
  every function compiles with the optimizing tier when the module loads. The builds with wide
  arithmetic add `--wasm-wide-arithmetic` to V8's flags and
  `javascript.options.wasm_wide_arithmetic` to Firefox's preferences.
- A run passes over its operand sets, of up to 256 KiB as natively, as many times as it takes to
  count 4 million instructions, which load does not change: sizing on cycles stopped early on a
  loaded machine.
- Each bench first counts 1,000 empty regions: the fewest of the last 248 is the fixed cost that
  every later count leaves out. Its noise is the difference of the fewest counts of two sets of 31
  empty counts, the largest of four pairs. A form whose difference counts under 50 times that noise
  runs 31 fresh repetitions of twice its operations, up to three times, and past them stops the
  sweep: the run grows until the cost of a count is a bounded share of the figure, as nanobench
  sizes its epochs. A form within the bound keeps its run, and a cell's `operations` name the run
  of its figure. A round of longer runs, like a round of more repetitions, runs every case again in
  the order of the first round and keeps the counts of the forms that need them: their repetitions
  meet the load over as long a span as the first round's, each after the same cases. With the
  noise held at 20,700 cycles, and the sibling hardware threads of the counted CPUs busy in bursts
  of 12 to 20 s for half of the sweep, rounds of those forms alone put 15.6% of the lengthened
  forms more than 10% above a natural sweep, and rounds of every case none.
- A figure of cycles is the lower rate of the fewest run of n operations and the fewest of 2n, each
  less the fewest empty count. A disturbance only adds counts (Chen and Revels, 2016), so each rate
  bounds the undisturbed one from above, where a difference moves by twice what the other run
  missed. At a load of 40 to 60, the median distance of a cell between two runs fell from 2.2% to
  0.5% in Chrome and from 3.3% to 1.3% in Firefox, the 95th percentile from 18.7% to 8.5% and from
  39.0% to 26.8%.
- A chain holds `x` in WebAssembly locals and stores a copy after each step, which keeps every
  intermediate `x`; it also runs with `x` in memory, and the faster counts.
- The driver needs to trace the browser: it starts the browser itself and adopts its orphans as
  their subreaper, which `kernel.yama.ptrace_scope` 1 allows.
- The document shows the WebAssembly builds in four tables, throughput and latency each: per
  operation and row, the subject against the fewest figure of the crates that keep the row's
  definition, as `Δ` in cycles and as a ratio; and the subject against its native build
  `Layout::wasm_native`, as a ratio.

## Measuring what changed

A selection names cells as `op/key/crate`, separated by commas, where a part may be `*`. It reaches
the module through an import, and the module stays whole: a selected cell runs the code of a full
run, on the operands of a full run, because each case on WebAssembly draws from a stream of its own.
A pattern that matches no cell stops the run.

A decision between two trees:

```sh
cargo run -p prime-field-bench-report -- modules --out <baseline>   # in the baseline's tree
cargo run -p prime-field-bench-report -- modules --out <change>     # in the change's tree
cargo run --release -p bench-tools --bin bench-browser -- decide \
  --baseline <baseline> --change <change> \
  --only 'mul_vartime/P-256 base/prime-field,mul_vartime/P-256 base/ark-ff' \
  --chrome <chrome> --firefox <firefox> --cpu 12 --browser-cpu 14 [--runs <runs>]
```

It runs both sides back to back in each of the four builds, `--runs` times, and prints each cell's
fewest figures and the load.

`bench-wasm --changed` is a content-addressed action cache, as in Bazel. A cell keeps its figures
while the record holds its key:

- its code: the hashes of its batch and chain loops, of what draws its inputs and of its setup, with
  all they call and all the data they address; the hash of its operands from a fixed seed; the bytes
  that its operation captures; and, when those can address memory that code fills, the hash of the
  function that offers its crate's cells;
- the browser's name, version and settings;
- the `RUSTFLAGS` of the build and the model of the CPU;
- the hash of the sources that measure: `HARNESS` in `src/changed.rs`.

A cell without a key counts as changed, and so does a cell whose loop can reach an indirect call. A
round runs the changed cells, the subject's columns in their rows, and each row's fewest figure and
best competitor per part; a module with none of them stays out, and a module where they hold a third
of its cells or more runs whole, once, for no more time than three runs of them. Fresh figures
replace old ones, with no minimum across runs. A round repeats while a kept figure leads a
re-measured row, 12 times at most; past that the tool stops with an error. `--dry-run` prints the
cells of the first round of each build. A `bench-wasm` without `--changed` measures every cell.

`report` refuses a record whose verdict compares figures of different runs: per build, operation,
row and part, one of the tied fewest figures and one of the best competitor's must come from the
subject's run. Each cell records its run. The setup table of `BENCHMARKS.md` gives each run's
commit, cells and load.

The hash in `src/identity.rs` is a Merkle hash over the reachable functions and data. It reads the
relocations that `KEEP_RELOCATIONS` (`-C link-arg=--emit-relocs`) keeps in each module, so each
address resolves to its symbol: layout leaves the hash, and the symbol's bytes enter it. The flag
adds custom sections and writes the zeros of `.bss` as a data segment. The two `num-bigint` crates
of the prime-field plain builds swap places in those modules, because `RUSTFLAGS` enters the file
names of the rlibs; every function body stays the same. An opcode or a relocation that the reader
does not know stops the tool with its offset. `bench-browser diff` names the functions that differ
between two modules, and `bench-browser edges` prints the call edges that the reader sees, to
compare with `wasm-tools print`.

`stamp-wasm` writes the keys of a record measured at an earlier commit, without measuring. It
refuses a record that ran dirty or that a sweep left unfinished, uncommitted changes in the measured
sources, and sources that differ from the record's `head`: a root `Cargo.toml` that differs beyond
its `members`, and a `Cargo.lock` where the measured package reaches a package or a version that it
reaches nowhere at `head`. The files of `bench-compare` and `bench-tools` stay out of that check:
run `bench-browser diff` between the modules of both commits first.

A selected cell takes the fewest figures of `SELECTED_RUNS`, 3, runs of its bench: repetitions
within one run can all meet one burst of load, where a full run spreads them over minutes. Cycles of
384 figures of 48 cells across 16 operations in two crates, against the full runs of the same module
before and after them, at load 29 to 96:

| Against each full run     | Median distance | 95th percentile | Median signed difference |
| ------------------------- | --------------- | --------------- | ------------------------ |
| The other full run        | 0.8%            | 6.3%            | -0.1%                    |
| 1 selected run            | 0.9%, 1.1%      | 13.2%, 12.9%    | +0.2%, +0.3%             |
| Fewest of 2 selected runs | 0.7%, 0.7%      | 5.5%, 6.7%      | -0.2%, -0.1%             |
| Fewest of 3 selected runs | 0.6%, 0.7%      | 5.3%, 5.9%      | -0.4%, -0.2%             |

Above the fewest figure of all seven runs of a cell, a full run stands at a median of 0.7% and a
95th percentile of 5.9%, 3 selected runs at 0.2% and 2.2%. A verdict compares figures of one run, so
the offset between runs stays out of it. Instructions differ by a median of 0.000%; a chain's count
follows its class, which each run picks by its cycles.

## Sweeps that stop

`bench-wasm` writes its record after each run of a bench in a build, marked `"unfinished": true`, by
renaming a complete file over the old one. `report` and `stamp-wasm` refuse such a record. The sweep
drops the mark at its end, once every cell whose key differed holds figures of one of its runs. A
run that its bench or its browser stops takes up to three attempts, `ATTEMPTS`, in `bench` as in
`bench-wasm`. Past them `bench` stops with the error; the sweep of `bench-wasm` hands out no more
runs, lets the running ones end and record their figures, and stops with the error.
`bench-wasm --changed` then measures what remains: a kept figure whose key differs holds no code
until a run measures it again, and a row whose rounds a stop cut short starts the first round. A
`bench-wasm` without `--changed` measures every cell again.

Each run of a bench runs in a process of its own, which reads its module from the standard input and
which the kernel kills when the sweep dies. A browser whose driver died waits for requests that
never come; the next driver kills it and removes its profile.

## Two streams

`--cpu 8,9 --browser-cpu 10,11` measures on two streams, CPUs 8 and 10 and CPUs 9 and 11, which take
the runs of every build from one queue. Each run is one bench of one build, and the setup table
names its CPU and load. A verdict compares figures of one run, so of one stream.

A sweep of the 2228 cells of biguint measured for 885 s on two streams, at load 22 to 25. Two sweeps
on one stream just before it measured for 2688 s, as the load rose from 25 to 41, and for 1672 s,
as it fell from 40 to 27. The figures of the two streams stand at a median distance of 0.34% and a
95th percentile of 4.4% from the second sweep on one stream, and of 1.1% and 7.8% from the first;
the two sweeps on one stream stand at 1.1% and 7.5% from each other.
