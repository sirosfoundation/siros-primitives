#![doc = include_str!("../README.md")]

use core::fmt;
use std::process::ExitCode;

use bench_compare::{Layout, Measured, Paths, markdown_table, sentence};
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

/// The files that the measured executable is built from, as git pathspecs.
const MEASURED: Measured = Measured {
    package: "biguint",
    bench: "compare",
    sources: "crates/biguint/src",
    pathspecs: &[
        "Cargo.toml",
        "Cargo.lock",
        "rust-toolchain.toml",
        "crates/biguint",
        "crates/bench-compare",
        ":(exclude)crates/biguint/BENCHMARKS.md",
        ":(exclude)crates/biguint/benchmarks",
    ],
    versions: &["ruint", "crypto-bigint", "bnum", "uint", "num-bigint"],
};

const LAYOUT: Layout<'static, Width> = Layout {
    subject: "biguint",
    tool: "biguint-bench-report",
    formulas: &[
        ("add", "a + b"),
        ("mul", "a × b"),
        ("div", "a / b"),
        ("mod", "a mod b"),
        ("mulmod", "a × b mod m"),
        ("add_mod", "a + b mod m"),
        ("sub_mod", "a − b mod m"),
    ],
    row_header: "Bits",
    rows_phrase: |widths| format!("at {} bits", sentence(widths)),
    method,
    notes: &[
        "- Each row compares functions of one timing class, the class of `biguint`'s function: \
         add, and mul up to 1024 bits, take a time that the width fixes; mul from 2048 bits, div, \
         mod, mulmod, add_mod and sub_mod take a time that depends on the values.",
        "- `native` is `u64` at 64 bits and `u128` at 128 bits.",
        "- `crypto-bigint` is constant-time; its div, mod and mulmod cases run its `*_vartime` \
         functions.",
        "- `num-bigint` allocates each result.",
        "- `uint` is `construct_uint!`, as in `primitive-types`.",
    ],
    axis_title: "bits",
    category_name: |width| format!("{} bits", width.bits),
    lines: true,
};

fn method(batch: u32) -> Vec<String> {
    let operands = [
        ["add, mul", "`a` and `b` uniform below 2^N"],
        [
            "div, mod",
            "a dividend with bit N − 1 set; a divisor below 2^(N/2) with bit N/2 − 1 set, held \
             in an N-bit integer",
        ],
        [
            "mulmod, add_mod, sub_mod",
            "a modulus `m` with bit N − 1 set; `a` and `b` uniform below `m`",
        ],
    ];
    let rows: Vec<Vec<String>> = operands
        .iter()
        .map(|row| row.iter().map(|text| (*text).to_owned()).collect())
        .collect();
    let mut lines = vec![
        format!(
            "Each case runs one crate's function over {batch} operand sets, the same for every \
             crate in a row, with operands and results behind `core::hint::black_box`. Callgrind \
             counts the instructions of the loop over the batch, including the functions it \
             calls, such as the allocator of `num-bigint`; each count is that total divided by \
             {batch}. The loop's own loads and stores are part of every count. Every instruction \
             counts as one, whatever its latency: a hardware divide weighs as much as an \
             addition. The counts are exact at any machine load. Every result equals \
             `num-bigint`'s, reduced modulo 2^N for the crates that wrap."
        ),
        String::new(),
    ];
    lines.extend(markdown_table(
        &["Operation".to_owned(), "Operands at width N".to_owned()],
        &rows,
        "ll",
    ));
    lines
}

fn main() -> ExitCode {
    bench_compare::execute(
        "biguint-bench-report",
        &Paths::new(env!("CARGO_MANIFEST_DIR"), "biguint.json"),
        &MEASURED,
        &LAYOUT,
    )
}
