# prime-field-bench-report

Measures `prime-field` against other prime-field crates with the CPU's performance counters and
writes `crates/prime-field/BENCHMARKS.md`.

```sh
cargo run -p prime-field-bench-report -- bench [--cpu <cpu>] [--machine <description>]
cargo run -p prime-field-bench-report -- report
```

`bench` runs the `compare` bench on one CPU (CPU 15 by default) with the flags of
[`bench-compare`](../../bench-compare), checks every case against an independent oracle, and
records the counts in `benchmarks/record.json` of the crate. `report` writes the document from the
record. `bench` needs Linux with `kernel.perf_event_paranoid` at 2 or below.
