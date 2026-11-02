# binary-field-bench-report

Measures `binary-field` against other binary-field crates with the CPU's performance counters, in
two builds: with carry-less multiplication and without it. It writes
`crates/binary-field/BENCHMARKS.md` with one SVG chart per operation and build.

## Usage

```sh
cargo run -p binary-field-bench-report -- bench [--cpu <cpu>] [--machine <description>]
cargo run -p binary-field-bench-report -- report
```

`bench` builds the `compare` bench of `binary-field` twice: with the flags of
[`bench-compare`](https://github.com/sirosfoundation/siros-primitives/tree/main/crates/bench-compare),
and with `-C target-feature=-pclmulqdq,-vpclmulqdq --cfg polyval_backend="soft"` added: the
second flag selects the software backend of `polyval`, which otherwise selects PCLMULQDQ at run
time. It runs each build on one CPU, CPU 15 unless `--cpu` names another: every case is
checked against a bit-serial reference, then measured. The record goes to
`target/benchmarks/binary-field.json`; `--machine` adds a description of the machine. `report`
reads the record and writes the document and the charts.

`bench` needs Linux with `kernel.perf_event_paranoid` at 2 or below.
