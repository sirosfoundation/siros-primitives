//! The registers of an element, which the plan chooses at compile time: a byte or a 16-bit word in
//! a general register when the default operations index tables, `clmul::Poly` in vector registers
//! when they run on carry-less products.

use core::fmt::Debug;
use core::hash::Hash;

use biguint::Uint;
use clmul::{Poly, Width};
#[cfg(feature = "precomputed")]
use typenum::{B0, B1, IsLessOrEqual, LeEq, U8, U16};

/// The registers of an element of a field of degree `N`, and the conversions that the kernels
/// need. A conversion between a general register and `Poly` moves the value between register
/// files; the plan calls one only where its algorithm needs the other file.
pub trait Storage<N: Width>: Copy + Eq + Hash + Default + Debug + 'static {
    /// `0`.
    const ZERO: Self;
    /// `1`.
    const ONE: Self;
    /// `x`.
    const X: Self;

    /// The element of the polynomial `poly`, of degree below `N`.
    fn from_poly(poly: Poly<N>) -> Self;

    /// The polynomial.
    fn poly(self) -> Poly<N>;

    /// The element of the polynomial `word`, of degree below `N`, for `N` up to 64.
    fn from_word(word: u64) -> Self;

    /// The polynomial in one word, for `N` up to 64.
    fn word(self) -> u64;

    /// The coefficients as an integer.
    fn uint(self) -> Uint<N>;

    /// The element of the integer `value`, below `2^N`.
    fn from_uint(value: Uint<N>) -> Self;

    /// `self + rhs`.
    #[must_use]
    fn add(self, rhs: Self) -> Self;

    /// Whether the element is `0`, without a branch.
    fn is_zero(self) -> bool;
}

/// Implements [`Storage`] for an unsigned integer of a general register.
macro_rules! word_storage {
    ($($word:ty),*) => {$(
        impl<N: Width> Storage<N> for $word {
            const ZERO: Self = 0;
            const ONE: Self = 1;
            const X: Self = 2;

            #[inline(always)]
            fn from_poly(poly: Poly<N>) -> Self {
                Self::from_uint(poly.to_uint())
            }

            #[inline(always)]
            fn poly(self) -> Poly<N> {
                Poly::from_uint(Uint::from_u64(u64::from(self)))
            }

            #[inline(always)]
            fn from_word(word: u64) -> Self {
                word as Self
            }

            #[inline(always)]
            fn word(self) -> u64 {
                u64::from(self)
            }

            #[inline(always)]
            fn uint(self) -> Uint<N> {
                Uint::from_u64(u64::from(self))
            }

            #[inline(always)]
            fn from_uint(value: Uint<N>) -> Self {
                value.as_limbs().first().copied().unwrap_or(0) as Self
            }

            #[inline(always)]
            fn add(self, rhs: Self) -> Self {
                self ^ rhs
            }

            #[inline(always)]
            fn is_zero(self) -> bool {
                self == 0
            }
        }
    )*};
}

word_storage!(u8, u16);

impl<N: Width> Storage<N> for Poly<N> {
    const ZERO: Self = Poly::<N>::ZERO;
    const ONE: Self = Poly::<N>::ONE;
    const X: Self = Poly::<N>::from_uint(Uint::from_u64(2));

    #[inline(always)]
    fn from_poly(poly: Poly<N>) -> Self {
        poly
    }

    #[inline(always)]
    fn poly(self) -> Poly<N> {
        self
    }

    #[inline(always)]
    fn from_word(word: u64) -> Self {
        Poly::<N>::from_uint(Uint::from_u64(word))
    }

    #[inline(always)]
    fn word(self) -> u64 {
        self.to_uint().as_limbs().first().copied().unwrap_or(0)
    }

    #[inline(always)]
    fn uint(self) -> Uint<N> {
        self.to_uint()
    }

    #[inline(always)]
    fn from_uint(value: Uint<N>) -> Self {
        Poly::<N>::from_uint(value)
    }

    #[inline(always)]
    fn add(self, rhs: Self) -> Self {
        self ^ rhs
    }

    #[inline(always)]
    fn is_zero(self) -> bool {
        Poly::is_zero(&self)
    }
}

/// A degree with the registers of its elements. Every `typenum` unsigned integer that is a
/// [`Width`] implements it.
pub trait Representation: Width {
    /// The registers of an element.
    type Element: Storage<Self>;
}

/// `Yes` for `B1`, `No` for `B0`.
#[cfg(feature = "precomputed")]
pub trait Pick<Yes, No> {
    /// The choice.
    type Output;
}

#[cfg(feature = "precomputed")]
impl<Yes, No> Pick<Yes, No> for B1 {
    type Output = Yes;
}

#[cfg(feature = "precomputed")]
impl<Yes, No> Pick<Yes, No> for B0 {
    type Output = No;
}

/// A byte up to degree 8, a 16-bit word up to degree 16, `Poly` above.
#[cfg(feature = "precomputed")]
type Chosen<N> = <LeEq<N, U16> as Pick<<LeEq<N, U8> as Pick<u8, u16>>::Output, Poly<N>>>::Output;

#[cfg(feature = "precomputed")]
impl<N> Representation for N
where
    N: Width + IsLessOrEqual<U8> + IsLessOrEqual<U16>,
    LeEq<N, U8>: Pick<u8, u16>,
    LeEq<N, U16>: Pick<<LeEq<N, U8> as Pick<u8, u16>>::Output, Poly<N>>,
    Chosen<N>: Storage<N>,
{
    type Element = Chosen<N>;
}

/// Without tables, every operation runs on carry-less products: `Poly` at every degree.
#[cfg(not(feature = "precomputed"))]
impl<N: Width> Representation for N {
    type Element = Poly<N>;
}
