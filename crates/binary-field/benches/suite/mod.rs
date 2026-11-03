//! The suite: the operations, the fields, the operand sets with their results from the bit-serial
//! reference, and the adapters of every crate.

mod basis;
mod libs;
mod reference;

use core::fmt;
use core::marker::PhantomData;

use bench_compare::{BATCH, Limbs, Rng, Row, Sample, Selected, Suite, Visitor, trim};
use serde::{Deserialize, Serialize};

use reference::Modulus;

/// The comparison of `binary-field`.
pub struct BinaryField;

/// A row: a field, by name, and the degree of its modulus.
#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct Field {
    /// The name.
    pub field: String,
    /// The degree of the modulus: the bits of an element.
    pub bits: usize,
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

    fn ops() -> Vec<Op> {
        Op::ALL
            .into_iter()
            .filter(|op| crate::KINDS.contains(&op.kind))
            .collect()
    }

    fn keys() -> Vec<Field> {
        FIELDS
            .iter()
            .map(|spec| Field {
                field: spec.name.to_owned(),
                bits: spec.modulus.degree,
            })
            .collect()
    }

    /// The operand sets of `op` over the field, with the results of the reference. The seed
    /// depends on the kind of operation and the field only, so every crate in both rows of an
    /// operation sees the same operands.
    fn samples(op: Op, key: &Field) -> Vec<Sample> {
        let (index, spec) = spec(key);
        let mut rng = Rng(((op.kind as u64) << 32) | index as u64);
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

    fn operands(op: Op, key: &Field, rng: &mut Rng) -> Vec<Limbs> {
        op.operands(&spec(key).1.modulus, rng)
    }

    fn offer<V: Visitor<Op, Field>>(row: Row<Op, Field, V>) -> Row<Op, Field, V> {
        libs::offer(row)
    }
}

/// An operation under comparison: a kind of operation, run by the functions of one timing class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Op {
    /// What it computes.
    pub kind: Kind,
    /// Whether the functions of its row take time that depends on the values.
    pub timing: Timing,
}

/// What an operation computes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Kind {
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

/// The timing class of a function, which its source decides.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Timing {
    /// No branch and no memory address depends on a value.
    Constant,
    /// A branch or a memory address, such as a table lookup, depends on a value.
    Variable,
}

impl Timing {
    /// The other class.
    pub const fn other(self) -> Self {
        match self {
            Self::Constant => Self::Variable,
            Self::Variable => Self::Constant,
        }
    }
}

impl Kind {
    /// The operation of this kind in the row of `timing`.
    pub const fn in_row(self, timing: Timing) -> Op {
        Op { kind: self, timing }
    }
}

impl Op {
    /// Every operation, in measurement order: each kind in its constant-time row, then in its
    /// variable-time row.
    pub const ALL: [Self; 10] = {
        let kinds = [Kind::Add, Kind::Mul, Kind::Square, Kind::Invert, Kind::Sqrt];
        let mut all = [Kind::Add.in_row(Timing::Constant); 10];
        let mut index = 0;
        while index < 5 {
            all[2 * index] = kinds[index].in_row(Timing::Constant);
            all[2 * index + 1] = kinds[index].in_row(Timing::Variable);
            index += 1;
        }
        all
    };

    /// One operand set over the field of `modulus`, with its result:
    /// - add: `a` and `b` uniform;
    /// - mul: `a` and `b` uniform and nonzero, so that a chain of products never reaches zero;
    /// - square, invert: `a` uniform and nonzero;
    /// - sqrt: `a = r²` for `r` uniform and nonzero, whose square root is `r` alone.
    fn sample(self, modulus: &Modulus, rng: &mut Rng) -> (Vec<Limbs>, Limbs) {
        match self.kind {
            Kind::Add => {
                let (a, b) = (modulus.uniform(rng), modulus.uniform(rng));
                let mut sum = a.clone();
                reference::xor(&mut sum, &b);
                (vec![a, b], sum)
            }
            Kind::Mul => {
                let (a, b) = (modulus.nonzero(rng), modulus.nonzero(rng));
                let product = modulus.mul(&a, &b);
                (vec![a, b], product)
            }
            Kind::Square => {
                let a = modulus.nonzero(rng);
                let result = modulus.square(&a);
                (vec![a], result)
            }
            Kind::Invert => {
                let a = modulus.nonzero(rng);
                let result = modulus.inverse(&a);
                (vec![a], result)
            }
            Kind::Sqrt => {
                let root = modulus.nonzero(rng);
                (vec![modulus.square(&root)], root)
            }
        }
    }

    /// One operand set over the field of `modulus`, drawn as [`sample`](Self::sample) draws it.
    fn operands(self, modulus: &Modulus, rng: &mut Rng) -> Vec<Limbs> {
        match self.kind {
            Kind::Add => vec![modulus.uniform(rng), modulus.uniform(rng)],
            Kind::Mul => vec![modulus.nonzero(rng), modulus.nonzero(rng)],
            Kind::Square | Kind::Invert => vec![modulus.nonzero(rng)],
            Kind::Sqrt => vec![modulus.square(&modulus.nonzero(rng))],
        }
    }
}

/// The kind, and `_vartime` for the variable-time row.
/// Whether this bench measures `kind`: it is in the bench's `KINDS`.
const fn measures(kind: Kind) -> bool {
    let mut index = 0;
    while index < crate::KINDS.len() {
        if crate::KINDS[index] as u8 == kind as u8 {
            return true;
        }
        index += 1;
    }
    false
}

/// A kind of operation as a type. Its cases compile in the benches that measure its kind.
pub trait KindOf: Copy {
    /// The kind.
    const KIND: Kind;
    /// Whether this bench measures the kind.
    const MEASURED: bool = measures(Self::KIND);

    /// The case of this kind in the row of `timing`.
    fn in_row(self, timing: Timing) -> InRow<Self> {
        InRow {
            timing,
            kind: PhantomData,
        }
    }
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

    kinds!(Add, Mul, Square, Invert, Sqrt);
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
        K::KIND.in_row(self.timing)
    }
}

impl fmt::Display for Op {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self.kind {
            Kind::Add => "add",
            Kind::Mul => "mul",
            Kind::Square => "square",
            Kind::Invert => "invert",
            Kind::Sqrt => "sqrt",
        })?;
        match self.timing {
            Timing::Constant => Ok(()),
            Timing::Variable => formatter.write_str("_vartime"),
        }
    }
}
