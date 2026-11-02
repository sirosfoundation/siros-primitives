//! Conversions between little-endian `u64` limbs and the integer types of each crate.

use core::num::NonZero;

use bench_compare::{Cells, Chain, Limbs, Visitor};
use biguint::generic_array::GenericArray;
use biguint::{Bits, ConstantTime, NonZeroUint, Uint};
use num_bigint::BigUint;

/// An operand type built from limbs.
pub trait FromLimbs {
    /// The value of `limbs`. Panics when the type has a different number of limbs, or when it
    /// refuses the value.
    fn from_limbs(limbs: &[u64]) -> Self;
}

/// A result type read back as limbs.
pub trait ToLimbs {
    /// The limbs of the result; `None` when the result holds no value.
    fn to_limbs(&self) -> Option<Limbs>;
}

/// Offers cases whose results read back through [`ToLimbs`].
pub trait Offer<O, K, V> {
    /// Offers the case that runs the crate's `function` for `op`. `prepare` builds the input of
    /// one operand set; `run` is the operation on one input.
    fn case<I, R: ToLimbs>(
        self,
        op: O,
        function: &'static str,
        prepare: impl Fn(&[Limbs]) -> I,
        run: impl Fn(&I) -> R + Copy,
    ) -> Self;

    /// Offers the case with its chain form, `x = op(x, y)`.
    fn chained<I: Chain<R>, R: ToLimbs>(
        self,
        op: O,
        function: &'static str,
        prepare: impl Fn(&[Limbs]) -> I,
        run: impl Fn(&I) -> R + Copy,
    ) -> Self;
}

impl<O: Copy + Eq + core::fmt::Display, K: bench_compare::Key, V: Visitor<O, K>> Offer<O, K, V>
    for Cells<O, K, V>
{
    fn case<I, R: ToLimbs>(
        self,
        op: O,
        function: &'static str,
        prepare: impl Fn(&[Limbs]) -> I,
        run: impl Fn(&I) -> R + Copy,
    ) -> Self {
        self.api(op, function, prepare, run, ToLimbs::to_limbs)
    }

    fn chained<I: Chain<R>, R: ToLimbs>(
        self,
        op: O,
        function: &'static str,
        prepare: impl Fn(&[Limbs]) -> I,
        run: impl Fn(&I) -> R + Copy,
    ) -> Self {
        self.chain(op, function, prepare, run, ToLimbs::to_limbs)
    }
}

/// `limbs` as an array of exactly `L` limbs.
fn array<const L: usize>(limbs: &[u64]) -> [u64; L] {
    <[u64; L]>::try_from(limbs)
        .unwrap_or_else(|_| panic!("{} limbs for a {L}-limb integer", limbs.len()))
}

impl<N: Bits> FromLimbs for Uint<N> {
    fn from_limbs(limbs: &[u64]) -> Self {
        let limbs = GenericArray::try_from_slice(limbs)
            .unwrap_or_else(|_| panic!("{} limbs for a {}-limb integer", limbs.len(), N::LIMBS));
        Self::from_generic_limbs(*limbs)
    }
}

impl<N: Bits> FromLimbs for NonZeroUint<N> {
    fn from_limbs(limbs: &[u64]) -> Self {
        Self::new(<Uint<N> as FromLimbs>::from_limbs(limbs)).expect("a nonzero divisor")
    }
}

impl<N: Bits> ToLimbs for Uint<N> {
    fn to_limbs(&self) -> Option<Limbs> {
        Some(self.as_limbs().to_vec())
    }
}

impl<T: FromLimbs> FromLimbs for ConstantTime<T> {
    fn from_limbs(limbs: &[u64]) -> Self {
        Self::new(T::from_limbs(limbs))
    }
}

impl<T: ToLimbs + Copy> ToLimbs for ConstantTime<T> {
    fn to_limbs(&self) -> Option<Limbs> {
        self.reveal().to_limbs()
    }
}

impl FromLimbs for u64 {
    fn from_limbs(limbs: &[u64]) -> Self {
        let [low] = array(limbs);
        low
    }
}

impl ToLimbs for u64 {
    fn to_limbs(&self) -> Option<Limbs> {
        Some(vec![*self])
    }
}

impl FromLimbs for u128 {
    fn from_limbs(limbs: &[u64]) -> Self {
        let [low, high] = array(limbs);
        Self::from(low) | (Self::from(high) << 64)
    }
}

impl ToLimbs for u128 {
    fn to_limbs(&self) -> Option<Limbs> {
        Some(vec![*self as u64, (*self >> 64) as u64])
    }
}

/// A primitive divisor or modulus: a nonzero value of the primitive type.
macro_rules! nonzero_primitive {
    ($($primitive:ty),*) => {
        $(
            impl FromLimbs for NonZero<$primitive> {
                fn from_limbs(limbs: &[u64]) -> Self {
                    Self::new(<$primitive>::from_limbs(limbs)).expect("a nonzero divisor")
                }
            }
        )*
    };
}

nonzero_primitive!(u64, u128);

impl<const B: usize, const L: usize> FromLimbs for ruint::Uint<B, L> {
    fn from_limbs(limbs: &[u64]) -> Self {
        Self::from_limbs(array(limbs))
    }
}

impl<const B: usize, const L: usize> ToLimbs for ruint::Uint<B, L> {
    fn to_limbs(&self) -> Option<Limbs> {
        Some(self.as_limbs().to_vec())
    }
}

impl<const L: usize> FromLimbs for crypto_bigint::Uint<L> {
    fn from_limbs(limbs: &[u64]) -> Self {
        Self::from_words(array(limbs))
    }
}

impl<const L: usize> FromLimbs for crypto_bigint::NonZero<crypto_bigint::Uint<L>> {
    fn from_limbs(limbs: &[u64]) -> Self {
        Self::new(crypto_bigint::Uint::from_limbs(limbs))
            .into_option()
            .expect("a nonzero divisor")
    }
}

impl<const L: usize> ToLimbs for crypto_bigint::Uint<L> {
    fn to_limbs(&self) -> Option<Limbs> {
        Some(self.to_words().to_vec())
    }
}

impl<const N: usize> FromLimbs for bnum::Uint<N> {
    fn from_limbs(limbs: &[u64]) -> Self {
        assert_eq!(
            limbs.len() * 8,
            N,
            "{} limbs for a {N}-byte integer",
            limbs.len()
        );
        let mut bytes = [0; N];
        for (chunk, limb) in bytes.as_chunks_mut::<8>().0.iter_mut().zip(limbs) {
            *chunk = limb.to_le_bytes();
        }
        Self::from_le_bytes(bytes)
    }
}

impl<const N: usize> ToLimbs for bnum::Uint<N> {
    fn to_limbs(&self) -> Option<Limbs> {
        Some(
            self.to_le_bytes()
                .as_chunks::<8>()
                .0
                .iter()
                .map(|chunk| u64::from_le_bytes(*chunk))
                .collect(),
        )
    }
}

impl FromLimbs for BigUint {
    fn from_limbs(limbs: &[u64]) -> Self {
        Self::new(
            limbs
                .iter()
                .flat_map(|limb| [*limb as u32, (*limb >> 32) as u32])
                .collect(),
        )
    }
}

impl ToLimbs for BigUint {
    fn to_limbs(&self) -> Option<Limbs> {
        Some(self.to_u64_digits())
    }
}
