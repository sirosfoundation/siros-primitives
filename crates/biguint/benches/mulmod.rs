//! Compares `biguint` with other big-integer crates on `a × b mod m`, in variable and constant
//! time, at 64 to 4096 bits. `biguint-bench-report` runs every bench of the crate.

mod suite;

use std::process::ExitCode;

/// The computation that this bench measures.
const BASE: suite::Op = suite::Op::MulMod;

/// The widths that this bench measures.
const WIDTHS: &[u32] = &suite::WIDTHS;

fn main() -> ExitCode {
    bench_compare::main::<suite::Biguint>("mulmod")
}
