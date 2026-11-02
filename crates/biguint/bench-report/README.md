# biguint-bench-report

Counts the instructions of the benchmarks that compare `biguint` with other big-integer crates and
writes `crates/biguint/BENCHMARKS.md` with one SVG chart per operation.

## Usage

```sh
cargo run -p biguint-bench-report -- bench [--machine <description>]
cargo run -p biguint-bench-report -- report
```

`bench` builds the `compare` bench of `biguint` with `-C target-cpu=native -C target-feature=-sse4a`
(Valgrind runs no `extrq`) in `target/benchmarks` and runs it. The bench counts the instructions of
every case under Valgrind's Callgrind, one process per case, on all the threads that the run may
use. `bench` records the counts, the commit, the compiler and the machine in
`target/benchmarks/biguint.json`. `--machine` adds a description of the machine.

`report` reads that record and writes the document and the charts.

`bench` needs Valgrind in `PATH`. The counts are exact at any machine load.
