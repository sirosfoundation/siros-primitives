//! Compares `prime-field` with other prime-field crates on `a²`, in variable and constant
//! time, over fields of 31 to 521 bits. `prime-field-bench-report` runs every bench of the crate.

mod suite;

use std::process::ExitCode;

/// The kind of operation that this bench measures.
const KIND: suite::Kind = suite::Kind::Square;

fn main() -> ExitCode {
    bench_compare::main::<suite::PrimeField>("square")
}
