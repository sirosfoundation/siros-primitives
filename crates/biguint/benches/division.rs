//! Compares `biguint` with other big-integer crates on `a / b` and `a mod b`, in variable and
//! constant time, at 64 to 4096 bits. `biguint-bench-report` runs every bench of the crate.

mod suite;

use std::process::ExitCode;

/// The computations that this bench measures. They share one binary: a crate's quotient and
/// remainder call one division, which then compiles as it did in a single bench, so its measured
/// loops keep their code.
const BASES: &[suite::Op] = &[suite::Op::Div, suite::Op::Mod];

fn main() -> ExitCode {
    bench_compare::main::<suite::Biguint>("division")
}
