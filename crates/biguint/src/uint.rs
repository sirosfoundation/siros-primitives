use const_default::ConstDefault;
use generic_array::sequence::GenericSequence;
use generic_array::{GenericArray, IntoArrayLength};
use typenum::Const;

use crate::bits::{Bits, Bits8, Bits16, Bits32, Bits64, Bits128, Bits256, Bits384, Bits512};
use crate::stream::{LimbStream, ZeroExtended, for_each_slot};

/// An unsigned integer of exactly `N` bits.
///
/// The value is held in `ceil(N / 64)` little-endian `u64` limbs. The bits of the top limb at and
/// above `N` are always zero. Arithmetic and the integer and limb constructors work modulo `2^N`.
/// The byte decoders and the checked conversions refuse a value that needs more bits.
#[repr(transparent)]
#[derive_where::derive_where(Clone, Copy, Default, Eq, Hash, PartialEq; GenericArray<u64, N::Limbs>)]
pub struct Uint<N: Bits> {
    pub(crate) limbs: GenericArray<u64, N::Limbs>,
}

/// An 8-bit unsigned integer.
pub type U8 = Uint<Bits8>;
/// A 16-bit unsigned integer.
pub type U16 = Uint<Bits16>;
/// A 32-bit unsigned integer.
pub type U32 = Uint<Bits32>;
/// A 64-bit unsigned integer.
pub type U64 = Uint<Bits64>;
/// A 128-bit unsigned integer.
pub type U128 = Uint<Bits128>;
/// A 256-bit unsigned integer.
pub type U256 = Uint<Bits256>;
/// A 384-bit unsigned integer.
pub type U384 = Uint<Bits384>;
/// A 512-bit unsigned integer.
pub type U512 = Uint<Bits512>;

impl<N: Bits> Uint<N> {
    /// `0`.
    pub const ZERO: Self = Self::from_u64(0);

    /// `1`.
    pub const ONE: Self = Self::from_u64(1);

    /// `value mod 2^N`.
    #[inline(always)]
    #[must_use]
    pub const fn from_u64(value: u64) -> Self {
        let mut limbs = GenericArray::<u64, N::Limbs>::DEFAULT;
        if let [low, ..] = limbs.as_mut_slice() {
            *low = value;
        }
        Self::from_generic_limbs(limbs)
    }

    /// `value mod 2^N`.
    #[inline(always)]
    #[must_use]
    pub const fn from_u128(value: u128) -> Self {
        let mut limbs = GenericArray::<u64, N::Limbs>::DEFAULT;
        match limbs.as_mut_slice() {
            [low, high, ..] => {
                *low = value as u64;
                *high = (value >> 64) as u64;
            }
            [low] => *low = value as u64,
            [] => {}
        }
        Self::from_generic_limbs(limbs)
    }

    /// The value of the little-endian `limbs`, modulo `2^N`.
    #[inline(always)]
    #[must_use]
    pub const fn from_limbs<const L: usize>(limbs: [u64; L]) -> Self
    where
        Const<L>: IntoArrayLength<ArrayLength = N::Limbs>,
    {
        Self::from_generic_limbs(GenericArray::from_array(limbs))
    }

    /// The value of the little-endian `limbs`, modulo `2^N`.
    #[inline(always)]
    #[must_use]
    pub const fn from_generic_limbs(mut limbs: GenericArray<u64, N::Limbs>) -> Self {
        if let Some(top) = limbs.as_mut_slice().last_mut() {
            *top &= N::TOP_MASK;
        }
        Self { limbs }
    }

    /// The little-endian limbs.
    #[inline(always)]
    #[must_use]
    pub const fn as_limbs(&self) -> &[u64] {
        self.limbs.as_slice()
    }

    /// The little-endian limbs.
    #[inline(always)]
    #[must_use]
    pub const fn into_limbs(self) -> GenericArray<u64, N::Limbs> {
        self.limbs
    }

    /// Zero-extends to `M` bits, or keeps the low `M` bits when `M < N`.
    #[inline(always)]
    #[must_use]
    pub fn resize<M: Bits>(self) -> Uint<M> {
        Uint::from_generic_limbs(GenericArray::generate(|index| {
            self.limbs.get(index).copied().unwrap_or(0)
        }))
    }

    /// Whether the value is `0`.
    #[inline(always)]
    #[must_use]
    pub fn is_zero(&self) -> bool {
        self.limbs.iter().all(|limb| *limb == 0)
    }

    /// Bit `index`, counted from the least significant bit; `false` at and above `N`.
    #[inline(always)]
    #[must_use]
    pub fn bit_at(&self, index: usize) -> bool {
        index < N::BITS
            && self
                .limbs
                .get(index / 64)
                .is_some_and(|&limb| limb & (1u64 << (index % 64)) != 0)
    }

    /// The number of bits needed to write the value: `0` for zero, else one more than the index
    /// of the highest set bit.
    #[inline(always)]
    #[must_use]
    pub fn bit_len(&self) -> usize {
        for (index, limb) in self.limbs.iter().copied().enumerate().rev() {
            if limb != 0 {
                return index * 64 + (64 - limb.leading_zeros() as usize);
            }
        }
        0
    }

    /// `N - bit_len()`.
    #[inline(always)]
    #[must_use]
    pub fn leading_zeros(&self) -> usize {
        N::BITS - self.bit_len()
    }

    /// The index of the lowest set bit; `N` for zero.
    #[inline(always)]
    #[must_use]
    pub fn trailing_zeros(&self) -> usize {
        for (index, limb) in self.limbs.iter().copied().enumerate() {
            if limb != 0 {
                return index * 64 + limb.trailing_zeros() as usize;
            }
        }
        N::BITS
    }

    /// The number of set bits.
    #[inline(always)]
    #[must_use]
    pub fn count_ones(&self) -> usize {
        self.limbs
            .iter()
            .map(|limb| limb.count_ones() as usize)
            .sum()
    }

    /// The number of clear bits below `N`.
    #[inline(always)]
    #[must_use]
    pub fn count_zeros(&self) -> usize {
        N::BITS - self.count_ones()
    }

    /// Collects the first `ceil(N / 64)` limbs of `stream`, modulo `2^N`. Up to four limbs the
    /// plain pull unrolls; longer pulls run in groups of four.
    #[inline(always)]
    pub(crate) fn from_stream(mut stream: impl LimbStream) -> Self {
        if N::LIMBS <= 4 {
            return Self::from_generic_limbs(GenericArray::generate(|_| stream.next_limb()));
        }
        let mut limbs = GenericArray::<u64, N::Limbs>::default();
        for_each_slot(&mut limbs, |slot| *slot = stream.next_limb());
        Self::from_generic_limbs(limbs)
    }

    /// The limbs, then `0` forever.
    #[inline(always)]
    pub(crate) fn stream(&self) -> ZeroExtended<'_> {
        ZeroExtended::new(&self.limbs)
    }
}
