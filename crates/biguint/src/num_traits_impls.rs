//! The `num-traits` traits of `Uint`, each the inherent method of its name.
//!
//! The shifts follow the traits' contract: `checked_shl` and `checked_shr` refuse `rhs >= N`, and
//! `wrapping_shl` and `wrapping_shr` shift by `rhs mod N`.

use generic_array::GenericArray;
use num_traits::ops::overflowing::{OverflowingAdd, OverflowingMul, OverflowingSub};
use num_traits::{
    Bounded, CheckedAdd, CheckedMul, CheckedNeg, CheckedShl, CheckedShr, CheckedSub, ConstOne,
    ConstZero, FromBytes, FromPrimitive, MulAdd, NumCast, Pow, SaturatingAdd, SaturatingMul,
    SaturatingSub, ToBytes, ToPrimitive, WrappingAdd, WrappingMul, WrappingNeg, WrappingShl,
    WrappingShr, WrappingSub,
};

use crate::bits::Bits;
use crate::uint::Uint;

impl<N: Bits> num_traits::One for Uint<N> {
    #[inline(always)]
    fn one() -> Self {
        Self::ONE
    }
}

impl<N: Bits> ConstOne for Uint<N> {
    const ONE: Self = Self::ONE;
}

impl<N: Bits> ConstZero for Uint<N> {
    const ZERO: Self = Self::ZERO;
}

impl<N: Bits> Bounded for Uint<N> {
    #[inline(always)]
    fn min_value() -> Self {
        Self::ZERO
    }

    #[inline(always)]
    fn max_value() -> Self {
        Self::MAX
    }
}

/// `self^exponent mod 2^N`.
impl<N: Bits> Pow<u32> for Uint<N> {
    type Output = Self;

    #[inline(always)]
    fn pow(self, exponent: u32) -> Self {
        self.wrapping_pow(exponent)
    }
}

/// `(self * a + b) mod 2^N`.
impl<N: Bits> MulAdd for Uint<N> {
    type Output = Self;

    #[inline(always)]
    fn mul_add(self, a: Self, b: Self) -> Self {
        Self::mul_add(self, a, b)
    }
}

/// The traits of `fn(&self, &Self) -> Output`.
macro_rules! binary {
    ($($trait:ident :: $method:ident -> $output:ty),* $(,)?) => {
        $(
            impl<N: Bits> $trait for Uint<N> {
                #[inline(always)]
                fn $method(&self, rhs: &Self) -> $output {
                    Self::$method(*self, *rhs)
                }
            }
        )*
    };
}

binary!(
    CheckedAdd::checked_add -> Option<Self>,
    CheckedSub::checked_sub -> Option<Self>,
    CheckedMul::checked_mul -> Option<Self>,
    WrappingAdd::wrapping_add -> Self,
    WrappingSub::wrapping_sub -> Self,
    WrappingMul::wrapping_mul -> Self,
    SaturatingAdd::saturating_add -> Self,
    SaturatingSub::saturating_sub -> Self,
    SaturatingMul::saturating_mul -> Self,
    OverflowingAdd::overflowing_add -> (Self, bool),
    OverflowingSub::overflowing_sub -> (Self, bool),
    OverflowingMul::overflowing_mul -> (Self, bool),
);

/// The traits of `fn(&self, u32) -> Output`.
macro_rules! shift {
    ($($trait:ident :: $method:ident -> $output:ty),* $(,)?) => {
        $(
            impl<N: Bits> $trait for Uint<N> {
                #[inline(always)]
                fn $method(&self, rhs: u32) -> $output {
                    Self::$method(*self, rhs)
                }
            }
        )*
    };
}

shift!(
    CheckedShl::checked_shl -> Option<Self>,
    CheckedShr::checked_shr -> Option<Self>,
    WrappingShl::wrapping_shl -> Self,
    WrappingShr::wrapping_shr -> Self,
);

impl<N: Bits> CheckedNeg for Uint<N> {
    #[inline(always)]
    fn checked_neg(&self) -> Option<Self> {
        Self::checked_neg(*self)
    }
}

impl<N: Bits> WrappingNeg for Uint<N> {
    #[inline(always)]
    fn wrapping_neg(&self) -> Self {
        Self::wrapping_neg(*self)
    }
}

impl<N: Bits> ToPrimitive for Uint<N> {
    #[inline(always)]
    fn to_i64(&self) -> Option<i64> {
        self.to_u64().and_then(|value| i64::try_from(value).ok())
    }

    #[inline(always)]
    fn to_u64(&self) -> Option<u64> {
        u64::try_from(*self).ok()
    }

    #[inline(always)]
    fn to_i128(&self) -> Option<i128> {
        self.to_u128().and_then(|value| i128::try_from(value).ok())
    }

    #[inline(always)]
    fn to_u128(&self) -> Option<u128> {
        u128::try_from(*self).ok()
    }

    #[inline(always)]
    fn to_f32(&self) -> Option<f32> {
        Some(Self::to_f32(self))
    }

    #[inline(always)]
    fn to_f64(&self) -> Option<f64> {
        Some(Self::to_f64(self))
    }
}

/// `None` for a negative value and for one that needs more than `N` bits; a float converts its
/// integer part.
impl<N: Bits> FromPrimitive for Uint<N> {
    #[inline(always)]
    fn from_i64(value: i64) -> Option<Self> {
        u128::try_from(value).ok().and_then(Self::checked_from_u128)
    }

    #[inline(always)]
    fn from_u64(value: u64) -> Option<Self> {
        Self::checked_from_u128(value.into())
    }

    #[inline(always)]
    fn from_i128(value: i128) -> Option<Self> {
        u128::try_from(value).ok().and_then(Self::checked_from_u128)
    }

    #[inline(always)]
    fn from_u128(value: u128) -> Option<Self> {
        Self::checked_from_u128(value)
    }

    #[inline(always)]
    fn from_f64(value: f64) -> Option<Self> {
        Self::checked_from_f64(value)
    }
}

/// Converts through [`ToPrimitive::to_u128`]: `None` for a source at or above `2^128`, whose exact
/// value the trait cannot read.
impl<N: Bits> NumCast for Uint<N> {
    #[inline(always)]
    fn from<T: ToPrimitive>(value: T) -> Option<Self> {
        value.to_u128().and_then(Self::checked_from_u128)
    }
}

/// The encodings in `ceil(N / 8)` bytes.
impl<N: Bits> ToBytes for Uint<N> {
    type Bytes = GenericArray<u8, N::Bytes>;

    #[inline(always)]
    fn to_be_bytes(&self) -> Self::Bytes {
        Self::to_be_bytes(self)
    }

    #[inline(always)]
    fn to_le_bytes(&self) -> Self::Bytes {
        Self::to_le_bytes(self)
    }
}

/// The decodings of `ceil(N / 8)` bytes, modulo `2^N`.
impl<N: Bits> FromBytes for Uint<N> {
    type Bytes = GenericArray<u8, N::Bytes>;

    #[inline(always)]
    fn from_be_bytes(bytes: &Self::Bytes) -> Self {
        Self::wrapping_from_be_bytes(bytes)
    }

    #[inline(always)]
    fn from_le_bytes(bytes: &Self::Bytes) -> Self {
        Self::wrapping_from_le_bytes(bytes)
    }
}
