#![doc = include_str!("../README.md")]

use core::fmt;
use std::process::ExitCode;

use bench_compare::{BATCH, Layout, Measured, NATIVE, Paths, markdown_table, sentence};
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
    package: "prime-field",
    bench: "compare",
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
    formulas: &[
        ("add", "a + b"),
        ("sub", "a − b"),
        ("mul", "a × b"),
        ("square", "a²"),
        ("invert", "a⁻¹, constant time"),
        ("invert_vartime", "a⁻¹, variable time"),
        ("sqrt", "√a"),
    ],
    row_header: "Field",
    rows_phrase: |fields| format!("over {}", sentence(fields)),
    method,
    notes: &[
        "- `prime-field`, `ark-ff`, `crypto-bigint`, `halo2curves` and `runtime-algebra` are \
         generic over the modulus; `k256`, `p256`, `p3-*` and `winter-math` are written for \
         their field.",
        "- `crypto-bigint` is `ConstMontyForm` with a `const_monty_params!` modulus.",
        "- `ark-ff` is `Fp` with `MontBackend` from the arkworks curve crates; its `inverse` and \
         `sqrt` run in variable time.",
        "- `plonky3` is `p3-mersenne-31`, `p3-baby-bear` and `p3-goldilocks` on their field; its \
         inverse is a Fermat exponentiation by an addition chain.",
        "- `runtime-algebra` and `compile-algebra` are longfellow-zk's prover-side and \
         compile-side fields: `FpGenericField` is their generic Montgomery field with the modulus \
         held at run time; `P256Field` adds a hand-written P-256 strategy; `compile-algebra`'s \
         `FpField` computes on `num-bigint`.",
    ],
    axis_title: "field",
    category_name: |field| field.field.clone(),
    lines: false,
};

fn method() -> Vec<String> {
    let operands = [
        ["add, sub, mul", "`a` and `b` uniform below `p`"],
        ["square, invert, invert_vartime", "`a` uniform in `[1, p)`"],
        [
            "sqrt",
            "`a = r²` for `r` uniform in `[1, p)`, so that a root exists",
        ],
    ];
    let rows: Vec<Vec<String>> = operands
        .iter()
        .map(|row| row.iter().map(|text| (*text).to_owned()).collect())
        .collect();
    let mut lines = vec![
        format!(
            "The operands are canonical residues. Before a case is measured, its results on \
             {BATCH} operand sets are checked against `num-bigint`: the exact residue, \
             `a^(p - 2)` for the inverses, and either root for `sqrt`. Elements enter and leave \
             each crate in its own representation; the conversions lie outside the counted \
             region. The functions that a case calls, such as the allocator of \
             `compile-algebra`, are part of its counts. The chains are those of add, sub, mul \
             and square."
        ),
        String::new(),
    ];
    lines.extend(markdown_table(
        &["Operation".to_owned(), "Operands".to_owned()],
        &rows,
        "ll",
    ));
    lines
}

fn main() -> ExitCode {
    bench_compare::execute(
        "prime-field-bench-report",
        &Paths::new(env!("CARGO_MANIFEST_DIR"), "prime-field.json"),
        &MEASURED,
        &LAYOUT,
    )
}
