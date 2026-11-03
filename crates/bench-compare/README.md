# bench-compare

Measures the operations of several crates side by side with the CPU's cycle counter, and writes
the comparison tables of a `BENCHMARKS.md`. Runs on Linux with access to user-mode `perf_event`
counters.

## Features

- `Suite`: what a crate's `compare` bench measures: its operations, its rows, the operand sets with
  their accepted results, and one adapter per compared crate.
- `Row::lib` offers one crate's cells:
  - `Cells::api` measures throughput, and `Cells::chain` throughput and the latency of
    `x = op(x, y)`;
  - `api_with` and `chain_with` add a setup shared by the inputs and the result reader;
  - `absent` gives the reason a crate has no function;
  - `deviating` marks a function that does less than the row defines.
- `bench_compare::main`: the bench binary. It checks every case against the accepted results, then
  counts cycles on a pinned CPU.
- `bench_compare::execute`: the report tool. `bench [--cpu <cpu>] [--machine <description>]`
  builds, runs and records `benchmarks/record.json`; `report` writes `BENCHMARKS.md`.
- `run_batch` and `run_chain`: the measured loops, for other tools such as `ct-check`.

## Usage

A crate's `compare` bench declares a `Suite`:

```rust,no_run
use bench_compare::{BATCH, Limbs, Rng, Row, Sample, Suite, Visitor, trim};

struct Add;

impl Suite for Add {
    type Op = &'static str;
    type Key = u32;
    const OPS: &'static [&'static str] = &["add"];

    fn keys() -> Vec<u32> {
        vec![64]
    }

    fn samples(op: &'static str, bits: &u32) -> Vec<Sample> {
        let mut rng = Rng(1);
        (0..BATCH)
            .map(|_| {
                let operands = Self::operands(op, bits, &mut rng);
                let sum = operands[0][0].wrapping_add(operands[1][0]);
                Sample { operands, accepted: vec![trim(&[sum]).to_vec()] }
            })
            .collect()
    }

    fn operands(_: &'static str, _: &u32, rng: &mut Rng) -> Vec<Limbs> {
        vec![rng.uniform(1), rng.uniform(1)]
    }

    fn offer<V: Visitor<&'static str, u32>>(
        row: Row<&'static str, u32, V>,
    ) -> Row<&'static str, u32, V> {
        row.lib("core", |cells| {
            cells.chain(
                "add",
                "wrapping_add",
                |set| (set[0][0], set[1][0]),
                |&(a, b): &(u64, u64)| a.wrapping_add(b),
                |sum| Some(vec![*sum]),
            )
        })
    }
}

fn main() -> std::process::ExitCode {
    bench_compare::main::<Add>("compare")
}
```

Its report tool runs `cargo run -p <tool> -- bench`, which builds, runs and records the bench in
`benchmarks/record.json`, and `cargo run -p <tool> -- report`, which writes `BENCHMARKS.md`:

```rust,no_run
use bench_compare::{Layout, Measured, NATIVE, Operation, Paths, sentence};

const MEASURED: Measured = Measured {
    package: "biguint",
    bench: "compare",
    sources: "crates/biguint/src",
    pathspecs: &["Cargo.toml", "Cargo.lock", "crates/biguint", "crates/bench-compare"],
    versions: &["ruint"],
    features: &[],
    forbidden: &["not-precomputed"],
    builds: &[NATIVE],
};

const LAYOUT: Layout<'static, u32> = Layout {
    subject: "biguint",
    tool: "biguint-bench-report",
    operations: &[
        Operation::new("add", "add", "a + b"),
        Operation::constant_time("add_ct", "add", "a + b"),
    ],
    row_header: "Bits",
    rows_phrase: |widths| format!("at {} bits", sentence(widths)),
    bits: |bits| *bits,
    method: &["Operands at width N: uniform below 2^N."],
};

fn main() -> std::process::ExitCode {
    bench_compare::execute(
        "biguint-bench-report",
        &Paths::new(env!("CARGO_MANIFEST_DIR")),
        &MEASURED,
        &LAYOUT,
    )
}
```

## Protocol

- A figure is the median over 31 repetitions of (cycles of 2n operations − cycles of n) / n,
  after nanoBench (Abel and Reineke, ISPASS 2020).
- Latency runs the chain `x = op(x, y)` with `x` held in registers between steps. Every case first
  matches the accepted results. The bench pins itself to CPU 15.
- A sweep stops on multiplexed counters, on long runs that count no more cycles than short runs,
  on a chain step under 0.75 cycles, and on a `[profile.bench]` without `lto = "fat"`.
- Bold ties the row's fewest. `*` deviates from the row's definition and is never bold. `°` is a
  constant-time function in a variable-time cell.
- `report` prints every odd cell to the terminal: a figure more than 10% above the crate's figure
  on a row of more bits.
