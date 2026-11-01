use core::ops::{Add, Div, Rem};

use const_default::ConstDefault;
use generic_array::ArrayLength;
use typenum::{Gr, IsGreater, Mod, NonZero, Quot, Sum, U0, U1, Unsigned};

type DivUp<A, B> = Sum<Quot<A, B>, Gr<Mod<A, B>, U0>>;
type LimbsFor<N> = DivUp<N, typenum::U64>;
type BytesFor<N> = DivUp<N, typenum::U8>;

pub(crate) mod sealed {
    use generic_array::ArrayLength;

    /// Seals [`super::Bits`] and carries the array length that only the division uses.
    pub trait Sealed {
        /// `ceil(N / 64) + 1`: a dividend shifted left until its divisor's top bit is set.
        type Window: ArrayLength;
    }
}

impl<N> sealed::Sealed for N
where
    N: Unsigned + Div<typenum::U64> + Rem<typenum::U64>,
    Mod<N, typenum::U64>: IsGreater<U0>,
    Quot<N, typenum::U64>: Add<Gr<Mod<N, typenum::U64>, U0>>,
    LimbsFor<N>: Add<U1>,
    Sum<LimbsFor<N>, U1>: ArrayLength,
{
    type Window = Sum<LimbsFor<N>, U1>;
}

/// A bit width `N >= 1`, given as a `typenum` unsigned integer, with its storage sizes.
///
/// It is sealed: exactly the nonzero `typenum` unsigned integers implement it.
pub trait Bits: sealed::Sealed + Unsigned + NonZero
where
    <Self::Limbs as ArrayLength>::ArrayType<u64>: Copy + ConstDefault,
{
    /// `ceil(N / 64)`: the little-endian `u64` limbs of a `Uint<N>`.
    type Limbs: ArrayLength;
    /// `ceil(N / 8)`: the bytes of an encoded `Uint<N>`.
    type Bytes: ArrayLength;

    /// `N`.
    const BITS: usize;
    /// `ceil(N / 64)`.
    const LIMBS: usize;
    /// `ceil(N / 8)`.
    const BYTES: usize;
    /// The bits of the top limb that lie below `N`.
    const TOP_MASK: u64;
}

impl<N> Bits for N
where
    N: sealed::Sealed + Unsigned + NonZero,
    N: Div<typenum::U64> + Rem<typenum::U64> + Div<typenum::U8> + Rem<typenum::U8>,
    Mod<N, typenum::U64>: IsGreater<U0>,
    Quot<N, typenum::U64>: Add<Gr<Mod<N, typenum::U64>, U0>>,
    LimbsFor<N>: ArrayLength,
    <LimbsFor<N> as ArrayLength>::ArrayType<u64>: Copy + ConstDefault,
    Mod<N, typenum::U8>: IsGreater<U0>,
    Quot<N, typenum::U8>: Add<Gr<Mod<N, typenum::U8>, U0>>,
    BytesFor<N>: ArrayLength,
{
    type Limbs = LimbsFor<N>;
    type Bytes = BytesFor<N>;

    const BITS: usize = N::USIZE;
    const LIMBS: usize = LimbsFor::<N>::USIZE;
    const BYTES: usize = BytesFor::<N>::USIZE;
    const TOP_MASK: u64 = if N::USIZE % 64 == 0 {
        u64::MAX
    } else {
        (1u64 << (N::USIZE % 64)) - 1
    };
}

/// 8 bits.
pub type Bits8 = typenum::U8;
/// 16 bits.
pub type Bits16 = typenum::U16;
/// 32 bits.
pub type Bits32 = typenum::U32;
/// 64 bits.
pub type Bits64 = typenum::U64;
/// 128 bits.
pub type Bits128 = typenum::U128;
/// 256 bits.
pub type Bits256 = typenum::U256;
/// 384 bits.
pub type Bits384 = typenum::U384;
/// 512 bits.
pub type Bits512 = typenum::U512;
