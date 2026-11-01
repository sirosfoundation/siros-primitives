//! One adapter per crate: the function that the crate runs for each operation, at each width.
//!
//! Each crate uses its own fastest public function for an operation. A crate without one gets an
//! absent cell with the reason.

use core::num::NonZero;
use core::ops::Add;

use biguint::typenum::{Sum, U1024 as Bits1024, U2048 as Bits2048, U4096 as Bits4096};
use biguint::{Bits, Bits64, Bits128, Bits256, Bits384, Bits512, Uint};
use num_bigint::BigUint;

use super::{Cells, FromLimbs, Lib, Limbs, Op, Row, Visitor};

/// Generates [`WIDTHS`] and [`offer`] from one line per width: the width in bits, the `biguint`
/// width type, the native adapter, the `u64` limbs, the bytes, and the `uint` crate's adapter.
macro_rules! widths {
    ($($bits:literal: $biguint:ty, $native:ident, $limbs:literal, $bytes:literal, $parity:ident;)*) => {
        /// The widths in bits.
        pub const WIDTHS: [usize; [$($bits),*].len()] = [$($bits),*];

        /// Offers the cells of every crate in `row`.
        pub fn offer<V: Visitor>(row: Row<V>) -> Row<V> {
            match row.bits() {
                $(
                    $bits => row
                        .lib(Lib::Biguint, this_crate::<$biguint, V>)
                        .lib(Lib::Native, $native)
                        .lib(Lib::Ruint, ruint::<$bits, $limbs, V>)
                        .lib(Lib::CryptoBigint, crypto_bigint::<$limbs, V>)
                        .lib(Lib::Bnum, bnum::<$bytes, V>)
                        .lib(Lib::Uint, parity::$parity)
                        .lib(Lib::NumBigint, num_bigint),
                )*
                bits => panic!("no adapters at {bits} bits"),
            }
        }
    };
}

widths! {
    64: Bits64, native_u64, 1, 8, uint_64;
    128: Bits128, native_u128, 2, 16, uint_128;
    256: Bits256, native_none, 4, 32, uint_256;
    384: Bits384, native_none, 6, 48, uint_384;
    512: Bits512, native_none, 8, 64, uint_512;
    1024: Bits1024, native_none, 16, 128, uint_1024;
    2048: Bits2048, native_none, 32, 256, uint_2048;
    4096: Bits4096, native_none, 64, 512, uint_4096;
}

/// The two operands of a set.
fn pair<T: FromLimbs>(set: &[Limbs]) -> (T, T) {
    two(set)
}

/// The two operands of a set, as `A` and `B`.
fn two<A: FromLimbs, B: FromLimbs>(set: &[Limbs]) -> (A, B) {
    match set {
        [a, b] => (A::from_limbs(a), B::from_limbs(b)),
        _ => panic!("{} operands where two were drawn", set.len()),
    }
}

/// The three operands of a set, the third as `M`.
fn three<T: FromLimbs, M: FromLimbs>(set: &[Limbs]) -> (T, T, M) {
    match set {
        [a, b, modulus] => (T::from_limbs(a), T::from_limbs(b), M::from_limbs(modulus)),
        _ => panic!("{} operands where three were drawn", set.len()),
    }
}

/// Offers each `Op => method` as `a.method(b)` on two operands of type `$t`, taken by value.
macro_rules! by_value {
    ($cells:expr, $t:ty, $($op:ident => $method:ident),+ $(,)?) => {
        $cells$(.api(Op::$op, stringify!($method), pair::<$t>, |&(a, b)| a.$method(b)))+
    };
}

/// Offers each `Op => method` as `a.method(&b)` on two operands of type `$t`, taken by reference.
macro_rules! by_ref {
    ($cells:expr, $t:ty, $($op:ident => $method:ident),+ $(,)?) => {
        $cells$(.api(Op::$op, stringify!($method), pair::<$t>, |(a, b)| a.$method(b)))+
    };
}

/// `biguint`, the crate under test, at `N` bits.
fn this_crate<N, V: Visitor>(cells: Cells<V>) -> Cells<V>
where
    N: Bits + Add<N>,
    Sum<N, N>: Bits,
{
    cells
        .api(Op::Add, "+", pair::<Uint<N>>, |&(a, b)| a + b)
        .api(Op::Mul, "*", pair::<Uint<N>>, |&(a, b)| a * b)
        .api(Op::Div, "div_rem", pair::<Uint<N>>, |&(a, b)| {
            a.div_rem(b).map(|(quotient, _)| quotient)
        })
        .api(Op::Mod, "div_rem", pair::<Uint<N>>, |&(a, b)| {
            a.div_rem(b).map(|(_, remainder)| remainder)
        })
        .api(
            Op::MulMod,
            "mul_mod",
            three::<Uint<N>, Uint<N>>,
            |&(a, b, modulus)| a.mul_mod(b, modulus),
        )
}

/// Offers add, mul, div and mod of the primitive `$t`. The divisor is a `NonZero<$t>`.
macro_rules! primitive {
    ($cells:expr, $t:ty) => {
        by_value!($cells, $t, Add => wrapping_add, Mul => wrapping_mul)
            .api(Op::Div, "/ NonZero", two::<$t, NonZero<$t>>, |&(a, divisor)| a / divisor)
            .api(Op::Mod, "% NonZero", two::<$t, NonZero<$t>>, |&(a, divisor)| a % divisor)
    };
}

/// `u64`.
fn native_u64<V: Visitor>(cells: Cells<V>) -> Cells<V> {
    primitive!(cells, u64).absent(Op::MulMod, NO_MUL_MOD)
}

/// `u128`.
fn native_u128<V: Visitor>(cells: Cells<V>) -> Cells<V> {
    primitive!(cells, u128).absent(Op::MulMod, NO_MUL_MOD)
}

/// The reason for an absent mulmod case.
const NO_MUL_MOD: &str = "no modular product";

/// The widths above 128 bits, which have no primitive integer.
fn native_none<V: Visitor>(cells: Cells<V>) -> Cells<V> {
    Op::ALL.into_iter().fold(cells, |cells, op| {
        cells.absent(op, "no primitive of this width")
    })
}

/// `ruint` at `B` bits in `L` limbs.
fn ruint<const B: usize, const L: usize, V: Visitor>(cells: Cells<V>) -> Cells<V> {
    by_value!(
        cells,
        ruint::Uint<B, L>,
        Add => wrapping_add,
        Mul => wrapping_mul,
        Div => wrapping_div,
        Mod => wrapping_rem,
    )
    .api(
        Op::MulMod,
        "mul_mod",
        three::<ruint::Uint<B, L>, ruint::Uint<B, L>>,
        |&(a, b, modulus)| a.mul_mod(b, modulus),
    )
}

/// `crypto-bigint` at `L` limbs. Division and mulmod run its variable-time functions.
fn crypto_bigint<const L: usize, V: Visitor>(cells: Cells<V>) -> Cells<V> {
    use crypto_bigint::{NonZero, Uint};
    by_ref!(cells, Uint<L>, Add => wrapping_add, Mul => wrapping_mul)
        .api(
            Op::Div,
            "wrapping_div_vartime",
            two::<Uint<L>, NonZero<Uint<L>>>,
            |(a, divisor)| a.wrapping_div_vartime(divisor),
        )
        .api(
            Op::Mod,
            "rem_vartime",
            two::<Uint<L>, NonZero<Uint<L>>>,
            |(a, divisor)| a.rem_vartime(divisor),
        )
        .api(
            Op::MulMod,
            "mul_mod_vartime",
            three::<Uint<L>, NonZero<Uint<L>>>,
            |(a, b, modulus)| a.mul_mod_vartime(b, modulus),
        )
}

/// `bnum` at `N` bytes.
fn bnum<const N: usize, V: Visitor>(cells: Cells<V>) -> Cells<V> {
    by_value!(
        cells,
        bnum::Uint<N>,
        Add => wrapping_add,
        Mul => wrapping_mul,
        Div => wrapping_div,
        Mod => wrapping_rem,
    )
    .absent(Op::MulMod, NO_MUL_MOD)
}

/// `num-bigint`, which allocates each result and returns it exactly.
fn num_bigint<V: Visitor>(cells: Cells<V>) -> Cells<V> {
    cells
        .api(Op::Add, "&a + &b", pair::<BigUint>, |(a, b)| a + b)
        .api(Op::Mul, "&a * &b", pair::<BigUint>, |(a, b)| a * b)
        .api(Op::Div, "&a / &b", pair::<BigUint>, |(a, b)| a / b)
        .api(Op::Mod, "&a % &b", pair::<BigUint>, |(a, b)| a % b)
        .absent(Op::MulMod, NO_MUL_MOD)
}

/// The parity `uint` crate: one `construct_uint!` type per width, and the double-width type of
/// each product.
#[expect(
    clippy::manual_div_ceil,
    clippy::reversed_empty_ranges,
    reason = "the code that `construct_uint!` expands to"
)]
mod parity {
    use super::{Cells, FromLimbs, Limbs, Op, Visitor, pair};
    use crate::suite::ToLimbs;

    /// The `construct_uint!` type `$t` of `$words` words, with its limb conversions.
    macro_rules! parity_type {
        ($t:ident($words:tt)) => {
            uint::construct_uint! {
                struct $t($words);
            }

            impl FromLimbs for $t {
                fn from_limbs(limbs: &[u64]) -> Self {
                    Self(limbs.try_into().unwrap_or_else(|_| {
                        panic!("{} limbs for a {}-word integer", limbs.len(), $words)
                    }))
                }
            }

            impl ToLimbs for $t {
                fn to_limbs(&self) -> Option<Limbs> {
                    Some(self.0.to_vec())
                }
            }
        };
    }

    /// Offers add, mul, div and mod of `$t`.
    macro_rules! parity_ops {
        ($cells:expr, $t:ident) => {
            $cells
                .api(Op::Add, "overflowing_add", pair::<$t>, |&(a, b)| {
                    a.overflowing_add(b).0
                })
                .api(Op::Mul, "overflowing_mul", pair::<$t>, |&(a, b)| {
                    a.overflowing_mul(b).0
                })
                .api(Op::Div, "/", pair::<$t>, |&(a, b)| a / b)
                .api(Op::Mod, "%", pair::<$t>, |&(a, b)| a % b)
        };
    }

    /// Generates each type and its adapter. A line names the adapter, the type and its `u64`
    /// words.
    macro_rules! parity {
        ($($adapter:ident: $t:ident($words:tt);)*) => {
            $(
                parity_type!($t($words));

                /// `uint` at the width of `$t`.
                pub(super) fn $adapter<V: Visitor>(cells: Cells<V>) -> Cells<V> {
                    parity_ops!(cells, $t).absent(Op::MulMod, super::NO_MUL_MOD)
                }
            )*
        };
    }

    parity! {
        uint_64: U64(1);
        uint_128: U128(2);
        uint_256: U256(4);
        uint_384: U384(6);
        uint_512: U512(8);
        uint_1024: U1024(16);
        uint_2048: U2048(32);
        uint_4096: U4096(64);
    }
}
