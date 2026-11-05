//! Values that are not zero: divisors and moduli that need no zero test, and the operations on a
//! nonzero `Uint` whose result is never zero.

use core::cmp::Ordering;
use core::fmt;
use core::hash::{Hash, Hasher};
use core::ops::BitOr;

use const_default::ConstDefault;
use generic_array::{ArrayLength, GenericArray};
use num_traits::Zero;

use crate::bits::Bits;
use crate::num::One;
use crate::uint::Uint;

/// A value that is not zero, held in the storage of [`Divisor::Stored`].
#[repr(transparent)]
pub struct NonZero<T: Divisor>(pub(crate) T::Stored);

/// An `N`-bit integer that is not zero.
pub type NonZeroUint<N> = NonZero<Uint<N>>;

/// A type whose nonzero values [`NonZero`] holds, with their storage.
pub trait Divisor: Zero + Copy {
    /// The storage of a nonzero value.
    type Stored: Copy;

    /// The storage of `self`, or `None` when it is zero.
    fn store(self) -> Option<Self::Stored>;

    /// The value that `stored` holds.
    fn load(stored: Self::Stored) -> Self;
}

/// The storage of a nonzero `Uint`: little-endian limbs.
pub trait Stored: Copy + Eq + Hash + One {
    /// The value of `limbs`, or `None` when it is zero. Limbs past the storage's are dropped.
    fn from_limbs(limbs: &[u64]) -> Option<Self>;

    /// The limbs, into `L` limbs.
    fn limbs<L: ArrayLength>(self) -> GenericArray<u64, L>;

    /// `self | bits`; limbs of `bits` past the storage's are dropped.
    #[must_use]
    fn or(self, bits: &[u64]) -> Self;

    /// The value as the divisor of the primitive division, when it has at most two limbs.
    fn native(self) -> Option<core::num::NonZero<u128>>;
}

impl Stored for core::num::NonZero<u64> {
    #[inline(always)]
    fn from_limbs(limbs: &[u64]) -> Option<Self> {
        Self::new(limbs.first().copied().unwrap_or(0))
    }

    #[inline(always)]
    fn limbs<L: ArrayLength>(self) -> GenericArray<u64, L> {
        let mut limbs = GenericArray::default();
        if let Some(slot) = limbs.first_mut() {
            *slot = self.get();
        }
        limbs
    }

    #[inline(always)]
    fn or(self, bits: &[u64]) -> Self {
        self | bits.first().copied().unwrap_or(0)
    }

    #[inline(always)]
    fn native(self) -> Option<core::num::NonZero<u128>> {
        Some(self.into())
    }
}

impl Stored for core::num::NonZero<u128> {
    #[inline(always)]
    fn from_limbs(limbs: &[u64]) -> Option<Self> {
        Self::new(wide(limbs))
    }

    #[inline(always)]
    fn limbs<L: ArrayLength>(self) -> GenericArray<u64, L> {
        let value = self.get();
        let mut limbs = GenericArray::default();
        for (slot, limb) in limbs.iter_mut().zip([value as u64, (value >> 64) as u64]) {
            *slot = limb;
        }
        limbs
    }

    #[inline(always)]
    fn or(self, bits: &[u64]) -> Self {
        self | wide(bits)
    }

    #[inline(always)]
    fn native(self) -> Option<core::num::NonZero<u128>> {
        Some(self)
    }
}

impl<L: ArrayLength> Stored for GenericArray<u64, L>
where
    L::ArrayType<u64>: Copy + ConstDefault,
{
    #[inline(always)]
    fn from_limbs(limbs: &[u64]) -> Option<Self> {
        let mut value = Self::default();
        for (slot, limb) in value.iter_mut().zip(limbs) {
            *slot = *limb;
        }
        value.iter().any(|limb| *limb != 0).then_some(value)
    }

    #[inline(always)]
    fn limbs<M: ArrayLength>(self) -> GenericArray<u64, M> {
        let mut limbs = GenericArray::default();
        for (slot, limb) in limbs.iter_mut().zip(self) {
            *slot = limb;
        }
        limbs
    }

    #[inline(always)]
    fn or(self, bits: &[u64]) -> Self {
        let mut value = self;
        for (slot, limb) in value.iter_mut().zip(bits) {
            *slot |= *limb;
        }
        value
    }

    #[inline(always)]
    fn native(self) -> Option<core::num::NonZero<u128>> {
        None
    }
}

/// The value of the low two limbs of `limbs`.
#[inline(always)]
fn wide(limbs: &[u64]) -> u128 {
    let low = limbs.first().copied().unwrap_or(0);
    let high = limbs.get(1).copied().unwrap_or(0);
    (u128::from(high) << 64) | u128::from(low)
}

impl<N: Bits> Divisor for Uint<N> {
    type Stored = N::NonZeroLimbs;

    #[inline(always)]
    fn store(self) -> Option<N::NonZeroLimbs> {
        N::NonZeroLimbs::from_limbs(&self.limbs)
    }

    #[inline(always)]
    fn load(stored: N::NonZeroLimbs) -> Self {
        Self::from_generic_limbs(stored.limbs())
    }
}

/// The value refused by [`NonZero`]'s `TryFrom`: it is zero.
#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
#[error("the value is zero")]
pub struct IsZero;

impl<T: Divisor> NonZero<T> {
    /// `value`, or `None` when it is zero.
    #[inline(always)]
    #[must_use]
    pub fn new(value: T) -> Option<Self> {
        value.store().map(Self)
    }

    /// The value.
    #[inline(always)]
    #[must_use]
    pub fn get(self) -> T {
        T::load(self.0)
    }
}

impl<T: Divisor> Clone for NonZero<T> {
    #[inline(always)]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: Divisor> Copy for NonZero<T> {}

impl<T: Divisor + PartialEq> PartialEq for NonZero<T> {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        self.get() == other.get()
    }
}

impl<T: Divisor + Eq> Eq for NonZero<T> {}

impl<T: Divisor + PartialOrd> PartialOrd for NonZero<T> {
    #[inline(always)]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.get().partial_cmp(&other.get())
    }
}

impl<T: Divisor + Ord> Ord for NonZero<T> {
    #[inline(always)]
    fn cmp(&self, other: &Self) -> Ordering {
        self.get().cmp(&other.get())
    }
}

impl<T: Divisor + Hash> Hash for NonZero<T> {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.get().hash(state);
    }
}

/// The value's own formatting.
impl<T: Divisor + fmt::Debug> fmt::Debug for NonZero<T> {
    #[inline]
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.get(), formatter)
    }
}

/// The value's own formatting.
impl<T: Divisor + fmt::Display> fmt::Display for NonZero<T> {
    #[inline]
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.get(), formatter)
    }
}

/// The value's one, which [`One`]'s contract keeps nonzero.
impl<T: Divisor<Stored: One>> One for NonZero<T> {
    const ONE: Self = Self(T::Stored::ONE);
}

impl One for core::num::NonZero<u64> {
    const ONE: Self = Self::MIN;
}

impl One for core::num::NonZero<u128> {
    const ONE: Self = Self::MIN;
}

impl<L: ArrayLength> One for GenericArray<u64, L>
where
    L::ArrayType<u64>: Copy + ConstDefault,
{
    const ONE: Self = {
        let mut limbs = <Self as ConstDefault>::DEFAULT;
        if let [low, ..] = limbs.as_mut_slice() {
            *low = 1;
        }
        limbs
    };
}

impl<N: Bits> From<NonZeroUint<N>> for Uint<N> {
    #[inline(always)]
    fn from(value: NonZeroUint<N>) -> Self {
        value.get()
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
        Self(self.0.or(&(Uint::<N>::ZERO | rhs).limbs))
    }
}

/// A nonzero value with the bits of another set: it stays nonzero.
impl<N: Bits, M: Bits> BitOr<NonZeroUint<M>> for NonZeroUint<N> {
    type Output = Self;

    #[inline(always)]
    fn bitor(self, rhs: NonZeroUint<M>) -> Self {
        self | rhs.get()
    }
}

impl<N: Bits> NonZeroUint<N> {
    /// `self + rhs`, or the largest value when the sum overflows.
    #[inline(always)]
    #[must_use]
    pub fn saturating_add(self, rhs: Uint<N>) -> Self {
        self.checked_add(rhs).unwrap_or(self | Uint::<N>::MAX)
    }

    /// `self + rhs`, or `None` when the sum overflows.
    #[inline(always)]
    #[must_use]
    pub fn checked_add(self, rhs: Uint<N>) -> Option<Self> {
        // Without overflow the sum is at least `self`, so `new` finds it nonzero.
        self.get().checked_add(rhs).and_then(Self::new)
    }

    /// `self * rhs`, or the largest value when the product overflows.
    #[inline(always)]
    #[must_use]
    pub fn saturating_mul(self, rhs: Self) -> Self {
        self.checked_mul(rhs).unwrap_or(self | Uint::<N>::MAX)
    }

    /// `self * rhs`, or `None` when the product overflows.
    #[inline(always)]
    #[must_use]
    pub fn checked_mul(self, rhs: Self) -> Option<Self> {
        // The exact product of two nonzero values is nonzero.
        self.get().checked_mul(rhs.get()).and_then(Self::new)
    }

    /// `self^exponent`, or `None` when it overflows. Squaring from the exponent's top bit keeps
    /// every partial power at most the result.
    #[inline(always)]
    #[must_use]
    pub fn checked_pow(self, exponent: u32) -> Option<Self> {
        let mut power = Self::ONE;
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
        self.get().leading_zeros()
    }

    /// The zero bits below the bottom set bit.
    #[inline(always)]
    #[must_use]
    pub fn trailing_zeros(self) -> usize {
        self.get().trailing_zeros()
    }

    /// `floor(log2(self))`: the index of the top set bit.
    #[inline(always)]
    #[must_use]
    pub fn ilog2(self) -> usize {
        self.get().bit_len() - 1
    }

    /// Whether exactly one bit is set.
    #[inline(always)]
    #[must_use]
    pub fn is_power_of_two(self) -> bool {
        self.get().count_ones() == 1
    }

    /// The value as the divisor of the primitive division, when it has at most two limbs.
    #[inline(always)]
    pub(crate) fn native(self) -> Option<core::num::NonZero<u128>> {
        self.0.native()
    }

    /// The limbs.
    #[inline(always)]
    pub(crate) fn limbs(self) -> GenericArray<u64, N::Limbs> {
        self.get().limbs
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
