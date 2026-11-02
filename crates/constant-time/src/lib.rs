#![doc = include_str!("../README.md")]
#![no_std]
#![forbid(unsafe_code)]
#![doc(test(attr(forbid(unsafe_code))))]

use core::ops::{
    Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Div, Mul,
    MulAssign, Neg, Not, Rem, Shl, ShlAssign, Shr, ShrAssign, Sub, SubAssign,
};

/// A value whose operations run in constant time over it: no branch and no memory address depends
/// on it.
///
/// It holds a `T` in the same memory; [`ConstantTime::new`] and [`ConstantTime::reveal`] cost
/// nothing. Each operator and method exists for the `T` that implements the matching `Ct*` trait.
/// The operators combine two `ConstantTime`s, or a `ConstantTime` and a [`Public`] value. Moduli,
/// divisors, shift amounts, public operands and widths may steer the time.
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct ConstantTime<T>(T);

impl<T> ConstantTime<T> {
    /// `value`, wrapped.
    #[inline(always)]
    #[must_use]
    pub const fn new(value: T) -> Self {
        Self(value)
    }

    /// The value.
    #[inline(always)]
    #[must_use]
    pub const fn reveal(self) -> T
    where
        T: Copy,
    {
        self.0
    }

    /// `self * self`.
    #[inline(always)]
    #[must_use]
    pub fn square(self) -> Self
    where
        T: CtSquare,
    {
        Self(self.0.ct_square())
    }

    /// `self + self`.
    #[inline(always)]
    #[must_use]
    pub fn double(self) -> Self
    where
        T: CtAdd<Output = T> + Copy,
    {
        Self(self.0.ct_add(self.0))
    }

    /// `self^exponent`; the implementation of [`CtPow`] names whether the exponent is wrapped.
    #[inline(always)]
    #[must_use]
    pub fn pow<E>(self, exponent: E) -> Self
    where
        T: CtPow<E>,
    {
        Self(self.0.ct_pow(exponent))
    }

    /// The multiplicative inverse, or `None` when it does not exist. Only the `Option` depends on
    /// the value.
    #[inline(always)]
    #[must_use]
    pub fn invert(self) -> Option<Self>
    where
        T: CtInvert,
    {
        self.0.ct_invert().map(Self)
    }

    /// A square root, in the form that [`CtSqrt::Root`] names.
    #[inline(always)]
    #[must_use]
    pub fn sqrt(self) -> T::Root
    where
        T: CtSqrt,
    {
        self.0.ct_sqrt()
    }

    /// `(self + rhs) mod modulus`, for operands of any value.
    #[inline(always)]
    #[must_use]
    pub fn add_mod<M>(self, rhs: Self, modulus: M) -> Self
    where
        T: CtAddMod<M>,
    {
        Self(self.0.ct_add_mod(rhs.0, modulus))
    }

    /// `(self - rhs) mod modulus`, for operands of any value.
    #[inline(always)]
    #[must_use]
    pub fn sub_mod<M>(self, rhs: Self, modulus: M) -> Self
    where
        T: CtSubMod<M>,
    {
        Self(self.0.ct_sub_mod(rhs.0, modulus))
    }

    /// `-self mod modulus`, for an operand of any value.
    #[inline(always)]
    #[must_use]
    pub fn neg_mod<M>(self, modulus: M) -> Self
    where
        T: CtNegMod<M>,
    {
        Self(self.0.ct_neg_mod(modulus))
    }

    /// `(self * rhs) mod modulus`, in the type that [`CtMulMod::Output`] names.
    #[inline(always)]
    #[must_use]
    pub fn mul_mod<R, M>(self, rhs: ConstantTime<R>, modulus: M) -> ConstantTime<T::Output>
    where
        T: CtMulMod<R, M>,
    {
        ConstantTime(self.0.ct_mul_mod(rhs.0, modulus))
    }

    /// `(self / divisor, self % divisor)`; the remainder has the type that
    /// [`CtDivRem::Remainder`] names.
    #[inline(always)]
    #[must_use]
    pub fn div_rem<D>(self, divisor: D) -> (Self, ConstantTime<T::Remainder>)
    where
        T: CtDivRem<D>,
    {
        let (quotient, remainder) = self.0.ct_div_rem(divisor);
        (Self(quotient), ConstantTime(remainder))
    }

    /// The value that `self` represents, as [`CtGet::Value`] names it.
    #[inline(always)]
    #[must_use]
    pub fn get(self) -> ConstantTime<T::Value>
    where
        T: CtGet,
    {
        ConstantTime(self.0.ct_get())
    }
}

/// The wrapped value of every public value that converts into a `T`.
impl<T, U: Public> From<U> for ConstantTime<T>
where
    T: From<U>,
{
    #[inline(always)]
    fn from(value: U) -> Self {
        Self(T::from(value))
    }
}

/// A value that may steer the time of an operation: the public operand of an operator.
pub trait Public: Copy {}

macro_rules! public {
    ($($primitive:ty),* $(,)?) => {
        $(impl Public for $primitive {})*
    };
}

public!(
    bool, u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize
);

impl<T: Public, const L: usize> Public for [T; L] {}

/// Defines a binary constant-time trait.
macro_rules! binary_trait {
    ($(#[$doc:meta])* $trait:ident, $method:ident) => {
        $(#[$doc])*
        ///
        /// The time depends on no value of `self` or `rhs`.
        pub trait $trait<Rhs = Self> {
            /// The result type.
            type Output;

            /// The operation.
            #[must_use]
            fn $method(self, rhs: Rhs) -> Self::Output;
        }
    };
}

binary_trait!(
    /// `self + rhs`.
    CtAdd,
    ct_add
);
binary_trait!(
    /// `self - rhs`.
    CtSub,
    ct_sub
);
binary_trait!(
    /// `self * rhs`.
    CtMul,
    ct_mul
);
binary_trait!(
    /// `self & rhs`.
    CtBitAnd,
    ct_bitand
);
binary_trait!(
    /// `self | rhs`.
    CtBitOr,
    ct_bitor
);
binary_trait!(
    /// `self ^ rhs`.
    CtBitXor,
    ct_bitxor
);

/// `-self`, in time that depends on no value of `self`.
pub trait CtNeg {
    /// `-self`.
    #[must_use]
    fn ct_neg(self) -> Self;
}

/// `!self`, in time that depends on no value of `self`.
pub trait CtNot {
    /// `!self`.
    #[must_use]
    fn ct_not(self) -> Self;
}

/// `self << amount` for a public amount, in time that depends on no value of `self`.
pub trait CtShl<A> {
    /// `self << amount`.
    #[must_use]
    fn ct_shl(self, amount: A) -> Self;
}

/// `self >> amount` for a public amount, in time that depends on no value of `self`.
pub trait CtShr<A> {
    /// `self >> amount`.
    #[must_use]
    fn ct_shr(self, amount: A) -> Self;
}

/// `self * self`, in time that depends on no value of `self`.
pub trait CtSquare {
    /// `self * self`.
    #[must_use]
    fn ct_square(self) -> Self;
}

/// `self^exponent`, in time that depends on no value of `self`, and on no value of the exponent
/// when `E` is a [`ConstantTime`].
pub trait CtPow<E> {
    /// `self^exponent`.
    #[must_use]
    fn ct_pow(self, exponent: E) -> Self;
}

/// The multiplicative inverse, in time that depends on no value of `self`. Only the `Option`
/// depends on the value.
pub trait CtInvert: Sized {
    /// The inverse, or `None` when it does not exist.
    #[must_use]
    fn ct_invert(self) -> Option<Self>;
}

/// A square root, in time that depends on no value of `self`.
pub trait CtSqrt {
    /// The result: `ConstantTime<Self>` where every value has a root, `Option<ConstantTime<Self>>`
    /// where the `Option` discloses whether a root exists.
    type Root;

    /// A square root.
    #[must_use]
    fn ct_sqrt(self) -> Self::Root;
}

/// `(self + rhs) mod modulus` for a public modulus, in time that depends on no value of `self` or
/// `rhs`.
pub trait CtAddMod<M>: Sized {
    /// `(self + rhs) mod modulus`.
    #[must_use]
    fn ct_add_mod(self, rhs: Self, modulus: M) -> Self;
}

/// `(self - rhs) mod modulus` for a public modulus, in time that depends on no value of `self` or
/// `rhs`.
pub trait CtSubMod<M>: Sized {
    /// `(self - rhs) mod modulus`.
    #[must_use]
    fn ct_sub_mod(self, rhs: Self, modulus: M) -> Self;
}

/// `-self mod modulus` for a public modulus, in time that depends on no value of `self`.
pub trait CtNegMod<M> {
    /// `-self mod modulus`.
    #[must_use]
    fn ct_neg_mod(self, modulus: M) -> Self;
}

/// `(self * rhs) mod modulus` for a public modulus, in time that depends on no value of `self` or
/// `rhs`.
pub trait CtMulMod<Rhs, M> {
    /// The type of the result.
    type Output;

    /// `(self * rhs) mod modulus`.
    #[must_use]
    fn ct_mul_mod(self, rhs: Rhs, modulus: M) -> Self::Output;
}

/// `(self / divisor, self % divisor)` for a public divisor, in time that depends on no value of
/// `self`.
pub trait CtDivRem<D>: Sized {
    /// The type of the remainder.
    type Remainder;

    /// `(self / divisor, self % divisor)`.
    #[must_use]
    fn ct_div_rem(self, divisor: D) -> (Self, Self::Remainder);
}

/// The value that `self` represents, in time that depends on no value of `self`.
pub trait CtGet {
    /// The type of the value.
    type Value;

    /// The value.
    #[must_use]
    fn ct_get(self) -> Self::Value;
}

/// Implements a binary operator of [`ConstantTime`]s through `$ct`: between two wrapped values,
/// and with a public right-hand operand, with the assigning forms.
macro_rules! operator {
    ($trait:ident, $method:ident, $assign_trait:ident, $assign:ident, $ct:ident, $ct_method:ident) => {
        impl<T: $ct<U>, U> $trait<ConstantTime<U>> for ConstantTime<T> {
            type Output = ConstantTime<T::Output>;

            #[inline(always)]
            fn $method(self, rhs: ConstantTime<U>) -> Self::Output {
                ConstantTime(self.0.$ct_method(rhs.0))
            }
        }

        impl<T: $ct<U>, U: Public> $trait<U> for ConstantTime<T> {
            type Output = ConstantTime<T::Output>;

            #[inline(always)]
            fn $method(self, rhs: U) -> Self::Output {
                ConstantTime(self.0.$ct_method(rhs))
            }
        }

        impl<T: $ct<U, Output = T> + Copy, U> $assign_trait<ConstantTime<U>> for ConstantTime<T> {
            #[inline(always)]
            fn $assign(&mut self, rhs: ConstantTime<U>) {
                self.0 = self.0.$ct_method(rhs.0);
            }
        }

        impl<T: $ct<U, Output = T> + Copy, U: Public> $assign_trait<U> for ConstantTime<T> {
            #[inline(always)]
            fn $assign(&mut self, rhs: U) {
                self.0 = self.0.$ct_method(rhs);
            }
        }
    };
}

operator!(Add, add, AddAssign, add_assign, CtAdd, ct_add);
operator!(Sub, sub, SubAssign, sub_assign, CtSub, ct_sub);
operator!(Mul, mul, MulAssign, mul_assign, CtMul, ct_mul);
operator!(
    BitAnd,
    bitand,
    BitAndAssign,
    bitand_assign,
    CtBitAnd,
    ct_bitand
);
operator!(BitOr, bitor, BitOrAssign, bitor_assign, CtBitOr, ct_bitor);
operator!(
    BitXor,
    bitxor,
    BitXorAssign,
    bitxor_assign,
    CtBitXor,
    ct_bitxor
);

impl<T: CtNeg> Neg for ConstantTime<T> {
    type Output = Self;

    #[inline(always)]
    fn neg(self) -> Self {
        Self(self.0.ct_neg())
    }
}

impl<T: CtNot> Not for ConstantTime<T> {
    type Output = Self;

    #[inline(always)]
    fn not(self) -> Self {
        Self(self.0.ct_not())
    }
}

impl<T: CtShl<A>, A> Shl<A> for ConstantTime<T> {
    type Output = Self;

    #[inline(always)]
    fn shl(self, amount: A) -> Self {
        Self(self.0.ct_shl(amount))
    }
}

impl<T: CtShr<A>, A> Shr<A> for ConstantTime<T> {
    type Output = Self;

    #[inline(always)]
    fn shr(self, amount: A) -> Self {
        Self(self.0.ct_shr(amount))
    }
}

impl<T: CtShl<A> + Copy, A> ShlAssign<A> for ConstantTime<T> {
    #[inline(always)]
    fn shl_assign(&mut self, amount: A) {
        self.0 = self.0.ct_shl(amount);
    }
}

impl<T: CtShr<A> + Copy, A> ShrAssign<A> for ConstantTime<T> {
    #[inline(always)]
    fn shr_assign(&mut self, amount: A) {
        self.0 = self.0.ct_shr(amount);
    }
}

/// `self / divisor` for a public divisor.
impl<T: CtDivRem<D>, D> Div<D> for ConstantTime<T> {
    type Output = Self;

    #[inline(always)]
    fn div(self, divisor: D) -> Self {
        Self(self.0.ct_div_rem(divisor).0)
    }
}

/// `self % divisor` for a public divisor.
impl<T: CtDivRem<D>, D> Rem<D> for ConstantTime<T> {
    type Output = ConstantTime<T::Remainder>;

    #[inline(always)]
    fn rem(self, divisor: D) -> ConstantTime<T::Remainder> {
        ConstantTime(self.0.ct_div_rem(divisor).1)
    }
}
