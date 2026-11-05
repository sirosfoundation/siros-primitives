//! A bench of two operations of `core` whose chain steps take a known number of cycles on an
//! x86-64 CPU: the `u64` add one, the `u64` multiply three. The browser driver calibrates against
//! it.

use core::fmt;

use bench_compare::{BATCH, Limbs, Rng, Row, Sample, Selected, Suite, Visitor, trim};
use serde::{Deserialize, Serialize};

/// The comparison.
struct Calibration;

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

/// An operation of `core` on two `u64`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Op {
    /// `wrapping_add`.
    Add,
    /// `wrapping_mul`.
    Mul,
}

impl Op {
    /// The operation on `a` and `b`.
    const fn apply(self, a: u64, b: u64) -> u64 {
        match self {
            Self::Add => a.wrapping_add(b),
            Self::Mul => a.wrapping_mul(b),
        }
    }
}

impl fmt::Display for Op {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Add => "add",
            Self::Mul => "mul",
        })
    }
}

/// The operation `add` as a type.
struct AddOp;

impl Selected<Op> for AddOp {
    const MEASURED: bool = true;

    fn op(&self) -> Op {
        Op::Add
    }
}

/// The operation `mul` as a type.
struct MulOp;

impl Selected<Op> for MulOp {
    const MEASURED: bool = true;

    fn op(&self) -> Op {
        Op::Mul
    }
}

impl Suite for Calibration {
    type Op = Op;
    type Key = Width;

    fn ops() -> Vec<Op> {
        vec![Op::Add, Op::Mul]
    }

    fn keys() -> Vec<Width> {
        vec![Width { bits: 64 }]
    }

    fn samples(op: Op, width: &Width) -> Vec<Sample> {
        let mut rng = Rng(1);
        (0..BATCH)
            .map(|_| {
                let operands = Self::operands(op, width, &mut rng);
                let result = op.apply(operands[0][0], operands[1][0]);
                Sample {
                    operands,
                    accepted: vec![trim(&[result]).to_vec()],
                }
            })
            .collect()
    }

    fn operands(_: Op, _: &Width, rng: &mut Rng) -> Vec<Limbs> {
        vec![rng.uniform(1), rng.uniform(1)]
    }

    fn offer<V: Visitor<Op, Width>>(row: Row<Op, Width, V>) -> Row<Op, Width, V> {
        row.lib("core", |cells| {
            let prepare = |set: &[Limbs]| (set[0][0], set[1][0]);
            let read = |result: &u64| Some(vec![*result]);
            cells
                .chain(
                    AddOp,
                    "wrapping_add",
                    prepare,
                    |&(a, b): &(u64, u64)| a.wrapping_add(b),
                    read,
                )
                .chain(
                    MulOp,
                    "wrapping_mul",
                    prepare,
                    |&(a, b): &(u64, u64)| a.wrapping_mul(b),
                    read,
                )
        })
    }
}

fn main() -> std::process::ExitCode {
    bench_compare::main::<Calibration>("calibration")
}
