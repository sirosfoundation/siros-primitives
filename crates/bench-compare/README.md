# bench-compare

The measuring library behind the `BENCHMARKS.md` of `biguint`, `prime-field` and `binary-field`:
it compares one crate's operations with other crates in user-mode cycles per operation, counted
by the CPU's performance counters. It needs Linux with `kernel.perf_event_paranoid` at 2 or below.

## Usage

A library's `compare` bench declares a `Suite`: its operations, its rows (widths or fields), a
generator of operand sets per row, a fixed checking batch with the accepted results, and one
adapter per crate that offers the crate's function for each operation or the reason it has none.
`bench_compare::main` runs the bench:

- it checks every case against the accepted results;
- it pins itself to one CPU (`--cpu`, CPU 15 by default);
- it prints every cell as one JSON line, then a line with the CPU, the 1-minute load average
  before and after the sweep, and the busy share of the CPU's SMT siblings.

A case whose result has the type of its first operand and whose other operands are `Copy` can
offer a chain form, which measures latency as the chain `x = op(x, y)`.
`Cells::deviating` declares how the next case's function deviates from the row's definition, such
as a skipped final reduction; the document marks a deviating figure below the measured crate's
with `*` and never counts it as the fewest of its row.

A library's report tool calls `bench_compare::execute` with the files it measures, the features
its builds enable and forbid, its builds and a `Layout` of words for the document.
`bench [--cpu <cpu>] [--machine <description>]` builds the bench of each build under `target`,
runs it and records the counts, the commit, the compiler, the CPU, the load and the target
features. A bench without an enabled feature, a unit with a forbidden one, or a build without AVX2
stops it. The native build takes `-C target-cpu=native -C codegen-units=1`. `report` writes the
document with one SVG chart per operation and build.

## Protocol

Each figure is the median over 31 repetitions of a run of 2n operations less a run of n,
divided by n. The protocol follows nanoBench (Abel and Reineke, ISPASS 2020) and ankerl::nanobench.
A sweep stops when the kernel shared the counters with other groups. The cycles of a sweep under
load compare within that sweep only.
