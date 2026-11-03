//! The suite: the operations, the fields, the operand sets with their results from `num-bigint`,
//! and the adapters of every crate.

mod libs;

use core::fmt;
use core::marker::PhantomData;

use bench_compare::{BATCH, Limbs, Rng, Row, Sample, Selected, Suite, Visitor, trim};
use num_bigint::BigUint;
use prime_field::Modulus;
use prime_field::standard::{
    BabyBear, Bls12381Base, Bn254Scalar, Goldilocks, Mersenne31, P256Base, P256Scalar, P521Base,
    Secp256k1Base, Tom256Base,
};
use serde::{Deserialize, Serialize};

/// The comparison of `prime-field`.
pub struct PrimeField;

/// A row: a field, by name, and the bit length of its modulus.
#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct Field {
    /// The name.
    pub field: String,
    /// The bit length of the modulus.
    pub bits: usize,
}

impl fmt::Display for Field {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.field)
    }
}

/// A field under comparison: its name and the little-endian limbs of its modulus.
pub struct Spec {
    pub name: &'static str,
    pub modulus: &'static [u64],
}

/// Every field, in row order: by size.
pub const FIELDS: [Spec; 10] = [
    Spec {
        name: "Mersenne-31",
        modulus: Mersenne31::MODULUS.as_limbs(),
    },
    Spec {
        name: "BabyBear",
        modulus: BabyBear::MODULUS.as_limbs(),
    },
    Spec {
        name: "Goldilocks",
        modulus: Goldilocks::MODULUS.as_limbs(),
    },
    Spec {
        name: "BN254 scalar",
        modulus: Bn254Scalar::MODULUS.as_limbs(),
    },
    Spec {
        name: "secp256k1 base",
        modulus: Secp256k1Base::MODULUS.as_limbs(),
    },
    Spec {
        name: "P-256 base",
        modulus: P256Base::MODULUS.as_limbs(),
    },
    Spec {
        name: "P-256 scalar",
        modulus: P256Scalar::MODULUS.as_limbs(),
    },
    Spec {
        name: "Tom-256 base",
        modulus: Tom256Base::MODULUS.as_limbs(),
    },
    Spec {
        name: "BLS12-381 base",
        modulus: Bls12381Base::MODULUS.as_limbs(),
    },
    Spec {
        name: "P-521 base",
        modulus: P521Base::MODULUS.as_limbs(),
    },
];

/// The specification of the field `key`.
pub fn spec(key: &Field) -> &'static Spec {
    FIELDS
        .iter()
        .find(|spec| spec.name == key.field)
        .unwrap_or_else(|| panic!("no field {}", key.field))
}

/// The bit length of the little-endian `limbs`.
fn bit_length(limbs: &[u64]) -> usize {
    let significant = trim(limbs);
    significant.last().map_or(0, |top| {
        64 * significant.len() - top.leading_zeros() as usize
    })
}

impl Suite for PrimeField {
    type Op = Op;
    type Key = Field;

    fn ops() -> Vec<Op> {
        Op::ALL
            .into_iter()
            .filter(|op| op.kind == crate::KIND)
            .collect()
    }

    fn keys() -> Vec<Field> {
        FIELDS
            .iter()
            .map(|spec| Field {
                field: spec.name.to_owned(),
                bits: bit_length(spec.modulus),
            })
            .collect()
    }

    /// The operand sets of `op` over the field, canonical residues, with the results from
    /// `num-bigint`. The seed depends on the kind of operation and the field only, so every crate
    /// in a row, and both timings of a kind, see the same operands.
    fn samples(op: Op, key: &Field) -> Vec<Sample> {
        let spec = spec(key);
        let index = FIELDS
            .iter()
            .position(|candidate| candidate.name == spec.name)
            .unwrap_or_else(|| panic!("no field {}", key.field));
        let p = big(spec.modulus);
        let mut rng = Rng(((op.kind as u64) << 32) | index as u64);
        (0..BATCH)
            .map(|_| {
                let (operands, accepted) = op.kind.sample(&p, spec.modulus, &mut rng);
                Sample {
                    operands,
                    accepted: accepted
                        .iter()
                        .map(|value| trim(&limbs(value, spec.modulus.len())).to_vec())
                        .collect(),
                }
            })
            .collect()
    }

    fn operands(op: Op, key: &Field, rng: &mut Rng) -> Vec<Limbs> {
        let spec = spec(key);
        op.kind.operands(&big(spec.modulus), spec.modulus, rng)
    }

    fn offer<V: Visitor<Op, Field>>(row: Row<Op, Field, V>) -> Row<Op, Field, V> {
        libs::offer(row)
    }
}

/// An operation under comparison: its kind and the timing class of the functions that a row
/// compares.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Op {
    /// What the operation computes.
    pub kind: Kind,
    /// The functions of the row: those whose time depends on the values, or those whose time
    /// depends on no value.
    pub timing: Timing,
}

/// What an operation computes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Kind {
    /// `a + b`.
    Add,
    /// `a - b`.
    Sub,
    /// `a * b`.
    Mul,
    /// `a * a`.
    Square,
    /// `a^-1`.
    Invert,
    /// A square root of `a`.
    Sqrt,
}

/// The timing class of a function, read from its source.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Timing {
    /// Its time may depend on the values.
    Variable,
    /// Its time depends on no value.
    Constant,
}

impl Op {
    /// Every operation, in measurement order: each kind in variable time, then in constant time.
    pub const ALL: [Self; 12] = {
        let kinds = [
            Kind::Add,
            Kind::Sub,
            Kind::Mul,
            Kind::Square,
            Kind::Invert,
            Kind::Sqrt,
        ];
        let mut all = [Self {
            kind: Kind::Add,
            timing: Timing::Variable,
        }; 12];
        let mut index = 0;
        while index < kinds.len() {
            all[2 * index] = Self {
                kind: kinds[index],
                timing: Timing::Variable,
            };
            all[2 * index + 1] = Self {
                kind: kinds[index],
                timing: Timing::Constant,
            };
            index += 1;
        }
        all
    };
}

impl Kind {
    /// One operand set over the field of `p`, with the accepted results:
    /// - add, sub, mul: `a` and `b` uniform below `p`, one result;
    /// - square, invert: `a` uniform in `[1, p)`, one result;
    /// - sqrt: `a = r^2` for `r` uniform in `[1, p)`, the results `r` and `p - r`.
    fn sample(self, p: &BigUint, modulus: &[u64], rng: &mut Rng) -> (Vec<Limbs>, Vec<BigUint>) {
        let draw = |rng: &mut Rng| rng.below(modulus);
        let nonzero = |rng: &mut Rng| loop {
            let value = rng.below(modulus);
            if value.iter().any(|limb| *limb != 0) {
                return value;
            }
        };
        match self {
            Self::Add | Self::Sub | Self::Mul => {
                let (a, b) = (draw(rng), draw(rng));
                let (x, y) = (big(&a), big(&b));
                let result = match self {
                    Self::Add => (&x + &y) % p,
                    Self::Sub => (&x + p - &y) % p,
                    Self::Mul | Self::Square | Self::Invert | Self::Sqrt => &x * &y % p,
                };
                (vec![a, b], vec![result])
            }
            Self::Square => {
                let a = nonzero(rng);
                let x = big(&a);
                (vec![a], vec![&x * &x % p])
            }
            Self::Invert => {
                let a = nonzero(rng);
                let x = big(&a);
                (vec![a], vec![x.modpow(&(p - 2u8), p)])
            }
            Self::Sqrt => {
                let r = big(&nonzero(rng));
                let a = &r * &r % p;
                (vec![limbs(&a, modulus.len())], vec![p - &r, r])
            }
        }
    }

    /// One operand set over the field of `p`, drawn as [`sample`](Self::sample) draws it.
    fn operands(self, p: &BigUint, modulus: &[u64], rng: &mut Rng) -> Vec<Limbs> {
        let nonzero = |rng: &mut Rng| loop {
            let value = rng.below(modulus);
            if value.iter().any(|limb| *limb != 0) {
                return value;
            }
        };
        match self {
            Self::Add | Self::Sub | Self::Mul => vec![rng.below(modulus), rng.below(modulus)],
            Self::Square | Self::Invert => vec![nonzero(rng)],
            Self::Sqrt => {
                let r = big(&nonzero(rng));
                vec![limbs(&(&r * &r % p), modulus.len())]
            }
        }
    }
}

/// A kind of operation as a type. Its cases compile in the bench of its kind alone.
pub trait KindOf: Copy {
    /// The kind.
    const KIND: Kind;
    /// Whether this bench measures the kind.
    const MEASURED: bool = Self::KIND as u8 == crate::KIND as u8;
}

/// The kinds of operation, one type each.
pub mod kind {
    use super::{Kind, KindOf};

    /// Declares each kind `$name` of `Kind::$name`.
    macro_rules! kinds {
        ($($name:ident),*) => {
            $(
                #[doc = concat!("The kind `Kind::", stringify!($name), "`.")]
                #[derive(Clone, Copy)]
                pub struct $name;

                impl KindOf for $name {
                    const KIND: Kind = Kind::$name;
                }
            )*
        };
    }

    kinds!(Add, Sub, Mul, Square, Invert, Sqrt);
}

/// The operation of a case: the kind `K` in the row of `timing`.
#[derive(Clone, Copy)]
pub struct InRow<K> {
    timing: Timing,
    kind: PhantomData<K>,
}

impl<K: KindOf> Selected<Op> for InRow<K> {
    const MEASURED: bool = K::MEASURED;

    fn op(&self) -> Op {
        Op {
            kind: K::KIND,
            timing: self.timing,
        }
    }
}

/// The case of `kind` in the variable-time row.
pub const fn var<K: KindOf>(_: K) -> InRow<K> {
    InRow {
        timing: Timing::Variable,
        kind: PhantomData,
    }
}

/// The case of `kind` in the constant-time row.
pub const fn ct<K: KindOf>(_: K) -> InRow<K> {
    InRow {
        timing: Timing::Constant,
        kind: PhantomData,
    }
}

impl fmt::Display for Op {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let kind = match self.kind {
            Kind::Add => "add",
            Kind::Sub => "sub",
            Kind::Mul => "mul",
            Kind::Square => "square",
            Kind::Invert => "invert",
            Kind::Sqrt => "sqrt",
        };
        let timing = match self.timing {
            Timing::Variable => "vartime",
            Timing::Constant => "ct",
        };
        write!(formatter, "{kind}_{timing}")
    }
}

/// The value of little-endian limbs.
pub fn big(limbs: &[u64]) -> BigUint {
    BigUint::from_slice(
        &limbs
            .iter()
            .flat_map(|limb| [*limb as u32, (*limb >> 32) as u32])
            .collect::<Vec<u32>>(),
    )
}

/// `value` in exactly `len` little-endian limbs.
pub fn limbs(value: &BigUint, len: usize) -> Limbs {
    let mut limbs = value.to_u64_digits();
    assert!(limbs.len() <= len, "{value} needs more than {len} limbs");
    limbs.resize(len, 0);
    limbs
}
