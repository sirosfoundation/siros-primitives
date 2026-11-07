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
  - `bench-wasm --chrome <executable> --firefox <executable> [--changed [--dry-run]] [--runs <runs>]
    [--target-dir <directory>] [--cpu <cpu>] [--browser-cpu <cpu>] [--machine <description>]` builds
    the builds of `WASM` and runs them in the two browsers, then records
    `benchmarks/record-wasm.json`;
  - `stamp-wasm --chrome <executable> [--target-dir <directory>]` writes the keys of the tree into
    the WebAssembly record;
  - `modules --out <directory> [--target-dir <directory>]` builds the modules of a decision;
  - `report` writes `BENCHMARKS.md` from the records.
- `bench-browser`: runs one WebAssembly bench in one browser and prints its lines. `decide` compares
  two directories of modules; `list`, `diff` and `edges` read modules.
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
  [--threads measuring | process] [--only <op/key/crate,...>] [--runs <runs>]
```

## Protocol

- A figure is (fewest cycles of 2n operations − fewest cycles of n) / n over 31 repetitions,
  after nanoBench (Abel and Reineke, ISPASS 2020). A busy sibling hardware thread doubled a
  one-`add` chain step: the fewest of each length is the run that met an idle sibling.
- Latency runs the chain `x = op(x, y)` with `x` held in registers between steps. Every case first
  matches the accepted results. The bench pins itself to CPU 15.
- A sweep stops on multiplexed counters, on long runs that count no more cycles than short runs,
  on a chain step under 0.75 cycles, and on a `[profile.bench]` without `lto = "fat"`.
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
- Each bench first counts 1,000 empty regions: a case whose difference counts under 50 times the
  noise of the fixed cost of a count stops the sweep. That noise is the difference of the fewest
  counts of two sets of 31 empty counts, the largest of four pairs.
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
cargo run --release -p bench-compare --bin bench-browser -- decide \
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
best competitor per part; a module with none of them stays out. Fresh figures replace old ones, with
no minimum across runs. A round repeats while a kept figure leads a re-measured row, 12 times at most;
past that the tool stops with an error. `--dry-run` prints the cells of the first round of each
build. A `bench-wasm` without `--changed` measures every cell.

`report` refuses a record whose verdict compares figures of different runs: per build, operation, row
and part, one of the tied fewest figures and one of the best competitor's must come from the subject's
run. Each cell records its run. The setup table of `BENCHMARKS.md` gives each run's commit, cells and
load.

The hash in `src/identity.rs` is a Merkle hash over the reachable functions and data. It reads the
relocations that `KEEP_RELOCATIONS` (`-C link-arg=--emit-relocs`) keeps in each module, so each
address resolves to its symbol: layout leaves the hash, and the symbol's bytes enter it. The flag
adds custom sections and writes the zeros of `.bss` as a data segment. The two `num-bigint` crates of
the prime-field plain builds swap places in those modules, because `RUSTFLAGS` enters the file names
of the rlibs; every function body stays the same. An opcode or a relocation that the reader does not
know stops the tool with its offset. `bench-browser diff` names the functions that differ between two
modules, and `bench-browser edges` prints the call edges that the reader sees, to compare with
`wasm-tools print`.

`stamp-wasm` writes the keys of a record measured at an earlier commit, without measuring. It refuses
a record that ran dirty, uncommitted changes in the measured sources, and sources that differ from
the record's `head`. The files of `bench-compare` stay out of that check: run `bench-browser diff`
between the modules of both commits first.

A selected cell takes the fewest figures of `SELECTED_RUNS`, 3, runs of its bench: repetitions within
one run meet one burst of load, where a full run spreads them over minutes. Cycles of 48 cells across
16 operations in two crates, against a full run back to back, at load 25 to 60:

| Against a full run | Figures | Median distance | 95th percentile | Median signed difference |
| ------------------ | ------- | --------------- | --------------- | ------------------------ |
| A second full run  | 176     | 1.9%            | 22.9%           | +0.1%                    |
| 1 selected run     | 192     | 2.4%            | 29.6%           | +0.2%                    |
| 3 selected runs    | 192     | 1.6%            | 18.4%           | -0.9%                    |
| 7 selected runs    | 96      | 4.7%            | 36.8%           | -3.0%                    |

Instruction counts differ by a median of 0.000% on each line. More runs read lower, as a minimum over
more samples does. A verdict compares figures of one run, so this offset between runs stays out of it.
