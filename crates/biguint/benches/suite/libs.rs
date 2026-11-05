//! One adapter per crate: the function that the crate runs for each operation, at each width.
//!
//! Each crate uses its own fastest public function for an operation. A crate without one gets an
//! absent cell with the reason. Each row compares functions of one timing class, the class of
//! `biguint`'s function: a crate's function of the other class leaves its cell absent.

use core::num::NonZero;
use core::ops::Add;

use bench_compare::{Cells, Chain, Limbs, Row, Visitor};
use biguint::typenum::{Sum, U1024 as Bits1024, U2048 as Bits2048, U4096 as Bits4096};
use biguint::{Bits, Bits64, Bits128, Bits256, Bits384, Bits512, ConstantTime, NonZeroUint, Uint};
use num_bigint::BigUint;

use super::limbs::{FromLimbs, Offer, ToLimbs};
use super::{Base, Ct, Op, Plain, Width, op};

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
        Op::Add
        | Op::AddCt
        | Op::MulCt
        | Op::DivCt
        | Op::ModCt
        | Op::MulModCt
        | Op::AddModCt
        | Op::SubModCt
        | Op::AddModReducedCt
        | Op::SubModReducedCt => Timing::Fixed,
        Op::Mul if bits <= 1024 => Timing::Fixed,
        Op::Mul
        | Op::Div
        | Op::Mod
        | Op::MulMod
        | Op::AddMod
        | Op::SubMod
        | Op::AddModReduced
        | Op::SubModReduced => Timing::Variable,
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

/// Offers cases of a stated timing class: a competitor's function `f` for the operation `op`
/// fills the table of `op` when its class is the row's, and the constant-time table of `op` when
/// its time is fixed.
trait Timed<V> {
    /// Offers the cases that run the crate's `function` for `op`, whose time is of the class
    /// `timing`, or absent cells where the class differs.
    fn timed<B: Base, I, R: ToLimbs>(
        self,
        op: B,
        timing: Timing,
        function: &'static str,
        prepare: impl Fn(&[Limbs]) -> I + Copy,
        run: impl Fn(&I) -> R + Copy,
    ) -> Self;

    /// Offers the cases of [`timed`](Self::timed) with their chain form, `x = op(x, y)`.
    fn timed_chained<B: Base, I: Chain<R>, R: ToLimbs>(
        self,
        op: B,
        timing: Timing,
        function: &'static str,
        prepare: impl Fn(&[Limbs]) -> I + Copy,
        run: impl Fn(&I) -> R + Copy,
    ) -> Self;

    /// Offers the variable-time `function` for `op` in the table of `op` alone, or an absent cell
    /// where the row's class differs; the crate offers its constant-time cell itself.
    fn variable<B: Base, I, R: ToLimbs>(
        self,
        op: B,
        function: &'static str,
        prepare: impl Fn(&[Limbs]) -> I,
        run: impl Fn(&I) -> R + Copy,
    ) -> Self;

    /// Offers the case of [`variable`](Self::variable) with its chain form.
    fn variable_chained<B: Base, I: Chain<R>, R: ToLimbs>(
        self,
        op: B,
        function: &'static str,
        prepare: impl Fn(&[Limbs]) -> I,
        run: impl Fn(&I) -> R + Copy,
    ) -> Self;

    /// Records the absent cells of `op` in both tables, for `reason`.
    fn missing(self, op: Op, reason: &'static str) -> Self;

    /// Offers a modular sum or difference `function` of the class `timing` as
    /// [`timed`](Self::timed) does, twice: on the rows of `any`, whose operands may take any
    /// value, deviating as `reduced_only` says when the function takes only operands below the
    /// modulus; and on the rows of `reduced`, whose operands lie below it.
    #[expect(
        clippy::too_many_arguments,
        reason = "the arguments of `timed`, for two rows"
    )]
    fn timed_modular<B: Base, BR: Base, I, R: ToLimbs>(
        self,
        any: B,
        reduced: BR,
        reduced_only: Option<&'static str>,
        timing: Timing,
        function: &'static str,
        prepare: impl Fn(&[Limbs]) -> I + Copy,
        run: impl Fn(&I) -> R + Copy,
    ) -> Self;
}

impl<V: Visitor<Op, Width>> Timed<V> for Crate<V> {
    fn timed<B: Base, I, R: ToLimbs>(
        self,
        op: B,
        timing: Timing,
        function: &'static str,
        prepare: impl Fn(&[Limbs]) -> I + Copy,
        run: impl Fn(&I) -> R + Copy,
    ) -> Self {
        let cells = match mismatch(B::OP, self.key().bits, timing) {
            None => self.case(Plain(op), function, prepare, run),
            Some(reason) => self.absent(B::OP, reason),
        };
        match timing {
            Timing::Fixed => cells.case(Ct(op), function, prepare, run),
            Timing::Variable => cells.absent(B::OP.constant_time(), VARIABLE_TIME),
        }
    }

    fn timed_chained<B: Base, I: Chain<R>, R: ToLimbs>(
        self,
        op: B,
        timing: Timing,
        function: &'static str,
        prepare: impl Fn(&[Limbs]) -> I + Copy,
        run: impl Fn(&I) -> R + Copy,
    ) -> Self {
        let cells = match mismatch(B::OP, self.key().bits, timing) {
            None => self.chained(Plain(op), function, prepare, run),
            Some(reason) => self.absent(B::OP, reason),
        };
        match timing {
            Timing::Fixed => cells.chained(Ct(op), function, prepare, run),
            Timing::Variable => cells.absent(B::OP.constant_time(), VARIABLE_TIME),
        }
    }

    fn variable<B: Base, I, R: ToLimbs>(
        self,
        op: B,
        function: &'static str,
        prepare: impl Fn(&[Limbs]) -> I,
        run: impl Fn(&I) -> R + Copy,
    ) -> Self {
        match mismatch(B::OP, self.key().bits, Timing::Variable) {
            None => self.case(Plain(op), function, prepare, run),
            Some(reason) => self.absent(B::OP, reason),
        }
    }

    fn variable_chained<B: Base, I: Chain<R>, R: ToLimbs>(
        self,
        op: B,
        function: &'static str,
        prepare: impl Fn(&[Limbs]) -> I,
        run: impl Fn(&I) -> R + Copy,
    ) -> Self {
        match mismatch(B::OP, self.key().bits, Timing::Variable) {
            None => self.chained(Plain(op), function, prepare, run),
            Some(reason) => self.absent(B::OP, reason),
        }
    }

    fn missing(self, op: Op, reason: &'static str) -> Self {
        self.absent(op, reason).absent(op.constant_time(), reason)
    }

    fn timed_modular<B: Base, BR: Base, I, R: ToLimbs>(
        self,
        any: B,
        reduced: BR,
        reduced_only: Option<&'static str>,
        timing: Timing,
        function: &'static str,
        prepare: impl Fn(&[Limbs]) -> I + Copy,
        run: impl Fn(&I) -> R + Copy,
    ) -> Self {
        let deviating = |cells: Self| match reduced_only {
            Some(deviation) => cells.deviating(deviation),
            None => cells,
        };
        let cells = match mismatch(B::OP, self.key().bits, timing) {
            None => deviating(self).case(Plain(any), function, prepare, run),
            Some(reason) => self.absent(B::OP, reason),
        };
        let cells = match timing {
            Timing::Fixed => deviating(cells).case(Ct(any), function, prepare, run),
            Timing::Variable => cells.absent(B::OP.constant_time(), VARIABLE_TIME),
        };
        cells.timed(reduced, timing, function, prepare, run)
    }
}

/// The deviation of a composition of a sum or difference and one correction, which brings only
/// operands below the modulus below it.
const COMPOSITION_FOR_REDUCED: &str = "unchecked precondition: operands must be below m; other inputs \
     give a wrong result (one subtraction or addition of m)";

/// The deviation of `crypto-bigint`'s `add_mod`, whose documentation assumes that `self + rhs` as
/// an unbounded integer is below `2p`.
const CRYPTO_BIGINT_ADD_MOD: &str = "unchecked precondition: operands must be below m; other inputs \
     give a wrong result (`Uint::add_mod`: \"Assumes `self + rhs` as unbounded integer is `< 2p`\")";

/// The deviation of `crypto-bigint`'s `sub_mod`, whose documentation assumes that `self - rhs` as
/// an unbounded signed integer lies in `[-p, p)`.
const CRYPTO_BIGINT_SUB_MOD: &str = "unchecked precondition: operands must be below m; other inputs \
     give a wrong result (`Uint::sub_mod`: \"Assumes `self - rhs` as unbounded signed integer is in \
     `[-p, p)`\")";

/// The reason for an absent constant-time cell of a function whose time depends on the values.
const VARIABLE_TIME: &str = "variable time";

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
        $cells$(.timed(op::$op, $timing, stringify!($method), pair::<$t>, #[inline(always)] |&(a, b)| a.$method(b)))+
    };
}

/// Offers each `Op => method` as [`by_value`] does, with its chain form.
macro_rules! chained_by_value {
    ($cells:expr, $t:ty, $($op:ident: $timing:expr => $method:ident),+ $(,)?) => {
        $cells$(.timed_chained(op::$op, $timing, stringify!($method), pair::<$t>, #[inline(always)] |&(a, b)| a.$method(b)))+
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
            Plain(op::Add),
            "+",
            pair::<Uint<N>>,
            #[inline(always)]
            |&(a, b)| a + b,
        )
        .case(
            Plain(op::Mul),
            "*",
            pair::<Uint<N>>,
            #[inline(always)]
            |&(a, b)| a * b,
        )
        .case(
            Plain(op::Div),
            "div_rem",
            two::<Uint<N>, NonZeroUint<N>>,
            #[inline(always)]
            |&(a, divisor)| a.div_rem(divisor).0,
        )
        .case(
            Plain(op::Mod),
            "div_rem",
            two::<Uint<N>, NonZeroUint<N>>,
            #[inline(always)]
            |&(a, divisor)| a.div_rem(divisor).1,
        )
        .chained(
            Plain(op::MulMod),
            "mul_mod",
            three::<Uint<N>, NonZeroUint<N>>,
            #[inline(always)]
            |&(a, b, modulus)| a.mul_mod(b, modulus),
        )
        .chained(
            Plain(op::AddMod),
            "add_mod",
            three::<Uint<N>, NonZeroUint<N>>,
            #[inline(always)]
            |&(a, b, modulus)| a.add_mod(b, modulus),
        )
        .chained(
            Plain(op::SubMod),
            "sub_mod",
            three::<Uint<N>, NonZeroUint<N>>,
            #[inline(always)]
            |&(a, b, modulus)| a.sub_mod(b, modulus),
        )
        .chained(
            Ct(op::Add),
            "ConstantTime +",
            pair::<ConstantTime<Uint<N>>>,
            #[inline(always)]
            |&(a, b)| a + b,
        )
        .case(
            Ct(op::Mul),
            "ConstantTime *",
            pair::<ConstantTime<Uint<N>>>,
            #[inline(always)]
            |&(a, b)| a * b,
        )
        .case(
            Ct(op::Div),
            "ConstantTime::div_rem",
            two::<ConstantTime<Uint<N>>, NonZeroUint<N>>,
            #[inline(always)]
            |&(a, divisor)| a.div_rem(divisor).0,
        )
        .case(
            Ct(op::Mod),
            "ConstantTime::div_rem",
            two::<ConstantTime<Uint<N>>, NonZeroUint<N>>,
            #[inline(always)]
            |&(a, divisor)| a.div_rem(divisor).1,
        )
        .chained(
            Ct(op::MulMod),
            "ConstantTime::mul_mod",
            three::<ConstantTime<Uint<N>>, NonZeroUint<N>>,
            #[inline(always)]
            |&(a, b, modulus)| a.mul_mod(b, modulus),
        )
        .chained(
            Ct(op::AddMod),
            "ConstantTime::add_mod",
            three::<ConstantTime<Uint<N>>, NonZeroUint<N>>,
            #[inline(always)]
            |&(a, b, modulus)| a.add_mod(b, modulus),
        )
        .chained(
            Ct(op::SubMod),
            "ConstantTime::sub_mod",
            three::<ConstantTime<Uint<N>>, NonZeroUint<N>>,
            #[inline(always)]
            |&(a, b, modulus)| a.sub_mod(b, modulus),
        )
        .residues::<N>()
}

/// Offers `biguint`'s residues, whose sums and differences take operands below the modulus by
/// their type: `Residue` and `ConstantTimeResidue` of the width's one modulus, in one scope that
/// encloses the cases.
trait Residues {
    /// Offers the residue cases at `N` bits.
    fn residues<N: Bits>(self) -> Self
    where
        Uint<N>: Copy;
}

impl<V: Visitor<Op, Width>> Residues for Crate<V> {
    fn residues<N: Bits>(self) -> Self
    where
        Uint<N>: Copy,
    {
        let bits = self.key().bits as usize;
        let modulus =
            biguint::Modulus::from(NonZeroUint::<N>::from_limbs(&super::reduced_modulus(bits)));
        modulus.scope(|ring| {
            let plain = move |set: &[Limbs]| {
                let (a, b, _) = three::<Uint<N>, Uint<N>>(set);
                (ring.reduce(a), ring.reduce(b))
            };
            let masked = move |set: &[Limbs]| {
                let (a, b, _) = three::<Uint<N>, Uint<N>>(set);
                (
                    ring.reduce_constant_time(ConstantTime::new(a)),
                    ring.reduce_constant_time(ConstantTime::new(b)),
                )
            };
            self.chain(
                Plain(op::AddModReduced),
                "Residue +",
                plain,
                #[inline(always)]
                |&(a, b)| a + b,
                |result| result.get().to_limbs(),
            )
            .chain(
                Plain(op::SubModReduced),
                "Residue -",
                plain,
                #[inline(always)]
                |&(a, b)| a - b,
                |result| result.get().to_limbs(),
            )
            .chain(
                Ct(op::AddModReduced),
                "ConstantTimeResidue +",
                masked,
                #[inline(always)]
                |&(a, b)| a + b,
                |result| result.get().reveal().to_limbs(),
            )
            .chain(
                Ct(op::SubModReduced),
                "ConstantTimeResidue -",
                masked,
                #[inline(always)]
                |&(a, b)| a - b,
                |result| result.get().reveal().to_limbs(),
            )
        })
    }
}

/// Offers add, mul, div and mod of the primitive `$t`. The divisor is a `NonZero<$t>`; the
/// hardware divide takes a time that depends on the values.
macro_rules! primitive {
    ($cells:expr, $t:ty) => {
        chained_by_value!($cells, $t, Add: Timing::Fixed => wrapping_add, Mul: Timing::Fixed => wrapping_mul)
            .timed(
                op::Div,
                Timing::Variable,
                "/ NonZero",
                two::<$t, NonZero<$t>>,
                #[inline(always)] |&(a, divisor)| a / divisor,
            )
            .timed(
                op::Mod,
                Timing::Variable,
                "% NonZero",
                two::<$t, NonZero<$t>>,
                #[inline(always)] |&(a, divisor)| a % divisor,
            )
            // The plain compositions that a user writes, for operands below `m`; they branch on
            // the values.
            .timed_modular(
                op::AddMod,
                op::AddModReduced,
                Some(COMPOSITION_FOR_REDUCED),
                Timing::Variable,
                "overflowing_add, then - m",
                three::<$t, $t>,
                #[inline(always)]
                |&(a, b, m)| {
                    let (s, c) = a.overflowing_add(b);
                    if c || s >= m { s.wrapping_sub(m) } else { s }
                },
            )
            .timed_modular(
                op::SubMod,
                op::SubModReduced,
                Some(COMPOSITION_FOR_REDUCED),
                Timing::Variable,
                "-, else wrapping_sub then + m",
                three::<$t, $t>,
                #[inline(always)]
                |&(a, b, m)| if a >= b { a - b } else { a.wrapping_sub(b).wrapping_add(m) },
            )
    };
}

/// `u64`.
fn native_u64<V: Visitor<Op, Width>>(cells: Crate<V>) -> Crate<V> {
    primitive!(cells, u64).timed_chained(
        op::MulMod,
        Timing::Variable,
        "u128 product % m",
        three::<u64, u64>,
        #[inline(always)]
        |&(a, b, m)| ((u128::from(a) * u128::from(b)) % u128::from(m)) as u64,
    )
}

/// `u128`.
fn native_u128<V: Visitor<Op, Width>>(cells: Crate<V>) -> Crate<V> {
    primitive!(cells, u128).missing(Op::MulMod, "no 256-bit native type")
}

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
        op::MulMod,
        Timing::Variable,
        "mul_mod",
        three::<ruint::Uint<B, L>, ruint::Uint<B, L>>,
        #[inline(always)]
        |&(a, b, modulus)| a.mul_mod(b, modulus),
    )
    .timed_chained(
        op::AddMod,
        Timing::Variable,
        "add_mod",
        three::<ruint::Uint<B, L>, ruint::Uint<B, L>>,
        #[inline(always)]
        |&(a, b, modulus)| a.add_mod(b, modulus),
    )
    .timed_chained(
        op::SubMod,
        Timing::Variable,
        "a.add_mod(m - b.reduce_mod(m), m)",
        three::<ruint::Uint<B, L>, ruint::Uint<B, L>>,
        #[inline(always)]
        |&(a, b, modulus)| a.add_mod(modulus - b.reduce_mod(modulus), modulus),
    )
    .timed_chained(
        op::AddModReduced,
        Timing::Variable,
        "add_mod",
        three::<ruint::Uint<B, L>, ruint::Uint<B, L>>,
        #[inline(always)]
        |&(a, b, modulus)| a.add_mod(b, modulus),
    )
    .timed_chained(
        op::SubModReduced,
        Timing::Variable,
        "a.add_mod(m - b, m)",
        three::<ruint::Uint<B, L>, ruint::Uint<B, L>>,
        #[inline(always)]
        |&(a, b, modulus)| a.add_mod(modulus - b, modulus),
    )
}

/// `crypto-bigint` at `L` limbs, constant-time: its sum, product, `add_mod` and `sub_mod` take a
/// fixed time. Division and mulmod run its variable-time functions.
fn crypto_bigint<const L: usize, V: Visitor<Op, Width>>(cells: Crate<V>) -> Crate<V> {
    use crypto_bigint::{NonZero, Uint};
    cells
        .timed_chained(
            op::Add,
            Timing::Fixed,
            "wrapping_add",
            pair::<Uint<L>>,
            #[inline(always)]
            |(a, b)| a.wrapping_add(b),
        )
        .timed(
            op::Mul,
            Timing::Fixed,
            "wrapping_mul",
            pair::<Uint<L>>,
            #[inline(always)]
            |(a, b)| a.wrapping_mul(b),
        )
        .variable(
            op::Div,
            "wrapping_div_vartime",
            two::<Uint<L>, NonZero<Uint<L>>>,
            #[inline(always)]
            |(a, divisor)| a.wrapping_div_vartime(divisor),
        )
        .case(
            Ct(op::Div),
            "div_rem",
            two::<Uint<L>, NonZero<Uint<L>>>,
            #[inline(always)]
            |(a, divisor)| a.div_rem(divisor).0,
        )
        .variable(
            op::Mod,
            "rem_vartime",
            two::<Uint<L>, NonZero<Uint<L>>>,
            #[inline(always)]
            |(a, divisor)| a.rem_vartime(divisor),
        )
        .case(
            Ct(op::Mod),
            "rem",
            two::<Uint<L>, NonZero<Uint<L>>>,
            #[inline(always)]
            |(a, divisor)| a.rem(divisor),
        )
        .variable_chained(
            op::MulMod,
            "mul_mod_vartime",
            three::<Uint<L>, NonZero<Uint<L>>>,
            #[inline(always)]
            |(a, b, modulus)| a.mul_mod_vartime(b, modulus),
        )
        .case(
            Ct(op::MulMod),
            "mul_mod",
            three::<Uint<L>, NonZero<Uint<L>>>,
            #[inline(always)]
            |(a, b, modulus)| a.mul_mod(b, modulus),
        )
        .timed_modular(
            op::AddMod,
            op::AddModReduced,
            Some(CRYPTO_BIGINT_ADD_MOD),
            Timing::Fixed,
            "add_mod",
            three::<Uint<L>, NonZero<Uint<L>>>,
            #[inline(always)]
            |(a, b, modulus)| a.add_mod(b, modulus),
        )
        .timed_modular(
            op::SubMod,
            op::SubModReduced,
            Some(CRYPTO_BIGINT_SUB_MOD),
            Timing::Fixed,
            "sub_mod",
            three::<Uint<L>, NonZero<Uint<L>>>,
            #[inline(always)]
            |(a, b, modulus)| a.sub_mod(b, modulus),
        )
}

/// Offers add_mod and sub_mod of `$t`, for operands below `m`, as the plain compositions of its
/// overflowing sum and difference and a comparison; they branch on the values.
macro_rules! modular_sums {
    ($cells:expr, $t:ty) => {
        $cells
            .timed_modular(
                op::AddMod,
                op::AddModReduced,
                Some(COMPOSITION_FOR_REDUCED),
                Timing::Variable,
                "overflowing_add, then - m",
                three::<$t, $t>,
                #[inline(always)]
                |&(a, b, m)| {
                    let (s, c) = a.overflowing_add(b);
                    if c || s >= m {
                        s.overflowing_sub(m).0
                    } else {
                        s
                    }
                },
            )
            .timed_modular(
                op::SubMod,
                op::SubModReduced,
                Some(COMPOSITION_FOR_REDUCED),
                Timing::Variable,
                "-, else overflowing_sub then + m",
                three::<$t, $t>,
                #[inline(always)]
                |&(a, b, m)| {
                    if a >= b {
                        a - b
                    } else {
                        a.overflowing_sub(b).0.overflowing_add(m).0
                    }
                },
            )
    };
}

/// `bnum` at `N` bytes: its sum and its wrapping product run every limb; its division branches on
/// the values.
fn bnum<const N: usize, V: Visitor<Op, Width>>(cells: Crate<V>) -> Crate<V> {
    let cells = chained_by_value!(cells, bnum::Uint<N>, Add: Timing::Fixed => wrapping_add);
    let cells = by_value!(
        cells,
        bnum::Uint<N>,
        Mul: Timing::Fixed => wrapping_mul,
        Div: Timing::Variable => wrapping_div,
        Mod: Timing::Variable => wrapping_rem,
    )
    .missing(Op::MulMod, NO_WIDE_PRODUCT);
    modular_sums!(cells, bnum::Uint<N>)
}

/// The reason for an absent mulmod of a crate whose product wraps at the width: no type holds the
/// 2N-bit product to reduce.
const NO_WIDE_PRODUCT: &str = "no 2N-bit product to reduce";

/// `num-bigint`, which allocates each result and returns it exactly. Every operation's time depends
/// on the lengths of its normalized operands.
fn num_bigint<V: Visitor<Op, Width>>(cells: Crate<V>) -> Crate<V> {
    cells
        .timed(
            op::Add,
            Timing::Variable,
            "&a + &b",
            pair::<BigUint>,
            #[inline(always)]
            |(a, b)| a + b,
        )
        .timed(
            op::Mul,
            Timing::Variable,
            "&a * &b",
            pair::<BigUint>,
            #[inline(always)]
            |(a, b)| a * b,
        )
        .timed(
            op::Div,
            Timing::Variable,
            "&a / &b",
            pair::<BigUint>,
            #[inline(always)]
            |(a, b)| a / b,
        )
        .timed(
            op::Mod,
            Timing::Variable,
            "&a % &b",
            pair::<BigUint>,
            #[inline(always)]
            |(a, b)| a % b,
        )
        .timed(
            op::MulMod,
            Timing::Variable,
            "(&a * &b) % &m",
            three::<BigUint, BigUint>,
            #[inline(always)]
            |(a, b, modulus)| (a * b) % modulus,
        )
        .timed_modular(
            op::AddMod,
            op::AddModReduced,
            None,
            Timing::Variable,
            "(&a + &b) % &m",
            three::<BigUint, BigUint>,
            #[inline(always)]
            |(a, b, modulus)| (a + b) % modulus,
        )
        .timed_modular(
            op::SubMod,
            op::SubModReduced,
            None,
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

    use super::{
        COMPOSITION_FOR_REDUCED, Crate, NO_WIDE_PRODUCT, Op, Timed, Timing, Width, op, pair, three,
    };
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
                    op::Add,
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
                    op::Mul,
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
                    op::Div,
                    Timing::Variable,
                    "/",
                    pair::<$t>,
                    #[inline(always)]
                    |&(a, b)| a / b,
                )
                .timed(
                    op::Mod,
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
                    let cells = parity_ops!(cells, $t, $words)
                        .missing(Op::MulMod, NO_WIDE_PRODUCT);
                    modular_sums!(cells, $t)
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
