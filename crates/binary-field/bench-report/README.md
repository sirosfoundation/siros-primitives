# binary-field-bench-report

Counts the instructions of the benchmarks that compare `binary-field` with other binary-field
crates and writes `crates/binary-field/BENCHMARKS.md` with one SVG chart per operation.

## Usage

```sh
cargo run -p binary-field-bench-report -- bench [--machine <description>]
cargo run -p binary-field-bench-report -- report
```

`bench` builds the `compare` bench of `binary-field` with
`-C target-cpu=native -C target-feature=-sse4a,-vpclmulqdq` (Valgrind runs no `extrq` and no
256-bit `vpclmulqdq`) in `target/benchmarks` and runs it: every case is checked against a
bit-serial reference and counted under Valgrind's Callgrind in its own process. The record goes
to `target/benchmarks/binary-field.json`; `--machine` adds a description of the machine. `report`
reads the record and writes the document and the charts.

`bench` needs Valgrind in `PATH`. The counts are exact at any machine load.
