//! The suite: the operations, the widths, the operand sets with their exact results from
//! `num-bigint`, and the adapters of every crate.

mod libs;
mod limbs;

use core::fmt;

use bench_compare::{BATCH, Limbs, Rng, Row, Sample, Suite, Visitor, trim};
use num_bigint::BigUint;
use serde::{Deserialize, Serialize};

pub use libs::WIDTHS;
pub use limbs::FromLimbs;

/// The comparison of `biguint`.
pub struct Biguint;

/// A row: a width in bits.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct Width {
    /// The width.
    pub bits: u32,
}

impl fmt::Display for Width {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.bits)
    }
}

impl Suite for Biguint {
    type Op = Op;
    type Key = Width;

    const OPS: &'static [Op] = &Op::ALL;

    fn keys() -> Vec<Width> {
        WIDTHS.into_iter().map(|bits| Width { bits }).collect()
    }

    /// The operand sets of `op` at the width, with their exact results. The seed depends on the
    /// operation and the width only, so every crate in a row sees the same operands. A crate
    /// that wraps modulo `2^bits` and a crate that returns the exact result both pass.
    fn samples(op: Op, key: &Width) -> Vec<Sample> {
        let bits = key.bits as usize;
        let mut rng = Rng(((op as u64) << 32) | bits as u64);
        (0..BATCH)
            .map(|_| {
                let operands = op.operands(bits, &mut rng);
                let values: Vec<BigUint> = operands
                    .iter()
                    .map(|operand| BigUint::from_limbs(operand))
                    .collect();
                let exact = op.exact(&values).to_u64_digits();
                let wrapped = trim(&exact[..exact.len().min(bits / 64)]).to_vec();
                let mut accepted = vec![wrapped];
                if accepted[0] != exact {
                    accepted.push(exact);
                }
                Sample { operands, accepted }
            })
            .collect()
    }

    fn offer<V: Visitor<Op, Width>>(row: Row<Op, Width, V>) -> Row<Op, Width, V> {
        libs::offer(row)
    }
}

/// An operation under comparison.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Op {
    /// `a + b`. Subtraction runs the same carry chain, so the bench measures addition only.
    Add,
    /// `a * b`.
    Mul,
    /// `a / b`.
    Div,
    /// `a % b`.
    Mod,
    /// `(a * b) mod m`.
    MulMod,
    /// `(a + b) mod m`.
    AddMod,
    /// `(a - b) mod m`.
    SubMod,
}

impl Op {
    /// Every operation, in measurement order.
    pub const ALL: [Self; 7] = [
        Self::Add,
        Self::Mul,
        Self::Div,
        Self::Mod,
        Self::MulMod,
        Self::AddMod,
        Self::SubMod,
    ];

    /// One operand set of width `bits`:
    /// - add, mul: two uniform `bits`-bit values;
    /// - div, mod: a dividend of exactly `bits` bits and a divisor of exactly `bits / 2` bits;
    /// - mulmod, add_mod, sub_mod: a modulus `m` of exactly `bits` bits and two uniform values
    ///   below `m`, as `[a, b, m]`.
    ///
    /// "Exactly `n` bits" means that bit `n - 1` is set.
    fn operands(self, bits: usize, rng: &mut Rng) -> Vec<Limbs> {
        let limbs = bits / 64;
        match self {
            Self::Add | Self::Mul => vec![rng.uniform(limbs), rng.uniform(limbs)],
            Self::Div | Self::Mod => vec![rng.exact(limbs, bits), rng.exact(limbs, bits / 2)],
            Self::MulMod | Self::AddMod | Self::SubMod => {
                let modulus = rng.exact(limbs, bits);
                vec![rng.below(&modulus), rng.below(&modulus), modulus]
            }
        }
    }

    /// The exact result of the operation on `operands`.
    fn exact(self, operands: &[BigUint]) -> BigUint {
        match (self, operands) {
            (Self::Add, [a, b]) => a + b,
            (Self::Mul, [a, b]) => a * b,
            (Self::Div, [a, b]) => a / b,
            (Self::Mod, [a, b]) => a % b,
            (Self::MulMod, [a, b, modulus]) => a * b % modulus,
            (Self::AddMod, [a, b, modulus]) => (a + b) % modulus,
            (Self::SubMod, [a, b, modulus]) => (a + modulus - b) % modulus,
            (op, operands) => panic!("{op} on {} operands", operands.len()),
        }
    }
}

impl fmt::Display for Op {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Add => "add",
            Self::Mul => "mul",
            Self::Div => "div",
            Self::Mod => "mod",
            Self::MulMod => "mulmod",
            Self::AddMod => "add_mod",
            Self::SubMod => "sub_mod",
        })
    }
}
