//! Compares `biguint` with other Rust big-integer crates: add, mul, div, mod and mulmod at 64 to
//! 4096 bits, in cycles per operation. `bench-compare` runs the comparison and
//! `biguint-bench-report` writes `BENCHMARKS.md` from its lines.

mod suite;

use std::process::ExitCode;

fn main() -> ExitCode {
    bench_compare::main::<suite::Biguint>("compare")
}
