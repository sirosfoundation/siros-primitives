//! The suite: the operations, the widths, the operand sets with their exact results from
//! `num-bigint`, and the adapters of every crate.

mod libs;
mod limbs;

use core::fmt;

use bench_compare::{BATCH, Limbs, Rng, Row, Sample, Selected, Suite, Visitor, trim};
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

    fn ops() -> Vec<Op> {
        Op::ALL
            .into_iter()
            .filter(|op| op.base() == crate::BASE)
            .collect()
    }

    fn keys() -> Vec<Width> {
        crate::WIDTHS.iter().map(|&bits| Width { bits }).collect()
    }

    /// The operand sets of `op` at the width, with their exact results. The seed depends on the
    /// operation and the width only, so every crate in a row sees the same operands. A crate
    /// that wraps modulo `2^bits` and a crate that returns the exact result both pass.
    fn samples(op: Op, key: &Width) -> Vec<Sample> {
        let bits = key.bits as usize;
        let mut rng = Rng(((op.base() as u64) << 32) | bits as u64);
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

    fn operands(op: Op, key: &Width, rng: &mut Rng) -> Vec<Limbs> {
        op.operands(key.bits as usize, rng)
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
    /// [`Add`](Self::Add) by functions whose time depends on no value.
    AddCt,
    /// [`Mul`](Self::Mul) in constant time.
    MulCt,
    /// [`Div`](Self::Div) in constant time.
    DivCt,
    /// [`Mod`](Self::Mod) in constant time.
    ModCt,
    /// [`MulMod`](Self::MulMod) in constant time.
    MulModCt,
    /// [`AddMod`](Self::AddMod) in constant time.
    AddModCt,
    /// [`SubMod`](Self::SubMod) in constant time.
    SubModCt,
}

impl Op {
    /// Every operation, in measurement order.
    pub const ALL: [Self; 14] = [
        Self::Add,
        Self::Mul,
        Self::Div,
        Self::Mod,
        Self::MulMod,
        Self::AddMod,
        Self::SubMod,
        Self::AddCt,
        Self::MulCt,
        Self::DivCt,
        Self::ModCt,
        Self::MulModCt,
        Self::AddModCt,
        Self::SubModCt,
    ];

    /// The operation that this one computes: itself, or the operation of a constant-time one.
    pub const fn base(self) -> Self {
        match self {
            Self::Add | Self::AddCt => Self::Add,
            Self::Mul | Self::MulCt => Self::Mul,
            Self::Div | Self::DivCt => Self::Div,
            Self::Mod | Self::ModCt => Self::Mod,
            Self::MulMod | Self::MulModCt => Self::MulMod,
            Self::AddMod | Self::AddModCt => Self::AddMod,
            Self::SubMod | Self::SubModCt => Self::SubMod,
        }
    }

    /// The constant-time operation of the same computation.
    pub const fn constant_time(self) -> Self {
        match self {
            Self::Add | Self::AddCt => Self::AddCt,
            Self::Mul | Self::MulCt => Self::MulCt,
            Self::Div | Self::DivCt => Self::DivCt,
            Self::Mod | Self::ModCt => Self::ModCt,
            Self::MulMod | Self::MulModCt => Self::MulModCt,
            Self::AddMod | Self::AddModCt => Self::AddModCt,
            Self::SubMod | Self::SubModCt => Self::SubModCt,
        }
    }

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
            Self::Add | Self::Mul | Self::AddCt | Self::MulCt => {
                vec![rng.uniform(limbs), rng.uniform(limbs)]
            }
            Self::Div | Self::Mod | Self::DivCt | Self::ModCt => {
                vec![rng.exact(limbs, bits), rng.exact(limbs, bits / 2)]
            }
            Self::MulMod
            | Self::AddMod
            | Self::SubMod
            | Self::MulModCt
            | Self::AddModCt
            | Self::SubModCt => {
                let modulus = rng.exact(limbs, bits);
                vec![rng.below(&modulus), rng.below(&modulus), modulus]
            }
        }
    }

    /// The exact result of the operation on `operands`.
    fn exact(self, operands: &[BigUint]) -> BigUint {
        match (self.base(), operands) {
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

/// A computation as a type: the operation of the plain table, whose constant-time operation fills
/// the constant-time table. Its cases compile in the bench of its computation alone.
pub trait Base: Copy {
    /// The operation of the plain table.
    const OP: Op;
    /// Whether this bench measures the computation.
    const MEASURED: bool = Self::OP as u8 == crate::BASE as u8;
}

/// The computations, one type each.
pub mod op {
    use super::{Base, Op};

    /// Declares each computation `$name` of the plain operation `Op::$name`.
    macro_rules! bases {
        ($($name:ident),*) => {
            $(
                #[doc = concat!("The computation of `Op::", stringify!($name), "`.")]
                #[derive(Clone, Copy)]
                pub struct $name;

                impl Base for $name {
                    const OP: Op = Op::$name;
                }
            )*
        };
    }

    bases!(Add, Mul, Div, Mod, MulMod, AddMod, SubMod);
}

/// The plain operation of the computation `B`.
#[derive(Clone, Copy)]
pub struct Plain<B>(pub B);

/// The constant-time operation of the computation `B`.
#[derive(Clone, Copy)]
pub struct Ct<B>(pub B);

impl<B: Base> Selected<Op> for Plain<B> {
    const MEASURED: bool = B::MEASURED;

    fn op(&self) -> Op {
        B::OP
    }
}

impl<B: Base> Selected<Op> for Ct<B> {
    const MEASURED: bool = B::MEASURED;

    fn op(&self) -> Op {
        B::OP.constant_time()
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
            Self::AddCt => "add_ct",
            Self::MulCt => "mul_ct",
            Self::DivCt => "div_ct",
            Self::ModCt => "mod_ct",
            Self::MulModCt => "mulmod_ct",
            Self::AddModCt => "add_mod_ct",
            Self::SubModCt => "sub_mod_ct",
        })
    }
}
