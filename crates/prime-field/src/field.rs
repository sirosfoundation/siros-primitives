use core::fmt;
use core::iter::{Product, Sum};
use core::ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign};

use biguint::generic_array::GenericArray;
use biguint::{Bits, OutOfRange, Uint};

use crate::kernel::{add_mod, bit, bit_len, equal, is_zero, neg_mod, sub_mod};
use crate::params::{Limbs, Shape};
use crate::width::Width;

/// A prime modulus. Declare a field by implementing this trait for a marker type: the field is
/// [`Fp`] of that type, and every other constant derives from `MODULUS` at compile time.
///
/// `MODULUS` is an odd prime. A composite odd modulus yields the ring of its residues, in which
/// [`Fp::invert`] returns `None` for every non-unit and [`Fp::sqrt`] is meaningless.
pub trait Modulus: Copy + 'static {
    /// The container width. Its limbs hold an element; the modulus fits the width.
    type Width: Width;
    /// The prime.
    const MODULUS: Uint<Self::Width>;
}

/// An element of the prime field of `M`, held as `L = ceil(width / 64)` limbs.
///
/// Fields whose modulus is `2^k - c` with `c < 2^64` and `c` small against `2^k` (Mersenne-31,
/// Goldilocks, secp256k1, P-521) hold plain residues and reduce products by folding. Every
/// other field holds Montgomery residues `x R mod p` with `R = 2^(64 L)` and reduces products
/// by Montgomery multiplication. The representation is private; the conversions take and return
/// canonical integers.
#[repr(transparent)]
#[derive_where::derive_where(Clone, Copy, Default, Eq, Hash, PartialEq; Limbs<M>)]
pub struct Fp<M: Modulus> {
    limbs: Limbs<M>,
}

/// A value refused as a field element: it is at or above the modulus, or wider than the
/// container.
#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
#[error("the value is not a canonical element: it is at or above the modulus")]
pub struct NotCanonical;

impl<M: Modulus> Fp<M> {
    /// `0`.
    pub const ZERO: Self = Self::from_limbs(M::Width::ZERO);
    /// `1`.
    pub const ONE: Self = Self::from_limbs(Shape::<M>::ONE);
    /// The modulus `p`.
    pub const MODULUS: Uint<M::Width> = M::MODULUS;
    /// The bit length of the modulus.
    pub const BITS: usize = Shape::<M>::BITS;

    /// The element whose representation is `limbs`.
    #[inline(always)]
    pub(crate) const fn from_limbs(limbs: Limbs<M>) -> Self {
        Self { limbs }
    }

    /// The representation.
    #[inline(always)]
    pub(crate) const fn limbs(&self) -> &[u64] {
        self.limbs.as_slice()
    }

    /// `value mod p`.
    #[inline(always)]
    #[must_use]
    pub const fn from_u64(value: u64) -> Self {
        let reduced = if Shape::<M>::LIMBS == 1 {
            value % crate::kernel::limb(Shape::<M>::P.as_slice(), 0)
        } else {
            value
        };
        Self::from_limbs(Shape::<M>::encode(
            Uint::<M::Width>::from_u64(reduced).as_limbs(),
        ))
    }

    /// The element `value`.
    ///
    /// # Errors
    ///
    /// [`NotCanonical`] when `value >= p`.
    #[inline(always)]
    pub const fn from_uint(value: Uint<M::Width>) -> Result<Self, NotCanonical> {
        if Shape::<M>::is_canonical(value.as_limbs()) {
            Ok(Self::from_limbs(Shape::<M>::encode(value.as_limbs())))
        } else {
            Err(NotCanonical)
        }
    }

    /// The canonical integer in `[0, p)`.
    #[inline(always)]
    #[must_use]
    pub const fn to_uint(self) -> Uint<M::Width> {
        Uint::from_generic_limbs(Shape::<M>::decode(self.limbs()))
    }

    /// Decodes the little-endian encoding of the canonical integer, in `ceil(width / 8)` bytes.
    ///
    /// # Errors
    ///
    /// [`NotCanonical`] when the bytes hold a value at or above `p`.
    #[inline]
    pub fn from_le_bytes(
        bytes: &GenericArray<u8, <M::Width as Bits>::Bytes>,
    ) -> Result<Self, NotCanonical> {
        Self::from_uint(Uint::from_le_bytes(bytes).map_err(|OutOfRange { .. }| NotCanonical)?)
    }

    /// Decodes the big-endian encoding of the canonical integer, in `ceil(width / 8)` bytes.
    ///
    /// # Errors
    ///
    /// [`NotCanonical`] when the bytes hold a value at or above `p`.
    #[inline]
    pub fn from_be_bytes(
        bytes: &GenericArray<u8, <M::Width as Bits>::Bytes>,
    ) -> Result<Self, NotCanonical> {
        Self::from_uint(Uint::from_be_bytes(bytes).map_err(|OutOfRange { .. }| NotCanonical)?)
    }

    /// The little-endian encoding of the canonical integer.
    #[inline]
    #[must_use]
    pub fn to_le_bytes(self) -> GenericArray<u8, <M::Width as Bits>::Bytes> {
        self.to_uint().to_le_bytes()
    }

    /// The big-endian encoding of the canonical integer.
    #[inline]
    #[must_use]
    pub fn to_be_bytes(self) -> GenericArray<u8, <M::Width as Bits>::Bytes> {
        self.to_uint().to_be_bytes()
    }

    /// Whether the element is `0`.
    #[inline(always)]
    #[must_use]
    pub const fn is_zero(&self) -> bool {
        is_zero(self.limbs())
    }

    /// `self + self`.
    #[inline(always)]
    #[must_use]
    pub const fn double(self) -> Self {
        self.add(self)
    }

    /// `self + rhs`, branch-free.
    #[inline(always)]
    #[must_use]
    pub const fn add(self, rhs: Self) -> Self {
        let mut out = M::Width::ZERO;
        add_mod(
            out.as_mut_slice(),
            self.limbs(),
            rhs.limbs(),
            Shape::<M>::P.as_slice(),
            Shape::<M>::SPARE,
        );
        Self::from_limbs(out)
    }

    /// `self - rhs`, branch-free.
    #[inline(always)]
    #[must_use]
    pub const fn sub(self, rhs: Self) -> Self {
        let mut out = M::Width::ZERO;
        sub_mod(
            out.as_mut_slice(),
            self.limbs(),
            rhs.limbs(),
            Shape::<M>::P.as_slice(),
        );
        Self::from_limbs(out)
    }

    /// `-self`, branch-free.
    #[inline(always)]
    #[must_use]
    pub const fn neg(self) -> Self {
        let mut out = M::Width::ZERO;
        neg_mod(out.as_mut_slice(), self.limbs(), Shape::<M>::P.as_slice());
        Self::from_limbs(out)
    }

    /// `self * rhs`, branch-free.
    #[inline(always)]
    #[must_use]
    pub const fn mul(self, rhs: Self) -> Self {
        Self::from_limbs(Shape::<M>::mul_raw(self.limbs(), rhs.limbs()))
    }

    /// `self * self`, branch-free.
    #[inline(always)]
    #[must_use]
    pub const fn square(self) -> Self {
        Self::from_limbs(Shape::<M>::square_raw(self.limbs()))
    }

    /// `self^exponent`, by sliding windows of odd powers over the bits of the exponent, from
    /// the top set bit. The time depends on the exponent, so the exponent is public.
    #[inline]
    #[must_use]
    pub fn pow<E: Bits>(self, exponent: Uint<E>) -> Self {
        let exponent = exponent.as_limbs();
        let bits = bit_len(exponent);
        let window = pow_window(bits);
        // `table[i] = self^(2 i + 1)`.
        let mut table = [self; 1 << (POW_WINDOW - 1)];
        let square = self.square();
        let mut previous = self;
        for entry in table.iter_mut().take(1 << (window - 1)).skip(1) {
            previous = previous.mul(square);
            *entry = previous;
        }
        let mut result = Self::ONE;
        let mut index = bits;
        while index > 0 {
            if !bit(exponent, index - 1) {
                result = result.square();
                index -= 1;
                continue;
            }
            let width = window.min(index);
            let mut value = 0;
            for offset in 0..width {
                value = (value << 1) | usize::from(bit(exponent, index - 1 - offset));
            }
            let trailing = value.trailing_zeros() as usize;
            let used = width - trailing;
            for _ in 0..used {
                result = result.square();
            }
            result = result.mul(table_entry(&table, (value >> trailing) / 2));
            index -= used;
        }
        result
    }

    /// `a` when `condition` holds, else `b`, without a branch.
    #[inline(always)]
    pub(crate) const fn select(condition: bool, a: Self, b: Self) -> Self {
        let mut out = M::Width::ZERO;
        crate::kernel::select(out.as_mut_slice(), condition, a.limbs(), b.limbs());
        Self::from_limbs(out)
    }

    /// Whether `self == rhs`, by one pass over the limbs.
    #[inline(always)]
    pub(crate) const fn equals(&self, rhs: &Self) -> bool {
        equal(self.limbs(), rhs.limbs())
    }
}

/// The widest window of [`Fp::pow`]: its table holds `2^(POW_WINDOW - 1)` odd powers.
const POW_WINDOW: usize = 6;

/// The window width that minimizes the table and the multiplications of a `bits`-bit exponent:
/// `2^(w - 1) - 1` table products and about `bits / (w + 1)` window products.
#[inline(always)]
const fn pow_window(bits: usize) -> usize {
    let mut best = 1;
    let mut window = 2;
    while window <= POW_WINDOW {
        let cost = (1 << (window - 1)) - 1 + bits / (window + 1);
        let best_cost = (1 << (best - 1)) - 1 + bits / (best + 1);
        if cost < best_cost {
            best = window;
        }
        window += 1;
    }
    best
}

/// Entry `index` of the window table; the base beyond the table.
#[inline(always)]
#[expect(
    clippy::indexing_slicing,
    reason = "the index is checked against the length"
)]
fn table_entry<M: Modulus>(table: &[Fp<M>; 1 << (POW_WINDOW - 1)], index: usize) -> Fp<M> {
    if index < table.len() {
        table[index]
    } else {
        table[0]
    }
}

impl<M: Modulus> fmt::Debug for Fp<M> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "Fp({:?})", self.to_uint())
    }
}

impl<M: Modulus> Add for Fp<M> {
    type Output = Self;

    #[inline(always)]
    fn add(self, rhs: Self) -> Self {
        Self::add(self, rhs)
    }
}

impl<M: Modulus> Sub for Fp<M> {
    type Output = Self;

    #[inline(always)]
    fn sub(self, rhs: Self) -> Self {
        Self::sub(self, rhs)
    }
}

impl<M: Modulus> Mul for Fp<M> {
    type Output = Self;

    #[inline(always)]
    fn mul(self, rhs: Self) -> Self {
        Self::mul(self, rhs)
    }
}

impl<M: Modulus> Neg for Fp<M> {
    type Output = Self;

    #[inline(always)]
    fn neg(self) -> Self {
        Self::neg(self)
    }
}

impl<M: Modulus> AddAssign for Fp<M> {
    #[inline(always)]
    fn add_assign(&mut self, rhs: Self) {
        *self = Self::add(*self, rhs);
    }
}

impl<M: Modulus> SubAssign for Fp<M> {
    #[inline(always)]
    fn sub_assign(&mut self, rhs: Self) {
        *self = Self::sub(*self, rhs);
    }
}

impl<M: Modulus> MulAssign for Fp<M> {
    #[inline(always)]
    fn mul_assign(&mut self, rhs: Self) {
        *self = Self::mul(*self, rhs);
    }
}

impl<M: Modulus> Sum for Fp<M> {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::ZERO, Self::add)
    }
}

impl<M: Modulus> Product for Fp<M> {
    fn product<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::ONE, Self::mul)
    }
}

impl<M: Modulus> From<u64> for Fp<M> {
    #[inline(always)]
    fn from(value: u64) -> Self {
        Self::from_u64(value)
    }
}

impl<M: Modulus> TryFrom<Uint<M::Width>> for Fp<M> {
    type Error = NotCanonical;

    #[inline(always)]
    fn try_from(value: Uint<M::Width>) -> Result<Self, NotCanonical> {
        Self::from_uint(value)
    }
}

impl<M: Modulus> From<Fp<M>> for Uint<M::Width> {
    #[inline(always)]
    fn from(element: Fp<M>) -> Self {
        element.to_uint()
    }
}
