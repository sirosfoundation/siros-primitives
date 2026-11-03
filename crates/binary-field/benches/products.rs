//! Compares `binary-field` with other binary-field crates on `a × b`, `a²`, `a⁻¹` and `√a`, in
//! variable and constant time, over fields from GF(2⁸) to GF(2⁵⁷¹). `binary-field-bench-report`
//! runs every bench of the crate.

mod suite;

use std::process::ExitCode;

/// The kinds of operation that this bench measures: the product and the powers that crates compose
/// from it. They share one binary: a crate function that two of them call compiles as it did in a
/// single bench, so its measured loops keep their code.
const KINDS: &[suite::Kind] = &[
    suite::Kind::Mul,
    suite::Kind::Square,
    suite::Kind::Invert,
    suite::Kind::Sqrt,
];

fn main() -> ExitCode {
    bench_compare::main::<suite::BinaryField>("products")
}
