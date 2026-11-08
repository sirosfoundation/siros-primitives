# bench-compare

What a bench target links to measure the operations of several crates side by side with the CPU's
cycle counter. Runs on Linux with access to user-mode `perf_event` counters, natively and in Chrome
and Firefox as WebAssembly. [`bench-tools`](../bench-tools) builds and runs the benches, and writes
the comparison tables of a `BENCHMARKS.md`.

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
- `run_batch` and `run_chain`: the measured loops, for other tools such as `ct-check`.

## Usage

A bench declares a `Suite` and the operations that it measures; `benches/calibration.rs` is a
complete bench of two operations.

## Protocol

- A native figure is (fewest cycles of 2n operations − fewest cycles of n) / n over 31
  repetitions, after nanoBench (Abel and Reineke, ISPASS 2020). A busy sibling hardware thread
  doubled a one-`add` chain step: the fewest of each length is the run that met an idle sibling.
  [`bench-tools`](../bench-tools) gives the WebAssembly figure.
- A case whose long runs count no more cycles than its short runs, or whose chain step counts under
  0.75 cycles, runs up to three more rounds of 31 repetitions. Each round runs every case again, in
  the order of the first round, and keeps the counts of that case alone: its repetitions meet the
  load over as long a span as the first round's, each after the same cases.
- Latency runs the chain `x = op(x, y)` with `x` held in registers between steps. Every case first
  matches the accepted results. The bench pins itself to CPU 15.
- A sweep stops on multiplexed counters, on long runs that count no more cycles than short runs,
  on a chain step under 0.75 cycles, on a WebAssembly difference under 50 times the noise of a
  count after three doublings of the run, and on a `[profile.bench]` without `lto = "fat"`.
- Bold ties the row's fewest. `*` deviates from the row's definition and is never bold. A crate
  without a variable-time function shows its constant-time one in the variable-time table.
- `report` prints every odd cell to the terminal: a figure more than 10% above the crate's figure
  on a row of more bits.
