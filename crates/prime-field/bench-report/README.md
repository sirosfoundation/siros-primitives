# prime-field-bench-report

Measures `prime-field` against other prime-field crates with the CPU's performance counters and
writes `crates/prime-field/BENCHMARKS.md`.

## Usage

```sh
cargo run -p prime-field-bench-report -- bench [--cpu <cpu>] [--machine <description>]
cargo run -p prime-field-bench-report -- report
```

`bench` builds the `compare` bench of `prime-field` with the flags of
[`bench-compare`](https://github.com/sirosfoundation/siros-primitives/tree/main/crates/bench-compare)
and runs it on one CPU, CPU 15 unless `--cpu` names another: every case is checked against an
independent oracle, then measured. The record goes to `benchmarks/record.json` of the crate;
`--machine` adds a description of the machine. `report` reads it, writes the document, and prints
to the terminal every cell where a crate's figure exceeds its figure on a field of more bits in the
same table by more than 10%.

`bench` needs Linux with `kernel.perf_event_paranoid` at 2 or below.
