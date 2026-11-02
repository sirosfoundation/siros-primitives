#![doc = include_str!("../README.md")]

use core::fmt;
use std::process::ExitCode;

use bench_compare::{Layout, Measured, Paths, markdown_table, sentence};
use serde::{Deserialize, Serialize};

/// A row: a field, by name.
#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
struct Field {
    field: String,
}

impl fmt::Display for Field {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.field)
    }
}

const MEASURED: Measured = Measured {
    package: "binary-field",
    bench: "compare",
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
};

const LAYOUT: Layout<'static, Field> = Layout {
    subject: "binary-field",
    tool: "binary-field-bench-report",
    formulas: &[
        ("add", "a + b"),
        ("mul", "a × b"),
        ("square", "a²"),
        ("invert", "a⁻¹"),
        ("sqrt", "√a"),
    ],
    row_header: "Field",
    rows_phrase: |fields| format!("over {}", sentence(fields)),
    method,
    notes: &[
        "- `gf256` is its `gf2p16` and `gf2p64`, and at 8 bits the AES field that its `gf` macro \
         declares; the macro's default modes are log and antilog tables up to 8 bits and Barrett \
         reduction above.",
        "- `galois_field_2pm` is its `gf2` module, which multiplies by carry-less products; \
         `galois_field_2pm lut` is its `gf2_lut` module, with log and antilog tables for a \
         primitive polynomial.",
        "- `g2p` multiplies by lookups in 256 × 256 tables of byte products.",
        "- `reed-solomon-erasure` is its `galois_8` module, GF(2⁸) with x⁸ + x⁴ + x³ + x² + 1, \
         and its `galois_16` module, the extension of that field by y² + 2y + 128.",
        "- `hekate-math` is its tower fields `Block8` to `Block256`: `Block8` is the AES field, \
         and each level above extends the one below by X² + X + τ.",
        "- `plonky3` is `p3-binary-field`: `Rijndael8b`, `Poly64` and `Ghash128` in the \
         polynomial bases of the rows, and `BinaryField16` in the Wiedemann tower.",
        "- `runtime-algebra`, `compile-algebra` and `core-algebra` are the `Gf2_128Field` of \
         longfellow-zk's prover, compiler and reference.",
        "- `polyval` is its `hazmat::FieldElement`, whose product is the crate's portable \
         constant-time code. An element `a` enters POLYVAL's representation (RFC 8452) as the \
         block of the bit reversal of `a·x⁻¹`, the map that carries POLYVAL's product to the \
         GCM field's.",
        "- Where a crate's basis differs from the row's (`reed-solomon-erasure`, `hekate-math` \
         from 16 bits, `plonky3` at 16 bits), operands and results pass through the field \
         isomorphism that the bench derives from the crate's tower: a root of the base \
         polynomial, then a root of each quadratic.",
    ],
    axis_title: "field",
    category_name: |field| field.field.clone(),
    lines: false,
};

fn method(batch: u32) -> Vec<String> {
    let table = |header: [&str; 2], rows: &[[&str; 2]]| {
        markdown_table(
            &header.map(str::to_owned),
            &rows
                .iter()
                .map(|row| row.iter().map(|text| (*text).to_owned()).collect())
                .collect::<Vec<_>>(),
            "ll",
        )
    };
    let mut lines = vec![
        format!(
            "Each case runs one crate's function over {batch} operand sets, the same elements \
             for every crate in a row, with operands and results behind `core::hint::black_box`. \
             Callgrind counts the instructions of the loop over the batch, including the \
             functions it calls; each count is that total divided by {batch}. The loop's own \
             loads and stores are part of every count. Every instruction counts as one, whatever \
             its latency: a table lookup weighs as much as an exclusive or, whatever the cache \
             does with it. The build leaves out VPCLMULQDQ, whose 256-bit `vpclmulqdq` Valgrind \
             cannot run. A native build may fuse two carry-less products into one such \
             instruction. The counts are exact at any machine load. Before a case is counted, \
             every result is checked against a bit-serial reference in the bench: the \
             shift-and-add product reduced bit by bit, the extended Euclidean inverse confirmed \
             by that product, and the root `r` of `a = r²`. Elements enter and leave each crate \
             in its own representation; the conversions lie outside the count."
        ),
        String::new(),
    ];
    lines.extend(table(
        ["Operation", "Operands"],
        &[
            ["add, mul", "`a` and `b` uniform"],
            ["square, invert", "`a` uniform and nonzero"],
            ["sqrt", "`a = r²` for `r` uniform and nonzero"],
        ],
    ));
    lines.extend([
        String::new(),
        "Each row is a field in the polynomial basis of its modulus: bit `i` of an element is \
         the coefficient of `xⁱ`."
            .to_owned(),
        String::new(),
    ]);
    lines.extend(table(
        ["Field", "Modulus"],
        &[
            ["GF(2⁸)", "x⁸ + x⁴ + x³ + x + 1, AES"],
            ["GF(2¹⁶)", "x¹⁶ + x⁵ + x³ + x² + 1"],
            ["GF(2⁶⁴)", "x⁶⁴ + x⁴ + x³ + x + 1"],
            ["GF(2¹²⁸)", "x¹²⁸ + x⁷ + x² + x + 1, GCM"],
            ["GF(2²³³)", "x²³³ + x⁷⁴ + 1, NIST B-233"],
            ["GF(2²⁵⁶)", "x²⁵⁶ + x¹⁰ + x⁵ + x² + 1"],
            ["GF(2⁵⁷¹)", "x⁵⁷¹ + x¹⁰ + x⁵ + x² + 1, NIST B-571"],
        ],
    ));
    lines
}

fn main() -> ExitCode {
    bench_compare::execute(
        "binary-field-bench-report",
        &Paths::new(env!("CARGO_MANIFEST_DIR"), "binary-field.json"),
        &MEASURED,
        &LAYOUT,
    )
}
