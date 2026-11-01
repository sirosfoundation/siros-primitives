//! Conversions between little-endian `u64` limbs and the integer types of each crate.

use core::num::NonZero;

use biguint::generic_array::GenericArray;
use biguint::{Bits, Uint};
use num_bigint::BigUint;

/// Little-endian `u64` limbs.
pub type Limbs = Vec<u64>;

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

/// `limbs` without its high zero limbs.
pub fn trim(limbs: &[u64]) -> &[u64] {
    let len = limbs
        .iter()
        .rposition(|limb| *limb != 0)
        .map_or(0, |top| top + 1);
    &limbs[..len]
}

/// `limbs` as an array of exactly `L` limbs.
fn array<const L: usize>(limbs: &[u64]) -> [u64; L] {
    <[u64; L]>::try_from(limbs)
        .unwrap_or_else(|_| panic!("{} limbs for a {L}-limb integer", limbs.len()))
}

impl<T: ToLimbs> ToLimbs for Option<T> {
    fn to_limbs(&self) -> Option<Limbs> {
        self.as_ref().and_then(ToLimbs::to_limbs)
    }
}

impl<N: Bits> FromLimbs for Uint<N> {
    fn from_limbs(limbs: &[u64]) -> Self {
        let limbs = GenericArray::try_from_slice(limbs)
            .unwrap_or_else(|_| panic!("{} limbs for a {}-limb integer", limbs.len(), N::LIMBS));
        Self::from_generic_limbs(*limbs)
    }
}

impl<N: Bits> ToLimbs for Uint<N> {
    fn to_limbs(&self) -> Option<Limbs> {
        Some(self.as_limbs().to_vec())
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
