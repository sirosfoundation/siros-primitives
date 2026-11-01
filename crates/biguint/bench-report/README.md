# biguint-bench-report

Runs the benchmarks that compare `biguint` with other big-integer crates and writes
`crates/biguint/BENCHMARKS.md` with one SVG chart per operation.

## Usage

```sh
cargo run -p biguint-bench-report -- bench [--machine <description>]
cargo run -p biguint-bench-report -- report [--provisional <reason>]
```

`bench` runs the `compare` bench of `biguint` twice with criterion's quick mode: for the default
target, and with `-C target-cpu=native`. Each build has its own target directory under
`target/benchmarks`. It records the commit, the compiler, the machine and the cells of the
comparison in `target/benchmarks/run.json`. `--machine` adds a description of the machine.

`report` reads that record and criterion's `estimates.json` files and writes the document and the
charts. `--provisional` marks the times as upper bounds and gives the reason, for example the load
of the machine; the record keeps it for later reports.

Timings need a quiet machine. On a busy one, pin the run to an idle CPU (for example with
`taskset -c <cpu>`) and pass `--provisional`.
