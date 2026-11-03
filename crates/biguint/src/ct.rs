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
use crate::kernel::{mask_of, select_masked};
use crate::modular::sum_mod_masked;
use crate::mul::{full_product, truncated_product};
use crate::nonzero::{NonZero, NonZeroUint};
use crate::shift::ShiftAmount;
use crate::stream::TwosComplement;
use crate::uint::Uint;

impl<N: Bits> Public for Uint<N> {}

impl<T: Public> Public for NonZero<T> {}

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

/// `value mod divisor`, for a divisor of `value`'s width.
#[inline(always)]
fn reduce<N: Bits>(divisor: &MaskedDivisor<N::Limbs>, value: &Uint<N>) -> Uint<N> {
    divisor.remainder::<N::Window, N>(&value.limbs, &mut DiscardQuotient)
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
        let divisor = MaskedDivisor::new(modulus.limbs());
        let (a, b) = (reduce(&divisor, &self), reduce(&divisor, &rhs));
        sum_mod_masked(&a, b.source(), modulus.as_ref())
    }
}

/// `(self - rhs) mod modulus`, for operands of any value.
impl<N: Bits> CtSubMod<NonZeroUint<N>> for Uint<N> {
    #[inline(always)]
    fn ct_sub_mod(self, rhs: Self, modulus: NonZeroUint<N>) -> Self {
        let divisor = MaskedDivisor::new(modulus.limbs());
        let (a, b) = (reduce(&divisor, &self), reduce(&divisor, &rhs));
        sum_mod_masked(&a, TwosComplement(b.source()), modulus.as_ref())
    }
}

/// `-self mod modulus`, for an operand of any value.
impl<N: Bits> CtNegMod<NonZeroUint<N>> for Uint<N> {
    #[inline(always)]
    fn ct_neg_mod(self, modulus: NonZeroUint<N>) -> Self {
        let a = reduce(&MaskedDivisor::new(modulus.limbs()), &self);
        sum_mod_masked(&Self::ZERO, TwosComplement(a.source()), modulus.as_ref())
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
        MaskedDivisor::new(modulus.limbs())
            .remainder::<<Sum<N, RhsN> as Sealed>::Window, ModN>(&full, &mut DiscardQuotient)
    }
}

/// `(self / divisor, self % divisor)`; the remainder takes the divisor's width.
impl<N: Bits, M: Bits> CtDivRem<NonZeroUint<M>> for Uint<N> {
    type Remainder = Uint<M>;

    #[inline(always)]
    fn ct_div_rem(self, divisor: NonZeroUint<M>) -> (Self, Uint<M>) {
        let mut quotient = GenericArray::<u64, N::Limbs>::default();
        let remainder = MaskedDivisor::new(divisor.limbs())
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
