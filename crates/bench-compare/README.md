# bench-compare

The machinery behind the `BENCHMARKS.md` of `biguint`, `prime-field` and `binary-field`: a
comparison of one crate's operations with other crates, in user-mode cycles and instructions per
operation that the CPU's performance counters count.

A library's `compare` bench declares a `Suite`: its operations, its rows (widths or fields), a
generator of operand sets per row, a fixed checking batch with the accepted results, and one
adapter per crate that offers the crate's function for each operation or the reason it has none.
A case whose result has the type of its first operand can offer a chain form, which measures
latency as the chain `x = op(x, y)`. Between two operations, an empty inline-assembly block holds
the result, which keeps the compiler from regrouping the chain into a tree. The chain runs twice,
with the result in general-purpose registers and in vector registers; the block can only add moves
between the two, so the latency is the faster chain, and the record names its class. A result of
more than 64 bytes goes through memory, in a single chain.

`bench_compare::main` runs the bench. It checks every case against the accepted results, pins
itself to one CPU (`--cpu`, CPU 15 by default), and counts every case with one perf_event group of
user-mode cycles, instructions and branch misses. It prints every cell as one JSON line, then a
line with the CPU, the 1-minute load average before and after the sweep, and the busy share of the
CPU's SMT siblings.

The protocol follows nanoBench (Abel and Reineke, ISPASS 2020) and the perf_event groups of
ankerl::nanobench:

- n starts at 16 operations and doubles until a run of n, counted after an uncounted run of as
  many, takes 2²⁰ cycles; n stops earlier where the operand sets of 2n operations fill 256 KiB;
- every run draws fresh operand sets, outside the counted region, so the branch predictor never
  replays a sequence;
- a repetition runs n operations uncounted, then counts a run of n and a run of 2n;
- a figure per operation is the median over 31 repetitions of the long run less the short run,
  divided by n, which cancels the fixed cost of a run;
- repetition r of every case runs before repetition r + 1 of any case, so a drift of the machine
  reaches all cases alike;
- a run during which the kernel shared the counters with other groups stops the sweep, and so do
  a case whose long runs count no more cycles than its short runs and a chain step under one
  cycle.

A library's report tool calls `bench_compare::execute` with the files it measures, the features
its builds enable and forbid, its builds and a `Layout` of words for the document.
`bench [--cpu <cpu>] [--machine <description>]` builds the bench of each build in its own directory
under `target`, runs it and records the counts, the commit, the compiler, the CPU, the load and
the target features that `rustc --print cfg` reports under the build's flags. It reads the
features of every compiled unit from cargo's JSON messages: a bench without an enabled feature, a
unit with a forbidden one, or a build without AVX2 stops it. The native build takes
`-C target-cpu=native -C codegen-units=1`; one codegen unit makes the code of each crate
independent of the rest of the bench. `report` reads the record and writes the document with one
SVG chart per operation and build.

Measuring needs Linux with `kernel.perf_event_paranoid` at 2 or below. The cycles of a sweep under
load compare within that sweep only.
