//! The arithmetic of a field on the registers its plan chooses: the default operations, which may
//! index tables and take time that depends on the values, and the constant-time ones of `Secret`.

use core::marker::PhantomData;
use core::ops::{BitAnd, BitXor, Shl, Shr};

use biguint::Uint;
use clmul::Poly;
use const_default::ConstDefault;
use typenum::{U64, U128};

use crate::frobenius::ChainStep;
use crate::plan::{Plan, Polynomial, Shape, Wide};
use crate::representation::{Representation, Storage};
use crate::table::Tables;

/// The registers of an element of the field with the polynomial `P`.
pub(crate) type Element<P> = <<P as Polynomial>::Degree as Representation>::Element;

/// The polynomial of an element of the field with the polynomial `P`.
pub(crate) type Lanes<P> = Poly<<P as Polynomial>::Degree>;

/// Runs `$body` with `$index` bound to each of `0..$len`. Optimized builds get one copy per index
/// below 8, so that constant lengths fold to straight-line code, and a loop beyond; debug builds
/// get the loop alone, which keeps the frames of the inlined operations small.
macro_rules! each {
    ($index:ident < $len:expr => $body:block) => {{
        let len: usize = $len;
        #[cfg(not(debug_assertions))]
        each!(@ $index, len, 8, $body; 0 1 2 3 4 5 6 7);
        #[cfg(debug_assertions)]
        each!(@ $index, len, 0, $body;);
    }};
    (@ $index:ident, $len:ident, $from:literal, $body:block; $($k:literal)*) => {
        $(
            if $k < $len {
                let $index: usize = $k;
                $body
            }
        )*
        let mut $index: usize = $from;
        while $index < $len {
            $body
            $index += 1;
        }
    };
}

pub(crate) use each;

/// A polynomial of at most 64 coefficients in one register: a general register's `u64`, or a
/// vector register's `Poly<U64>`.
pub(crate) trait Word:
    Copy
    + BitXor<Output = Self>
    + BitAnd<Output = Self>
    + Shl<usize, Output = Self>
    + Shr<usize, Output = Self>
{
    /// `0`.
    const ZERO: Self;
    /// The polynomial of the bits of `word`.
    fn of(word: u64) -> Self;
}

impl Word for u64 {
    const ZERO: Self = 0;

    #[inline(always)]
    fn of(word: u64) -> Self {
        word
    }
}

impl Word for Poly<U64> {
    const ZERO: Self = Self::ZERO;

    #[inline(always)]
    fn of(word: u64) -> Self {
        Self::from_uint(Uint::from_u64(word))
    }
}

/// The bits below `x^bits` of a word, for `bits` up to 64.
#[inline(always)]
const fn low_mask(bits: usize) -> u64 {
    if bits >= 64 {
        u64::MAX
    } else {
        (1 << bits) - 1
    }
}

/// The arithmetic of the field with the polynomial `P`.
pub(crate) struct Field<P>(PhantomData<P>);

impl<P: Polynomial> Field<P> {
    /// `h r mod x^n` and the bits of `h r` at and above `x^n`, for `h` of degree below `n`, by one
    /// shift per tap: with `t` the overflow, `(h + t) r mod x^n` is `h r mod f`, since `t r` stays
    /// below `x^n` when `2d <= n`. Words of at most 64 bits.
    #[inline(always)]
    fn fold_word<W: Word>(low: W, high: W) -> W {
        let degree = Plan::<P>::DEGREE;
        let taps = Plan::<P>::TAPS;
        let mut overflow = W::ZERO;
        each!(index < taps.len() => {
            if let Some(&tap) = taps.get(index)
                && tap > 0
            {
                overflow = overflow ^ (high >> (degree - tap));
            }
        });
        let high = high ^ overflow;
        let mut sum = low ^ high;
        each!(index < taps.len() => {
            if let Some(&tap) = taps.get(index)
                && tap > 0
            {
                sum = sum ^ (high << tap);
            }
        });
        if degree >= 64 {
            sum
        } else {
            sum & W::of(low_mask(degree))
        }
    }

    /// `p mod f` for a product `p` of degree below `2n` in one word: `n` up to 32.
    #[inline(always)]
    fn reduce_word<W: Word>(product: W) -> W {
        let degree = Plan::<P>::DEGREE;
        Self::fold_word(product & W::of(low_mask(degree)), product >> degree)
    }

    // ---------------------------------------------------------------------------------------
    // The polynomial of an element.

    /// The product of two polynomials of the field.
    #[inline(always)]
    pub(crate) fn lanes_mul(a: Lanes<P>, b: Lanes<P>) -> Lanes<P> {
        match Plan::<P>::SHAPE {
            Shape::Word => {
                let product = a.clmul_words(0, b, 0).resize::<U64>();
                Self::reduce_word(product).resize()
            }
            Shape::Words => Self::fold_words(a.clmul_words(0, b, 0)),
            Shape::Lane => Self::lane_mul(a.resize(), b.resize()).resize(),
            Shape::Lanes => Self::reduce_lanes(a.widening_mul(b)),
        }
    }

    /// The square of a polynomial of the field.
    #[inline(always)]
    pub(crate) fn lanes_square(a: Lanes<P>) -> Lanes<P> {
        match Plan::<P>::SHAPE {
            Shape::Word => {
                let square = a.clmul_words(0, a, 0).resize::<U64>();
                Self::reduce_word(square).resize()
            }
            Shape::Words => Self::fold_words(a.clmul_words(0, a, 0)),
            Shape::Lane => {
                let square = a.resize::<U128>().widening_square();
                Self::fold_lane(square.resize(), (square >> 128).resize()).resize()
            }
            Shape::Lanes => Self::reduce_lanes(a.widening_square()),
        }
    }

    /// `p mod f` for `n` from 33 to 64 and a product `p` of two words.
    #[inline(always)]
    fn fold_words(product: Poly<U128>) -> Lanes<P> {
        let degree = Plan::<P>::DEGREE;
        let low = product.resize::<U64>() & Poly::of(low_mask(degree));
        let high = (product >> degree).resize::<U64>();
        Self::fold_word(low, high).resize()
    }

    /// `a b mod f` for `n = 128` and `d` below 64: Karatsuba's three word products give
    /// `low + middle x^64 + high x^128`; the top word of `high` folds by its product with `r` into
    /// the middle, and the top word of the middle into the bottom (Gueron and Kounavis, 2010).
    #[inline(always)]
    fn lane_mul(a: Poly<U128>, b: Poly<U128>) -> Poly<U128> {
        let r = Poly::<U64>::of(Plan::<P>::R_WORD);
        let low = a.clmul_words(0, b, 0);
        let high = a.clmul_words(1, b, 1);
        let (a_sum, b_sum) = (a ^ (a >> 64), b ^ (b >> 64));
        let middle = a_sum.clmul_words(0, b_sum, 0) ^ low ^ high;
        let middle = middle ^ (high << 64) ^ high.clmul_words(1, r, 0);
        low ^ (middle << 64) ^ middle.clmul_words(1, r, 0)
    }

    /// `low + high x^128 mod f` for `n = 128` and `d` below 64, by shifts within words: the carries
    /// out of the low word and the overflow of the high word come from the same shifts, with the
    /// overflow added to `high` first.
    #[inline(always)]
    fn fold_lane(low: Poly<U128>, high: Poly<U128>) -> Poly<U128> {
        let taps = Plan::<P>::TAPS;
        let mut spill = Poly::<U128>::ZERO;
        each!(index < taps.len() => {
            if let Some(&tap) = taps.get(index)
                && tap > 0
            {
                spill ^= high.shr_within_words((64 - tap) as u32);
            }
        });
        let high = high ^ (spill >> 64);
        let mut sum = low ^ high ^ (spill << 64);
        each!(index < taps.len() => {
            if let Some(&tap) = taps.get(index)
                && tap > 0
            {
                sum ^= high.shl_within_words(tap as u32);
            }
        });
        sum
    }

    /// `p mod f` for a product `p` of two polynomials of the field, by rounds of folds.
    #[inline(always)]
    fn reduce_lanes(product: Poly<Wide<P>>) -> Lanes<P> {
        let degree = Plan::<P>::DEGREE;
        let mut low = product.resize::<P::Degree>();
        if Plan::<P>::R_DEGREE < 64 && Plan::<P>::ONE_SPILL {
            // Two products with `r`: the high half's, and its overflow's, below 64 bits.
            let r = Poly::<U64>::of(Plan::<P>::R_WORD);
            let folded = (product >> degree).resize::<P::Degree>().widening_mul(r);
            low ^= folded.resize();
            let spill = (folded >> degree).resize::<U64>().widening_mul(r);
            low ^ spill.resize()
        } else {
            // Rounds of shifted copies, one per tap.
            let mut rest = product >> degree;
            each!(_round < Plan::<P>::ROUNDS => {
                let mut folded = Poly::<Wide<P>>::ZERO;
                let taps = Plan::<P>::TAPS;
                each!(index < taps.len() => {
                    if let Some(&tap) = taps.get(index) {
                        folded ^= rest << tap;
                    }
                });
                low ^= folded.resize();
                rest = folded >> degree;
            });
            low
        }
    }

    /// `sqrt(a) = e(x) + sqrt(x) o(x)` for the even and odd coefficients `e(x^2)` and `x o(x^2)` of
    /// `a`, which the masks and shifts within words gather, a word pair at a time.
    #[inline(always)]
    pub(crate) fn lanes_sqrt(a: Lanes<P>) -> Lanes<P> {
        if Plan::<P>::SQRT_BY_SPLIT {
            let even = gather(a);
            let odd = gather(a.shr_within_words(1));
            even ^ Self::lanes_mul(odd, Tables::<P>::sqrt_x())
        } else {
            let mut power = a;
            each!(_step < Plan::<P>::DEGREE - 1 => {
                power = Self::lanes_square(power);
            });
            power
        }
    }

    // ---------------------------------------------------------------------------------------
    // The word of an element of a field of degree up to 16.

    /// `a b mod f`.
    #[inline(always)]
    pub(crate) fn word_mul(a: u64, b: u64) -> u64 {
        Self::reduce_word(clmul::clmul64(a, b) as u64)
    }

    /// `a^2 mod f`: the constant-time linear map of squaring up to degree 8, a carry-less square
    /// above, where the map takes four parity gathers.
    #[inline(always)]
    pub(crate) fn word_square(a: u64) -> u64 {
        if Plan::<P>::DEGREE <= 8 {
            Tables::<P>::frobenius(a, 1)
        } else {
            Self::reduce_word(clmul::clmul64(a, a) as u64)
        }
    }

    /// `a^(2^k)`: squarings for `k = 1`, the constant-time linear map of the power above.
    #[inline(always)]
    fn word_frobenius(a: u64, k: usize) -> u64 {
        if k == 1 {
            Self::word_square(a)
        } else {
            Tables::<P>::frobenius(a, k)
        }
    }

    /// The square root `a^(2^(n-1))`, by the constant-time linear map.
    #[inline(always)]
    pub(crate) fn word_sqrt(a: u64) -> u64 {
        Tables::<P>::frobenius(a, Plan::<P>::DEGREE - 1)
    }

    /// `a^(2^k)` by `k` squarings.
    #[inline(always)]
    fn lanes_frobenius(a: Lanes<P>, k: usize) -> Lanes<P> {
        let mut power = a;
        let mut step = 0;
        while step < k {
            power = Self::lanes_square(power);
            step += 1;
        }
        power
    }

    // ---------------------------------------------------------------------------------------
    // The operations on elements: the defaults, and those of `Secret`.

    /// `a b`, the default.
    #[inline(always)]
    pub(crate) fn mul(a: Element<P>, b: Element<P>) -> Element<P> {
        if Plan::<P>::MUL_BY_TABLE {
            Element::<P>::from_word(Tables::<P>::mul(a.word(), b.word()))
        } else {
            Self::ct_mul(a, b)
        }
    }

    /// `a b` in constant time.
    #[inline(always)]
    pub(crate) fn ct_mul(a: Element<P>, b: Element<P>) -> Element<P> {
        if Plan::<P>::SMALL {
            Element::<P>::from_word(Self::word_mul(a.word(), b.word()))
        } else {
            Element::<P>::from_poly(Self::lanes_mul(a.poly(), b.poly()))
        }
    }

    /// `a^2`, the default.
    #[inline(always)]
    pub(crate) fn square(a: Element<P>) -> Element<P> {
        if Plan::<P>::SMALL {
            Element::<P>::from_word(Tables::<P>::square(a.word()))
        } else {
            Self::ct_square(a)
        }
    }

    /// `a^2` in constant time.
    #[inline(always)]
    pub(crate) fn ct_square(a: Element<P>) -> Element<P> {
        if Plan::<P>::SMALL {
            Element::<P>::from_word(Self::word_square(a.word()))
        } else {
            Element::<P>::from_poly(Self::lanes_square(a.poly()))
        }
    }

    /// The square root, the default.
    #[inline(always)]
    pub(crate) fn sqrt(a: Element<P>) -> Element<P> {
        if Plan::<P>::SMALL {
            Element::<P>::from_word(Tables::<P>::sqrt(a.word()))
        } else {
            Self::ct_sqrt(a)
        }
    }

    /// The square root in constant time.
    #[inline(always)]
    pub(crate) fn ct_sqrt(a: Element<P>) -> Element<P> {
        if Plan::<P>::SMALL {
            Element::<P>::from_word(Self::word_sqrt(a.word()))
        } else {
            Element::<P>::from_poly(Self::lanes_sqrt(a.poly()))
        }
    }

    /// The inverse of a nonzero `a`, and `0` for `0`, the default.
    #[inline(always)]
    pub(crate) fn invert(a: Element<P>) -> Element<P> {
        if Plan::<P>::SMALL {
            Element::<P>::from_word(Tables::<P>::invert(a.word()))
        } else if Plan::<P>::INVERT_BY_TABLES {
            Element::<P>::from_poly(Self::lanes_invert(a.poly()))
        } else {
            Self::ct_invert(a)
        }
    }

    /// The chain of Itoh and Tsujii of a field of one register, whose powers from `x^(2^3)` on
    /// read the tables of the chain's steps. The tables cover degrees up to 128, so the chain has
    /// at most seven steps.
    #[inline(always)]
    fn lanes_invert(a: Lanes<P>) -> Lanes<P> {
        let bits = Plan::<P>::CHAIN_BITS;
        let exponent = Plan::<P>::DEGREE - 1;
        let mut power = a;
        macro_rules! steps {
            ($($step:literal)*) => {$(
                if $step + 1 < bits {
                    let index = bits - 2 - $step;
                    let doubled = if ChainStep::<P, $step>::TABULATED {
                        ChainStep::<P, $step>::apply(power)
                    } else {
                        Self::lanes_frobenius(power, ChainStep::<P, $step>::K)
                    };
                    power = Self::lanes_mul(doubled, power);
                    if (exponent >> index) & 1 == 1 {
                        power = Self::lanes_mul(Self::lanes_square(power), a);
                    }
                }
            )*};
        }
        steps!(0 1 2 3 4 5 6);
        Self::lanes_square(power)
    }

    /// The inverse of a nonzero `a`, and `0` for `0`, in constant time: `a^(2^n - 2)` by the chain
    /// of Itoh and Tsujii.
    #[inline(always)]
    pub(crate) fn ct_invert(a: Element<P>) -> Element<P> {
        if Plan::<P>::SMALL {
            Element::<P>::from_word(crate::invert::itoh_tsujii::<P, _>(
                a.word(),
                Self::word_frobenius,
                Self::word_mul,
            ))
        } else {
            Element::<P>::from_poly(crate::invert::itoh_tsujii::<P, _>(
                a.poly(),
                Self::lanes_frobenius,
                Self::lanes_mul,
            ))
        }
    }
}

/// The masks of [`gather`] at the width `N`.
struct Masks<N>(PhantomData<N>);

impl<N: clmul::Width> Masks<N> {
    /// The masks of the gather within words: every other bit, pair, nibble, byte, 16 bits and 32
    /// bits of each word.
    const WITHIN: [Poly<N>; 6] = [
        Poly::from_uint(repeated::<N>(0x5555_5555_5555_5555)),
        Poly::from_uint(repeated::<N>(0x3333_3333_3333_3333)),
        Poly::from_uint(repeated::<N>(0x0F0F_0F0F_0F0F_0F0F)),
        Poly::from_uint(repeated::<N>(0x00FF_00FF_00FF_00FF)),
        Poly::from_uint(repeated::<N>(0x0000_FFFF_0000_FFFF)),
        Poly::from_uint(repeated::<N>(0x0000_0000_FFFF_FFFF)),
    ];

    /// The masks of the packing across words: the low halves of the blocks of 128, 256, 512,
    /// 1024 and 2048 bits.
    const BLOCKS: [Poly<N>; 5] = [
        Poly::from_uint(blocks::<N>(64)),
        Poly::from_uint(blocks::<N>(128)),
        Poly::from_uint(blocks::<N>(256)),
        Poly::from_uint(blocks::<N>(512)),
        Poly::from_uint(blocks::<N>(1024)),
    ];
}

/// The coefficients at the even positions of `a`, moved to half their position: the gather
/// within each word (Warren, Hacker's Delight, 7-2), which leaves 32 coefficients in the low half
/// of each word, then the packing of those halves into words, of words into word pairs, and so on.
#[inline(always)]
fn gather<N: clmul::Width>(a: Poly<N>) -> Poly<N> {
    let within = Masks::<N>::WITHIN;
    let mut x = a & within[0];
    each!(level < 5 => {
        if let Some(&mask) = within.get(level + 1) {
            x = (x | x.shr_within_words(1u32 << level)) & mask;
        }
    });
    let half = N::BITS.div_ceil(2);
    each!(level < 5 => {
        let chunk = 32 << level;
        if chunk < half
            && let Some(&mask) = Masks::<N>::BLOCKS.get(level)
        {
            x = (x | (x >> chunk)) & mask;
        }
    });
    x
}

/// `pattern` in every word of an `N`-bit integer.
const fn repeated<N: clmul::Width>(pattern: u64) -> Uint<N> {
    let mut limbs = generic_array::GenericArray::<u64, N::Limbs>::DEFAULT;
    let mut slots = limbs.as_mut_slice();
    while let [slot, rest @ ..] = slots {
        *slot = pattern;
        slots = rest;
    }
    Uint::from_generic_limbs(limbs)
}

/// The bits whose position modulo `2 size` lies below `size`, for `size` a multiple of 64.
const fn blocks<N: clmul::Width>(size: usize) -> Uint<N> {
    let mut limbs = generic_array::GenericArray::<u64, N::Limbs>::DEFAULT;
    let mut slots = limbs.as_mut_slice();
    let mut index = 0;
    while let [slot, rest @ ..] = slots {
        *slot = if (index * 64) % (2 * size) < size {
            u64::MAX
        } else {
            0
        };
        index += 1;
        slots = rest;
    }
    Uint::from_generic_limbs(limbs)
}
