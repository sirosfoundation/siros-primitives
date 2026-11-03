#![doc = include_str!("../README.md")]

use core::fmt;
use std::process::ExitCode;

use bench_compare::{Layout, Measured, NATIVE, Operation, Paths, sentence};
use serde::{Deserialize, Serialize};

/// A row: a field, by name, and the bit length of its modulus.
#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
struct Field {
    field: String,
    bits: u32,
}

impl fmt::Display for Field {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.field)
    }
}

const MEASURED: Measured = Measured {
    package: "prime-field",
    benches: &["sums", "products"],
    sources: "crates/prime-field/src",
    pathspecs: &[
        "Cargo.toml",
        "Cargo.lock",
        "rust-toolchain.toml",
        "crates/biguint/src",
        "crates/biguint/Cargo.toml",
        "crates/prime-field",
        "crates/bench-compare",
        ":(exclude)crates/prime-field/BENCHMARKS.md",
        ":(exclude)crates/prime-field/benchmarks",
    ],
    versions: &[
        "ark-ff",
        "crypto-bigint",
        "k256",
        "p256",
        "halo2curves",
        "p3-field",
        "winter-math",
        "runtime-algebra",
        "compile-algebra",
    ],
    features: &[],
    // It strips the lookup tables of `biguint`.
    forbidden: &["not-precomputed"],
    builds: &[NATIVE],
};

const LAYOUT: Layout<'static, Field> = Layout {
    subject: "prime-field",
    tool: "prime-field-bench-report",
    // Variable time is the default and carries no label: its tables come first.
    operations: &[
        Operation::new("add_vartime", "add", "a + b"),
        Operation::new("sub_vartime", "sub", "a − b"),
        Operation::new("mul_vartime", "mul", "a × b"),
        Operation::new("square_vartime", "square", "a²"),
        Operation::new("invert_vartime", "invert", "a⁻¹"),
        Operation::new("sqrt_vartime", "sqrt", "√a"),
        Operation::constant_time("add_ct", "add", "a + b"),
        Operation::constant_time("sub_ct", "sub", "a − b"),
        Operation::constant_time("mul_ct", "mul", "a × b"),
        Operation::constant_time("square_ct", "square", "a²"),
        Operation::constant_time("invert_ct", "invert", "a⁻¹"),
        Operation::constant_time("sqrt_ct", "sqrt", "√a"),
    ],
    row_header: "Field",
    rows_phrase: |fields| format!("over {}", sentence(fields)),
    bits: |field| field.bits,
    method: &[
        "Operands: canonical residues, the same for every crate.",
        "Every result matches `num-bigint` before it is measured.",
        "`prime-field` runs `Fp` in the tables without a label and `ConstantTime` in the \
         constant-time tables.",
    ],
};

fn main() -> ExitCode {
    bench_compare::execute(
        "prime-field-bench-report",
        &Paths::new(env!("CARGO_MANIFEST_DIR")),
        &MEASURED,
        &LAYOUT,
    )
}
