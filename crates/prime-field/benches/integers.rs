//! Compares `prime-field` with other prime-field crates on `a × k`, `a + k`, `a − k` for an integer
//! `k` of 64 bits and of the bit length of `p`, and on the element of a `u64`, in variable and
//! constant time, over fields of 31 to 521 bits. `prime-field-bench-report` runs every bench of
//! the crate.

mod suite;

use std::process::ExitCode;

/// The kinds of operation that this bench measures.
const KINDS: &[suite::Kind] = &[
    suite::Kind::MulU64,
    suite::Kind::MulUint,
    suite::Kind::AddU64,
    suite::Kind::AddUint,
    suite::Kind::SubU64,
    suite::Kind::SubUint,
    suite::Kind::FromU64,
];

fn main() -> ExitCode {
    bench_compare::main::<suite::PrimeField>("integers")
}
