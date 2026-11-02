//! Polynomials of fields in common use, and the GCM and POLYVAL views of `GF(2^128)`.

use typenum::{U8, U16, U64, U128, U233, U256, U571};

use crate::element::Gf2;
use crate::plan::Polynomial;

/// Declares a polynomial type.
macro_rules! polynomial {
    ($(#[$attribute:meta])* $name:ident, $degree:ty, [$($tap:literal),*]) => {
        $(#[$attribute])*
        #[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
        pub struct $name;

        impl Polynomial for $name {
            type Degree = $degree;
            const TAPS: &'static [usize] = &[$($tap),*];
        }
    };
}

polynomial! {
    /// `x^8 + x^4 + x^3 + x + 1`, the polynomial of AES.
    Aes, U8, [4, 3, 1, 0]
}

polynomial! {
    /// `x^16 + x^5 + x^3 + x^2 + 1`.
    Poly16, U16, [5, 3, 2, 0]
}

polynomial! {
    /// `x^64 + x^4 + x^3 + x + 1`.
    Poly64, U64, [4, 3, 1, 0]
}

polynomial! {
    /// `x^128 + x^7 + x^2 + x + 1`, the polynomial of GCM.
    Gcm, U128, [7, 2, 1, 0]
}

polynomial! {
    /// `x^233 + x^74 + 1`, the polynomial of the NIST curve B-233.
    B233, U233, [74, 0]
}

polynomial! {
    /// `x^256 + x^10 + x^5 + x^2 + 1`.
    Poly256, U256, [10, 5, 2, 0]
}

polynomial! {
    /// `x^571 + x^10 + x^5 + x^2 + 1`, the polynomial of the NIST curve B-571.
    B571, U571, [10, 5, 2, 0]
}

/// `GF(2^8)` with the polynomial of AES.
pub type Gf2p8 = Gf2<Aes>;
/// `GF(2^16)` with [`Poly16`].
pub type Gf2p16 = Gf2<Poly16>;
/// `GF(2^64)` with [`Poly64`].
pub type Gf2p64 = Gf2<Poly64>;
/// `GF(2^128)` with the polynomial of GCM.
pub type Gf2p128 = Gf2<Gcm>;
/// `GF(2^233)` with the polynomial of B-233.
pub type Gf2p233 = Gf2<B233>;
/// `GF(2^256)` with [`Poly256`].
pub type Gf2p256 = Gf2<Poly256>;
/// `GF(2^571)` with the polynomial of B-571.
pub type Gf2p571 = Gf2<B571>;

/// The 128-bit value of an element of `GF(2^128)`.
#[inline(always)]
fn to_u128(element: Gf2p128) -> u128 {
    let [low, high]: [u64; 2] = element.into_limbs().into_array();
    u128::from(low) | (u128::from(high) << 64)
}

impl Gf2p128 {
    /// The element that a GCM block denotes: GCM reads the most significant bit of the first byte
    /// as the coefficient of `x^0`, so the block is the bit reversal of the big-endian value.
    #[inline]
    #[must_use]
    pub fn from_gcm_block(block: [u8; 16]) -> Self {
        Self::from(biguint::U128::from_u128(
            u128::from_be_bytes(block).reverse_bits(),
        ))
    }

    /// The GCM block that denotes the element.
    #[inline]
    #[must_use]
    pub fn to_gcm_block(self) -> [u8; 16] {
        to_u128(self).reverse_bits().to_be_bytes()
    }

    /// The element that a POLYVAL block denotes. POLYVAL (RFC 8452) works in the field of the
    /// reciprocal polynomial `x^128 + x^127 + x^126 + x^121 + 1` with little-endian bytes and bits,
    /// and its product carries a factor `x^-128`. The map `a -> x * reverse(a)` is an
    /// isomorphism onto this field, so POLYVAL's sum and product are this field's.
    #[inline]
    #[must_use]
    pub fn from_polyval_block(block: [u8; 16]) -> Self {
        Self::from(biguint::U128::from_u128(
            u128::from_le_bytes(block).reverse_bits(),
        ))
        .times_x()
    }

    /// The POLYVAL block that denotes the element.
    #[inline]
    #[must_use]
    pub fn to_polyval_block(self) -> [u8; 16] {
        to_u128(self.over_x()).reverse_bits().to_le_bytes()
    }
}
