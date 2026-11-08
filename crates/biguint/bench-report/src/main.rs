#![doc = include_str!("../README.md")]

use core::fmt;
use std::process::ExitCode;

use bench_compare::{Layout, Measured, NATIVE, Operation, Paths, sentence};
use serde::{Deserialize, Serialize};

/// A row: a width in bits.
#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
struct Width {
    bits: u32,
}

impl fmt::Display for Width {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.bits)
    }
}

/// What the bench measures.
const MEASURED: Measured = Measured {
    package: "biguint",
    benches: &["add", "mul", "division", "mulmod", "add_mod", "sub_mod"],
    versions: &["ruint", "crypto-bigint", "bnum", "uint", "num-bigint"],
    features: &[],
    // It strips the lookup tables of `biguint`.
    forbidden: &["not-precomputed"],
    builds: &[NATIVE],
};

const LAYOUT: Layout<'static, Width> = Layout {
    subject: "biguint",
    tool: "biguint-bench-report",
    operations: &[
        Operation::new("add", "add", "a + b"),
        Operation::new("mul", "mul", "a × b"),
        Operation::new("div", "div", "a / b"),
        Operation::new("mod", "mod", "a mod b"),
        Operation::new("mulmod", "mulmod", "a × b mod m"),
        Operation::new("add_mod", "add_mod", "a + b mod m"),
        Operation::new("sub_mod", "sub_mod", "a − b mod m"),
        Operation::new(
            "add_mod_reduced",
            "add_mod, reduced operands",
            "a + b mod m, a and b below m",
        ),
        Operation::new(
            "sub_mod_reduced",
            "sub_mod, reduced operands",
            "a − b mod m, a and b below m",
        ),
        Operation::constant_time("add_ct", "add", "a + b"),
        Operation::constant_time("mul_ct", "mul", "a × b"),
        Operation::constant_time("div_ct", "div", "a / b"),
        Operation::constant_time("mod_ct", "mod", "a mod b"),
        Operation::constant_time("mulmod_ct", "mulmod", "a × b mod m"),
        Operation::constant_time("add_mod_ct", "add_mod", "a + b mod m"),
        Operation::constant_time("sub_mod_ct", "sub_mod", "a − b mod m"),
        Operation::constant_time(
            "add_mod_reduced_ct",
            "add_mod, reduced operands",
            "a + b mod m, a and b below m",
        ),
        Operation::constant_time(
            "sub_mod_reduced_ct",
            "sub_mod, reduced operands",
            "a − b mod m, a and b below m",
        ),
    ],
    row_header: "Bits",
    rows_phrase: |widths| format!("at {} bits", sentence(widths)),
    bits: |width| width.bits,
    method: &[
        "Operands at width N: uniform below 2^N; a divisor of N/2 bits; a modulus of N bits with \
         operands below it.",
        "Every result matches `num-bigint` before it is measured.",
        "A row compares the functions of `biguint`'s timing class: fixed for add and for mul up \
         to 1024 bits, variable for the rest.",
        "A constant-time table compares `ConstantTime<Uint>` with every function whose time the \
         width fixes; divisors and moduli are public.",
        "add_mod and sub_mod take operands of any value. A `*` there marks a function with an \
         unchecked precondition, operands below the modulus, which gives a wrong result for \
         other inputs: those cells compare against an unchecked precondition.",
        "The rows of reduced operands take the one modulus of the width. `biguint` runs \
         `Residue` and `ConstantTimeResidue` of a ring that `Modulus::scope` prepares outside \
         the measured loop: the type holds each operand below the modulus, with no check.",
        "`Residue` keeps a reference to its ring and reads the modulus through it, which makes \
         modulus mixing a compile error. `native` holds the modulus of its 64-bit and 128-bit \
         rows in a register, which leads `Residue -` at 128 bits natively, and the rows of reduced \
         operands at 64 and 128 bits in WebAssembly.",
        "`biguint` takes the operands of `add` and `mul` by value, and in WebAssembly LLVM loads \
         them into locals ahead of the operation. `crypto-bigint`'s product at 384 bits reads its \
         operands by reference as it goes.",
        "With wide arithmetic, `biguint` compiles to the same WebAssembly as `ruint` for `mul` at \
         256, 384 and 512 bits and as `crypto-bigint` for `add` at 384 and 512 bits: those figures \
         differ by where the browser places the code.",
        "Where the browsers favour different forms, `biguint` takes the form with the fewest cycles \
         summed over Chrome and Firefox: without wide arithmetic for reduced `add_mod` at 256 to 512 \
         bits, `add` at 512 bits and reduced constant-time `sub_mod` at 384 bits, and with it for \
         `add` at 384 and 1024 bits and `mulmod` at 512 bits.",
        "Without wide arithmetic, `add` at 512 bits runs `ruint`'s code with the operands in the \
         order of the faster native sum, which costs latency in WebAssembly.",
        "With wide arithmetic, `add` at 1024 bits sums 128-bit words, which costs latency in both \
         browsers.",
    ],
    wasm_native: "",
};

fn main() -> ExitCode {
    bench_compare::execute(
        "biguint-bench-report",
        &Paths::new(env!("CARGO_MANIFEST_DIR")),
        &MEASURED,
        &LAYOUT,
    )
}
