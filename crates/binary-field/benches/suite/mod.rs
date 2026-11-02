//! The suite: the operations, the fields, the operand sets with their results from the bit-serial
//! reference, and the adapters of every crate.

mod basis;
mod libs;
mod reference;

use core::fmt;

use bench_compare::{BATCH, Limbs, Rng, Row, Sample, Suite, Visitor, trim};
use serde::{Deserialize, Serialize};

use reference::Modulus;

/// The comparison of `binary-field`.
pub struct BinaryField;

/// A row: a field, by name.
#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct Field {
    /// The name.
    pub field: String,
}

impl fmt::Display for Field {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.field)
    }
}

/// A field under comparison: its name and its modulus.
pub struct Spec {
    pub name: &'static str,
    pub modulus: Modulus,
}

pub const GF8: &str = "GF(2⁸)";
pub const GF16: &str = "GF(2¹⁶)";
pub const GF64: &str = "GF(2⁶⁴)";
pub const GF128: &str = "GF(2¹²⁸)";
pub const GF233: &str = "GF(2²³³)";
pub const GF256: &str = "GF(2²⁵⁶)";
pub const GF571: &str = "GF(2⁵⁷¹)";

/// `x⁸ + x⁴ + x³ + x + 1`, the modulus of AES (FIPS 197).
pub const AES: Modulus = Modulus {
    degree: 8,
    taps: &[4, 3, 1, 0],
};

/// Every field, in row order: by degree. The moduli come from the standards that name them.
pub const FIELDS: [Spec; 7] = [
    Spec {
        name: GF8,
        modulus: AES,
    },
    Spec {
        name: GF16,
        modulus: Modulus {
            degree: 16,
            taps: &[5, 3, 2, 0],
        },
    },
    Spec {
        name: GF64,
        modulus: Modulus {
            degree: 64,
            taps: &[4, 3, 1, 0],
        },
    },
    // NIST SP 800-38D, GCM.
    Spec {
        name: GF128,
        modulus: Modulus {
            degree: 128,
            taps: &[7, 2, 1, 0],
        },
    },
    // FIPS 186-4, curve B-233.
    Spec {
        name: GF233,
        modulus: Modulus {
            degree: 233,
            taps: &[74, 0],
        },
    },
    Spec {
        name: GF256,
        modulus: Modulus {
            degree: 256,
            taps: &[10, 5, 2, 0],
        },
    },
    // FIPS 186-4, curve B-571.
    Spec {
        name: GF571,
        modulus: Modulus {
            degree: 571,
            taps: &[10, 5, 2, 0],
        },
    },
];

/// The index and the specification of the field `key`.
pub fn spec(key: &Field) -> (usize, &'static Spec) {
    FIELDS
        .iter()
        .enumerate()
        .find(|(_, spec)| spec.name == key.field)
        .unwrap_or_else(|| panic!("no field {}", key.field))
}

impl Suite for BinaryField {
    type Op = Op;
    type Key = Field;

    const OPS: &'static [Op] = &Op::ALL;

    fn keys() -> Vec<Field> {
        FIELDS
            .iter()
            .map(|spec| Field {
                field: spec.name.to_owned(),
            })
            .collect()
    }

    /// The operand sets of `op` over the field, with the results of the reference. The seed
    /// depends on the operation and the field only, so every crate in a row sees the same
    /// operands.
    fn samples(op: Op, key: &Field) -> Vec<Sample> {
        let (index, spec) = spec(key);
        let mut rng = Rng(((op as u64) << 32) | index as u64);
        (0..BATCH)
            .map(|_| {
                let (operands, accepted) = op.sample(&spec.modulus, &mut rng);
                Sample {
                    operands,
                    accepted: vec![trim(&accepted).to_vec()],
                }
            })
            .collect()
    }

    fn offer<V: Visitor<Op, Field>>(row: Row<Op, Field, V>) -> Row<Op, Field, V> {
        libs::offer(row)
    }
}

/// An operation under comparison.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Op {
    /// `a + b`.
    Add,
    /// `a * b`.
    Mul,
    /// `a * a`.
    Square,
    /// `a^-1`.
    Invert,
    /// The square root of `a`.
    Sqrt,
}

impl Op {
    /// Every operation, in measurement order.
    pub const ALL: [Self; 5] = [Self::Add, Self::Mul, Self::Square, Self::Invert, Self::Sqrt];

    /// One operand set over the field of `modulus`, with its result:
    /// - add, mul: `a` and `b` uniform;
    /// - square, invert: `a` uniform and nonzero;
    /// - sqrt: `a = r²` for `r` uniform and nonzero, whose square root is `r` alone.
    fn sample(self, modulus: &Modulus, rng: &mut Rng) -> (Vec<Limbs>, Limbs) {
        match self {
            Self::Add => {
                let (a, b) = (modulus.uniform(rng), modulus.uniform(rng));
                let mut sum = a.clone();
                reference::xor(&mut sum, &b);
                (vec![a, b], sum)
            }
            Self::Mul => {
                let (a, b) = (modulus.uniform(rng), modulus.uniform(rng));
                let product = modulus.mul(&a, &b);
                (vec![a, b], product)
            }
            Self::Square => {
                let a = modulus.nonzero(rng);
                let result = modulus.square(&a);
                (vec![a], result)
            }
            Self::Invert => {
                let a = modulus.nonzero(rng);
                let result = modulus.inverse(&a);
                (vec![a], result)
            }
            Self::Sqrt => {
                let root = modulus.nonzero(rng);
                (vec![modulus.square(&root)], root)
            }
        }
    }
}

impl fmt::Display for Op {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Add => "add",
            Self::Mul => "mul",
            Self::Square => "square",
            Self::Invert => "invert",
            Self::Sqrt => "sqrt",
        })
    }
}
