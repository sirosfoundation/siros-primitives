//! One adapter per crate: the function that the crate runs for each operation, at each width.
//!
//! Each crate uses its own fastest public function for an operation. A crate without one gets an
//! absent cell with the reason. Each row compares functions of one timing class, the class of
//! `biguint`'s function: a crate's function of the other class leaves its cell absent.

use core::num::NonZero;
use core::ops::Add;

use bench_compare::{Cells, Chain, Limbs, Row, Visitor};
use biguint::typenum::{Sum, U1024 as Bits1024, U2048 as Bits2048, U4096 as Bits4096};
use biguint::{Bits, Bits64, Bits128, Bits256, Bits384, Bits512, NonZeroUint, Uint};
use num_bigint::BigUint;

use super::limbs::{FromLimbs, Offer, ToLimbs};
use super::{Op, Width};

/// The cells of one crate in one row.
type Crate<V> = Cells<Op, Width, V>;

/// Whether the time of a function depends on the values it computes on.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Timing {
    /// The time is fixed by the width.
    Fixed,
    /// The time depends on the values.
    Variable,
}

/// The timing of `biguint`'s function for `op` at `bits` bits, the class of the row. Its sum and
/// its products up to 1024 bits are straight-line code. Its products from 2048 bits run Karatsuba,
/// which compares the halves and stops each carry where it ends; its divisions and modular
/// operations branch on the values.
const fn row_timing(op: Op, bits: u32) -> Timing {
    match op {
        Op::Add => Timing::Fixed,
        Op::Mul if bits <= 1024 => Timing::Fixed,
        Op::Mul | Op::Div | Op::Mod | Op::MulMod | Op::AddMod | Op::SubMod => Timing::Variable,
    }
}

/// The reason for the absent cell of a function of the class `timing` for `op` at `bits` bits;
/// `None` when the row has its class.
const fn mismatch(op: Op, bits: u32, timing: Timing) -> Option<&'static str> {
    match (row_timing(op, bits), timing) {
        (Timing::Fixed, Timing::Fixed) | (Timing::Variable, Timing::Variable) => None,
        (Timing::Fixed, Timing::Variable) => Some(VARIABLE_IN_FIXED_ROW),
        (Timing::Variable, Timing::Fixed) => Some(FIXED_IN_VARIABLE_ROW),
    }
}

/// Offers cases of a stated timing class.
trait Timed<V> {
    /// Offers the case that runs the crate's `function` for `op`, whose time is of the class
    /// `timing`, or an absent cell when the row's class differs.
    fn timed<I, R: ToLimbs>(
        self,
        op: Op,
        timing: Timing,
        function: &'static str,
        prepare: impl Fn(&[Limbs]) -> I,
        run: impl Fn(&I) -> R + Copy,
    ) -> Self;

    /// Offers the case of [`timed`](Self::timed) with its chain form, `x = op(x, y)`.
    fn timed_chained<I: Chain<R>, R: ToLimbs>(
        self,
        op: Op,
        timing: Timing,
        function: &'static str,
        prepare: impl Fn(&[Limbs]) -> I,
        run: impl Fn(&I) -> R + Copy,
    ) -> Self;
}

impl<V: Visitor<Op, Width>> Timed<V> for Crate<V> {
    fn timed<I, R: ToLimbs>(
        self,
        op: Op,
        timing: Timing,
        function: &'static str,
        prepare: impl Fn(&[Limbs]) -> I,
        run: impl Fn(&I) -> R + Copy,
    ) -> Self {
        match mismatch(op, self.key().bits, timing) {
            None => self.case(op, function, prepare, run),
            Some(reason) => self.absent(op, reason),
        }
    }

    fn timed_chained<I: Chain<R>, R: ToLimbs>(
        self,
        op: Op,
        timing: Timing,
        function: &'static str,
        prepare: impl Fn(&[Limbs]) -> I,
        run: impl Fn(&I) -> R + Copy,
    ) -> Self {
        match mismatch(op, self.key().bits, timing) {
            None => self.chained(op, function, prepare, run),
            Some(reason) => self.absent(op, reason),
        }
    }
}

/// The reason for a cell whose function's time depends on the values, in a fixed-time row.
const VARIABLE_IN_FIXED_ROW: &str = "variable time in a fixed-time row";

/// The reason for a cell whose function's time is fixed, in a variable-time row.
const FIXED_IN_VARIABLE_ROW: &str = "fixed time in a variable-time row";

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

/// Offers each `Op => method` as `a.method(b)` on two operands of type `$t`, taken by value, with
/// the timing class of each.
macro_rules! by_value {
    ($cells:expr, $t:ty, $($op:ident: $timing:expr => $method:ident),+ $(,)?) => {
        $cells$(.timed(Op::$op, $timing, stringify!($method), pair::<$t>, #[inline(always)] |&(a, b)| a.$method(b)))+
    };
}

/// Offers each `Op => method` as [`by_value`] does, with its chain form.
macro_rules! chained_by_value {
    ($cells:expr, $t:ty, $($op:ident: $timing:expr => $method:ident),+ $(,)?) => {
        $cells$(.timed_chained(Op::$op, $timing, stringify!($method), pair::<$t>, #[inline(always)] |&(a, b)| a.$method(b)))+
    };
}

/// `biguint`, the crate under test, at `N` bits. Its timing defines each row's.
fn this_crate<N, V: Visitor<Op, Width>>(cells: Crate<V>) -> Crate<V>
where
    N: Bits + Add<N>,
    Sum<N, N>: Bits,
{
    cells
        .chained(
            Op::Add,
            "+",
            pair::<Uint<N>>,
            #[inline(always)]
            |&(a, b)| a + b,
        )
        .case(
            Op::Mul,
            "*",
            pair::<Uint<N>>,
            #[inline(always)]
            |&(a, b)| a * b,
        )
        .case(
            Op::Div,
            "div_rem",
            two::<Uint<N>, NonZeroUint<N>>,
            #[inline(always)]
            |&(a, divisor)| a.div_rem(divisor).0,
        )
        .case(
            Op::Mod,
            "div_rem",
            two::<Uint<N>, NonZeroUint<N>>,
            #[inline(always)]
            |&(a, divisor)| a.div_rem(divisor).1,
        )
        .chained(
            Op::MulMod,
            "mul_mod",
            three::<Uint<N>, NonZeroUint<N>>,
            #[inline(always)]
            |&(a, b, modulus)| a.mul_mod(b, modulus),
        )
        .chained(
            Op::AddMod,
            "add_mod",
            three::<Uint<N>, NonZeroUint<N>>,
            #[inline(always)]
            |&(a, b, modulus)| a.add_mod(b, modulus),
        )
        .chained(
            Op::SubMod,
            "sub_mod",
            three::<Uint<N>, NonZeroUint<N>>,
            #[inline(always)]
            |&(a, b, modulus)| a.sub_mod(b, modulus),
        )
}

/// Offers add, mul, div and mod of the primitive `$t`. The divisor is a `NonZero<$t>`; the
/// hardware divide takes a time that depends on the values.
macro_rules! primitive {
    ($cells:expr, $t:ty) => {
        chained_by_value!($cells, $t, Add: Timing::Fixed => wrapping_add, Mul: Timing::Fixed => wrapping_mul)
            .timed(
                Op::Div,
                Timing::Variable,
                "/ NonZero",
                two::<$t, NonZero<$t>>,
                #[inline(always)] |&(a, divisor)| a / divisor,
            )
            .timed(
                Op::Mod,
                Timing::Variable,
                "% NonZero",
                two::<$t, NonZero<$t>>,
                #[inline(always)] |&(a, divisor)| a % divisor,
            )
            .absent(Op::MulMod, NO_MUL_MOD)
            .absent(Op::AddMod, NO_ADD_MOD)
            .absent(Op::SubMod, NO_ADD_MOD)
    };
}

/// `u64`.
fn native_u64<V: Visitor<Op, Width>>(cells: Crate<V>) -> Crate<V> {
    primitive!(cells, u64)
}

/// `u128`.
fn native_u128<V: Visitor<Op, Width>>(cells: Crate<V>) -> Crate<V> {
    primitive!(cells, u128)
}

/// The reason for an absent mulmod case.
const NO_MUL_MOD: &str = "no modular product";

/// The reason for an absent add_mod or sub_mod case.
const NO_ADD_MOD: &str = "no modular sum or difference";

/// The widths above 128 bits, which have no primitive integer.
fn native_none<V: Visitor<Op, Width>>(cells: Crate<V>) -> Crate<V> {
    Op::ALL.into_iter().fold(cells, |cells, op| {
        cells.absent(op, "no primitive of this width")
    })
}

/// `ruint` at `B` bits in `L` limbs. Its sum is one carry chain. Its product is unrolled
/// schoolbook up to 8 limbs; above, it skips zero limbs and stops each carry where it ends. Its
/// division, `mul_mod` and `add_mod` branch on the values. It has no modular difference: the
/// sub_mod case adds the negation of the reduced subtrahend.
fn ruint<const B: usize, const L: usize, V: Visitor<Op, Width>>(cells: Crate<V>) -> Crate<V> {
    let product = if L <= 8 {
        Timing::Fixed
    } else {
        Timing::Variable
    };
    let cells = chained_by_value!(cells, ruint::Uint<B, L>, Add: Timing::Fixed => wrapping_add);
    by_value!(
        cells,
        ruint::Uint<B, L>,
        Mul: product => wrapping_mul,
        Div: Timing::Variable => wrapping_div,
        Mod: Timing::Variable => wrapping_rem,
    )
    .timed_chained(
        Op::MulMod,
        Timing::Variable,
        "mul_mod",
        three::<ruint::Uint<B, L>, ruint::Uint<B, L>>,
        #[inline(always)]
        |&(a, b, modulus)| a.mul_mod(b, modulus),
    )
    .timed_chained(
        Op::AddMod,
        Timing::Variable,
        "add_mod",
        three::<ruint::Uint<B, L>, ruint::Uint<B, L>>,
        #[inline(always)]
        |&(a, b, modulus)| a.add_mod(b, modulus),
    )
    .timed_chained(
        Op::SubMod,
        Timing::Variable,
        "a.add_mod(m - b.reduce_mod(m), m)",
        three::<ruint::Uint<B, L>, ruint::Uint<B, L>>,
        #[inline(always)]
        |&(a, b, modulus)| a.add_mod(modulus - b.reduce_mod(modulus), modulus),
    )
}

/// `crypto-bigint` at `L` limbs, constant-time: its sum, product, `add_mod` and `sub_mod` take a
/// fixed time. Division and mulmod run its variable-time functions.
fn crypto_bigint<const L: usize, V: Visitor<Op, Width>>(cells: Crate<V>) -> Crate<V> {
    use crypto_bigint::{NonZero, Uint};
    cells
        .timed_chained(
            Op::Add,
            Timing::Fixed,
            "wrapping_add",
            pair::<Uint<L>>,
            #[inline(always)]
            |(a, b)| a.wrapping_add(b),
        )
        .timed(
            Op::Mul,
            Timing::Fixed,
            "wrapping_mul",
            pair::<Uint<L>>,
            #[inline(always)]
            |(a, b)| a.wrapping_mul(b),
        )
        .timed(
            Op::Div,
            Timing::Variable,
            "wrapping_div_vartime",
            two::<Uint<L>, NonZero<Uint<L>>>,
            #[inline(always)]
            |(a, divisor)| a.wrapping_div_vartime(divisor),
        )
        .timed(
            Op::Mod,
            Timing::Variable,
            "rem_vartime",
            two::<Uint<L>, NonZero<Uint<L>>>,
            #[inline(always)]
            |(a, divisor)| a.rem_vartime(divisor),
        )
        .timed_chained(
            Op::MulMod,
            Timing::Variable,
            "mul_mod_vartime",
            three::<Uint<L>, NonZero<Uint<L>>>,
            #[inline(always)]
            |(a, b, modulus)| a.mul_mod_vartime(b, modulus),
        )
        .timed(
            Op::AddMod,
            Timing::Fixed,
            "add_mod",
            three::<Uint<L>, NonZero<Uint<L>>>,
            #[inline(always)]
            |(a, b, modulus)| a.add_mod(b, modulus),
        )
        .timed(
            Op::SubMod,
            Timing::Fixed,
            "sub_mod",
            three::<Uint<L>, NonZero<Uint<L>>>,
            #[inline(always)]
            |(a, b, modulus)| a.sub_mod(b, modulus),
        )
}

/// `bnum` at `N` bytes: its sum and its wrapping product run every limb; its division branches on
/// the values.
fn bnum<const N: usize, V: Visitor<Op, Width>>(cells: Crate<V>) -> Crate<V> {
    let cells = chained_by_value!(cells, bnum::Uint<N>, Add: Timing::Fixed => wrapping_add);
    by_value!(
        cells,
        bnum::Uint<N>,
        Mul: Timing::Fixed => wrapping_mul,
        Div: Timing::Variable => wrapping_div,
        Mod: Timing::Variable => wrapping_rem,
    )
    .absent(Op::MulMod, NO_MUL_MOD)
    .absent(Op::AddMod, NO_ADD_MOD)
    .absent(Op::SubMod, NO_ADD_MOD)
}

/// `num-bigint`, which allocates each result and returns it exactly. Every operation's time depends
/// on the lengths of its normalized operands.
fn num_bigint<V: Visitor<Op, Width>>(cells: Crate<V>) -> Crate<V> {
    cells
        .timed(
            Op::Add,
            Timing::Variable,
            "&a + &b",
            pair::<BigUint>,
            #[inline(always)]
            |(a, b)| a + b,
        )
        .timed(
            Op::Mul,
            Timing::Variable,
            "&a * &b",
            pair::<BigUint>,
            #[inline(always)]
            |(a, b)| a * b,
        )
        .timed(
            Op::Div,
            Timing::Variable,
            "&a / &b",
            pair::<BigUint>,
            #[inline(always)]
            |(a, b)| a / b,
        )
        .timed(
            Op::Mod,
            Timing::Variable,
            "&a % &b",
            pair::<BigUint>,
            #[inline(always)]
            |(a, b)| a % b,
        )
        .absent(Op::MulMod, NO_MUL_MOD)
        .timed(
            Op::AddMod,
            Timing::Variable,
            "(&a + &b) % &m",
            three::<BigUint, BigUint>,
            #[inline(always)]
            |(a, b, modulus)| (a + b) % modulus,
        )
        .timed(
            Op::SubMod,
            Timing::Variable,
            "(&a + &m - &b) % &m",
            three::<BigUint, BigUint>,
            #[inline(always)]
            |(a, b, modulus)| (a + modulus - b) % modulus,
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
    use bench_compare::{Limbs, Visitor};

    use super::{Crate, NO_ADD_MOD, NO_MUL_MOD, Op, Timed, Timing, Width, pair};
    use crate::suite::limbs::{FromLimbs, ToLimbs};

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

    /// Offers add, mul, div and mod of `$t` of `$words` words. Its sum adds the carry only into
    /// a limb that receives one, from the second word on; at 8 words its product skips the steps
    /// of zero limbs that receive no carry. Its division branches on the values.
    macro_rules! parity_ops {
        ($cells:expr, $t:ident, $words:tt) => {
            $cells
                .timed_chained(
                    Op::Add,
                    if $words == 1 {
                        Timing::Fixed
                    } else {
                        Timing::Variable
                    },
                    "overflowing_add",
                    pair::<$t>,
                    #[inline(always)]
                    |&(a, b)| a.overflowing_add(b).0,
                )
                .timed(
                    Op::Mul,
                    if $words == 8 {
                        Timing::Variable
                    } else {
                        Timing::Fixed
                    },
                    "overflowing_mul",
                    pair::<$t>,
                    #[inline(always)]
                    |&(a, b)| a.overflowing_mul(b).0,
                )
                .timed(
                    Op::Div,
                    Timing::Variable,
                    "/",
                    pair::<$t>,
                    #[inline(always)]
                    |&(a, b)| a / b,
                )
                .timed(
                    Op::Mod,
                    Timing::Variable,
                    "%",
                    pair::<$t>,
                    #[inline(always)]
                    |&(a, b)| a % b,
                )
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
                    parity_ops!(cells, $t, $words)
                        .absent(Op::MulMod, NO_MUL_MOD)
                        .absent(Op::AddMod, NO_ADD_MOD)
                        .absent(Op::SubMod, NO_ADD_MOD)
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
