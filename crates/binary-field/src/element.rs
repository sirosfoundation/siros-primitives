//! The field element and its default arithmetic.

use core::fmt;
use core::ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign};

use biguint::{Bits, One, Square, Uint};
use clmul::Poly;
use const_default::ConstDefault;
use generic_array::GenericArray;

use crate::arith::{Element, Field};
use crate::error::OutOfField;
use crate::kernel::{bit, mask, masked_parity, set_bit, truncate, xor_shifted};
use crate::plan::{Plan, Polynomial};
use crate::representation::Storage;

/// The limbs of an element of the field with the polynomial `P`.
pub type Limbs<P> = <<P as Polynomial>::Degree as Bits>::Limbs;
/// The bytes of an element of the field with the polynomial `P`.
pub type Bytes<P> = <<P as Polynomial>::Degree as Bits>::Bytes;

/// An element of `GF(2^n) = GF(2)[x] / (f(x))`, for the polynomial `f` that `P` declares.
///
/// The element is a polynomial of degree below `n`; bit `i` of its value is the coefficient of
/// `x^i`. The field's plan picks its registers at compile time: a byte or a 16-bit word in a
/// general register for degrees up to 16 (64 on WebAssembly), and `clmul::Poly`'s vector registers
/// above.
///
/// The operations take the fastest algorithm of the plan, and their time may depend on the
/// values: table lookups, and branches on zero. [`ConstantTime`](crate::ConstantTime) holds an
/// element whose operations take one path for every value.
#[repr(transparent)]
#[derive_where::derive_where(Clone, Copy, Default, Eq, Hash, PartialEq; Element<P>)]
pub struct Gf2<P: Polynomial> {
    value: Element<P>,
}

impl<P: Polynomial> Gf2<P> {
    /// The degree `n` of the field.
    pub const DEGREE: usize = Plan::<P>::DEGREE;
    /// `0`.
    pub const ZERO: Self = Self {
        value: Element::<P>::ZERO,
    };
    /// `1`.
    pub const ONE: Self = Self {
        value: Element::<P>::ONE,
    };
    /// `x`, the class of the indeterminate.
    pub const X: Self = Self {
        value: Element::<P>::X,
    };

    /// `r(x) = f(x) - x^n`, in the limbs of an element.
    const R: GenericArray<u64, Limbs<P>> = r_limbs::<P>();
    /// `x^-1 = x^(n-1) + (r(x) - 1) / x`.
    const X_INVERSE: GenericArray<u64, Limbs<P>> = x_inverse::<P>();
    /// Bit `i` is the trace of `x^i`.
    const TRACE_MASK: GenericArray<u64, Limbs<P>> = trace_mask::<P>();

    /// The element of the registers `value`.
    #[inline(always)]
    pub(crate) const fn of(value: Element<P>) -> Self {
        Self { value }
    }

    /// The registers.
    #[inline(always)]
    pub(crate) const fn value(self) -> Element<P> {
        self.value
    }

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
        Ok(Self::of(Element::<P>::from_uint(value.try_resize()?)))
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
            Ok(Self::of(Element::<P>::from_uint(value)))
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
        Ok(Self::of(Element::<P>::from_uint(Uint::from_le_bytes(
            bytes,
        )?)))
    }

    /// The element whose big-endian `bytes` hold its polynomial.
    ///
    /// # Errors
    ///
    /// A bit at or above `n` is set.
    #[inline]
    pub fn from_be_bytes(bytes: &GenericArray<u8, Bytes<P>>) -> Result<Self, OutOfField> {
        Ok(Self::of(Element::<P>::from_uint(Uint::from_be_bytes(
            bytes,
        )?)))
    }

    /// The polynomial as an integer of `n` bits.
    #[inline]
    #[must_use]
    pub fn into_uint(self) -> Uint<P::Degree> {
        self.value.uint()
    }

    /// The element of the polynomial `poly`: every `Poly<n>` has degree below `n`.
    #[inline]
    #[must_use]
    pub fn from_poly(poly: Poly<P::Degree>) -> Self {
        Self::of(Element::<P>::from_poly(poly))
    }

    /// The polynomial in `clmul`'s vector registers, where the element lives above degree 16; up to
    /// degree 16 it moves from a general register.
    #[inline]
    #[must_use]
    pub fn into_poly(self) -> Poly<P::Degree> {
        self.value.poly()
    }

    /// The little-endian limbs of the polynomial.
    #[inline]
    #[must_use]
    pub fn into_limbs(self) -> GenericArray<u64, Limbs<P>> {
        self.into_uint().into_limbs()
    }

    /// The little-endian bytes of the polynomial.
    #[inline]
    #[must_use]
    pub fn to_le_bytes(&self) -> GenericArray<u8, Bytes<P>> {
        self.into_uint().to_le_bytes()
    }

    /// The big-endian bytes of the polynomial.
    #[inline]
    #[must_use]
    pub fn to_be_bytes(&self) -> GenericArray<u8, Bytes<P>> {
        self.into_uint().to_be_bytes()
    }

    /// Whether the element is `0`.
    #[inline]
    #[must_use]
    pub fn is_zero(&self) -> bool {
        self.value == Element::<P>::ZERO
    }

    /// `self * x`: a shift, with `r` added when the top coefficient falls off.
    #[inline]
    #[must_use]
    pub fn times_x(self) -> Self {
        let value = self.into_limbs();
        let carried = bit(&value, Plan::<P>::DEGREE - 1);
        let mut limbs = GenericArray::<u64, Limbs<P>>::DEFAULT;
        xor_shifted(&mut limbs, &value, 1);
        truncate(&mut limbs, Plan::<P>::DEGREE);
        Self::from_limbs_masked(limbs, &Self::R, carried)
    }

    /// `self / x`: a shift, with `x^-1` added when the constant coefficient falls off.
    #[inline]
    #[must_use]
    pub fn over_x(self) -> Self {
        let mut limbs = self.into_limbs();
        let constant = bit(&limbs, 0);
        shift_right_one(&mut limbs);
        Self::from_limbs_masked(limbs, &Self::X_INVERSE, constant)
    }

    /// The square root. Every element of a field of characteristic 2 has exactly one.
    #[inline(always)]
    #[must_use]
    pub fn sqrt(self) -> Self {
        Self::of(Field::<P>::sqrt(self.value))
    }

    /// The trace `a + a^2 + a^4 + ... + a^(2^(n-1))`, which is `0` or `1`.
    #[inline]
    #[must_use]
    pub fn trace(&self) -> bool {
        masked_parity(&self.into_limbs(), &Self::TRACE_MASK)
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
        Self::of(Element::<P>::from_uint(Uint::from_generic_limbs(limbs)))
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

impl<P: Polynomial> fmt::Debug for Gf2<P> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Gf2({:?})", self.into_uint())
    }
}

impl<P: Polynomial> From<Uint<P::Degree>> for Gf2<P> {
    #[inline]
    fn from(value: Uint<P::Degree>) -> Self {
        Self::of(Element::<P>::from_uint(value))
    }
}

impl<P: Polynomial> From<Gf2<P>> for Uint<P::Degree> {
    #[inline]
    fn from(element: Gf2<P>) -> Self {
        element.into_uint()
    }
}

/// A nonzero element holds itself.
impl<P: Polynomial> biguint::Divisor for Gf2<P> {
    type Stored = Self;

    #[inline(always)]
    fn store(self) -> Option<Self> {
        (!num_traits::Zero::is_zero(&self)).then_some(self)
    }

    #[inline(always)]
    fn load(stored: Self) -> Self {
        stored
    }
}

impl<P: Polynomial> biguint::Field for Gf2<P> {
    #[inline(always)]
    fn invert_nonzero(self) -> Self {
        Self::of(Field::<P>::invert(self.value))
    }

    #[inline(always)]
    fn ct_invert_nonzero(self) -> Self {
        Self::of(Field::<P>::ct_invert(self.value))
    }
}

/// `is_zero` runs in time independent of the value.
impl<P: Polynomial> num_traits::Zero for Gf2<P> {
    #[inline(always)]
    fn zero() -> Self {
        Self::ZERO
    }

    #[inline(always)]
    fn is_zero(&self) -> bool {
        Storage::is_zero(self.value)
    }
}

impl<P: Polynomial> One for Gf2<P> {
    const ONE: Self = Self::ONE;
}

impl<P: Polynomial> Add for Gf2<P> {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self {
        Self::of(self.value.add(rhs.value))
    }
}

impl<P: Polynomial> AddAssign for Gf2<P> {
    #[inline]
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

    #[inline]
    fn sub(self, rhs: Self) -> Self {
        self + rhs
    }
}

#[expect(
    clippy::suspicious_op_assign_impl,
    reason = "subtraction in characteristic 2 is addition"
)]
impl<P: Polynomial> SubAssign for Gf2<P> {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        *self += rhs;
    }
}

impl<P: Polynomial> Neg for Gf2<P> {
    type Output = Self;

    #[inline]
    fn neg(self) -> Self {
        self
    }
}

impl<P: Polynomial> Mul for Gf2<P> {
    type Output = Self;

    #[inline(always)]
    fn mul(self, rhs: Self) -> Self {
        Self::of(Field::<P>::mul(self.value, rhs.value))
    }
}

impl<P: Polynomial> MulAssign for Gf2<P> {
    #[inline]
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}

impl<P: Polynomial> Square for Gf2<P> {
    type Output = Self;

    /// `self * self`: in characteristic 2 the cross terms cancel.
    #[inline]
    fn square(self) -> Self {
        Self::of(Field::<P>::square(self.value))
    }
}
