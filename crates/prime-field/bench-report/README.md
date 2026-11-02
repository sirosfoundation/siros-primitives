# prime-field-bench-report

Counts the instructions of the benchmarks that compare `prime-field` with other prime-field
crates and writes `crates/prime-field/BENCHMARKS.md` with one SVG chart per operation.

## Usage

```sh
cargo run -p prime-field-bench-report -- bench [--machine <description>]
cargo run -p prime-field-bench-report -- report
```

`bench` builds the `compare` bench of `prime-field` with
`-C target-cpu=native -C target-feature=-sse4a,-vpclmulqdq` (Valgrind runs no `extrq` and no
256-bit `vpclmulqdq`) in `target/benchmarks` and runs it: every case is checked against an
independent oracle and counted under Valgrind's Callgrind in its own process. The record goes to
`target/benchmarks/prime-field.json`. `report` reads it and writes the document and the charts.

`bench` needs Valgrind in `PATH`. The counts are exact at any machine load.
