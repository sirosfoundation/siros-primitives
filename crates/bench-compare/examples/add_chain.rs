//! A bench of one operation: the `u64` add of `core`, whose chain step takes one cycle on an
//! x86-64 CPU. The browser driver calibrates against it.

use core::fmt;

use bench_compare::{BATCH, Limbs, Rng, Row, Sample, Selected, Suite, Visitor, trim};
use serde::{Deserialize, Serialize};

/// The comparison.
struct Add;

/// A row: a width in bits, which a cell records as `"bits": 64`.
#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
struct Width {
    bits: u32,
}

impl fmt::Display for Width {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.bits)
    }
}

/// The operation `add` as a type.
struct AddOp;

impl Selected<&'static str> for AddOp {
    const MEASURED: bool = true;

    fn op(&self) -> &'static str {
        "add"
    }
}

impl Suite for Add {
    type Op = &'static str;
    type Key = Width;

    fn ops() -> Vec<&'static str> {
        vec!["add"]
    }

    fn keys() -> Vec<Width> {
        vec![Width { bits: 64 }]
    }

    fn samples(op: &'static str, width: &Width) -> Vec<Sample> {
        let mut rng = Rng(1);
        (0..BATCH)
            .map(|_| {
                let operands = Self::operands(op, width, &mut rng);
                let sum = operands[0][0].wrapping_add(operands[1][0]);
                Sample {
                    operands,
                    accepted: vec![trim(&[sum]).to_vec()],
                }
            })
            .collect()
    }

    fn operands(_: &'static str, _: &Width, rng: &mut Rng) -> Vec<Limbs> {
        vec![rng.uniform(1), rng.uniform(1)]
    }

    fn offer<V: Visitor<&'static str, Width>>(
        row: Row<&'static str, Width, V>,
    ) -> Row<&'static str, Width, V> {
        row.lib("core", |cells| {
            cells.chain(
                AddOp,
                "wrapping_add",
                |set| (set[0][0], set[1][0]),
                |&(a, b): &(u64, u64)| a.wrapping_add(b),
                |sum| Some(vec![*sum]),
            )
        })
    }
}

fn main() -> std::process::ExitCode {
    bench_compare::main::<Add>("add")
}
