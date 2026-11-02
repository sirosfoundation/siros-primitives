//! One adapter per crate: the function that the crate runs for each operation, at each width.
//!
//! Each crate uses its own fastest public function for an operation. A crate without one gets an
//! absent cell with the reason.

use core::num::NonZero;
use core::ops::Add;

use bench_compare::{Cells, Gpr, Limbs, Row, Visitor};
use biguint::typenum::{Sum, U1024 as Bits1024, U2048 as Bits2048, U4096 as Bits4096};
use biguint::{Bits, Bits64, Bits128, Bits256, Bits384, Bits512, Uint};
use num_bigint::BigUint;

use super::limbs::{FromLimbs, Offer};
use super::{Op, Width};

/// The cells of one crate in one row.
type Crate<V> = Cells<Op, Width, V>;

/// Generates [`WIDTHS`] and [`offer`] from one line per width: the width in bits, the `biguint`
/// width type, the native adapter, the `u64` limbs, the bytes, and the `uint` crate's adapter.
macro_rules! widths {
    ($($bits:literal: $biguint:ty, $native:ident, $limbs:literal, $bytes:literal, $parity:ident;)*) => {
        /// The widths in bits.
        pub const WIDTHS: [u32; [$($bits),*].len()] = [$($bits),*];

        /// Offers the cells of every crate in `row`.
        pub fn offer<V: Visitor<Op, Width>>(row: Row<Op, Width, V>) -> Row<Op, Width, V> {
            match row.key().bits {
                $(
                    $bits => row
                        .lib("biguint", this_crate::<$biguint, V>)
                        .lib("native", $native)
                        .lib("ruint", ruint::<$bits, $limbs, V>)
                        .lib("crypto-bigint", crypto_bigint::<$limbs, V>)
                        .lib("bnum", bnum::<$bytes, V>)
                        .lib("uint", parity::$parity)
                        .lib("num-bigint", num_bigint),
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
        $cells$(.case(Op::$op, stringify!($method), pair::<$t>, |&(a, b)| a.$method(b)))+
    };
}

/// Offers each `Op => method` as `a.method(&b)` on two operands of type `$t`, taken by reference.
macro_rules! by_ref {
    ($cells:expr, $t:ty, $($op:ident => $method:ident),+ $(,)?) => {
        $cells$(.case(Op::$op, stringify!($method), pair::<$t>, |(a, b)| a.$method(b)))+
    };
}

/// Offers each `Op => method` as [`by_value`] does, with its chain form through `$barrier`.
macro_rules! chained_by_value {
    ($cells:expr, $t:ty, $barrier:expr, $($op:ident => $method:ident),+ $(,)?) => {
        $cells$(.chained(Op::$op, stringify!($method), pair::<$t>, |&(a, b)| a.$method(b), $barrier))+
    };
}

/// Offers each `Op => method` as [`by_ref`] does, with its chain form through `$barrier`.
macro_rules! chained_by_ref {
    ($cells:expr, $t:ty, $barrier:expr, $($op:ident => $method:ident),+ $(,)?) => {
        $cells$(.chained(Op::$op, stringify!($method), pair::<$t>, |(a, b)| a.$method(b), $barrier))+
    };
}

/// `biguint`, the crate under test, at `N` bits.
fn this_crate<N, V: Visitor<Op, Width>>(cells: Crate<V>) -> Crate<V>
where
    N: Bits + Add<N>,
    Sum<N, N>: Bits,
{
    cells
        .chained(Op::Add, "+", pair::<Uint<N>>, |&(a, b)| a + b, Gpr)
        .case(Op::Mul, "*", pair::<Uint<N>>, |&(a, b)| a * b)
        .case(Op::Div, "div_rem", pair::<Uint<N>>, |&(a, b)| {
            a.div_rem(b).map(|(quotient, _)| quotient)
        })
        .case(Op::Mod, "div_rem", pair::<Uint<N>>, |&(a, b)| {
            a.div_rem(b).map(|(_, remainder)| remainder)
        })
        .case(
            Op::MulMod,
            "mul_mod",
            three::<Uint<N>, Uint<N>>,
            |&(a, b, modulus)| a.mul_mod(b, modulus),
        )
}

/// Offers add, mul, div and mod of the primitive `$t`. The divisor is a `NonZero<$t>`. The
/// hardware multiply takes the same time on any operands, so the chain of products applies.
macro_rules! primitive {
    ($cells:expr, $t:ty) => {
        chained_by_value!($cells, $t, Gpr, Add => wrapping_add, Mul => wrapping_mul)
            .case(Op::Div, "/ NonZero", two::<$t, NonZero<$t>>, |&(a, divisor)| a / divisor)
            .case(Op::Mod, "% NonZero", two::<$t, NonZero<$t>>, |&(a, divisor)| a % divisor)
    };
}

/// `u64`.
fn native_u64<V: Visitor<Op, Width>>(cells: Crate<V>) -> Crate<V> {
    primitive!(cells, u64).absent(Op::MulMod, NO_MUL_MOD)
}

/// `u128`.
fn native_u128<V: Visitor<Op, Width>>(cells: Crate<V>) -> Crate<V> {
    primitive!(cells, u128).absent(Op::MulMod, NO_MUL_MOD)
}

/// The reason for an absent mulmod case.
const NO_MUL_MOD: &str = "no modular product";

/// The widths above 128 bits, which have no primitive integer.
fn native_none<V: Visitor<Op, Width>>(cells: Crate<V>) -> Crate<V> {
    Op::ALL.into_iter().fold(cells, |cells, op| {
        cells.absent(op, "no primitive of this width")
    })
}

/// `ruint` at `B` bits in `L` limbs.
fn ruint<const B: usize, const L: usize, V: Visitor<Op, Width>>(cells: Crate<V>) -> Crate<V> {
    let cells = chained_by_value!(cells, ruint::Uint<B, L>, Gpr, Add => wrapping_add);
    by_value!(
        cells,
        ruint::Uint<B, L>,
        Mul => wrapping_mul,
        Div => wrapping_div,
        Mod => wrapping_rem,
    )
    .chained(
        Op::MulMod,
        "mul_mod",
        three::<ruint::Uint<B, L>, ruint::Uint<B, L>>,
        |&(a, b, modulus)| a.mul_mod(b, modulus),
        Gpr,
    )
}

/// `crypto-bigint` at `L` limbs. Division and mulmod run its variable-time functions.
fn crypto_bigint<const L: usize, V: Visitor<Op, Width>>(cells: Crate<V>) -> Crate<V> {
    use crypto_bigint::{NonZero, Uint};
    let cells = chained_by_ref!(cells, Uint<L>, Gpr, Add => wrapping_add);
    by_ref!(cells, Uint<L>, Mul => wrapping_mul)
        .case(
            Op::Div,
            "wrapping_div_vartime",
            two::<Uint<L>, NonZero<Uint<L>>>,
            |(a, divisor)| a.wrapping_div_vartime(divisor),
        )
        .case(
            Op::Mod,
            "rem_vartime",
            two::<Uint<L>, NonZero<Uint<L>>>,
            |(a, divisor)| a.rem_vartime(divisor),
        )
        .chained(
            Op::MulMod,
            "mul_mod_vartime",
            three::<Uint<L>, NonZero<Uint<L>>>,
            |(a, b, modulus)| a.mul_mod_vartime(b, modulus),
            Gpr,
        )
}

/// `bnum` at `N` bytes.
fn bnum<const N: usize, V: Visitor<Op, Width>>(cells: Crate<V>) -> Crate<V> {
    let cells = chained_by_value!(cells, bnum::Uint<N>, Gpr, Add => wrapping_add);
    by_value!(
        cells,
        bnum::Uint<N>,
        Mul => wrapping_mul,
        Div => wrapping_div,
        Mod => wrapping_rem,
    )
    .absent(Op::MulMod, NO_MUL_MOD)
}

/// `num-bigint`, which allocates each result and returns it exactly.
fn num_bigint<V: Visitor<Op, Width>>(cells: Crate<V>) -> Crate<V> {
    cells
        .chained(Op::Add, "&a + &b", pair::<BigUint>, |(a, b)| a + b, Gpr)
        .case(Op::Mul, "&a * &b", pair::<BigUint>, |(a, b)| a * b)
        .case(Op::Div, "&a / &b", pair::<BigUint>, |(a, b)| a / b)
        .case(Op::Mod, "&a % &b", pair::<BigUint>, |(a, b)| a % b)
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
    use bench_compare::{Gpr, Limbs, Visitor};

    use super::{Crate, Op, Width, pair};
    use crate::suite::limbs::{FromLimbs, Offer, ToLimbs};

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
                .chained(
                    Op::Add,
                    "overflowing_add",
                    pair::<$t>,
                    |&(a, b)| a.overflowing_add(b).0,
                    Gpr,
                )
                .case(Op::Mul, "overflowing_mul", pair::<$t>, |&(a, b)| {
                    a.overflowing_mul(b).0
                })
                .case(Op::Div, "/", pair::<$t>, |&(a, b)| a / b)
                .case(Op::Mod, "%", pair::<$t>, |&(a, b)| a % b)
        };
    }

    /// Generates each type and its adapter. A line names the adapter, the type and its `u64`
    /// words.
    macro_rules! parity {
        ($($adapter:ident: $t:ident($words:tt);)*) => {
            $(
                parity_type!($t($words));

                /// `uint` at the width of `$t`.
                pub(super) fn $adapter<V: Visitor<Op, Width>>(cells: Crate<V>) -> Crate<V> {
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
