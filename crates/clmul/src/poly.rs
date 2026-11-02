//! Polynomials over GF(2) in the registers of the build's vector unit.

use core::fmt;
use core::hash::{Hash, Hasher};
use core::ops::{
    Add, BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Div, Shl, Shr,
};

use biguint::generic_array::{ArrayLength, GenericArray};
use biguint::typenum::{Quot, Sum, U127, U128, Unsigned};
use biguint::{Bits, OutOfRange, Uint};
use const_default::ConstDefault;

use crate::backend::{self, Register};

/// Runs `$body` with `$index` bound to each of `0..$len`: one copy per index below 16, so that
/// constant lengths fold to straight-line code, and a loop beyond.
macro_rules! each {
    ($index:ident < $len:expr => $body:block) => {{
        let len: usize = $len;
        each!(@ $index, len, $body; 0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15);
    }};
    (@ $index:ident, $len:ident, $body:block; $($k:literal)*) => {
        $(
            if $k < $len {
                let $index: usize = $k;
                $body
            }
        )*
        let mut $index: usize = 16;
        while $index < $len {
            $body
            $index += 1;
        }
    };
}

/// One register of a [`Poly`]: 128 bits, two 64-bit words.
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct Lane(Register);

impl ConstDefault for Lane {
    const DEFAULT: Self = Self(backend::ZERO);
}

/// A bit width `N` with the registers of a [`Poly<N>`]: `ceil(N / 128)`.
///
/// Exactly the nonzero `typenum` unsigned integers implement it.
pub trait Width: Bits
where
    <Self::Lanes as ArrayLength>::ArrayType<Lane>: Copy + ConstDefault,
{
    /// `ceil(N / 128)`.
    type Lanes: ArrayLength;

    /// `ceil(N / 128)`.
    const LANES: usize;
}

impl<N> Width for N
where
    N: Bits + Add<U127>,
    Sum<N, U127>: Div<U128>,
    Quot<Sum<N, U127>, U128>: ArrayLength,
    <Quot<Sum<N, U127>, U128> as ArrayLength>::ArrayType<Lane>: Copy + ConstDefault,
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
    lanes: GenericArray<Lane, N::Lanes>,
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
        let mut lanes = GenericArray::<Lane, N::Lanes>::DEFAULT;
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
            *slot = Lane(backend::register([low, high]));
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
        each!(index < M::LANES => {
            out.set(index, self.lane(index));
        });
        if M::BITS < N::BITS {
            out.truncated()
        } else {
            out
        }
    }

    /// The carry-less product of word `i` of `self` and word `j` of `rhs`: two coefficients of
    /// `x^(64 i)` and `x^(64 j)` times `x^(64 (i + j))` are the 128 bits of the result.
    #[inline(always)]
    #[must_use]
    pub fn clmul_words<M: Width>(self, i: usize, rhs: Poly<M>, j: usize) -> Poly<U128> {
        let mut out = Poly::<U128>::ZERO;
        out.set(
            0,
            backend::clmul(self.lane(i / 2), i % 2, rhs.lane(j / 2), j % 2),
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
        each!(i < N::LANES => {
            let a = self.lane(i);
            let a_full = 2 * i + 1 < a_words;
            let a_sum = backend::xor(a, backend::down(a));
            each!(j < M::LANES => {
                let b = rhs.lane(j);
                let b_full = 2 * j + 1 < b_words;
                let low = backend::clmul(a, 0, b, 0);
                even.add_to(i + j, low);
                match (a_full, b_full) {
                    (true, true) => {
                        let high = backend::clmul(a, 1, b, 1);
                        let b_sum = backend::xor(b, backend::down(b));
                        let middle = backend::xor(
                            backend::xor(backend::clmul(a_sum, 0, b_sum, 0), low),
                            high,
                        );
                        even.add_to(i + j + 1, high);
                        odd.add_to(i + j, middle);
                    }
                    (true, false) => odd.add_to(i + j, backend::clmul(a, 1, b, 0)),
                    (false, true) => odd.add_to(i + j, backend::clmul(a, 0, b, 1)),
                    (false, false) => {}
                }
            });
        });
        let lanes = <Sum<N, M> as Width>::LANES;
        each!(k < lanes => {
            let value = backend::xor(backend::up(odd.lane(k)), even.lane(k));
            let carried = if k == 0 {
                value
            } else {
                backend::xor(value, backend::down(odd.lane(k - 1)))
            };
            even.set(k, carried);
        });
        even
    }

    /// The whole square `self * self`, in `2N` bits: the cross terms cancel, so each word squares
    /// alone onto its register.
    #[inline(always)]
    #[must_use]
    pub fn widening_square(self) -> Poly<Sum<N, N>>
    where
        N: Add<N>,
        Sum<N, N>: Width,
    {
        let mut out = Poly::<Sum<N, N>>::ZERO;
        each!(word < N::LIMBS => {
            let lane = self.lane(word / 2);
            out.set(word, backend::clmul(lane, word % 2, lane, word % 2));
        });
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
            return out.truncated();
        }
        // Whole registers, then one word, then the bits below a word.
        each!(k < N::LANES => {
            let source = |index: usize| -> Register {
                if index < lanes { backend::ZERO } else { self.lane(index - lanes) }
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
                let lane = backend::xor(backend::shl_words(worded(k), shift), backend::up(carry));
                if k == 0 {
                    lane
                } else {
                    let below = backend::shr_words(worded(k - 1), 64 - shift);
                    backend::xor(lane, backend::down(below))
                }
            };
            out.set(k, value);
        });
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
        each!(k < N::LANES => {
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
        });
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
                each!(index < N::LANES => {
                    self.set(index, $op(self.lane(index), rhs.lane(index)));
                });
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
