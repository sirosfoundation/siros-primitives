# bench-compare

Measures the operations of several crates side by side with the CPU's cycle counter, and writes
the comparison tables of a `BENCHMARKS.md`. Runs on Linux with access to user-mode `perf_event`
counters, natively and in Chrome and Firefox as WebAssembly.

## Features

- `Suite`: what a bench measures: its operations, its rows, the operand sets with their accepted
  results, and one adapter per compared crate. A comparison splits over several bench binaries, one
  per group of operations, that build in parallel. Operations whose cases call one crate function
  belong in one group: LLVM decides the inlining of that function from all its callers, and a
  split moves the code of the measured loops.
- `Selected`: the operation of a case as a type, with whether the bench measures it as a constant.
  A case of an operation that the bench leaves out compiles to nothing.
- `Row::lib` offers one crate's cells:
  - `Cells::api` measures throughput, and `Cells::chain` throughput and the latency of
    `x = op(x, y)`;
  - `api_with` and `chain_with` add a setup shared by the inputs and the result reader;
  - `absent` gives the reason a crate has no function;
  - `deviating` marks a function that does less than the row defines.
- `bench_compare::main`: the bench binary. It checks every case against the accepted results, then
  counts cycles on a pinned CPU. Built for `wasm32-unknown-unknown`, it is a module that a browser
  runs and that sends its counts and its lines through its imports.
- `bench_compare::execute`: the report tool.
  - `bench [--cpu <cpu>] [--machine <description>]` builds, runs and records
    `benchmarks/record.json`;
  - `bench-wasm --chrome <executable> --firefox <executable> [--cpu <cpu>] [--browser-cpu <cpu>]
    [--machine <description>]` builds the builds of `WASM` and runs them in the two browsers,
    then records `benchmarks/record-wasm.json`;
  - `report` writes `BENCHMARKS.md` from the records.
- `bench-browser`: runs one WebAssembly bench in one browser and prints its lines.
- `run_batch` and `run_chain`: the measured loops, for other tools such as `ct-check`.

## Usage

A bench declares a `Suite` and the operations that it measures; `examples/calibration.rs` is a
complete bench of two operations. Its report tool declares what it measures and how the document
names it:

```rust,no_run
use bench_compare::{Layout, Measured, NATIVE, Operation, Paths, sentence};

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
    bench_compare::execute(
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

One bench in one browser, such as the example:

```sh
cargo build --release --target wasm32-unknown-unknown -p bench-compare --example calibration
cargo run --release -p bench-compare --bin bench-browser -- \
  target/wasm32-unknown-unknown/release/examples/calibration.wasm --firefox <firefox> \
  --cpu 12 --browser-cpu 14 [--build baseline | wide] [--tiering optimizing | default] \
  [--threads measuring | process]
```

## Protocol

- A figure is (fewest cycles of 2n operations − fewest cycles of n) / n over 31 repetitions,
  after nanoBench (Abel and Reineke, ISPASS 2020). A busy sibling hardware thread doubled a
  one-`add` chain step: the fewest of each length is the run that met an idle sibling.
- Latency runs the chain `x = op(x, y)` with `x` held in registers between steps. Every case first
  matches the accepted results. The bench pins itself to CPU 15.
- A sweep stops on multiplexed counters, natively on long runs that count no more cycles than short
  runs, on a chain step under 0.75 cycles, and on a `[profile.bench]` without `lto = "fat"`.
- Bold ties the row's fewest. `*` deviates from the row's definition and is never bold. A crate
  without a variable-time function shows its constant-time one in the variable-time table.
- `report` prints every odd cell to the terminal: a figure more than 10% above the crate's figure
  on a row of more bits.

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
  every later count leaves out. The setup table gives its noise, the difference of the fewest counts
  of two sets of 31 empty counts, the largest of four pairs.
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
