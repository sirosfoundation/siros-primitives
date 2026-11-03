//! Compares `prime-field` with other prime-field crates on `a + b` and `a − b`, in variable and
//! constant time, over fields of 31 to 521 bits. `prime-field-bench-report` runs every bench of
//! the crate.

mod suite;

use std::process::ExitCode;

/// The kinds of operation that this bench measures.
const KINDS: &[suite::Kind] = &[suite::Kind::Add, suite::Kind::Sub];

fn main() -> ExitCode {
    bench_compare::main::<suite::PrimeField>("sums")
}
