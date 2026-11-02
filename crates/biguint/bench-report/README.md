# biguint-bench-report

Measures `biguint` against other big-integer crates with the CPU's performance counters and writes
`crates/biguint/BENCHMARKS.md`.

## Usage

```sh
cargo run -p biguint-bench-report -- bench [--cpu <cpu>] [--machine <description>]
cargo run -p biguint-bench-report -- report
```

`bench` builds the `compare` bench of `biguint` with the flags of
[`bench-compare`](https://github.com/sirosfoundation/siros-primitives/tree/main/crates/bench-compare)
and runs it on one CPU, CPU 15 unless `--cpu` names another. It records the counts, the commit,
the compiler, the CPU and the load in `benchmarks/record.json` of the crate. `--machine` adds a
description of the machine.

`report` reads that record, writes the document, and prints to the terminal every cell where a
crate's figure exceeds its figure at a greater width in the same table by more than 10%.

`bench` needs Linux with `kernel.perf_event_paranoid` at 2 or below.
