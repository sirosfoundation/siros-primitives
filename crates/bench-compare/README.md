# bench-compare

The machinery behind the `BENCHMARKS.md` of `biguint` and `prime-field`: a comparison of one
crate's operations with other crates, in instructions per operation that Callgrind counts.

A library's `compare` bench declares a `Suite`: its operations, its rows (widths or fields), the
operand sets of each row with their accepted results, and one adapter per crate that offers the
crate's function for each operation or the reason it has none. `bench_compare::main` runs the
bench: it checks every case against the accepted results, counts each one under Callgrind in its
own process, and prints every cell as one JSON line.

A library's report tool calls `bench_compare::execute` with the files it measures and a `Layout` of
words for the document. `bench` builds the bench with
`-C target-cpu=native -C target-feature=-sse4a` (Valgrind runs no `extrq`) in `target/benchmarks`,
runs it and records the counts, the commit, the compiler and the machine in the run record.
`report` reads the record and writes the document with one SVG chart per operation.

`bench` needs Valgrind in `PATH`. The counts are exact at any machine load.
