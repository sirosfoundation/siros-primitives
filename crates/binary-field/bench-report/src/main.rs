#![doc = include_str!("../README.md")]

use core::fmt;
use std::process::ExitCode;

use bench_compare::{Build, Layout, Measured, NATIVE, Operation, Paths, sentence};
use serde::{Deserialize, Serialize};

/// A row: a field, by name, and the degree of its modulus.
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
    package: "binary-field",
    benches: &["add", "products"],
    sources: "crates/binary-field/src",
    pathspecs: &[
        "Cargo.toml",
        "Cargo.lock",
        "rust-toolchain.toml",
        "crates/biguint/src",
        "crates/biguint/Cargo.toml",
        "crates/clmul/src",
        "crates/clmul/Cargo.toml",
        "crates/binary-field",
        "crates/bench-compare",
        ":(exclude)crates/binary-field/BENCHMARKS.md",
        ":(exclude)crates/binary-field/benchmarks",
    ],
    versions: &[
        "gf256",
        "galois_field_2pm",
        "g2p",
        "reed-solomon-erasure",
        "hekate-math",
        "p3-binary-field",
        "runtime-algebra",
        "compile-algebra",
        "core-algebra",
        "polyval",
    ],
    // The comparison measures the full tables of `precomputed`.
    features: &["precomputed"],
    // It strips every lookup table.
    forbidden: &["not-precomputed"],
    builds: &[
        Build {
            name: "with carry-less multiplication",
            ..NATIVE
        },
        Build {
            name: "without carry-less multiplication",
            slug: "without-clmul",
            // polyval selects its PCLMULQDQ backend at run time unless its backend is "soft".
            rustflags: "-C target-cpu=native -C target-feature=-pclmulqdq,-vpclmulqdq \
                        -C codegen-units=1 -C llvm-args=-align-loops=64 \
                        --cfg polyval_backend=\"soft\"",
        },
    ],
};

const LAYOUT: Layout<'static, Field> = Layout {
    subject: "binary-field",
    tool: "binary-field-bench-report",
    // Variable time is the default and carries no label: its tables come first.
    operations: &[
        Operation::new("add_vartime", "add", "a + b"),
        Operation::new("mul_vartime", "mul", "a × b"),
        Operation::new("square_vartime", "square", "a²"),
        Operation::new("invert_vartime", "invert", "a⁻¹"),
        Operation::new("sqrt_vartime", "sqrt", "√a"),
        Operation::constant_time("add", "add", "a + b"),
        Operation::constant_time("mul", "mul", "a × b"),
        Operation::constant_time("square", "square", "a²"),
        Operation::constant_time("invert", "invert", "a⁻¹"),
        Operation::constant_time("sqrt", "sqrt", "√a"),
    ],
    row_header: "Field",
    rows_phrase: |fields| format!("over {}", sentence(fields)),
    bits: |field| field.bits,
    method: &[
        "Every field in the polynomial basis of its modulus: GF(2⁸) of AES, GF(2¹²⁸) of GCM, \
         GF(2²³³) and GF(2⁵⁷¹) of NIST B-233 and B-571.",
        "Every result matches a bit-serial reference before it is measured.",
        "The build without carry-less multiplication drops PCLMULQDQ and VPCLMULQDQ.",
    ],
};

fn main() -> ExitCode {
    bench_compare::execute(
        "binary-field-bench-report",
        &Paths::new(env!("CARGO_MANIFEST_DIR")),
        &MEASURED,
        &LAYOUT,
    )
}
