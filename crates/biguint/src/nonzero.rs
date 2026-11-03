//! Values that are not zero: divisors and moduli that need no zero test, and the operations on a
//! nonzero `Uint` whose result is never zero.

use core::fmt;
use core::ops::BitOr;

use generic_array::GenericArray;
use num_traits::Zero;

use crate::bits::Bits;
use crate::num::One;
use crate::uint::Uint;

/// A value that is not zero.
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NonZero<T>(pub(crate) T);

/// An `N`-bit integer that is not zero.
pub type NonZeroUint<N> = NonZero<Uint<N>>;

/// The value refused by [`NonZero`]'s `TryFrom`: it is zero.
#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
#[error("the value is zero")]
pub struct IsZero;

impl<T: Zero> NonZero<T> {
    /// `value`, or `None` when it is zero.
    #[inline(always)]
    #[must_use]
    pub fn new(value: T) -> Option<Self> {
        (!value.is_zero()).then_some(Self(value))
    }

    /// The value.
    #[inline(always)]
    #[must_use]
    pub fn get(self) -> T {
        self.0
    }

    /// A reference to the value.
    #[inline(always)]
    #[must_use]
    pub const fn as_ref(&self) -> &T {
        &self.0
    }
}

/// The value's own formatting.
impl<T: fmt::Debug> fmt::Debug for NonZero<T> {
    #[inline]
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.0, formatter)
    }
}

/// The value's own formatting.
impl<T: fmt::Display> fmt::Display for NonZero<T> {
    #[inline]
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, formatter)
    }
}

/// The value's one, which [`One`]'s contract keeps nonzero.
impl<T: One> One for NonZero<T> {
    const ONE: Self = Self(T::ONE);
}

impl<N: Bits> From<NonZeroUint<N>> for Uint<N> {
    #[inline(always)]
    fn from(value: NonZeroUint<N>) -> Self {
        value.0
    }
}

impl<N: Bits> TryFrom<Uint<N>> for NonZeroUint<N> {
    type Error = IsZero;

    #[inline(always)]
    fn try_from(value: Uint<N>) -> Result<Self, IsZero> {
        Self::new(value).ok_or(IsZero)
    }
}

impl<N: Bits> Zero for Uint<N> {
    #[inline(always)]
    fn zero() -> Self {
        Self::ZERO
    }

    #[inline(always)]
    fn is_zero(&self) -> bool {
        Uint::is_zero(self)
    }
}

/// A nonzero value with bits set: it stays nonzero.
impl<N: Bits, M: Bits> BitOr<Uint<M>> for NonZeroUint<N> {
    type Output = Self;

    #[inline(always)]
    fn bitor(self, rhs: Uint<M>) -> Self {
        Self(self.0 | rhs)
    }
}

/// A nonzero value with the bits of another set: it stays nonzero.
impl<N: Bits, M: Bits> BitOr<NonZeroUint<M>> for NonZeroUint<N> {
    type Output = Self;

    #[inline(always)]
    fn bitor(self, rhs: NonZeroUint<M>) -> Self {
        Self(self.0 | rhs.0)
    }
}

impl<N: Bits> NonZeroUint<N> {
    /// `self + rhs`, or the largest value when the sum overflows.
    #[inline(always)]
    #[must_use]
    pub fn saturating_add(self, rhs: Uint<N>) -> Self {
        Self(self.0.saturating_add(rhs))
    }

    /// `self + rhs`, or `None` when the sum overflows.
    #[inline(always)]
    #[must_use]
    pub fn checked_add(self, rhs: Uint<N>) -> Option<Self> {
        // Without overflow the sum is at least `self`.
        self.0.checked_add(rhs).map(Self)
    }

    /// `self * rhs`, or the largest value when the product overflows.
    #[inline(always)]
    #[must_use]
    pub fn saturating_mul(self, rhs: Self) -> Self {
        Self(self.0.saturating_mul(rhs.0))
    }

    /// `self * rhs`, or `None` when the product overflows.
    #[inline(always)]
    #[must_use]
    pub fn checked_mul(self, rhs: Self) -> Option<Self> {
        // The exact product of two nonzero values is nonzero.
        self.0.checked_mul(rhs.0).map(Self)
    }

    /// `self^exponent`, or `None` when it overflows. Squaring from the exponent's top bit keeps
    /// every partial power at most the result.
    #[inline(always)]
    #[must_use]
    pub fn checked_pow(self, exponent: u32) -> Option<Self> {
        let mut power = Self(Uint::ONE);
        for bit in (0..u32::BITS - exponent.leading_zeros()).rev() {
            power = power.checked_mul(power)?;
            if exponent >> bit & 1 == 1 {
                power = power.checked_mul(self)?;
            }
        }
        Some(power)
    }

    /// The zero bits above the top set bit.
    #[inline(always)]
    #[must_use]
    pub fn leading_zeros(self) -> usize {
        self.0.leading_zeros()
    }

    /// The zero bits below the bottom set bit.
    #[inline(always)]
    #[must_use]
    pub fn trailing_zeros(self) -> usize {
        self.0.trailing_zeros()
    }

    /// `floor(log2(self))`: the index of the top set bit.
    #[inline(always)]
    #[must_use]
    pub fn ilog2(self) -> usize {
        self.0.bit_len() - 1
    }

    /// Whether exactly one bit is set.
    #[inline(always)]
    #[must_use]
    pub fn is_power_of_two(self) -> bool {
        self.0.count_ones() == 1
    }

    /// The value as the divisor of the primitive division, when it has at most two limbs.
    #[inline(always)]
    pub(crate) fn native(self) -> Option<core::num::NonZero<u128>> {
        match self.0.as_limbs() {
            [low] => core::num::NonZero::new(u128::from(*low)),
            [low, high] => core::num::NonZero::new(u128::from(*high) << 64 | u128::from(*low)),
            _ => None,
        }
    }

    /// The limbs.
    #[inline(always)]
    pub(crate) fn limbs(&self) -> &GenericArray<u64, N::Limbs> {
        &self.0.limbs
    }
}

/// The count of the limbs of a nonzero value up to its top nonzero one.
///
/// The count feeds every step of a division. A scan from the top with a branch per limb predicts
/// it from the divisors before; a branch-free scan puts its latency on the division's path, which
/// costs 60 cycles at 512 bits.
#[inline(always)]
pub(crate) fn significant_len(limbs: &[u64]) -> usize {
    // A nonzero value whose upper limbs are zero has its nonzero limb at the bottom.
    limbs
        .iter()
        .skip(1)
        .rposition(|limb| *limb != 0)
        .map_or(1, |index| index + 2)
}
