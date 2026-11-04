//! Compares `prime-field` with other prime-field crates on `a / k` for a nonzero integer `k` of 64
//! bits and of one bit less than `p`, in variable and constant time, over fields of 31 to 521
//! bits. `prime-field-bench-report` runs every bench of the crate.

mod suite;

use std::process::ExitCode;

/// The kinds of operation that this bench measures.
const KINDS: &[suite::Kind] = &[suite::Kind::DivU64, suite::Kind::DivUint];

fn main() -> ExitCode {
    bench_compare::main::<suite::PrimeField>("divisions")
}
