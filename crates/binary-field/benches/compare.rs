//! Compares `binary-field` with other Rust binary-field crates: add, mul, square, invert and sqrt
//! over fields from GF(2⁸) to GF(2⁵⁷¹), in instructions per operation. `bench-compare` runs the
//! comparison and `binary-field-bench-report` writes `BENCHMARKS.md` from its lines.

mod suite;

use std::process::ExitCode;

fn main() -> ExitCode {
    bench_compare::main::<suite::BinaryField>("compare")
}
