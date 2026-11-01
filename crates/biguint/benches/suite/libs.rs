//! One adapter per crate: the function that the crate runs for each operation, at each width.
//!
//! Each crate uses its own fastest public function for an operation. A crate without one gets
//! the obvious composition of its public functions, or an absent cell with the reason.

use core::num::NonZero;
use core::ops::Add;

use biguint::typenum::Sum;
use biguint::{Bits, Bits64, Bits128, Bits256, Bits384, Bits512, Uint};
use bnum::cast::As;
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
                        .lib(Lib::Bnum, bnum::<$bytes, { 2 * $bytes }, V>)
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

fn this_crate<N, V: Visitor>(cells: Cells<V>) -> Cells<V>
where
    N: Bits + Add<N>,
    Sum<N, N>: Bits,
{
    cells
        .api(Op::Add, "+", pair::<Uint<N>>, |&(a, b)| a + b)
        .api(Op::Sub, "-", pair::<Uint<N>>, |&(a, b)| a - b)
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

/// Offers add, sub, mul, div and mod of the primitive `$t`. The divisor is a `NonZero<$t>`.
macro_rules! primitive {
    ($cells:expr, $t:ty) => {
        by_value!($cells, $t, Add => wrapping_add, Sub => wrapping_sub, Mul => wrapping_mul)
            .api(Op::Div, "/ NonZero", two::<$t, NonZero<$t>>, |&(a, divisor)| a / divisor)
            .api(Op::Mod, "% NonZero", two::<$t, NonZero<$t>>, |&(a, divisor)| a % divisor)
    };
}

fn native_u64<V: Visitor>(cells: Cells<V>) -> Cells<V> {
    primitive!(cells, u64).composed(
        Op::MulMod,
        "`u128` product, then `% NonZero<u128>`",
        three::<u64, NonZero<u64>>,
        |&(a, b, modulus)| {
            let product = u128::from(a) * u128::from(b);
            (product % NonZero::<u128>::from(modulus)) as u64
        },
    )
}

fn native_u128<V: Visitor>(cells: Cells<V>) -> Cells<V> {
    primitive!(cells, u128).absent(Op::MulMod, "no 256-bit primitive")
}

fn native_none<V: Visitor>(cells: Cells<V>) -> Cells<V> {
    Op::ALL.into_iter().fold(cells, |cells, op| {
        cells.absent(op, "no primitive of this width")
    })
}

fn ruint<const B: usize, const L: usize, V: Visitor>(cells: Cells<V>) -> Cells<V> {
    by_value!(
        cells,
        ruint::Uint<B, L>,
        Add => wrapping_add,
        Sub => wrapping_sub,
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

fn crypto_bigint<const L: usize, V: Visitor>(cells: Cells<V>) -> Cells<V> {
    use crypto_bigint::{NonZero, Uint};
    by_ref!(cells, Uint<L>, Add => wrapping_add, Sub => wrapping_sub, Mul => wrapping_mul)
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

/// `N` bytes wide; `W = 2 * N` is the width of the product in the composed mulmod.
fn bnum<const N: usize, const W: usize, V: Visitor>(cells: Cells<V>) -> Cells<V> {
    by_value!(
        cells,
        bnum::Uint<N>,
        Add => wrapping_add,
        Sub => wrapping_sub,
        Mul => wrapping_mul,
        Div => wrapping_div,
        Mod => wrapping_rem,
    )
    .composed(
        Op::MulMod,
        "`widening_mul`, the halves joined at twice the width, then `wrapping_rem`",
        three::<bnum::Uint<N>, bnum::Uint<N>>,
        |&(a, b, modulus)| bnum_mul_mod::<N, W>(a, b, modulus),
    )
}

/// `a * b mod modulus` from bnum's public API: the two halves of `widening_mul` joined into a
/// `W`-byte integer, reduced by `wrapping_rem`, then cast back to `N` bytes.
fn bnum_mul_mod<const N: usize, const W: usize>(
    a: bnum::Uint<N>,
    b: bnum::Uint<N>,
    modulus: bnum::Uint<N>,
) -> bnum::Uint<N> {
    let (low, high) = a.widening_mul(b);
    let mut bytes = [0; W];
    let (low_bytes, high_bytes) = bytes.split_at_mut(N);
    low_bytes.copy_from_slice(low.as_bytes());
    high_bytes.copy_from_slice(high.as_bytes());
    bnum::Uint::<W>::from_le_bytes(bytes)
        .wrapping_rem(modulus.as_::<bnum::Uint<W>>())
        .as_()
}

fn num_bigint<V: Visitor>(cells: Cells<V>) -> Cells<V> {
    cells
        .api(Op::Add, "&a + &b", pair::<BigUint>, |(a, b)| a + b)
        .api(Op::Sub, "&a - &b", pair::<BigUint>, |(a, b)| a - b)
        .api(Op::Mul, "&a * &b", pair::<BigUint>, |(a, b)| a * b)
        .api(Op::Div, "&a / &b", pair::<BigUint>, |(a, b)| a / b)
        .api(Op::Mod, "&a % &b", pair::<BigUint>, |(a, b)| a % b)
        .composed(
            Op::MulMod,
            "`&a * &b`, then `%`",
            three::<BigUint, BigUint>,
            |(a, b, modulus)| a * b % modulus,
        )
}

/// The parity `uint` crate: one `construct_uint!` type per width, and the double-width type of
/// each product.
#[expect(
    clippy::manual_div_ceil,
    clippy::reversed_empty_ranges,
    reason = "the code that `construct_uint!` expands to"
)]
mod parity {
    use super::{Cells, FromLimbs, Limbs, Op, Visitor, pair, three};
    use crate::suite::ToLimbs;

    /// Generates each type, its limb conversions, its `mul_mod`, and its adapter. A line names
    /// the adapter, the type, its `u64` words, and the type of its full product.
    macro_rules! parity {
        ($($adapter:ident: $t:ident($words:tt) => $wide:ident;)*) => {
            $(
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

                impl $t {
                    /// `self * rhs mod modulus`: the full product as `primitive-types` builds it
                    /// (`uint_full_mul_reg!`), the modulus zero-extended to the product's width,
                    /// then `%`.
                    fn mul_mod(self, rhs: Self, modulus: Self) -> Self {
                        let product = $wide(uint::uint_full_mul_reg!($t, $words, self, rhs));
                        let mut wide_modulus = [0; 2 * $words];
                        wide_modulus[..$words].copy_from_slice(&modulus.0);
                        let remainder = product % $wide(wide_modulus);
                        Self(core::array::from_fn(|index| remainder.0[index]))
                    }
                }

                pub(super) fn $adapter<V: Visitor>(cells: Cells<V>) -> Cells<V> {
                    cells
                        .api(Op::Add, "overflowing_add", pair::<$t>, |&(a, b)| {
                            a.overflowing_add(b).0
                        })
                        .api(Op::Sub, "overflowing_sub", pair::<$t>, |&(a, b)| {
                            a.overflowing_sub(b).0
                        })
                        .api(Op::Mul, "overflowing_mul", pair::<$t>, |&(a, b)| {
                            a.overflowing_mul(b).0
                        })
                        .api(Op::Div, "/", pair::<$t>, |&(a, b)| a / b)
                        .api(Op::Mod, "%", pair::<$t>, |&(a, b)| a % b)
                        .composed(
                            Op::MulMod,
                            "`uint_full_mul_reg!` (`full_mul` in primitive-types), then `%`",
                            three::<$t, $t>,
                            |&(a, b, modulus)| a.mul_mod(b, modulus),
                        )
                }
            )*
        };
    }

    uint::construct_uint! {
        struct U768(12);
    }

    uint::construct_uint! {
        struct U1024(16);
    }

    parity! {
        uint_64: U64(1) => U128;
        uint_128: U128(2) => U256;
        uint_256: U256(4) => U512;
        uint_384: U384(6) => U768;
        uint_512: U512(8) => U1024;
    }
}
