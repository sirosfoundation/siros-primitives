use core::ops::Add;

use biguint::{Bits, Indices};

use const_default::ConstDefault;
use generic_array::{ArrayLength, GenericArray};
use typenum::{Sum, U1};

/// A container width for field elements: a [`Bits`] width with the array lengths of the work
/// areas of its arithmetic. Every `typenum` width implements it.
pub trait Width: Bits
where
    <Self::Wide as ArrayLength>::ArrayType<u64>: Copy,
    <Self::Extended as ArrayLength>::ArrayType<u64>: Copy,
{
    /// `2 L` limbs: a full product.
    type Wide: ArrayLength + Indices;
    /// `L + 1` limbs: a sum of two elements, or a partly folded product.
    type Extended: ArrayLength + Indices;

    /// `L` zero limbs.
    const ZERO: GenericArray<u64, Self::Limbs>;
    /// `2 L` zero limbs.
    const WIDE_ZERO: GenericArray<u64, Self::Wide>;
    /// `L + 1` zero limbs.
    const EXTENDED_ZERO: GenericArray<u64, Self::Extended>;
}

impl<N> Width for N
where
    N: Bits,
    N::Limbs: Add<N::Limbs> + Add<U1>,
    Sum<N::Limbs, N::Limbs>: ArrayLength + Indices,
    Sum<N::Limbs, U1>: ArrayLength + Indices,
    <N::Limbs as ArrayLength>::ArrayType<u64>: ConstDefault,
    <Sum<N::Limbs, N::Limbs> as ArrayLength>::ArrayType<u64>: Copy + ConstDefault,
    <Sum<N::Limbs, U1> as ArrayLength>::ArrayType<u64>: Copy + ConstDefault,
{
    type Wide = Sum<N::Limbs, N::Limbs>;
    type Extended = Sum<N::Limbs, U1>;

    const ZERO: GenericArray<u64, Self::Limbs> = GenericArray::DEFAULT;
    const WIDE_ZERO: GenericArray<u64, Self::Wide> = GenericArray::DEFAULT;
    const EXTENDED_ZERO: GenericArray<u64, Self::Extended> = GenericArray::DEFAULT;
}
