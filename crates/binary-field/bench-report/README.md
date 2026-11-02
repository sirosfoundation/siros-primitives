# binary-field-bench-report

Measures `binary-field` against other binary-field crates with the CPU's performance counters, with
and without carry-less multiplication, and writes `crates/binary-field/BENCHMARKS.md`.

```sh
cargo run -p binary-field-bench-report -- bench [--cpu <cpu>] [--machine <description>]
cargo run -p binary-field-bench-report -- report
```

`bench` builds the `compare` bench with the feature `precomputed` twice: with the flags of
[`bench-compare`](../../bench-compare), and with
`-C target-feature=-pclmulqdq,-vpclmulqdq --cfg polyval_backend="soft"` added. It runs each build
on one CPU (CPU 15 by default), checks every case against a bit-serial reference, and records the
counts in `benchmarks/record.json` of the crate. `report` writes the document from the record.
`bench` needs Linux with `kernel.perf_event_paranoid` at 2 or below; a unit built with
`not-precomputed` stops it.
