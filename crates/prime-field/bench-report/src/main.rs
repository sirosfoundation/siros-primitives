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
    benches: &["sums", "products", "integers", "divisions"],
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
        Operation::new("mul_u64_vartime", "mul_u64", "a × k, k of 64 bits"),
        Operation::new(
            "mul_uint_vartime",
            "mul_uint",
            "a × k, k of the bit length of p",
        ),
        Operation::new("add_u64_vartime", "add_u64", "a + k, k of 64 bits"),
        Operation::new(
            "add_uint_vartime",
            "add_uint",
            "a + k, k of the bit length of p",
        ),
        Operation::new("sub_u64_vartime", "sub_u64", "a − k, k of 64 bits"),
        Operation::new(
            "sub_uint_vartime",
            "sub_uint",
            "a − k, k of the bit length of p",
        ),
        Operation::new("from_u64_vartime", "from_u64", "k mod p, k of 64 bits"),
        Operation::new("div_u64_vartime", "div_u64", "a / k, k of 64 bits"),
        Operation::new(
            "div_uint_vartime",
            "div_uint",
            "a / k, k of one bit less than p",
        ),
        Operation::constant_time("add_ct", "add", "a + b"),
        Operation::constant_time("sub_ct", "sub", "a − b"),
        Operation::constant_time("mul_ct", "mul", "a × b"),
        Operation::constant_time("square_ct", "square", "a²"),
        Operation::constant_time("invert_ct", "invert", "a⁻¹"),
        Operation::constant_time("sqrt_ct", "sqrt", "√a"),
        Operation::constant_time("mul_u64_ct", "mul_u64", "a × k, k of 64 bits"),
        Operation::constant_time("mul_uint_ct", "mul_uint", "a × k, k of the bit length of p"),
        Operation::constant_time("add_u64_ct", "add_u64", "a + k, k of 64 bits"),
        Operation::constant_time("add_uint_ct", "add_uint", "a + k, k of the bit length of p"),
        Operation::constant_time("sub_u64_ct", "sub_u64", "a − k, k of 64 bits"),
        Operation::constant_time("sub_uint_ct", "sub_uint", "a − k, k of the bit length of p"),
        Operation::constant_time("from_u64_ct", "from_u64", "k mod p, k of 64 bits"),
        Operation::constant_time("div_u64_ct", "div_u64", "a / k, k of 64 bits"),
        Operation::constant_time("div_uint_ct", "div_uint", "a / k, k of one bit less than p"),
    ],
    row_header: "Field",
    rows_phrase: |fields| format!("over {}", sentence(fields)),
    bits: |field| field.bits,
    method: &[
        "Operands: canonical residues, the same for every crate.",
        "Integer operands `k`: uniform of exactly 64 bits, or of exactly the bit length of `p`, which \
         may reach `p`. A crate without an operation on an integer runs its own conversion of `k` \
         and then its operation on two elements.",
        "Divisors `k`: nonzero, uniform of exactly 64 bits over the fields of more than 64 bits, or \
         of exactly one bit less than `p`. A crate without a division by an integer runs its own \
         conversion of `k`, its inverse and a product; `prime-field × invert` is that composition \
         in `prime-field`.",
        "Every result matches `num-bigint` before it is measured.",
        "`prime-field` runs `Fp` in the tables without a label and `ConstantTime` in the \
         constant-time tables.",
        "`prime-field` holds each element in Montgomery form, which favours `mul`. `k256/p256` holds \
         a P-256 scalar as a plain integer, which favours `add_u64`, `sub_u64`, `add_uint`, \
         `sub_uint` and `from_u64` over that field.",
    ],
    wasm_native: "",
};

fn main() -> ExitCode {
    bench_compare::execute(
        "prime-field-bench-report",
        &Paths::new(env!("CARGO_MANIFEST_DIR")),
        &MEASURED,
        &LAYOUT,
    )
}
