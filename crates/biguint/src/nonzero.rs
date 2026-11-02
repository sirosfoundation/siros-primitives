//! Nonzero integers: divisors and moduli that need no zero test.
//!
//! The representation follows the limb count. One limb is a `NonZero<u64>` and two limbs a
//! `NonZero<u128>`, so a division by them reaches the hardware divide as the primitive division by
//! `NonZero` does. Wider values keep their limbs; the division finds their top significant limb,
//! which a nonzero value always has.

use core::fmt;
use core::hash::Hash;
use core::num::NonZero;
use core::ops::Deref;

use generic_array::{ArrayLength, GenericArray};
use typenum::{B0, B1, Bit, UInt, UTerm, Unsigned};

use crate::bits::Bits;
use crate::scan;
use crate::uint::Uint;

/// An `N`-bit integer that is not zero.
#[derive_where::derive_where(Clone, Copy, Eq, Hash, PartialEq; <N as crate::bits::sealed::Sealed>::NonZeroLimbs)]
pub struct NonZeroUint<N: Bits> {
    value: <N as crate::bits::sealed::Sealed>::NonZeroLimbs,
}

impl<N: Bits> NonZeroUint<N> {
    /// `value`, or `None` when it is zero.
    #[inline(always)]
    #[must_use]
    pub fn new(value: Uint<N>) -> Option<Self> {
        NonZeroLimbs::new(value.limbs).map(|value| Self { value })
    }

    /// The value.
    #[inline(always)]
    #[must_use]
    pub fn get(self) -> Uint<N> {
        Uint {
            limbs: self.value.limbs(),
        }
    }

    /// The value as the divisor of the primitive division, when it has at most two limbs.
    #[inline(always)]
    pub(crate) fn native(self) -> Option<NonZero<u128>> {
        self.value.native()
    }

    /// The limbs, in place above two limbs.
    #[inline(always)]
    pub(crate) fn limbs(&self) -> LimbsOf<'_, N::Limbs> {
        self.value.limbs_of()
    }
}

/// The limbs of a nonzero value: its stored limbs, or the limbs of its stored primitive.
pub enum LimbsOf<'a, L: ArrayLength> {
    /// The stored limbs.
    InPlace(&'a GenericArray<u64, L>),
    /// The limbs of the stored primitive.
    Copied(GenericArray<u64, L>),
}

impl<L: ArrayLength> Deref for LimbsOf<'_, L> {
    type Target = GenericArray<u64, L>;

    #[inline(always)]
    fn deref(&self) -> &GenericArray<u64, L> {
        match self {
            Self::InPlace(limbs) => limbs,
            Self::Copied(limbs) => limbs,
        }
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

/// Lowercase hexadecimal with a `0x` prefix, as [`Uint`] prints.
impl<N: Bits> fmt::Debug for NonZeroUint<N> {
    #[inline]
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.get(), formatter)
    }
}

/// The nonzero values of `L` limbs.
pub trait NonZeroLimbs<L: ArrayLength>: Copy + Eq + Hash {
    /// `limbs`, or `None` when they are all zero.
    fn new(limbs: GenericArray<u64, L>) -> Option<Self>;

    /// The limbs.
    fn limbs(self) -> GenericArray<u64, L>;

    /// The limbs, in place when they are stored as limbs.
    fn limbs_of(&self) -> LimbsOf<'_, L>;

    /// The value as a `NonZero<u128>`; `None` above two limbs.
    fn native(self) -> Option<NonZero<u128>>;
}

/// The representation of the nonzero values of a limb count.
pub trait NonZeroFor: ArrayLength {
    /// The representation.
    type Repr: NonZeroLimbs<Self>;
}

impl NonZeroFor for UInt<UTerm, B1> {
    type Repr = NonZero<u64>;
}

impl NonZeroFor for UInt<UInt<UTerm, B1>, B0> {
    type Repr = NonZero<u128>;
}

impl NonZeroFor for UInt<UInt<UTerm, B1>, B1> {
    type Repr = Limbs<Self>;
}

/// Four limbs and more.
impl<U: Unsigned, A: Bit, B: Bit, C: Bit> NonZeroFor for UInt<UInt<UInt<U, A>, B>, C>
where
    Self: ArrayLength,
    GenericArray<u64, Self>: Copy + Eq + Hash,
{
    type Repr = Limbs<Self>;
}

impl NonZeroLimbs<UInt<UTerm, B1>> for NonZero<u64> {
    #[inline(always)]
    fn new(limbs: GenericArray<u64, UInt<UTerm, B1>>) -> Option<Self> {
        let [low] = limbs.into_array();
        Self::new(low)
    }

    #[inline(always)]
    fn limbs(self) -> GenericArray<u64, UInt<UTerm, B1>> {
        GenericArray::from_array([self.get()])
    }

    #[inline(always)]
    fn limbs_of(&self) -> LimbsOf<'_, UInt<UTerm, B1>> {
        LimbsOf::Copied(self.limbs())
    }

    #[inline(always)]
    fn native(self) -> Option<NonZero<u128>> {
        Some(NonZero::<u128>::from(self))
    }
}

impl NonZeroLimbs<UInt<UInt<UTerm, B1>, B0>> for NonZero<u128> {
    #[inline(always)]
    fn new(limbs: GenericArray<u64, UInt<UInt<UTerm, B1>, B0>>) -> Option<Self> {
        let [low, high] = limbs.into_array();
        Self::new(u128::from(high) << 64 | u128::from(low))
    }

    #[inline(always)]
    fn limbs(self) -> GenericArray<u64, UInt<UInt<UTerm, B1>, B0>> {
        let value = self.get();
        GenericArray::from_array([value as u64, (value >> 64) as u64])
    }

    #[inline(always)]
    fn limbs_of(&self) -> LimbsOf<'_, UInt<UInt<UTerm, B1>, B0>> {
        LimbsOf::Copied(self.limbs())
    }

    #[inline(always)]
    fn native(self) -> Option<NonZero<u128>> {
        Some(self)
    }
}

/// Three limbs and more, not all zero.
#[derive_where::derive_where(Clone, Copy, Eq, Hash, PartialEq; GenericArray<u64, L>)]
pub struct Limbs<L: ArrayLength>(GenericArray<u64, L>);

impl<L: ArrayLength> NonZeroLimbs<L> for Limbs<L>
where
    GenericArray<u64, L>: Copy + Eq + Hash,
{
    #[inline(always)]
    fn new(limbs: GenericArray<u64, L>) -> Option<Self> {
        (!scan::is_zero(&limbs)).then_some(Self(limbs))
    }

    #[inline(always)]
    fn limbs(self) -> GenericArray<u64, L> {
        self.0
    }

    #[inline(always)]
    fn limbs_of(&self) -> LimbsOf<'_, L> {
        LimbsOf::InPlace(&self.0)
    }

    #[inline(always)]
    fn native(self) -> Option<NonZero<u128>> {
        None
    }
}
