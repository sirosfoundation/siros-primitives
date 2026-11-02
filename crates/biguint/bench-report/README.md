# biguint-bench-report

Measures `biguint` against other big-integer crates with the CPU's performance counters and writes
`crates/biguint/BENCHMARKS.md` with one SVG chart per operation.

## Usage

```sh
cargo run -p biguint-bench-report -- bench [--cpu <cpu>] [--machine <description>]
cargo run -p biguint-bench-report -- report
```

`bench` builds the `compare` bench of `biguint` with the flags of
[`bench-compare`](https://github.com/sirosfoundation/siros-primitives/tree/main/crates/bench-compare)
and runs it on one CPU, CPU 15 unless `--cpu` names another. It records the counts, the commit,
the compiler, the CPU and the load in `target/benchmarks/biguint.json`. `--machine` adds a
description of the machine.

`report` reads that record and writes the document and the charts.

`bench` needs Linux with `kernel.perf_event_paranoid` at 2 or below.
