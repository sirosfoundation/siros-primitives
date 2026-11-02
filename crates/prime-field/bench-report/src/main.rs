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
};

const LAYOUT: Layout<'static, Field> = Layout {
    subject: "prime-field",
    tool: "prime-field-bench-report",
    formulas: &[
        ("add_vartime", "a + b, variable time"),
        ("add_ct", "a + b, constant time"),
        ("sub_vartime", "a − b, variable time"),
        ("sub_ct", "a − b, constant time"),
        ("mul_vartime", "a × b, variable time"),
        ("mul_ct", "a × b, constant time"),
        ("square_vartime", "a², variable time"),
        ("square_ct", "a², constant time"),
        ("invert_vartime", "a⁻¹, variable time"),
        ("invert_ct", "a⁻¹, constant time"),
        ("sqrt_vartime", "√a, variable time"),
        ("sqrt_ct", "√a, constant time"),
    ],
    row_header: "Field",
    rows_phrase: |fields| format!("over {}", sentence(fields)),
    method,
    notes: &[
        "- A variable-time row compares the functions whose time may depend on the values; a \
         constant-time row those whose time depends on no value. Each function sits in the row of \
         its class, as its source shows it: a branch, a loop bound or a memory address that \
         depends on a value makes it variable time.",
        "- `prime-field` runs `Fp` in the variable-time rows and `Secret` in the constant-time \
         rows.",
        "- `prime-field`, `ark-ff`, `crypto-bigint`, `halo2curves` and `runtime-algebra` are \
         generic over the modulus; `k256`, `p256`, `p3-*` and `winter-math` are written for \
         their field.",
        "- `crypto-bigint` is `ConstMontyForm` with a `const_monty_params!` modulus: masks and \
         constant-time selections, except `invert_vartime`.",
        "- `ark-ff` is `Fp` with `MontBackend` from the arkworks curve crates: `+` and `*` branch \
         on their final subtraction, `-` on its borrow, `inverse` and `sqrt` throughout.",
        "- `k256`'s `+` and `-` add limbs without carries and leave the reduction to a later \
         `normalize`. `p256`'s scalar adds `invert_vartime`, Stein's algorithm.",
        "- `halo2curves` selects by masks in `+`, `-` and `*`, and runs the constant-time `sqrt` \
         of `ff`; its `invert` runs variable-time division steps.",
        "- `plonky3` is `p3-mersenne-31`, `p3-baby-bear` and `p3-goldilocks` on their field: \
         `+`, `*`, `square`, `try_inverse` and `try_sqrt` decide with `if` on the value; \
         Mersenne-31's `-` is arithmetic alone.",
        "- `winter-math` corrects by masks, and inverts by a fixed addition chain.",
        "- `runtime-algebra` and `compile-algebra` are longfellow-zk's prover-side and \
         compile-side fields: `FpGenericField` is their generic Montgomery field with the modulus \
         held at run time, whose `addf`, `subf` and `mulf` select by `cmov` in inline assembly \
         and whose `invert` branches; `P256Field` adds a hand-written P-256 strategy; \
         `compile-algebra`'s `FpField` computes on `num-bigint`.",
    ],
    axis_title: "field",
    category_name: |field| field.field.clone(),
    lines: false,
};

fn method(batch: u32) -> Vec<String> {
    let operands = [
        ["add, sub, mul", "`a` and `b` uniform below `p`"],
        ["square, invert", "`a` uniform in `[1, p)`"],
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
            "Each case runs one crate's function over {batch} operand sets, the same canonical \
             residues for every crate in both rows of an operation, with operands and results behind \
             `core::hint::black_box`. Callgrind counts the instructions of the loop over the \
             batch, including the functions it calls, such as the allocator of \
             `compile-algebra`; each count is that total divided by {batch}. The loop's own \
             loads and stores are part of every count. Every instruction counts as one, \
             whatever its latency. The counts are exact at any machine load. Before a case is \
             counted, every result is checked against `num-bigint`: the exact residue, \
             `a^(p - 2)` for the inverses, and either root for `sqrt`. Elements enter and leave \
             each crate in its own representation; the conversions lie outside the count."
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
