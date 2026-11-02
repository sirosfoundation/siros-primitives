# binary-field-bench-report

Measures `binary-field` against other binary-field crates with the CPU's performance counters, in
two builds: with carry-less multiplication and without it. It writes
`crates/binary-field/BENCHMARKS.md`.

## Usage

```sh
cargo run -p binary-field-bench-report -- bench [--cpu <cpu>] [--machine <description>]
cargo run -p binary-field-bench-report -- report
```

`bench` builds the `compare` bench of `binary-field` with its feature `precomputed`, whose full
tables the comparison measures; a unit built with `not-precomputed` stops it. It builds the bench
twice: with the flags of
[`bench-compare`](https://github.com/sirosfoundation/siros-primitives/tree/main/crates/bench-compare),
and with `-C target-feature=-pclmulqdq,-vpclmulqdq --cfg polyval_backend="soft"` added: the
second flag selects the software backend of `polyval`, which otherwise selects PCLMULQDQ at run
time. It runs each build on one CPU, CPU 15 unless `--cpu` names another: every case is
checked against a bit-serial reference, then measured. The record goes to
`benchmarks/record.json` of the crate; `--machine` adds a description of the machine. `report`
reads the record, writes the document, and prints to the terminal every cell where a crate's
figure exceeds its figure on a field of higher degree in the same table by more than 10%.

`bench` needs Linux with `kernel.perf_event_paranoid` at 2 or below.
