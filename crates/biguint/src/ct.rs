//! The constant-time operations of `Uint`, through which
//! [`ConstantTime<Uint<N>>`](ConstantTime) computes.
//!
//! The sums, the differences, the bitwise operations and the shifts by a public amount run the
//! same instructions as `Uint`'s operators: their carry chains and limb loops never branch on a
//! value. Products run the quadratic product, and Karatsuba from its threshold with the signs of
//! the differences and the carries under masks. Reductions and divisions divide by the public
//! divisor with [`MaskedDivisor`], and modular sums correct under masks.

use core::ops::{Add, BitAnd, BitOr, BitXor, Mul, Sub};

use constant_time::{
    ConstantTime, CtAdd, CtAddMod, CtBitAnd, CtBitOr, CtBitXor, CtDivRem, CtMul, CtMulMod,
    CtNegMod, CtNot, CtPow, CtShl, CtShr, CtSquare, CtSub, CtSubMod, Public,
};
use generic_array::GenericArray;
use typenum::Sum;

use crate::bits::Bits;
use crate::bits::sealed::Sealed;
use crate::div::DiscardQuotient;
use crate::div::ct::MaskedDivisor;
use crate::kernel::{
    add_flipped, add_flipped_into, add_masked, fits, mask_of, select_masked, sub_into,
    subtract_when_above_masked,
};
use crate::modular::sum_mod_masked;
use crate::mul::{full_product, truncated_product};
use crate::nonzero::{Divisor, NonZero, NonZeroUint};
use crate::shift::ShiftAmount;
use crate::stream::TwosComplement;
use crate::uint::Uint;

impl<N: Bits> Public for Uint<N> {}

impl<T: Public + Divisor> Public for NonZero<T> {}

/// `(a * b) mod 2^N`, with the corrections under masks.
#[inline(always)]
pub(crate) fn product<N: Bits, R: Bits>(a: &Uint<N>, b: &Uint<R>) -> Uint<N> {
    Uint::from_generic_limbs(truncated_product::<N, R::Limbs, true>(&a.limbs, &b.limbs))
}

/// Bit `index` of `value`, as `0` or `1`.
#[inline(always)]
pub(crate) fn bit<N: Bits>(value: &Uint<N>, index: usize) -> u64 {
    value.limbs.get(index / 64).copied().unwrap_or(0) >> (index % 64) & 1
}

/// A public modulus prepared to reduce operands of its width in constant time over the operands.
enum Reduction<'a, N: Bits> {
    /// A modulus of at least `2^(N - 1)`: an operand below `2^N` is below twice the modulus, and
    /// one subtraction under a mask reduces it.
    Subtract(&'a Uint<N>),
    /// A smaller modulus, which reduces each operand by a division.
    Divide(MaskedDivisor<N::Limbs>),
}

impl<'a, N: Bits> Reduction<'a, N> {
    #[inline(always)]
    fn new(modulus: &'a Uint<N>) -> Self {
        if bit(modulus, N::BITS - 1) == 1 {
            Self::Subtract(modulus)
        } else {
            Self::Divide(MaskedDivisor::new(&modulus.limbs))
        }
    }

    /// `value mod modulus`.
    #[inline(always)]
    fn reduce(&self, value: &Uint<N>) -> Uint<N> {
        match self {
            Self::Subtract(modulus) => {
                let mut reduced = Uint::ZERO;
                subtract_when_above_masked(&mut reduced.limbs, &value.limbs, &modulus.limbs, 0);
                reduced
            }
            Self::Divide(divisor) => {
                divisor.remainder::<N::Window, N>(&value.limbs, &mut DiscardQuotient)
            }
        }
    }
}

/// Implements a constant-time operator trait of `Uint` by its operator, which runs the same
/// instructions for every value.
macro_rules! by_operator {
    ($($ct:ident, $method:ident, $op:ident, $op_method:ident;)*) => {
        $(
            impl<N: Bits, RhsN: Bits> $ct<Uint<RhsN>> for Uint<N> {
                type Output = Self;

                #[inline(always)]
                fn $method(self, rhs: Uint<RhsN>) -> Self {
                    $op::$op_method(self, rhs)
                }
            }
        )*
    };
}

by_operator!(
    CtAdd, ct_add, Add, add;
    CtSub, ct_sub, Sub, sub;
    CtBitAnd, ct_bitand, BitAnd, bitand;
    CtBitOr, ct_bitor, BitOr, bitor;
    CtBitXor, ct_bitxor, BitXor, bitxor;
);

/// `(self * rhs) mod 2^N`.
impl<N: Bits, RhsN: Bits> CtMul<Uint<RhsN>> for Uint<N> {
    type Output = Self;

    #[inline(always)]
    fn ct_mul(self, rhs: Uint<RhsN>) -> Self {
        product(&self, &rhs)
    }
}

/// `!self` within `N` bits.
impl<N: Bits> CtNot for Uint<N> {
    #[inline(always)]
    fn ct_not(self) -> Self {
        !self
    }
}

/// `(self * 2^amount) mod 2^N`: `0` when `amount >= N` or `amount < 0`.
impl<N: Bits, A: ShiftAmount> CtShl<A> for Uint<N> {
    #[inline(always)]
    fn ct_shl(self, amount: A) -> Self {
        self << amount
    }
}

/// `floor(self / 2^amount)`: `0` when `amount >= N` or `amount < 0`.
impl<N: Bits, A: ShiftAmount> CtShr<A> for Uint<N> {
    #[inline(always)]
    fn ct_shr(self, amount: A) -> Self {
        self >> amount
    }
}

/// `(self * self) mod 2^N`.
impl<N: Bits> CtSquare for Uint<N> {
    #[inline(always)]
    fn ct_square(self) -> Self {
        product(&self, &self)
    }
}

/// `self^exponent mod 2^N` for a wrapped exponent: square-and-multiply over all `E` bits, each
/// product taken under the mask of its bit.
impl<N: Bits, E: Bits> CtPow<ConstantTime<Uint<E>>> for Uint<N> {
    #[inline]
    fn ct_pow(self, exponent: ConstantTime<Uint<E>>) -> Self {
        let exponent = exponent.reveal();
        (0..E::BITS).rev().fold(Self::ONE, |power, index| {
            let mut power = product(&power, &power);
            let multiplied = product(&power, &self);
            select_masked(
                &mut power.limbs,
                &multiplied.limbs,
                mask_of(bit(&exponent, index)),
            );
            power
        })
    }
}

/// `(self + rhs) mod modulus`, for operands of any value.
impl<N: Bits> CtAddMod<NonZeroUint<N>> for Uint<N> {
    #[inline(always)]
    fn ct_add_mod(self, rhs: Self, modulus: NonZeroUint<N>) -> Self {
        let modulus = modulus.get();
        let reduction = Reduction::new(&modulus);
        // Natively, a modulus that reduces by one subtraction takes `a + b ≡ a - (m - b)`: `b`
        // reduces, `m - b` lies in `(0, m]`, and `difference_mod` corrects. That is one masked
        // reduction where the sum of both reduced operands takes three: at 2048 bits, 300 cycles
        // per operation on Zen 3 against 380 to 640, and fewer at every width. WebAssembly,
        // unmeasured in this form, reduces both operands and corrects their sum.
        if cfg!(not(target_arch = "wasm32"))
            && let Reduction::Subtract(subtracted) = reduction
        {
            let b = reduction.reduce(&rhs);
            let mut negated = Uint::<N>::ZERO;
            fits(sub_into(&mut negated.limbs, &subtracted.limbs, &b.limbs));
            return difference_mod(&self, &negated, subtracted);
        }
        let (a, b) = (reduction.reduce(&self), reduction.reduce(&rhs));
        sum_mod_masked(&a, b.source(), &modulus)
    }
}

/// `(self - rhs) mod modulus`, for operands of any value.
impl<N: Bits> CtSubMod<NonZeroUint<N>> for Uint<N> {
    #[inline(always)]
    fn ct_sub_mod(self, rhs: Self, modulus: NonZeroUint<N>) -> Self {
        let modulus = modulus.get();
        let reduction = Reduction::new(&modulus);
        if const { chained::<N>() }
            && let Reduction::Subtract(subtracted) = reduction
        {
            return difference_mod_chained(&self, &rhs, subtracted);
        }
        let b = reduction.reduce(&rhs);
        match reduction {
            Reduction::Subtract(subtracted) => difference_mod_masked(&self, &b, subtracted),
            Reduction::Divide(_) => sum_mod_masked(
                &reduction.reduce(&self),
                TwosComplement(b.source()),
                &modulus,
            ),
        }
    }
}

/// `(a - b) mod modulus` under masks, for `a` below twice the modulus and `b` at most the modulus.
#[inline(always)]
fn difference_mod<N: Bits>(a: &Uint<N>, b: &Uint<N>, modulus: &Uint<N>) -> Uint<N> {
    if const { chained::<N>() } {
        difference_mod_chained(a, b, modulus)
    } else {
        difference_mod_masked(a, b, modulus)
    }
}

/// The lengths of [`difference_mod_chained`], whose three carry chains run in place. Longer, the
/// chains through memory take more cycles on Zen 3 than [`difference_mod_masked`], whose select
/// vectorizes: at 4096 bits 489 against 445 per operation. At one limb, the reduction of `b` in
/// [`difference_mod_masked`] runs beside the chain of `a`, whose latency is 8.9 cycles there against
/// 16 here.
const CHAINED_DIFFERENCE_LIMBS: core::ops::RangeInclusive<usize> = 2..=16;

/// Whether a difference of `N` bits runs [`difference_mod_chained`]. Tested in a `const` block, the
/// condition keeps the other arm out of the code of each length: evaluated at run time, it reaches
/// LLVM, which then allocates the registers of 2048 bits otherwise.
const fn chained<N: Bits>() -> bool {
    *CHAINED_DIFFERENCE_LIMBS.start() <= N::LIMBS && N::LIMBS <= *CHAINED_DIFFERENCE_LIMBS.end()
}

/// `(a - b) mod modulus` under masks, for `a` and `b` below twice the modulus: the difference `d`
/// lies strictly between `-2 * modulus` and `2 * modulus`. A borrow puts `d` below zero, and
/// `d + modulus` follows; without one, `d - modulus` does. Either lies between `-modulus` and
/// `modulus`, and is below zero exactly when its correction does not carry out of the top limb;
/// there, adding the modulus gives the result.
///
/// The flip of the first correction is the borrow less one: it feeds the complement and the carry
/// into the chain, which LLVM keeps arithmetic, where [`mask_of`] would add a store and a load to
/// its latency. The mask of the second correction selects limbs, and LLVM turns it into a
/// conditional move without [`mask_of`].
#[inline(always)]
fn difference_mod_chained<N: Bits>(a: &Uint<N>, b: &Uint<N>, modulus: &Uint<N>) -> Uint<N> {
    let mut difference = Uint::<N>::ZERO;
    let borrow = sub_into(&mut difference.limbs, &a.limbs, &b.limbs);
    let carry = add_flipped(
        &mut difference.limbs,
        &modulus.limbs,
        borrow.wrapping_sub(1),
    );
    fits(add_masked(
        &mut difference.limbs,
        &modulus.limbs,
        mask_of(carry ^ 1),
    ));
    difference
}

/// `(a - b) mod modulus` under masks, for `a` below twice the modulus and `b` below it: the
/// difference `d` lies between `-modulus` and `2 * modulus`. A borrow puts `d` below zero, where
/// `d + modulus` is the result; without one, `d - modulus` is the result when `d >= modulus`. Each
/// correction carries out of the top limb exactly when it is the result.
#[inline(always)]
fn difference_mod_masked<N: Bits>(a: &Uint<N>, b: &Uint<N>, modulus: &Uint<N>) -> Uint<N> {
    let mut difference = Uint::<N>::ZERO;
    let borrow = sub_into(&mut difference.limbs, &a.limbs, &b.limbs);
    let mut corrected = Uint::<N>::ZERO;
    let carry = add_flipped_into(
        &mut corrected.limbs,
        &difference.limbs,
        &modulus.limbs,
        mask_of(borrow ^ 1),
    );
    select_masked(&mut difference.limbs, &corrected.limbs, mask_of(carry));
    difference
}

/// `-self mod modulus`, for an operand of any value.
impl<N: Bits> CtNegMod<NonZeroUint<N>> for Uint<N> {
    #[inline(always)]
    fn ct_neg_mod(self, modulus: NonZeroUint<N>) -> Self {
        let modulus = modulus.get();
        let a = Reduction::new(&modulus).reduce(&self);
        sum_mod_masked(&Self::ZERO, TwosComplement(a.source()), &modulus)
    }
}

/// `(self * rhs) mod modulus`, for operands of any width and any value: the full product divided
/// by the modulus.
impl<N, RhsN, ModN> CtMulMod<Uint<RhsN>, NonZeroUint<ModN>> for Uint<N>
where
    N: Bits + Add<RhsN>,
    RhsN: Bits,
    ModN: Bits,
    Sum<N, RhsN>: Bits,
{
    type Output = Uint<ModN>;

    #[inline(always)]
    fn ct_mul_mod(self, rhs: Uint<RhsN>, modulus: NonZeroUint<ModN>) -> Uint<ModN> {
        let full = full_product::<Sum<N, RhsN>, _, _, true>(&self.limbs, &rhs.limbs);
        MaskedDivisor::new(&modulus.limbs())
            .remainder::<<Sum<N, RhsN> as Sealed>::Window, ModN>(&full, &mut DiscardQuotient)
    }
}

/// `(self / divisor, self % divisor)`; the remainder takes the divisor's width.
impl<N: Bits, M: Bits> CtDivRem<NonZeroUint<M>> for Uint<N> {
    type Remainder = Uint<M>;

    #[inline(always)]
    fn ct_div_rem(self, divisor: NonZeroUint<M>) -> (Self, Uint<M>) {
        let mut quotient = GenericArray::<u64, N::Limbs>::default();
        let remainder = MaskedDivisor::new(&divisor.limbs())
            .remainder::<N::Window, M>(&self.limbs, &mut quotient);
        (Uint::from_generic_limbs(quotient), remainder)
    }
}

/// Implements an operator with a public `Uint` on the left and a [`ConstantTime`] on the right.
macro_rules! public_left {
    ($($op:ident, $method:ident, $ct:ident, $ct_method:ident;)*) => {
        $(
            impl<N: Bits, RhsN: Bits> $op<ConstantTime<Uint<RhsN>>> for Uint<N> {
                type Output = ConstantTime<Self>;

                #[inline(always)]
                fn $method(self, rhs: ConstantTime<Uint<RhsN>>) -> ConstantTime<Self> {
                    ConstantTime::new($ct::$ct_method(self, rhs.reveal()))
                }
            }
        )*
    };
}

public_left!(
    Add, add, CtAdd, ct_add;
    Sub, sub, CtSub, ct_sub;
    Mul, mul, CtMul, ct_mul;
    BitAnd, bitand, CtBitAnd, ct_bitand;
    BitOr, bitor, CtBitOr, ct_bitor;
    BitXor, bitxor, CtBitXor, ct_bitxor;
);
