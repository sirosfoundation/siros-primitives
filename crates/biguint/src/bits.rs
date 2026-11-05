use core::ops::{Add, Div, Mul, Rem};

use const_default::ConstDefault;
use generic_array::{ArrayLength, GenericArray};
use typenum::{
    B0, B1, Gr, IsGreater, IsLessOrEqual, LeEq, Mod, NonZero, Prod, Quot, Sum, U0, U1, U2, U3, U4,
    U8, U64, U128, Unsigned,
};

use crate::nonzero::Stored;

type DivUp<A, B> = Sum<Quot<A, B>, Gr<Mod<A, B>, U0>>;
type LimbsFor<N> = DivUp<N, typenum::U64>;
type BytesFor<N> = DivUp<N, typenum::U8>;
/// `k * limbs + 8`.
type ScratchFor<N, K> = Sum<Prod<LimbsFor<N>, K>, U8>;

// Without a niche type the compiler adds a zero test before the division on its own: the storage
// of a nonzero value of one or two limbs is `core::num::NonZero`, which tells it the divisor is
// nonzero.
/// The storage of a nonzero value above 64 bits: `NonZero<u128>` up to 128 bits, the limbs above.
type Wider<N> =
    <LeEq<N, U128> as Pick<core::num::NonZero<u128>, GenericArray<u64, LimbsFor<N>>>>::Output;
/// The storage of a nonzero `N`-bit value.
type NonZeroFor<N> = <LeEq<N, U64> as Pick<core::num::NonZero<u64>, Wider<N>>>::Output;

/// `Yes` for `B1`, `No` for `B0`.
pub trait Pick<Yes, No> {
    /// The choice.
    type Output;
}

impl<Yes, No> Pick<Yes, No> for B1 {
    type Output = Yes;
}

impl<Yes, No> Pick<Yes, No> for B0 {
    type Output = No;
}

pub(crate) mod sealed {
    use generic_array::ArrayLength;

    /// Seals [`super::Bits`] and carries the array lengths of the work areas.
    pub trait Sealed {
        /// `ceil(N / 64) + 1`: a dividend shifted left until its divisor's top bit is set.
        type Window: ArrayLength;
        /// `ceil(N / 64) + 8`: the work area of a full product into `N` bits of two equally long
        /// operands.
        type BalancedScratch: ArrayLength;
        /// `2 * ceil(N / 64) + 8`: the work area of a full product into `N` bits of two operands
        /// of different lengths.
        type ProductScratch: ArrayLength;
        /// `3 * ceil(N / 64) + 8`: the work area of a product modulo `2^N`.
        type TruncatedScratch: ArrayLength;
        /// `4 * ceil(N / 64) + 8`: the work area of a division by an `N`-bit divisor.
        type DivisionScratch: ArrayLength;
        /// `2 * ceil(N / 64) + 1`: the product of two `N`-bit values with the limb above it.
        type ProductWindow: ArrayLength;
    }
}

impl<N> sealed::Sealed for N
where
    N: Unsigned + Div<typenum::U64> + Rem<typenum::U64>,
    Mod<N, typenum::U64>: IsGreater<U0>,
    Quot<N, typenum::U64>: Add<Gr<Mod<N, typenum::U64>, U0>>,
    LimbsFor<N>: Add<U1> + Add<U8> + Mul<U2> + Mul<U3> + Mul<U4>,
    Sum<LimbsFor<N>, U1>: ArrayLength,
    Sum<LimbsFor<N>, U8>: ArrayLength,
    Prod<LimbsFor<N>, U2>: Add<U8> + Add<U1>,
    Sum<Prod<LimbsFor<N>, U2>, U1>: ArrayLength,
    Prod<LimbsFor<N>, U3>: Add<U8>,
    Prod<LimbsFor<N>, U4>: Add<U8>,
    ScratchFor<N, U2>: ArrayLength,
    ScratchFor<N, U3>: ArrayLength,
    ScratchFor<N, U4>: ArrayLength,
{
    type Window = Sum<LimbsFor<N>, U1>;
    type BalancedScratch = Sum<LimbsFor<N>, U8>;
    type ProductScratch = ScratchFor<N, U2>;
    type TruncatedScratch = ScratchFor<N, U3>;
    type DivisionScratch = ScratchFor<N, U4>;
    type ProductWindow = Sum<Prod<LimbsFor<N>, U2>, U1>;
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
    /// The storage of a nonzero `Uint<N>`: `core::num::NonZero<u64>` up to 64 bits,
    /// `core::num::NonZero<u128>` up to 128 bits, the limbs above.
    type NonZeroLimbs: Stored;

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
    N: IsLessOrEqual<U64> + IsLessOrEqual<U128>,
    LeEq<N, U128>: Pick<core::num::NonZero<u128>, GenericArray<u64, LimbsFor<N>>>,
    LeEq<N, U64>: Pick<core::num::NonZero<u64>, Wider<N>>,
    NonZeroFor<N>: Stored,
{
    type Limbs = LimbsFor<N>;
    type Bytes = BytesFor<N>;
    type NonZeroLimbs = NonZeroFor<N>;

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
