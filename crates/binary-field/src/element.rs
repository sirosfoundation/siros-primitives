//! The field element and its arithmetic.

use core::fmt;
use core::ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign};

use biguint::{Bits, One, Square, Uint};
use const_default::ConstDefault;
use generic_array::GenericArray;

use crate::error::OutOfField;
use crate::invert::invert;
use crate::kernel::{
    bit, copy, gather, head, is_zero, mask, masked_parity, set_bit, spread, square_limbs, truncate,
    xor_shifted,
};
use crate::mul::{Depth, Levels};
use crate::plan::{Plan, Polynomial, Scratch, Wide};
use crate::reduce::{const_reduce, reduce};

/// The limbs of an element of the field with the polynomial `P`.
pub type Limbs<P> = <<P as Polynomial>::Degree as Bits>::Limbs;
/// The bytes of an element of the field with the polynomial `P`.
pub type Bytes<P> = <<P as Polynomial>::Degree as Bits>::Bytes;
/// The limbs of a product before reduction.
pub(crate) type WideLimbs<P> = <Wide<P> as Bits>::Limbs;
/// The limbs of the scratch of a product.
pub(crate) type ScratchLimbs<P> = <Scratch<P> as Bits>::Limbs;

/// An element of `GF(2^n) = GF(2)[x] / (f(x))`, for the polynomial `f` that `P` declares.
///
/// The element is a polynomial of degree below `n` in `ceil(n / 64)` little-endian `u64` limbs;
/// bit `i` of the value is the coefficient of `x^i`. Addition is XOR. The products, squares,
/// inverses, square roots and traces take the algorithms that the degree and the taps select at
/// compile time; see the crate documentation.
#[repr(transparent)]
#[derive_where::derive_where(Clone, Copy, Default, Eq, Hash, PartialEq; Uint<P::Degree>)]
pub struct Gf2<P: Polynomial> {
    value: Uint<P::Degree>,
}

impl<P: Polynomial> Gf2<P> {
    /// The degree `n` of the field.
    pub const DEGREE: usize = Plan::<P>::DEGREE;
    /// `0`.
    pub const ZERO: Self = Self { value: Uint::ZERO };
    /// `1`.
    pub const ONE: Self = Self { value: Uint::ONE };
    /// `x`, the class of the indeterminate.
    pub const X: Self = Self {
        value: Uint::from_u64(2),
    };

    /// `r(x) = f(x) - x^n`, in the limbs of an element.
    const R: GenericArray<u64, Limbs<P>> = r_limbs::<P>();
    /// `x^-1 = x^(n-1) + (r(x) - 1) / x`.
    const X_INVERSE: GenericArray<u64, Limbs<P>> = x_inverse::<P>();
    /// Bit `i` is the trace of `x^i`.
    const TRACE_MASK: GenericArray<u64, Limbs<P>> = trace_mask::<P>();
    /// `sqrt(x) = x^(2^(n-1))`.
    const SQRT_X: GenericArray<u64, Limbs<P>> = sqrt_x::<P>();

    /// The element with the polynomial `value`, whose degree must be below `n`.
    ///
    /// # Errors
    ///
    /// A bit of `value` at or above `n` is set.
    #[inline]
    pub fn from_u64(value: u64) -> Result<Self, OutOfField> {
        Self::from_uint(Uint::<biguint::Bits64>::from_u64(value))
    }

    /// The element with the polynomial `value`, whose degree must be below `n`.
    ///
    /// # Errors
    ///
    /// A bit of `value` at or above `n` is set.
    #[inline]
    pub fn from_u128(value: u128) -> Result<Self, OutOfField> {
        Self::from_uint(Uint::<biguint::Bits128>::from_u128(value))
    }

    /// The element with the polynomial `value` of any width, whose degree must be below `n`.
    ///
    /// # Errors
    ///
    /// A bit of `value` at or above `n` is set.
    #[inline]
    pub fn from_uint<M: Bits>(value: Uint<M>) -> Result<Self, OutOfField> {
        Ok(Self {
            value: value.try_resize()?,
        })
    }

    /// The element whose little-endian `limbs` hold its polynomial.
    ///
    /// # Errors
    ///
    /// A bit at or above `n` is set.
    #[inline]
    pub fn from_limbs(limbs: GenericArray<u64, Limbs<P>>) -> Result<Self, OutOfField> {
        let value = Uint::from_generic_limbs(limbs);
        if value.into_limbs() == limbs {
            Ok(Self { value })
        } else {
            Err(OutOfField)
        }
    }

    /// The element whose little-endian `bytes` hold its polynomial.
    ///
    /// # Errors
    ///
    /// A bit at or above `n` is set.
    #[inline]
    pub fn from_le_bytes(bytes: &GenericArray<u8, Bytes<P>>) -> Result<Self, OutOfField> {
        Ok(Self {
            value: Uint::from_le_bytes(bytes)?,
        })
    }

    /// The element whose big-endian `bytes` hold its polynomial.
    ///
    /// # Errors
    ///
    /// A bit at or above `n` is set.
    #[inline]
    pub fn from_be_bytes(bytes: &GenericArray<u8, Bytes<P>>) -> Result<Self, OutOfField> {
        Ok(Self {
            value: Uint::from_be_bytes(bytes)?,
        })
    }

    /// The polynomial as an integer of `n` bits.
    #[inline(always)]
    #[must_use]
    pub const fn into_uint(self) -> Uint<P::Degree> {
        self.value
    }

    /// The little-endian limbs of the polynomial.
    #[inline(always)]
    #[must_use]
    pub const fn as_limbs(&self) -> &[u64] {
        self.value.as_limbs()
    }

    /// The little-endian limbs of the polynomial.
    #[inline(always)]
    #[must_use]
    pub const fn into_limbs(self) -> GenericArray<u64, Limbs<P>> {
        self.value.into_limbs()
    }

    /// The little-endian bytes of the polynomial.
    #[inline]
    #[must_use]
    pub fn to_le_bytes(&self) -> GenericArray<u8, Bytes<P>> {
        self.value.to_le_bytes()
    }

    /// The big-endian bytes of the polynomial.
    #[inline]
    #[must_use]
    pub fn to_be_bytes(&self) -> GenericArray<u8, Bytes<P>> {
        self.value.to_be_bytes()
    }

    /// Whether the element is `0`.
    #[inline(always)]
    #[must_use]
    pub fn is_zero(&self) -> bool {
        is_zero(self.as_limbs())
    }

    /// `self * x`: a shift, with `r` added when the top coefficient falls off.
    #[inline]
    #[must_use]
    pub fn times_x(self) -> Self {
        let carried = bit(self.as_limbs(), Plan::<P>::DEGREE - 1);
        let mut limbs = GenericArray::<u64, Limbs<P>>::DEFAULT;
        xor_shifted(&mut limbs, self.as_limbs(), 1);
        truncate(&mut limbs, Plan::<P>::DEGREE);
        Self::from_limbs_masked(limbs, &Self::R, carried)
    }

    /// `self / x`: a shift, with `x^-1` added when the constant coefficient falls off.
    #[inline]
    #[must_use]
    pub fn over_x(self) -> Self {
        let constant = bit(self.as_limbs(), 0);
        let mut limbs = self.into_limbs();
        shift_right_one(&mut limbs);
        Self::from_limbs_masked(limbs, &Self::X_INVERSE, constant)
    }

    /// The multiplicative inverse, or `None` for `0`.
    #[inline]
    #[must_use]
    pub fn invert(self) -> Option<Self> {
        invert(self)
    }

    /// The square root. Every element of a field of characteristic 2 has exactly one.
    #[inline]
    #[must_use]
    pub fn sqrt(self) -> Self {
        if Plan::<P>::SQRT_BY_SPLIT {
            // `a = e(x^2) + x o(x^2)` for the even and odd coefficients, so
            // `sqrt(a) = e(x) + sqrt(x) o(x)`.
            let mut even = GenericArray::<u64, Limbs<P>>::DEFAULT;
            let mut odd = GenericArray::<u64, Limbs<P>>::DEFAULT;
            gather(&mut even, self.as_limbs(), 0);
            gather(&mut odd, self.as_limbs(), 1);
            Self::from_reduced(even) + Self::from_reduced(odd) * Self::from_reduced(Self::SQRT_X)
        } else {
            (1..Plan::<P>::DEGREE).fold(self, |power, _| power.square())
        }
    }

    /// The trace `a + a^2 + a^4 + ... + a^(2^(n-1))`, which is `0` or `1`.
    #[inline]
    #[must_use]
    pub fn trace(&self) -> bool {
        masked_parity(self.as_limbs(), &Self::TRACE_MASK)
    }

    /// The element of reduced `limbs`.
    #[inline(always)]
    pub(crate) const fn from_reduced(limbs: GenericArray<u64, Limbs<P>>) -> Self {
        Self {
            value: Uint::from_generic_limbs(limbs),
        }
    }

    /// `limbs`, plus `addend` when `add` holds, without a branch.
    #[inline(always)]
    fn from_limbs_masked(
        mut limbs: GenericArray<u64, Limbs<P>>,
        addend: &GenericArray<u64, Limbs<P>>,
        add: bool,
    ) -> Self {
        let take = mask(add);
        for (slot, &limb) in limbs.iter_mut().zip(addend.iter()) {
            *slot ^= limb & take;
        }
        Self::from_reduced(limbs)
    }

    /// The reduced product `self * rhs`.
    #[inline(always)]
    fn product(&self, rhs: &Self) -> Self {
        let mut wide = GenericArray::<u64, WideLimbs<P>>::DEFAULT;
        let mut scratch = GenericArray::<u64, ScratchLimbs<P>>::DEFAULT;
        Levels::product(
            &mut wide,
            self.as_limbs(),
            rhs.as_limbs(),
            &mut scratch,
            Plan::<P>::KARATSUBA_THRESHOLD,
        );
        Self::reduced(wide, &mut scratch)
    }

    /// The element of the product `wide`, reduced.
    #[inline(always)]
    fn reduced(mut wide: GenericArray<u64, WideLimbs<P>>, scratch: &mut [u64]) -> Self {
        reduce::<P>(&mut wide, scratch);
        let mut limbs = GenericArray::<u64, Limbs<P>>::DEFAULT;
        copy(&mut limbs, head(&wide, Plan::<P>::LIMBS));
        Self::from_reduced(limbs)
    }
}

/// Shifts `limbs` right by one bit.
#[inline(always)]
fn shift_right_one(limbs: &mut [u64]) {
    let mut rest = limbs;
    while let [limb, tail @ ..] = rest {
        let next = tail.first().copied().unwrap_or(0);
        *limb = (*limb >> 1) | (next << 63);
        rest = tail;
    }
}

/// `r` in the limbs of an element.
const fn r_limbs<P: Polynomial>() -> GenericArray<u64, Limbs<P>> {
    let mut limbs = GenericArray::<u64, Limbs<P>>::DEFAULT;
    let mut taps = Plan::<P>::TAPS;
    while let [tap, rest @ ..] = taps {
        set_bit(limbs.as_mut_slice(), *tap);
        taps = rest;
    }
    limbs
}

/// `x^-1 = x^(n-1) + (r - 1) / x`: its product with `x` is `x^n + r - 1 = f - 1 = 1`.
const fn x_inverse<P: Polynomial>() -> GenericArray<u64, Limbs<P>> {
    let mut limbs = GenericArray::<u64, Limbs<P>>::DEFAULT;
    set_bit(limbs.as_mut_slice(), Plan::<P>::DEGREE - 1);
    let mut taps = Plan::<P>::TAPS;
    while let [tap, rest @ ..] = taps {
        if *tap > 0 {
            set_bit(limbs.as_mut_slice(), *tap - 1);
        }
        taps = rest;
    }
    limbs
}

/// The traces of `x^k` for `k < n`, by Newton's identities over GF(2): with `e_j` the coefficient
/// of `x^(n-j)` in `f`, the power sum of the roots `p_k = Tr(x^k)` satisfies
/// `p_k = sum(e_j p_(k-j), j = 1..k-1) + k e_k`, and `p_0 = n`.
const fn trace_mask<P: Polynomial>() -> GenericArray<u64, Limbs<P>> {
    let degree = Plan::<P>::DEGREE;
    let mut mask = GenericArray::<u64, Limbs<P>>::DEFAULT;
    if degree % 2 == 1 {
        set_bit(mask.as_mut_slice(), 0);
    }
    let mut k = 1;
    while k < degree {
        // `e_j = 1` exactly when `n - j` is a tap, so the sum runs over the taps `t = n - j` with
        // `j = n - t <= k - 1`, that is `t >= n - k + 1`.
        let mut parity = false;
        let mut taps = Plan::<P>::TAPS;
        while let [tap, rest @ ..] = taps {
            let j = degree - *tap;
            if j < k {
                parity ^= bit(mask.as_slice(), k - j);
            } else if j == k {
                parity ^= k % 2 == 1;
            }
            taps = rest;
        }
        if parity {
            set_bit(mask.as_mut_slice(), k);
        }
        k += 1;
    }
    mask
}

/// `sqrt(x) = x^(2^(n-1))`, by `n - 1` squarings, each a bit spread and a shift-and-add
/// reduction. Degrees above the plan's limit skip it, and their square roots square at runtime.
const fn sqrt_x<P: Polynomial>() -> GenericArray<u64, Limbs<P>> {
    let mut power = GenericArray::<u64, Limbs<P>>::DEFAULT;
    if !Plan::<P>::SQRT_BY_SPLIT {
        return power;
    }
    set_bit(power.as_mut_slice(), 1);
    let mut wide = GenericArray::<u64, WideLimbs<P>>::DEFAULT;
    let mut scratch = GenericArray::<u64, WideLimbs<P>>::DEFAULT;
    let mut count = 1;
    while count < Plan::<P>::DEGREE {
        spread(wide.as_mut_slice(), power.as_slice());
        const_reduce::<P>(wide.as_mut_slice(), scratch.as_mut_slice());
        copy(
            power.as_mut_slice(),
            head(wide.as_slice(), Plan::<P>::LIMBS),
        );
        count += 1;
    }
    power
}

impl<P: Polynomial> fmt::Debug for Gf2<P> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Gf2({:?})", self.value)
    }
}

impl<P: Polynomial> From<Uint<P::Degree>> for Gf2<P> {
    #[inline(always)]
    fn from(value: Uint<P::Degree>) -> Self {
        Self { value }
    }
}

impl<P: Polynomial> From<Gf2<P>> for Uint<P::Degree> {
    #[inline(always)]
    fn from(element: Gf2<P>) -> Self {
        element.value
    }
}

impl<P: Polynomial> One for Gf2<P> {
    const ONE: Self = Self::ONE;
}

#[expect(
    clippy::suspicious_arithmetic_impl,
    reason = "addition in characteristic 2 is XOR"
)]
impl<P: Polynomial> Add for Gf2<P> {
    type Output = Self;

    #[inline(always)]
    fn add(self, rhs: Self) -> Self {
        Self {
            value: self.value ^ rhs.value,
        }
    }
}

impl<P: Polynomial> AddAssign for Gf2<P> {
    #[inline(always)]
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

#[expect(
    clippy::suspicious_arithmetic_impl,
    reason = "subtraction in characteristic 2 is addition"
)]
impl<P: Polynomial> Sub for Gf2<P> {
    type Output = Self;

    #[inline(always)]
    fn sub(self, rhs: Self) -> Self {
        self + rhs
    }
}

#[expect(
    clippy::suspicious_op_assign_impl,
    reason = "subtraction in characteristic 2 is addition"
)]
impl<P: Polynomial> SubAssign for Gf2<P> {
    #[inline(always)]
    fn sub_assign(&mut self, rhs: Self) {
        *self += rhs;
    }
}

impl<P: Polynomial> Neg for Gf2<P> {
    type Output = Self;

    #[inline(always)]
    fn neg(self) -> Self {
        self
    }
}

impl<P: Polynomial> Mul for Gf2<P> {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: Self) -> Self {
        self.product(&rhs)
    }
}

impl<P: Polynomial> MulAssign for Gf2<P> {
    #[inline]
    fn mul_assign(&mut self, rhs: Self) {
        *self = self.product(&rhs);
    }
}

impl<P: Polynomial> Square for Gf2<P> {
    type Output = Self;

    /// `self * self`: in characteristic 2 the cross terms cancel, so the square of each limb is
    /// its own carry-less product, or the spread of its bits to the even positions.
    #[inline]
    fn square(self) -> Self {
        let mut wide = GenericArray::<u64, WideLimbs<P>>::DEFAULT;
        let mut scratch = GenericArray::<u64, ScratchLimbs<P>>::DEFAULT;
        if Plan::<P>::SQUARE_BY_CLMUL {
            square_limbs(&mut wide, self.as_limbs(), |limb| {
                clmul::clmul64(limb, limb)
            });
        } else {
            spread(&mut wide, self.as_limbs());
        }
        Self::reduced(wide, &mut scratch)
    }
}
