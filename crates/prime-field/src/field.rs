use core::fmt;
use core::hash::{Hash, Hasher};
use core::iter::{Product, Sum};
use core::ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign};

use biguint::generic_array::GenericArray;
use biguint::{Bits, OutOfRange, Uint};

use crate::kernel::{bit, bit_len, equal, limb};
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
/// The operations take the path with the fewest instructions, and their time may depend on the
/// values; [`Secret`](crate::Secret) holds an element whose operations run in constant time. The
/// `const_*` methods compute in constant contexts.
///
/// A one-limb Mersenne modulus `2^k - 1` (Mersenne-31) holds plain residues in `[1, p]`, with
/// `0` held as `p`. A modulus `2^k - c` of several limbs with `c < 2^64` small against `2^k`
/// (secp256k1, P-521) holds plain residues and reduces products by folding. Every other field
/// holds Montgomery residues `x R mod p` with `R = 2^(64 L)` and reduces products by Montgomery
/// multiplication. The representation is private; the conversions take and return canonical
/// integers.
#[repr(transparent)]
#[derive_where::derive_where(Clone, Copy, Eq; Limbs<M>)]
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
    pub const ZERO: Self = Self::from_limbs(Shape::<M>::ZERO);
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

    /// The representation, as an array.
    #[inline(always)]
    pub(crate) const fn limb_array(&self) -> &Limbs<M> {
        &self.limbs
    }

    /// `value mod p`.
    #[inline(always)]
    #[must_use]
    pub const fn from_u64(value: u64) -> Self {
        let reduced = if Shape::<M>::LIMBS == 1 {
            value % limb(Shape::<M>::P.as_slice(), 0)
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
    #[inline(always)]
    pub fn from_le_bytes(
        bytes: &GenericArray<u8, <M::Width as Bits>::Bytes>,
    ) -> Result<Self, NotCanonical> {
        Self::from_integer(Uint::from_le_bytes(bytes).map_err(|OutOfRange { .. }| NotCanonical)?)
    }

    /// Decodes the big-endian encoding of the canonical integer, in `ceil(width / 8)` bytes.
    ///
    /// # Errors
    ///
    /// [`NotCanonical`] when the bytes hold a value at or above `p`.
    #[inline(always)]
    pub fn from_be_bytes(
        bytes: &GenericArray<u8, <M::Width as Bits>::Bytes>,
    ) -> Result<Self, NotCanonical> {
        Self::from_integer(Uint::from_be_bytes(bytes).map_err(|OutOfRange { .. }| NotCanonical)?)
    }

    /// The little-endian encoding of the canonical integer.
    #[inline(always)]
    #[must_use]
    pub fn to_le_bytes(self) -> GenericArray<u8, <M::Width as Bits>::Bytes> {
        self.integer().to_le_bytes()
    }

    /// The big-endian encoding of the canonical integer.
    #[inline(always)]
    #[must_use]
    pub fn to_be_bytes(self) -> GenericArray<u8, <M::Width as Bits>::Bytes> {
        self.integer().to_be_bytes()
    }

    /// [`Self::from_uint`] by the run-time kernels, in time that depends on whether `value` is
    /// canonical alone.
    #[inline(always)]
    fn from_integer(value: Uint<M::Width>) -> Result<Self, NotCanonical> {
        let limbs = value.into_limbs();
        if Shape::<M>::below_p(&limbs) {
            Ok(Self::from_limbs(Shape::<M>::representation_of(&limbs)))
        } else {
            Err(NotCanonical)
        }
    }

    /// [`Self::to_uint`] by the run-time kernels, in constant time.
    #[inline(always)]
    fn integer(self) -> Uint<M::Width> {
        Uint::from_generic_limbs(Shape::<M>::integer_of(&self.limbs))
    }

    /// Whether the element is `0`.
    #[inline(always)]
    #[must_use]
    pub const fn is_zero(&self) -> bool {
        equal(self.limbs(), Shape::<M>::ZERO.as_slice())
    }

    /// `self + rhs`.
    #[inline(always)]
    #[must_use]
    #[expect(
        clippy::should_implement_trait,
        reason = "`Add` is implemented too; the method serves method-call syntax"
    )]
    pub fn add(self, rhs: Self) -> Self {
        self.add_with(rhs)
    }

    /// `self - rhs`.
    #[inline(always)]
    #[must_use]
    #[expect(
        clippy::should_implement_trait,
        reason = "`Sub` is implemented too; the method serves method-call syntax"
    )]
    pub fn sub(self, rhs: Self) -> Self {
        self.sub_with(rhs)
    }

    /// `-self`.
    #[inline(always)]
    #[must_use]
    #[expect(
        clippy::should_implement_trait,
        reason = "`Neg` is implemented too; the method serves method-call syntax"
    )]
    pub fn neg(self) -> Self {
        self.neg_with()
    }

    /// `self + self`.
    #[inline(always)]
    #[must_use]
    pub fn double(self) -> Self {
        self.add_with(self)
    }

    /// `self * rhs`.
    #[inline(always)]
    #[must_use]
    #[expect(
        clippy::should_implement_trait,
        reason = "`Mul` is implemented too; the method serves method-call syntax"
    )]
    pub fn mul(self, rhs: Self) -> Self {
        self.mul_with::<false>(rhs)
    }

    /// `self * self`.
    #[inline(always)]
    #[must_use]
    pub fn square(self) -> Self {
        self.square_with::<false>()
    }

    /// `self^exponent`, by sliding windows of odd powers over the bits of the exponent, from
    /// the top set bit.
    #[inline(always)]
    #[must_use]
    pub fn pow<E: Bits>(self, exponent: Uint<E>) -> Self {
        self.pow_with::<false, E>(exponent)
    }

    /// `self + rhs` in a constant context.
    #[inline(always)]
    #[must_use]
    pub const fn const_add(self, rhs: Self) -> Self {
        Self::from_limbs(Shape::<M>::add_raw(self.limbs(), rhs.limbs()))
    }

    /// `self - rhs` in a constant context.
    #[inline(always)]
    #[must_use]
    pub const fn const_sub(self, rhs: Self) -> Self {
        Self::from_limbs(Shape::<M>::sub_raw(self.limbs(), rhs.limbs()))
    }

    /// `-self` in a constant context.
    #[inline(always)]
    #[must_use]
    pub const fn const_neg(self) -> Self {
        Self::from_limbs(Shape::<M>::neg_raw(self.limbs()))
    }

    /// `self + self` in a constant context.
    #[inline(always)]
    #[must_use]
    pub const fn const_double(self) -> Self {
        self.const_add(self)
    }

    /// `self * rhs` in a constant context.
    #[inline(always)]
    #[must_use]
    pub const fn const_mul(self, rhs: Self) -> Self {
        Self::from_limbs(Shape::<M>::mul_raw(self.limbs(), rhs.limbs()))
    }

    /// `self * self` in a constant context.
    #[inline(always)]
    #[must_use]
    pub const fn const_square(self) -> Self {
        Self::from_limbs(Shape::<M>::square_raw(self.limbs()))
    }

    /// `self + rhs`, in constant time: the default and the `Secret` operation alike.
    #[inline(always)]
    pub(crate) fn add_with(self, rhs: Self) -> Self {
        Self::from_limbs(Shape::<M>::add(&self.limbs, &rhs.limbs))
    }

    /// `self - rhs`, in constant time: the default and the `Secret` operation alike.
    #[inline(always)]
    pub(crate) fn sub_with(self, rhs: Self) -> Self {
        Self::from_limbs(Shape::<M>::sub(&self.limbs, &rhs.limbs))
    }

    /// `-self`, in constant time: the default and the `Secret` operation alike.
    #[inline(always)]
    pub(crate) fn neg_with(self) -> Self {
        Self::from_limbs(Shape::<M>::neg(&self.limbs))
    }

    /// `self * rhs`, in constant time when `CT` holds.
    #[inline(always)]
    pub(crate) fn mul_with<const CT: bool>(self, rhs: Self) -> Self {
        Self::from_limbs(Shape::<M>::mul::<CT>(&self.limbs, &rhs.limbs))
    }

    /// `self * self`, in constant time when `CT` holds.
    #[inline(always)]
    pub(crate) fn square_with<const CT: bool>(self) -> Self {
        Self::from_limbs(Shape::<M>::square::<CT>(&self.limbs))
    }

    /// `self^exponent` by sliding windows, in time that depends on the exponent alone when `CT`
    /// holds.
    #[inline(always)]
    pub(crate) fn pow_with<const CT: bool, E: Bits>(self, exponent: Uint<E>) -> Self {
        let exponent = exponent.as_limbs();
        let bits = bit_len(exponent);
        let window = pow_window(bits);
        // `table[i] = self^(2 i + 1)`.
        let mut table = [self; 1 << (POW_WINDOW - 1)];
        let square = self.square_with::<CT>();
        let mut previous = self;
        for entry in table.iter_mut().take(1 << (window - 1)).skip(1) {
            previous = previous.mul_with::<CT>(square);
            *entry = previous;
        }
        let mut result = Self::ONE;
        let mut index = bits;
        while index > 0 {
            if !bit(exponent, index - 1) {
                result = result.square_with::<CT>();
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
                result = result.square_with::<CT>();
            }
            result = result.mul_with::<CT>(table_entry(&table, (value >> trailing) / 2));
            index -= used;
        }
        result
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

impl<M: Modulus> Default for Fp<M> {
    /// `0`.
    #[inline(always)]
    fn default() -> Self {
        Self::ZERO
    }
}

impl<M: Modulus> PartialEq for Fp<M> {
    /// One pass that folds the differences of the limbs, which the compiler vectorizes for
    /// elements in memory; the comparison of the derived equality calls `bcmp` above 64 bytes.
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        self.equals(other)
    }
}

impl<M: Modulus> Hash for Fp<M> {
    #[inline(always)]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.limbs.hash(state);
    }
}

impl<M: Modulus> fmt::Debug for Fp<M> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "Fp({:?})", self.integer())
    }
}

impl<M: Modulus> Add for Fp<M> {
    type Output = Self;

    #[inline(always)]
    fn add(self, rhs: Self) -> Self {
        self.add_with(rhs)
    }
}

impl<M: Modulus> Sub for Fp<M> {
    type Output = Self;

    #[inline(always)]
    fn sub(self, rhs: Self) -> Self {
        self.sub_with(rhs)
    }
}

impl<M: Modulus> Mul for Fp<M> {
    type Output = Self;

    #[inline(always)]
    fn mul(self, rhs: Self) -> Self {
        self.mul_with::<false>(rhs)
    }
}

impl<M: Modulus> Neg for Fp<M> {
    type Output = Self;

    #[inline(always)]
    fn neg(self) -> Self {
        self.neg_with()
    }
}

impl<M: Modulus> AddAssign for Fp<M> {
    #[inline(always)]
    fn add_assign(&mut self, rhs: Self) {
        *self = self.add_with(rhs);
    }
}

impl<M: Modulus> SubAssign for Fp<M> {
    #[inline(always)]
    fn sub_assign(&mut self, rhs: Self) {
        *self = self.sub_with(rhs);
    }
}

impl<M: Modulus> MulAssign for Fp<M> {
    #[inline(always)]
    fn mul_assign(&mut self, rhs: Self) {
        *self = self.mul_with::<false>(rhs);
    }
}

impl<M: Modulus> Sum for Fp<M> {
    #[inline(always)]
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::ZERO, |sum, item| sum.add_with(item))
    }
}

impl<M: Modulus> Product for Fp<M> {
    #[inline(always)]
    fn product<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::ONE, |product, item| product.mul_with::<false>(item))
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
        Self::from_integer(value)
    }
}

impl<M: Modulus> From<Fp<M>> for Uint<M::Width> {
    #[inline(always)]
    fn from(element: Fp<M>) -> Self {
        element.integer()
    }
}
