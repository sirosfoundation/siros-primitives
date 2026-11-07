# prime-field-bench-report

Measures `prime-field` against other prime-field crates with the CPU's performance counters and
writes `crates/prime-field/BENCHMARKS.md`.

```sh
cargo run -p prime-field-bench-report -- bench [--cpu <cpu>] [--machine <description>]
cargo run -p prime-field-bench-report -- bench-wasm --chrome <executable> --firefox <executable> \
  [--changed [--dry-run]] [--runs <runs>] [--target-dir <directory>] [--cpu <cpu>] \
  [--browser-cpu <cpu>] [--machine <description>]
cargo run -p prime-field-bench-report -- stamp-wasm --chrome <executable> [--target-dir <directory>]
cargo run -p prime-field-bench-report -- modules --out <directory> [--target-dir <directory>]
cargo run -p prime-field-bench-report -- report
```

`bench` runs the benches of the crate, one per group of operations, on one CPU (CPU 15 by default)
with the flags of [`bench-compare`](../../bench-compare), checks every case against an
independent oracle, and records the counts in `benchmarks/record.json` of the crate. `report`
writes the document from the record and prints every cell more than 10% above the same crate's
cell on a field of more bits in the same table. `bench` needs Linux with `kernel.perf_event_paranoid` at 2 or below.

`bench-wasm` builds the benches for `wasm32-unknown-unknown`, with and without wide arithmetic,
runs each build in Chrome and in Firefox, and records the counts in `benchmarks/record-wasm.json`;
`report` adds their tables. `--changed` measures the cells whose key differs from the record, with
the figures they are compared with; `stamp-wasm` writes the keys of the tree into a record of the same
sources; `modules` writes the modules of a `bench-browser decide`.
[`bench-compare`](../../bench-compare) describes the browser driver and these keys.
