# biguint-bench-report

Measures `biguint` against other big-integer crates with the CPU's performance counters and
writes `crates/biguint/BENCHMARKS.md`.

```sh
cargo run -p biguint-bench-report -- bench [--cpu <cpu>] [--machine <description>]
cargo run -p biguint-bench-report -- report
```

`bench` runs the benches of the crate, one per group of computations, on one CPU (CPU 15 by
default) with the flags of [`bench-compare`](../../bench-compare) and records the counts in
`benchmarks/record.json` of the crate. `report` writes the document from the record and prints
every cell more than 10% above the same crate's cell at a greater width in the same table. `bench` needs Linux with
`kernel.perf_event_paranoid` at 2 or below.
