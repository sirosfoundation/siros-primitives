//! Compares `prime-field` with other Rust prime-field crates: add, sub, mul, square, invert,
//! invert_vartime and sqrt over fields from 31 to 521 bits, in instructions per operation.
//! `bench-compare` runs the comparison and `prime-field-bench-report` writes `BENCHMARKS.md` from
//! its lines.

mod suite;

use std::process::ExitCode;

fn main() -> ExitCode {
    bench_compare::main::<suite::PrimeField>("compare")
}
