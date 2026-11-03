//! Compares `binary-field` with other binary-field crates on `a²`, in variable and constant
//! time, over fields from GF(2⁸) to GF(2⁵⁷¹). `binary-field-bench-report` runs every bench of the
//! crate.

mod suite;

use std::process::ExitCode;

/// The kind of operation that this bench measures.
const KIND: suite::Kind = suite::Kind::Square;

fn main() -> ExitCode {
    bench_compare::main::<suite::BinaryField>("square")
}
