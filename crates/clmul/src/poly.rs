//! Polynomials over GF(2) in the registers of the build's vector unit.

use core::fmt;
use core::hash::{Hash, Hasher};
use core::marker::PhantomData;
use core::ops::{
    Add, BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Div, Shl, Shr,
};

use biguint::generic_array::{ArrayLength, GenericArray};
use biguint::typenum::{Quot, Sum, U1, U127, U128, Unsigned};
use biguint::{Bits, Indices, OutOfRange, Uint};
use const_default::ConstDefault;

use crate::backend::{self, Register};
use crate::nibble::NibbleMap;

/// One vector register that holds a polynomial over GF(2) of degree below `N`, for `N` up to 128.
/// The bits at and above `N` are zero, so the conversions to and from `Poly<N>` take no
/// instruction. A `Lane` passes between functions in a vector register, where a `Poly` passes in
/// memory. A [`Poly`] keeps each of its registers as a `Lane<U128>`.
///
/// ```
/// use biguint::typenum::U64;
/// use biguint::Uint;
/// use clmul::{Lane, Poly};
///
/// let a = Poly::<U64>::from_uint(Uint::from_u64(0b11));
/// let lane = Lane::from_poly(a);
/// assert_eq!(lane.into_poly(), a);
/// assert_eq!(Poly::from(Lane::<U64>::ZERO), Poly::ZERO);
/// ```
#[repr(transparent)]
pub struct Lane<N>(Register, PhantomData<N>);

impl<N> Lane<N> {
    /// The lane of `register`, whose bits at and above `N` are zero.
    #[inline(always)]
    const fn of(register: Register) -> Self {
        Self(register, PhantomData)
    }
}

impl<N> Clone for Lane<N> {
    #[inline(always)]
    fn clone(&self) -> Self {
        *self
    }
}

impl<N> Copy for Lane<N> {}

impl<N> ConstDefault for Lane<N> {
    const DEFAULT: Self = Self::of(backend::ZERO);
}

impl<N: Width<Lanes = U1>> Lane<N> {
    /// `0`.
    pub const ZERO: Self = Self::of(backend::ZERO);

    /// The register of `poly`.
    #[inline(always)]
    #[must_use]
    pub const fn from_poly(poly: Poly<N>) -> Self {
        let [lane] = poly.lanes.into_array::<1>();
        Self::of(lane.0)
    }

    /// The polynomial in the register.
    #[inline(always)]
    #[must_use]
    pub const fn into_poly(self) -> Poly<N> {
        Poly {
            lanes: GenericArray::<Lane<U128>, U1>::from_array([Lane::of(self.0)]),
        }
    }
}

impl<N: Width<Lanes = U1>> Default for Lane<N> {
    #[inline(always)]
    fn default() -> Self {
        Self::ZERO
    }
}

impl<N: Width<Lanes = U1>> PartialEq for Lane<N> {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        self.into_poly() == other.into_poly()
    }
}

impl<N: Width<Lanes = U1>> Eq for Lane<N> {}

impl<N: Width<Lanes = U1>> Hash for Lane<N> {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.into_poly().hash(state);
    }
}

impl<N: Width<Lanes = U1>> fmt::Debug for Lane<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Lane({:?})", self.into_poly().to_uint())
    }
}

impl<N: Width<Lanes = U1>> From<Poly<N>> for Lane<N> {
    #[inline(always)]
    fn from(poly: Poly<N>) -> Self {
        Self::from_poly(poly)
    }
}

impl<N: Width<Lanes = U1>> From<Lane<N>> for Poly<N> {
    #[inline(always)]
    fn from(lane: Lane<N>) -> Self {
        lane.into_poly()
    }
}

/// A bit width `N` with the registers of a [`Poly<N>`]: `ceil(N / 128)`.
///
/// Exactly the nonzero `typenum` unsigned integers implement it.
pub trait Width: Bits
where
    <Self::Lanes as ArrayLength>::ArrayType<Lane<U128>>: Copy + ConstDefault,
{
    /// `ceil(N / 128)`.
    type Lanes: ArrayLength + Indices;

    /// `ceil(N / 128)`.
    const LANES: usize;
}

impl<N> Width for N
where
    N: Bits + Add<U127>,
    Sum<N, U127>: Div<U128>,
    Quot<Sum<N, U127>, U128>: ArrayLength + Indices,
    <Quot<Sum<N, U127>, U128> as ArrayLength>::ArrayType<Lane<U128>>: Copy + ConstDefault,
{
    type Lanes = Quot<Sum<N, U127>, U128>;

    const LANES: usize = <Quot<Sum<N, U127>, U128> as Unsigned>::USIZE;
}

/// A polynomial over GF(2) of degree below `N`: bit `i` is the coefficient of `x^i`.
///
/// It is `ceil(N / 128)` registers of the build's vector unit: SSE registers with PCLMULQDQ,
/// Advanced SIMD registers with PMULL, and pairs of `u64` words elsewhere. Up to 128 bits it is one
/// register. The bits at and above `N` are zero.
///
/// ```
/// use biguint::typenum::{U64, U128};
/// use biguint::Uint;
/// use clmul::Poly;
///
/// // (x + 1)^2 = x^2 + 1, and x^63 * x = x^64 in the widened product.
/// let x_plus_1 = Poly::<U64>::from_uint(Uint::from_u64(0b11));
/// assert_eq!(x_plus_1.widening_square(), Poly::<U128>::from_uint(Uint::from_u64(0b101)));
/// let x63 = Poly::<U64>::from_uint(Uint::from_u64(1 << 63));
/// let x = Poly::<U64>::from_uint(Uint::from_u64(2));
/// assert_eq!(x63.widening_mul(x), Poly::<U128>::ONE << 64);
/// ```
#[repr(transparent)]
pub struct Poly<N: Width> {
    lanes: GenericArray<Lane<U128>, N::Lanes>,
}

/// The most registers whose loops run one copy per register; wider polynomials loop.
const COPIED_LANES: usize = 4;

/// Runs `body` on each register index of a `Poly<N>`: one copy per register up to
/// [`COPIED_LANES`] in builds without debug assertions ([`Indices`]), a loop beyond.
#[inline(always)]
fn each_lane<N: Width>(mut body: impl FnMut(usize)) {
    if N::LANES > COPIED_LANES {
        for index in 0..N::LANES {
            body(index);
        }
    } else {
        N::Lanes::each(body);
    }
}

/// The bits of word `word` that lie below `bits`.
#[inline(always)]
const fn word_mask(bits: usize, word: usize) -> u64 {
    let low = word * 64;
    if bits >= low + 64 {
        u64::MAX
    } else if bits <= low {
        0
    } else {
        (1 << (bits - low)) - 1
    }
}

/// The bits of word `word` of a width of `bits` bits: up to 64.
#[inline(always)]
const fn word_bits(bits: usize, word: usize) -> usize {
    let above = bits.saturating_sub(word * 64);
    if above > 64 { 64 } else { above }
}

/// The carry-less product of word `i` of `a`, of `a_bits` bits, and word `j` of `b`, of `b_bits`
/// bits: the narrow product when either word is below `2^32`.
#[inline(always)]
fn word_product(
    a: Register,
    i: usize,
    a_bits: usize,
    b: Register,
    j: usize,
    b_bits: usize,
) -> Register {
    if a_bits <= 32 {
        backend::clmul_narrow(a, i % 2, b, j % 2)
    } else if b_bits <= 32 {
        backend::clmul_narrow(b, j % 2, a, i % 2)
    } else {
        backend::clmul(a, i % 2, b, j % 2)
    }
}

/// The product of two registers as Karatsuba's `[low, middle, high]`, whose sum
/// `low + middle x^64 + high x^128` it is: three word products, the middle one of the sums of the
/// words.
#[cfg(not(all(
    target_arch = "x86_64",
    not(target_feature = "pclmulqdq"),
    target_feature = "avx2"
)))]
#[inline(always)]
fn karatsuba(a: Register, b: Register) -> [Register; 3] {
    let low = backend::clmul(a, 0, b, 0);
    let high = backend::clmul(a, 1, b, 1);
    let a_sum = backend::xor(a, backend::down(a));
    let b_sum = backend::xor(b, backend::down(b));
    let middle = backend::xor(backend::xor(backend::clmul(a_sum, 0, b_sum, 0), low), high);
    [low, middle, high]
}

/// The product of two registers as Karatsuba's `[low, middle, high]`: the vector comb's nine
/// piece products on AVX2 registers.
#[cfg(all(
    target_arch = "x86_64",
    not(target_feature = "pclmulqdq"),
    target_feature = "avx2"
))]
#[inline(always)]
fn karatsuba(a: Register, b: Register) -> [Register; 3] {
    backend::karatsuba(a, b)
}

/// Whether register `index` of a width of `bits` bits holds bits at and above `bits`.
#[inline(always)]
const fn partial(bits: usize, index: usize) -> bool {
    bits < 128 * (index + 1)
}

impl<N: Width> Poly<N> {
    /// `0`.
    pub const ZERO: Self = Self {
        lanes: GenericArray::DEFAULT,
    };

    /// `1`.
    pub const ONE: Self = Self::from_uint(Uint::ONE);

    /// The polynomial whose coefficients are the bits of `value`.
    #[inline(always)]
    #[must_use]
    pub const fn from_uint(value: Uint<N>) -> Self {
        let mut lanes = GenericArray::<Lane<U128>, N::Lanes>::DEFAULT;
        let mut slots = lanes.as_mut_slice();
        let mut limbs = value.as_limbs();
        while let [slot, slots_rest @ ..] = slots {
            let (low, limbs_rest) = match limbs {
                [low, rest @ ..] => (*low, rest),
                [] => (0, limbs),
            };
            let (high, limbs_rest) = match limbs_rest {
                [high, rest @ ..] => (*high, rest),
                [] => (0, limbs_rest),
            };
            *slot = Lane::of(backend::register([low, high]));
            limbs = limbs_rest;
            slots = slots_rest;
        }
        Self { lanes }
    }

    /// The integer whose bits are the coefficients.
    #[inline(always)]
    #[must_use]
    pub const fn to_uint(self) -> Uint<N> {
        let mut limbs = GenericArray::<u64, N::Limbs>::DEFAULT;
        let mut slots = limbs.as_mut_slice();
        let mut lanes = self.lanes.as_slice();
        while let [lane, lanes_rest @ ..] = lanes {
            let [low, high] = backend::words(lane.0);
            slots = match slots {
                [first, second, rest @ ..] => {
                    *first = low;
                    *second = high;
                    rest
                }
                [first, rest @ ..] => {
                    *first = low;
                    rest
                }
                [] => slots,
            };
            lanes = lanes_rest;
        }
        Uint::from_generic_limbs(limbs)
    }

    /// The polynomial of the little-endian `bytes`.
    ///
    /// # Errors
    ///
    /// A bit at or above `N` is set.
    #[inline(always)]
    pub fn from_le_bytes(bytes: &GenericArray<u8, N::Bytes>) -> Result<Self, OutOfRange> {
        Uint::from_le_bytes(bytes).map(Self::from_uint)
    }

    /// The polynomial of the big-endian `bytes`.
    ///
    /// # Errors
    ///
    /// A bit at or above `N` is set.
    #[inline(always)]
    pub fn from_be_bytes(bytes: &GenericArray<u8, N::Bytes>) -> Result<Self, OutOfRange> {
        Uint::from_be_bytes(bytes).map(Self::from_uint)
    }

    /// The little-endian bytes.
    #[inline(always)]
    #[must_use]
    pub fn to_le_bytes(&self) -> GenericArray<u8, N::Bytes> {
        self.to_uint().to_le_bytes()
    }

    /// The big-endian bytes.
    #[inline(always)]
    #[must_use]
    pub fn to_be_bytes(&self) -> GenericArray<u8, N::Bytes> {
        self.to_uint().to_be_bytes()
    }

    /// Whether every coefficient is zero.
    #[inline(always)]
    #[must_use]
    pub fn is_zero(&self) -> bool {
        self.lanes.iter().fold(0, |acc, lane| {
            let [low, high] = backend::words(lane.0);
            acc | low | high
        }) == 0
    }

    /// Register `index`, or zero above the top.
    #[inline(always)]
    fn lane(&self, index: usize) -> Register {
        self.lanes.get(index).map_or(backend::ZERO, |lane| lane.0)
    }

    /// Sets register `index`; above the top it does nothing.
    #[inline(always)]
    fn set(&mut self, index: usize, value: Register) {
        if let Some(lane) = self.lanes.get_mut(index) {
            lane.0 = value;
        }
    }

    /// Adds `value` to register `index`; above the top it does nothing.
    #[inline(always)]
    fn add_to(&mut self, index: usize, value: Register) {
        if let Some(lane) = self.lanes.get_mut(index) {
            lane.0 = backend::xor(lane.0, value);
        }
    }

    /// Clears the bits at and above `N` of the top register.
    #[inline(always)]
    fn truncated(mut self) -> Self {
        let top = N::LANES - 1;
        if partial(N::BITS, top) {
            let mask =
                backend::register([word_mask(N::BITS, 2 * top), word_mask(N::BITS, 2 * top + 1)]);
            let value = backend::and(self.lane(top), mask);
            self.set(top, value);
        }
        self
    }

    /// The polynomial modulo `x^M`, or zero-extended to `M` bits.
    #[inline(always)]
    #[must_use]
    pub fn resize<M: Width>(self) -> Poly<M> {
        let mut out = Poly::<M>::ZERO;
        each_lane::<M>(
            #[inline(always)]
            |index| {
                out.set(index, self.lane(index));
            },
        );
        if M::BITS < N::BITS {
            out.truncated()
        } else {
            out
        }
    }

    /// Every 64-bit word shifted left by `bits`, below 64, on its own: the coefficients that leave
    /// a word drop, and the result keeps the bits below `N`.
    #[inline(always)]
    #[must_use]
    pub fn shl_within_words(mut self, bits: u32) -> Self {
        each_lane::<N>(
            #[inline(always)]
            |index| {
                self.set(index, backend::shl_words(self.lane(index), bits));
            },
        );
        if N::BITS % 64 == 0 {
            self
        } else {
            self.truncated()
        }
    }

    /// Every byte replaced by the image of its low nibble under `map`: one PSHUFB with SSSE3, one
    /// TBL on aarch64, and the images selected by the bits of the nibble elsewhere. The bytes at and
    /// above `N` index nibble 0, whose image is 0; when `N` is not a multiple of 8 the result's
    /// bits at and above `N` are cleared.
    #[inline(always)]
    #[must_use]
    pub fn map_low_nibbles(mut self, map: NibbleMap) -> Self {
        let low = backend::register([0x0F0F_0F0F_0F0F_0F0F; 2]);
        each_lane::<N>(
            #[inline(always)]
            |index| {
                self.set(index, map.apply(backend::and(self.lane(index), low)));
            },
        );
        if N::BITS % 8 == 0 {
            self
        } else {
            self.truncated()
        }
    }

    /// Every byte replaced by the image of its high nibble under `map`, as
    /// [`map_low_nibbles`](Self::map_low_nibbles).
    #[inline(always)]
    #[must_use]
    pub fn map_high_nibbles(mut self, map: NibbleMap) -> Self {
        let low = backend::register([0x0F0F_0F0F_0F0F_0F0F; 2]);
        each_lane::<N>(
            #[inline(always)]
            |index| {
                let nibbles = backend::and(backend::shr_words(self.lane(index), 4), low);
                self.set(index, map.apply(nibbles));
            },
        );
        if N::BITS % 8 == 0 {
            self
        } else {
            self.truncated()
        }
    }

    /// Every 64-bit word shifted right by `bits`, below 64, on its own: the coefficients that
    /// leave a word drop.
    #[inline(always)]
    #[must_use]
    pub fn shr_within_words(mut self, bits: u32) -> Self {
        each_lane::<N>(
            #[inline(always)]
            |index| {
                self.set(index, backend::shr_words(self.lane(index), bits));
            },
        );
        self
    }

    /// The carry-less product of word `i` of `self` and word `j` of `rhs`: two coefficients of
    /// `x^(64 i)` and `x^(64 j)` times `x^(64 (i + j))` are the 128 bits of the result.
    #[inline(always)]
    #[must_use]
    pub fn clmul_words<M: Width>(self, i: usize, rhs: Poly<M>, j: usize) -> Poly<U128> {
        let mut out = Poly::<U128>::ZERO;
        out.set(
            0,
            word_product(
                self.lane(i / 2),
                i,
                word_bits(N::BITS, i),
                rhs.lane(j / 2),
                j,
                word_bits(M::BITS, j),
            ),
        );
        out
    }

    /// The whole product `self * rhs`, in `N + M` bits. Two-word registers multiply by
    /// Karatsuba's three word products; the registers combine by the schoolbook.
    #[inline(always)]
    #[must_use]
    pub fn widening_mul<M>(self, rhs: Poly<M>) -> Poly<Sum<N, M>>
    where
        N: Add<M>,
        M: Width,
        Sum<N, M>: Width,
    {
        // Products whose weight is an even number of words land on whole registers; the others,
        // at `x^(64 (2 k + 1))`, wait in `odd[k]` and shift up by one word at the end.
        let mut even = Poly::<Sum<N, M>>::ZERO;
        let mut odd = Poly::<Sum<N, M>>::ZERO;
        let (a_words, b_words) = (N::LIMBS, M::LIMBS);
        // One pass over the register pairs `(i, j)`.
        each_lane::<N>(
            #[inline(always)]
            |i| {
                each_lane::<M>(
                    #[inline(always)]
                    |j| {
                        let (a, b) = (self.lane(i), rhs.lane(j));
                        let (a_full, b_full) = (2 * i + 1 < a_words, 2 * j + 1 < b_words);
                        if a_full && b_full {
                            let [low, middle, high] = karatsuba(a, b);
                            even.add_to(i + j, low);
                            even.add_to(i + j + 1, high);
                            odd.add_to(i + j, middle);
                        } else {
                            let bits = |width: usize, word: usize| word_bits(width, word);
                            let (a_low, b_low) = (bits(N::BITS, 2 * i), bits(M::BITS, 2 * j));
                            even.add_to(i + j, word_product(a, 0, a_low, b, 0, b_low));
                            if a_full {
                                let a_high = bits(N::BITS, 2 * i + 1);
                                odd.add_to(i + j, word_product(a, 1, a_high, b, 0, b_low));
                            } else if b_full {
                                let b_high = bits(M::BITS, 2 * j + 1);
                                odd.add_to(i + j, word_product(a, 0, a_low, b, 1, b_high));
                            }
                        }
                    },
                );
            },
        );
        each_lane::<Sum<N, M>>(
            #[inline(always)]
            |k| {
                let value = backend::xor(backend::up(odd.lane(k)), even.lane(k));
                let carried = if k == 0 {
                    value
                } else {
                    backend::xor(value, backend::down(odd.lane(k - 1)))
                };
                even.set(k, carried);
            },
        );
        even
    }

    /// The whole square `self * self`, in `2N` bits: the cross terms cancel, so each word squares
    /// alone onto its register, bit `i` at bit `2 i`.
    #[inline(always)]
    #[must_use]
    pub fn widening_square(self) -> Poly<Sum<N, N>>
    where
        N: Add<N>,
        Sum<N, N>: Width,
    {
        let mut out = Poly::<Sum<N, N>>::ZERO;
        each_lane::<N>(
            #[inline(always)]
            |index| {
                let (first, second) = backend::spread(self.lane(index));
                out.set(2 * index, first);
                out.set(2 * index + 1, second);
            },
        );
        out
    }

    /// `self * x^bits mod x^N`.
    #[inline(always)]
    fn shifted_left(self, bits: usize) -> Self {
        if bits >= N::BITS {
            return Self::ZERO;
        }
        let (lanes, words, shift) = (bits / 128, (bits / 64) % 2, (bits % 64) as u32);
        let mut out = Self::ZERO;
        if N::BITS <= 64 {
            // One word: a word shift drops what leaves it.
            out.set(0, backend::shl_words(self.lane(0), shift));
            return if N::BITS < 64 { out.truncated() } else { out };
        }
        // Whole registers, then one word, then the bits below a word.
        each_lane::<N>(
            #[inline(always)]
            |k| {
                let source = |index: usize| -> Register {
                    if index < lanes {
                        backend::ZERO
                    } else {
                        self.lane(index - lanes)
                    }
                };
                let worded = |index: usize| -> Register {
                    if words == 0 {
                        source(index)
                    } else if index == 0 {
                        backend::up(source(0))
                    } else {
                        backend::xor(backend::up(source(index)), backend::down(source(index - 1)))
                    }
                };
                let value = if shift == 0 {
                    worded(k)
                } else {
                    let carry = backend::shr_words(worded(k), 64 - shift);
                    let lane =
                        backend::xor(backend::shl_words(worded(k), shift), backend::up(carry));
                    if k == 0 {
                        lane
                    } else {
                        let below = backend::shr_words(worded(k - 1), 64 - shift);
                        backend::xor(lane, backend::down(below))
                    }
                };
                out.set(k, value);
            },
        );
        out.truncated()
    }

    /// `self / x^bits`, rounded down.
    #[inline(always)]
    fn shifted_right(self, bits: usize) -> Self {
        if bits >= N::BITS {
            return Self::ZERO;
        }
        let (lanes, words, shift) = (bits / 128, (bits / 64) % 2, (bits % 64) as u32);
        let mut out = Self::ZERO;
        if N::BITS <= 64 {
            out.set(0, backend::shr_words(self.lane(0), shift));
            return out;
        }
        each_lane::<N>(
            #[inline(always)]
            |k| {
                let source = |index: usize| -> Register { self.lane(index + lanes) };
                let worded = |index: usize| -> Register {
                    if words == 0 {
                        source(index)
                    } else {
                        backend::xor(backend::down(source(index)), backend::up(source(index + 1)))
                    }
                };
                let value = if shift == 0 {
                    worded(k)
                } else {
                    let carry = backend::shl_words(worded(k), 64 - shift);
                    let lane =
                        backend::xor(backend::shr_words(worded(k), shift), backend::down(carry));
                    let above = backend::shl_words(worded(k + 1), 64 - shift);
                    backend::xor(lane, backend::up(above))
                };
                out.set(k, value);
            },
        );
        out
    }
}

impl<N: Width> Clone for Poly<N> {
    #[inline(always)]
    fn clone(&self) -> Self {
        *self
    }
}

impl<N: Width> Copy for Poly<N> {}

impl<N: Width> Default for Poly<N> {
    #[inline(always)]
    fn default() -> Self {
        Self::ZERO
    }
}

impl<N: Width> PartialEq for Poly<N> {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        self.lanes
            .iter()
            .zip(other.lanes.iter())
            .all(|(a, b)| backend::words(a.0) == backend::words(b.0))
    }
}

impl<N: Width> Eq for Poly<N> {}

impl<N: Width> Hash for Poly<N> {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.to_uint().hash(state);
    }
}

impl<N: Width> fmt::Debug for Poly<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Poly({:?})", self.to_uint())
    }
}

impl<N: Width> From<Uint<N>> for Poly<N> {
    #[inline(always)]
    fn from(value: Uint<N>) -> Self {
        Self::from_uint(value)
    }
}

impl<N: Width> From<Poly<N>> for Uint<N> {
    #[inline(always)]
    fn from(poly: Poly<N>) -> Self {
        poly.to_uint()
    }
}

/// Implements a bitwise operator and its assignment form lane by lane.
macro_rules! bitwise {
    ($trait:ident, $method:ident, $assign_trait:ident, $assign:ident, $op:path) => {
        impl<N: Width> $trait for Poly<N> {
            type Output = Self;

            #[inline(always)]
            fn $method(mut self, rhs: Self) -> Self {
                each_lane::<N>(
                    #[inline(always)]
                    |index| {
                        self.set(index, $op(self.lane(index), rhs.lane(index)));
                    },
                );
                self
            }
        }

        impl<N: Width> $assign_trait for Poly<N> {
            #[inline(always)]
            fn $assign(&mut self, rhs: Self) {
                *self = $trait::$method(*self, rhs);
            }
        }
    };
}

bitwise!(BitXor, bitxor, BitXorAssign, bitxor_assign, backend::xor);
bitwise!(BitAnd, bitand, BitAndAssign, bitand_assign, backend::and);
bitwise!(BitOr, bitor, BitOrAssign, bitor_assign, backend::or);

/// `self * x^bits mod x^N`.
impl<N: Width> Shl<usize> for Poly<N> {
    type Output = Self;

    #[inline(always)]
    fn shl(self, bits: usize) -> Self {
        self.shifted_left(bits)
    }
}

/// `self / x^bits`, rounded down.
impl<N: Width> Shr<usize> for Poly<N> {
    type Output = Self;

    #[inline(always)]
    fn shr(self, bits: usize) -> Self {
        self.shifted_right(bits)
    }
}
