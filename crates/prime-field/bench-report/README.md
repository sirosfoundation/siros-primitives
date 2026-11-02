# prime-field-bench-report

Measures `prime-field` against other prime-field crates with the CPU's performance counters and
writes `crates/prime-field/BENCHMARKS.md` with one SVG chart per operation.

## Usage

```sh
cargo run -p prime-field-bench-report -- bench [--cpu <cpu>] [--machine <description>]
cargo run -p prime-field-bench-report -- report
```

`bench` builds the `compare` bench of `prime-field` with the flags of
[`bench-compare`](https://github.com/sirosfoundation/siros-primitives/tree/main/crates/bench-compare)
and runs it on one CPU, CPU 15 unless `--cpu` names another: every case is checked against an
independent oracle, then measured. The record goes to `benchmarks/record.json` of the crate;
`--machine` adds a description of the machine. `report` reads it and writes the document and the
charts.

`bench` needs Linux with `kernel.perf_event_paranoid` at 2 or below.
