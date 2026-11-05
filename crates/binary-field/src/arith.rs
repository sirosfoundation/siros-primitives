//! The arithmetic of a field on the registers its plan chooses: the default operations, which may
//! index tables and take time that depends on the values, and the constant-time ones of
//! `ConstantTime`.

use core::marker::PhantomData;
use core::ops::{BitAnd, BitXor, Shl, Shr};

use biguint::Uint;
use clmul::Poly;
use const_default::ConstDefault;
use typenum::{U8, U64, U96, U128};

use crate::frobenius::Frobenius;
use crate::invert::{Inversion, inverse};
use crate::kernel::{clmul8, clmul16, spread_half};
use crate::plan::{Half, Plan, Polynomial, Shape, Wide};
use crate::representation::{Representation, Storage};
use crate::subfield::Subfield;
use crate::table::Tables;

/// The registers of an element of the field with the polynomial `P`.
pub(crate) type Element<P> = <<P as Polynomial>::Degree as Representation>::Element;

/// The polynomial of an element of the field with the polynomial `P`.
pub(crate) type Lanes<P> = Poly<<P as Polynomial>::Degree>;

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

/// Runs `body` on each index below `len`, in order. Builds without debug assertions call it at a
/// literal index for each index below 8 and loop past 8, so that a constant `len` folds to
/// straight-line code before the optimizer schedules it: with plain loops, which LLVM unrolls late,
/// a GF(2^128) product without carry-less multiplication took 8% more cycles. Debug builds loop.
#[inline(always)]
pub(crate) fn each(len: usize, mut body: impl FnMut(usize)) {
    if cfg!(debug_assertions) {
        for index in 0..len {
            body(index);
        }
        return;
    }
    if len > 0 {
        body(0);
    }
    if len > 1 {
        body(1);
    }
    if len > 2 {
        body(2);
    }
    if len > 3 {
        body(3);
    }
    if len > 4 {
        body(4);
    }
    if len > 5 {
        body(5);
    }
    if len > 6 {
        body(6);
    }
    if len > 7 {
        body(7);
    }
    for index in 8..len {
        body(index);
    }
}

/// Where a squaring runs: alone, or in a run of squarings, a Frobenius power.
#[derive(Clone, Copy)]
pub(crate) enum Squaring {
    /// One squaring.
    Alone,
    /// A squaring of a run.
    Run,
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
        each(
            taps.len(),
            #[inline(always)]
            |index| {
                if let Some(&tap) = taps.get(index)
                    && tap > 0
                {
                    overflow = overflow ^ (high >> (degree - tap));
                }
            },
        );
        let high = high ^ overflow;
        let mut sum = low ^ high;
        each(
            taps.len(),
            #[inline(always)]
            |index| {
                if let Some(&tap) = taps.get(index)
                    && tap > 0
                {
                    sum = sum ^ (high << tap);
                }
            },
        );
        if degree >= 64 {
            sum
        } else {
            sum & W::of(low_mask(degree))
        }
    }

    /// `p mod f` for a product `p` of degree below `2n` in one word: `n` up to 32. One fold with the
    /// overflow added first when `2d <= n`; otherwise the plan's rounds, each of which moves the
    /// bits at and above `x^n` down by `x^n = r`.
    #[inline(always)]
    fn reduce_word<W: Word>(product: W) -> W {
        let degree = Plan::<P>::DEGREE;
        let mask = W::of(low_mask(degree));
        if Plan::<P>::ONE_SPILL {
            return Self::fold_word(product & mask, product >> degree);
        }
        let taps = Plan::<P>::TAPS;
        let (mut sum, mut rest) = (product & mask, product >> degree);
        each(
            Plan::<P>::ROUNDS,
            #[inline(always)]
            |_round| {
                let mut folded = W::ZERO;
                each(
                    taps.len(),
                    #[inline(always)]
                    |index| {
                        if let Some(&tap) = taps.get(index) {
                            folded = folded ^ (rest << tap);
                        }
                    },
                );
                sum = sum ^ (folded & mask);
                rest = folded >> degree;
            },
        );
        sum
    }

    // ---------------------------------------------------------------------------------------
    // The polynomial of an element.

    /// The product of two polynomials of the field.
    #[cfg_attr(debug_assertions, inline(never))]
    #[cfg_attr(not(debug_assertions), inline(always))]
    pub(crate) fn lanes_mul(a: Lanes<P>, b: Lanes<P>) -> Lanes<P> {
        match Plan::<P>::SHAPE {
            Shape::Word => {
                let product = a.clmul_words(0, b, 0).resize::<U64>();
                Self::reduce_word(product).resize()
            }
            Shape::Words => Self::fold_words(a.clmul_words(0, b, 0)),
            Shape::Lane if Plan::<P>::FOLD_BY_PRODUCTS => {
                Self::lane_mul(a.resize(), b.resize()).resize()
            }
            Shape::Lane => {
                let product = a.resize::<U128>().widening_mul(b.resize::<U128>());
                Self::fold_lane(product.resize(), (product >> 128).resize()).resize()
            }
            Shape::Lanes => Self::reduce_lanes(a.widening_mul(b)),
        }
    }

    /// The square of a polynomial of the field.
    #[cfg_attr(debug_assertions, inline(never))]
    #[cfg_attr(not(debug_assertions), inline(always))]
    pub(crate) fn lanes_square(a: Lanes<P>) -> Lanes<P> {
        Self::square_in(a, Squaring::Alone)
    }

    /// The square of a polynomial of the field, as a squaring of a run.
    #[inline(always)]
    pub(crate) fn run_square(a: Lanes<P>) -> Lanes<P> {
        Self::square_in(a, Squaring::Run)
    }

    /// The square of a polynomial of the field, in the context `context`.
    #[inline(always)]
    fn square_in(a: Lanes<P>, context: Squaring) -> Lanes<P> {
        match Plan::<P>::SHAPE {
            Shape::Word => {
                let square = Self::word_poly_square(a, context).resize::<U64>();
                Self::reduce_word(square).resize()
            }
            Shape::Words if Plan::<P>::SPILL_BY_NIBBLES => {
                // The top nibble of `h` holds the squares of the top two bits of `a`, so their
                // map gives the spill without waiting on the square.
                let degree = Plan::<P>::DEGREE;
                let spill = (a >> (degree - 2)).map_low_nibbles(Tables::<P>::SQUARE_SPILL);
                Self::fold_spilled(Self::word_poly_square(a, context), spill.resize())
            }
            Shape::Words => Self::fold_words(Self::word_poly_square(a, context)),
            Shape::Lane => {
                let square = a.resize::<U128>().widening_square();
                Self::fold_lane(square.resize(), (square >> 128).resize()).resize()
            }
            Shape::Lanes => Self::reduce_lanes(a.widening_square()),
        }
    }

    /// The square of a polynomial of at most 64 coefficients, in 128 bits. A square alone takes
    /// `clmul`'s spread of the bits to twice their position, a byte shuffle of a nibble table with
    /// SSSE3, which leaves the carry-less multiplier to the fold; a run of squarings takes the
    /// carry-less product with hardware word products, which takes fewer instructions.
    #[inline(always)]
    fn word_poly_square(a: Lanes<P>, context: Squaring) -> Poly<U128> {
        if clmul::HARDWARE && matches!(context, Squaring::Run) {
            a.clmul_words(0, a, 0)
        } else {
            a.resize::<U64>().widening_square()
        }
    }

    /// `p mod f` for `n` from 33 to 64 and a product `p` of two words. With hardware word products,
    /// `(h + t) r mod x^n` for the part `h` at and above `x^n` and the overflow `t` of `h r` is one
    /// word product, with `t` from shifts of `h`; otherwise shifts per tap give it.
    #[inline(always)]
    fn fold_words(product: Poly<U128>) -> Lanes<P> {
        let degree = Plan::<P>::DEGREE;
        let mask = Poly::of(low_mask(degree));
        if Plan::<P>::SPILL_BY_NIBBLES {
            let spill = (product >> (2 * degree - 4)).map_low_nibbles(Tables::<P>::SPILL);
            Self::fold_spilled(product, spill)
        } else if Plan::<P>::FOLD_BY_PRODUCTS && degree == 64 {
            // `h` is word 1, so the shifts within words give `t` in word 1 without a move.
            let r = Poly::<U64>::of(Plan::<P>::R_WORD);
            let raised = product ^ Self::overflow(product);
            (product ^ raised.clmul_words(1, r, 0)).resize()
        } else if Plan::<P>::FOLD_BY_PRODUCTS {
            let r = Poly::<U64>::of(Plan::<P>::R_WORD);
            let high = (product >> degree).resize::<U64>();
            let raised = high ^ Self::overflow(high);
            let folded = raised.clmul_words(0, r, 0).resize::<U64>();
            ((product.resize::<U64>() ^ folded) & mask).resize()
        } else {
            let low = product.resize::<U64>() & mask;
            let high = (product >> degree).resize::<U64>();
            Self::fold_word(low, high).resize()
        }
    }

    /// `p mod f` for `n` from 33 to 64, `d` at most 4 and a product `p` of two words, as
    /// `low + (h r mod x^n) + t r`, given `t r`, the image of the top nibble of `h` under the
    /// spill map, which runs in parallel with the fold of `h`.
    #[inline(always)]
    fn fold_spilled(product: Poly<U128>, spill: Poly<U128>) -> Lanes<P> {
        let degree = Plan::<P>::DEGREE;
        let r = Poly::<U64>::of(Plan::<P>::R_WORD);
        let folded = if Plan::<P>::FOLD_BY_PRODUCTS && degree == 64 {
            product.clmul_words(1, r, 0)
        } else if Plan::<P>::FOLD_BY_PRODUCTS {
            (product >> degree).clmul_words(0, r, 0)
        } else {
            let high = (product >> degree).resize::<U64>();
            let mut sum = high;
            each(
                Plan::<P>::TAPS.len(),
                #[inline(always)]
                |index| {
                    if let Some(&tap) = Plan::<P>::TAPS.get(index)
                        && tap > 0
                    {
                        sum ^= high << tap;
                    }
                },
            );
            sum.resize()
        };
        (product ^ folded ^ spill).resize()
    }

    /// The overflow of `h r` past `x^n` for `h` below `x^n` in each word of `high`, by one shift per
    /// tap: `n` up to 64.
    #[inline(always)]
    fn overflow<N: clmul::Width>(high: Poly<N>) -> Poly<N> {
        let degree = Plan::<P>::DEGREE;
        let taps = Plan::<P>::TAPS;
        let mut overflow = Poly::<N>::ZERO;
        each(
            taps.len(),
            #[inline(always)]
            |index| {
                if let Some(&tap) = taps.get(index)
                    && tap > 0
                {
                    overflow ^= high.shr_within_words((degree - tap) as u32);
                }
            },
        );
        overflow
    }

    /// `a b mod f` for `n = 128` and `d` below 64, as `a0 b + a1 c` with `c = x^64 b mod f =
    /// b0 x^64 + b1 r`, which one word product gives. The four word products by `a0` and `a1`
    /// sum to `u + w x^64`, and the top word of `w` folds once by its product with `r`: it is below
    /// `x^63`, so the fold leaves no overflow. The products of `w` come first, as the fold waits
    /// on them.
    #[inline(always)]
    fn lane_mul(a: Poly<U128>, b: Poly<U128>) -> Poly<U128> {
        let r = Poly::<U64>::of(Plan::<P>::R_WORD);
        let c = (b << 64) ^ b.clmul_words(1, r, 0);
        let w = a.clmul_words(0, b, 1) ^ a.clmul_words(1, c, 1);
        let folded = w.clmul_words(1, r, 0);
        let u = a.clmul_words(0, b, 0) ^ a.clmul_words(1, c, 0);
        u ^ (w << 64) ^ folded
    }

    /// `p mod f` for `n` up to 64 and a product `p` whose part `h` at and above `x^n` has `h r`
    /// below `x^n`: one shifted copy per tap, without the overflow of a full product. The odd
    /// product of a square root is one when `2d <= n`, as `h` keeps at most `n / 2` bits.
    #[inline(always)]
    fn fold_short(product: Poly<U128>) -> Lanes<P> {
        let degree = Plan::<P>::DEGREE;
        let taps = Plan::<P>::TAPS;
        let high = (product >> degree).resize::<U64>();
        let mut sum = (product.resize::<U64>() & Poly::of(low_mask(degree))) ^ high;
        each(
            taps.len(),
            #[inline(always)]
            |index| {
                if let Some(&tap) = taps.get(index)
                    && tap > 0
                {
                    sum ^= high << tap;
                }
            },
        );
        sum.resize()
    }

    /// `a c mod f` for `n = 128`, `d` below 64 and `a` below `x^64`: two word products give
    /// `low + high x^128` with `high` below `x^63`, whose product with `r` stays below `x^128`, so
    /// one fold ends it.
    #[inline(always)]
    fn word_mul_lane(a: Poly<U128>, c: Poly<U128>) -> Poly<U128> {
        let middle = a.clmul_words(0, c, 1);
        let low = a.clmul_words(0, c, 0) ^ (middle << 64);
        let high = middle >> 64;
        if Plan::<P>::FOLD_BY_PRODUCTS {
            low ^ high.clmul_words(0, Poly::<U64>::of(Plan::<P>::R_WORD), 0)
        } else {
            Self::fold_lane(low, high)
        }
    }

    /// `low + high x^128 mod f` for `n = 128` and `d` below 64, by shifts within words, as
    /// `(high + t) r mod x^128` for the overflow `t` of `high r`: `t r` stays below `x^128`. The
    /// shifts by each tap give the overflow in the high word and the carries out of the low word.
    /// When `d` is at most 32 the overflow, below `x^d`, holds no bit that a shift carries, and one
    /// pass of shifts gives both; above, the carries come from `high + t`.
    #[inline(always)]
    fn fold_lane(low: Poly<U128>, high: Poly<U128>) -> Poly<U128> {
        let taps = Plan::<P>::TAPS;
        let spills = |value: Poly<U128>| {
            let mut spill = Poly::<U128>::ZERO;
            each(
                taps.len(),
                #[inline(always)]
                |index| {
                    if let Some(&tap) = taps.get(index)
                        && tap > 0
                    {
                        spill ^= value.shr_within_words((64 - tap) as u32);
                    }
                },
            );
            spill
        };
        let spill = spills(high);
        let high = high ^ (spill >> 64);
        let carries = if Plan::<P>::R_DEGREE <= 32 {
            spill
        } else {
            spills(high)
        };
        let mut sum = low ^ high ^ (carries << 64);
        each(
            taps.len(),
            #[inline(always)]
            |index| {
                if let Some(&tap) = taps.get(index)
                    && tap > 0
                {
                    sum ^= high.shl_within_words(tap as u32);
                }
            },
        );
        sum
    }

    /// `p mod f` for a product `p` of two polynomials of the field: two rounds of products with
    /// `r` by hardware word products; otherwise one fold of shifted copies with its overflow added
    /// first when `2d <= n`, and rounds of them beyond.
    #[inline(always)]
    fn reduce_lanes(product: Poly<Wide<P>>) -> Lanes<P> {
        let degree = Plan::<P>::DEGREE;
        let mut low = product.resize::<P::Degree>();
        if Plan::<P>::FOLD_BY_PRODUCTS && Plan::<P>::ONE_SPILL {
            // Two products with `r`: the high half's, and its overflow's, below 64 bits.
            let r = Poly::<U64>::of(Plan::<P>::R_WORD);
            let folded = (product >> degree).resize::<P::Degree>().widening_mul(r);
            low ^= folded.resize();
            let spill = (folded >> degree).resize::<U64>().widening_mul(r);
            low ^ spill.resize()
        } else if Plan::<P>::ONE_SPILL {
            low ^ Self::fold_lanes((product >> degree).resize::<P::Degree>())
        } else {
            // Rounds of shifted copies, one per tap.
            let mut rest = product >> degree;
            each(
                Plan::<P>::ROUNDS,
                #[inline(always)]
                |_round| {
                    let mut folded = Poly::<Wide<P>>::ZERO;
                    let taps = Plan::<P>::TAPS;
                    each(
                        taps.len(),
                        #[inline(always)]
                        |index| {
                            if let Some(&tap) = taps.get(index) {
                                folded ^= rest << tap;
                            }
                        },
                    );
                    low ^= folded.resize();
                    rest = folded >> degree;
                },
            );
            low
        }
    }

    /// `h x^n mod f` for `h` below `x^n` and `2d <= n`, by shifts of the whole polynomial, as
    /// `(h + t) r mod x^n` for the overflow `t` of `h r`: `t r` stays below `x^n`. Each shift
    /// works on `n` bits, the width of `h`.
    #[inline(always)]
    fn fold_lanes(high: Lanes<P>) -> Lanes<P> {
        let degree = Plan::<P>::DEGREE;
        let taps = Plan::<P>::TAPS;
        let mut overflow = Lanes::<P>::ZERO;
        each(
            taps.len(),
            #[inline(always)]
            |index| {
                if let Some(&tap) = taps.get(index)
                    && tap > 0
                {
                    overflow ^= high >> (degree - tap);
                }
            },
        );
        let high = high ^ overflow;
        let mut sum = high;
        each(
            taps.len(),
            #[inline(always)]
            |index| {
                if let Some(&tap) = taps.get(index)
                    && tap > 0
                {
                    sum ^= high << tap;
                }
            },
        );
        sum
    }

    /// `sqrt(a) = e(x) + sqrt(x) o(x)` for the even and odd coefficients `e(x^2)` and `x o(x^2)` of
    /// `a`, which the masks and shifts within words gather. Up to 64 bits one gather takes `a` and
    /// `a / x` side by side in the two words of a register, and one word product multiplies `o`.
    /// At 128 bits `o` fills one word: two word products and one fold. Above, `o` keeps
    /// `ceil(n / 2)` bits, and its product takes the registers of those bits alone.
    #[inline(always)]
    pub(crate) fn lanes_sqrt(a: Lanes<P>) -> Lanes<P> {
        if Plan::<P>::SQRT_BY_SPLIT {
            let root_x = Tables::<P>::sqrt_x();
            match Plan::<P>::SHAPE {
                Shape::Word | Shape::Words => {
                    let wide = a.resize::<U128>();
                    let halves = gather_within(wide ^ (wide.shr_within_words(1) << 64));
                    // Word 1 holds `o`, below `x^32`: a comb takes the narrow product when the
                    // width says so.
                    let product = if clmul::HARDWARE {
                        halves.clmul_words(1, root_x, 0)
                    } else {
                        halves.resize::<U96>().clmul_words(1, root_x, 0)
                    };
                    let odd = if Plan::<P>::ONE_SPILL {
                        Self::fold_short(product)
                    } else if matches!(Plan::<P>::SHAPE, Shape::Word) {
                        Self::reduce_word(product.resize::<U64>()).resize()
                    } else {
                        Self::fold_words(product)
                    };
                    halves.resize::<P::Degree>() ^ odd
                }
                Shape::Lane => {
                    let even = gather(a);
                    let odd = gather(a.shr_within_words(1));
                    even ^ Self::word_mul_lane(odd.resize(), root_x.resize()).resize()
                }
                Shape::Lanes => {
                    let even = gather(a);
                    let odd = gather(a.shr_within_words(1)).resize::<Half<P>>();
                    even ^ Self::reduce_lanes(odd.widening_mul(root_x).resize())
                }
            }
        } else {
            let mut power = a;
            each(
                Plan::<P>::DEGREE - 1,
                #[inline(always)]
                |_step| {
                    power = Self::run_square(power);
                },
            );
            power
        }
    }

    // ---------------------------------------------------------------------------------------
    // The word of an element of a field of degree up to 16.

    /// `a b mod f`: one hardware word product, or without one, the integer products of narrow bit
    /// lanes, which take fewer instructions than the comb of 64-bit words.
    #[inline(always)]
    pub(crate) fn word_mul(a: u64, b: u64) -> u64 {
        if clmul::HARDWARE && Plan::<P>::NIBBLE_MAPS {
            // In a vector register: the product, and its fold by two nibble maps.
            let byte = |value: u64| Poly::<U8>::from_uint(Uint::from_u64(value));
            Tables::<P>::fold_byte(byte(a).clmul_words(0, byte(b), 0))
        } else if clmul::HARDWARE {
            Self::reduce_word(clmul::clmul64(a, b) as u64)
        } else if Plan::<P>::DEGREE <= 8 {
            Self::reduce_word(clmul8(a, b))
        } else {
            Self::reduce_word(clmul16(a, b))
        }
    }

    /// `a^2 mod f`: the constant-time linear map of squaring up to degree 8; above, where the map
    /// takes four parity gathers, a carry-less square with hardware word products and the spread
    /// of the bits otherwise.
    #[inline(always)]
    pub(crate) fn word_square(a: u64) -> u64 {
        if Plan::<P>::DEGREE <= 8 {
            Tables::<P>::frobenius(a, 1)
        } else if clmul::HARDWARE {
            Self::reduce_word(clmul::clmul64(a, a) as u64)
        } else {
            Self::reduce_word(spread_half(a))
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

    // ---------------------------------------------------------------------------------------
    // The operations on elements: the defaults, and those of `ConstantTime`.

    /// `a b`, the default.
    #[cfg_attr(debug_assertions, inline(never))]
    #[cfg_attr(not(debug_assertions), inline(always))]
    pub(crate) fn mul(a: Element<P>, b: Element<P>) -> Element<P> {
        if Plan::<P>::MUL_BY_PRODUCTS {
            Element::<P>::from_word(Tables::<P>::mul(a.word(), b.word()))
        } else if Plan::<P>::MUL_BY_SUBFIELD_PRODUCTS {
            Element::<P>::from_word(Tables::<P>::nibble_split_mul(a.word(), b.word()))
        } else if Plan::<P>::MUL_BY_SUBFIELD_LOGARITHMS {
            Element::<P>::from_word(Tables::<P>::composite_mul(a.word(), b.word()))
        } else if Plan::<P>::MUL_BY_LOGARITHMS {
            Element::<P>::from_word(Tables::<P>::log_mul(a.word(), b.word()))
        } else if Plan::<P>::MUL_BY_FOLDS {
            Element::<P>::from_word(Tables::<P>::fold(clmul::clmul64(a.word(), b.word()) as u64))
        } else {
            Self::ct_mul(a, b)
        }
    }

    /// `a b` in constant time.
    #[cfg_attr(debug_assertions, inline(never))]
    #[cfg_attr(not(debug_assertions), inline(always))]
    pub(crate) fn ct_mul(a: Element<P>, b: Element<P>) -> Element<P> {
        if Plan::<P>::SMALL {
            Element::<P>::from_word(Self::word_mul(a.word(), b.word()))
        } else {
            Element::<P>::from_poly(Self::lanes_mul(a.poly(), b.poly()))
        }
    }

    /// `a^2`, the default.
    #[cfg_attr(debug_assertions, inline(never))]
    #[cfg_attr(not(debug_assertions), inline(always))]
    pub(crate) fn square(a: Element<P>) -> Element<P> {
        if Plan::<P>::ELEMENT_TABLES {
            Element::<P>::from_word(Tables::<P>::square(a.word()))
        } else if Plan::<P>::BYTE_MAPS {
            Element::<P>::from_word(Tables::<P>::square_by_bytes(a.word()))
        } else {
            Self::ct_square(a)
        }
    }

    /// `a^2` in constant time.
    #[cfg_attr(debug_assertions, inline(never))]
    #[cfg_attr(not(debug_assertions), inline(always))]
    pub(crate) fn ct_square(a: Element<P>) -> Element<P> {
        if Plan::<P>::SMALL {
            Element::<P>::from_word(Self::word_square(a.word()))
        } else {
            Element::<P>::from_poly(Self::lanes_square(a.poly()))
        }
    }

    /// The square root, the default.
    #[cfg_attr(debug_assertions, inline(never))]
    #[cfg_attr(not(debug_assertions), inline(always))]
    pub(crate) fn sqrt(a: Element<P>) -> Element<P> {
        if Plan::<P>::ELEMENT_TABLES {
            Element::<P>::from_word(Tables::<P>::sqrt(a.word()))
        } else if Plan::<P>::BYTE_MAPS {
            Element::<P>::from_word(Tables::<P>::sqrt_by_bytes(a.word()))
        } else {
            Self::ct_sqrt(a)
        }
    }

    /// The square root in constant time.
    #[cfg_attr(debug_assertions, inline(never))]
    #[cfg_attr(not(debug_assertions), inline(always))]
    pub(crate) fn ct_sqrt(a: Element<P>) -> Element<P> {
        if Plan::<P>::SMALL {
            Element::<P>::from_word(Self::word_sqrt(a.word()))
        } else {
            Element::<P>::from_poly(Self::lanes_sqrt(a.poly()))
        }
    }

    /// The inverse of a nonzero `a`, the default: the table of inverses, or the field's chain with
    /// the tables of its Frobenius powers.
    #[cfg_attr(debug_assertions, inline(never))]
    #[cfg_attr(not(debug_assertions), inline(always))]
    pub(crate) fn invert(a: Element<P>) -> Element<P> {
        if Plan::<P>::ELEMENT_TABLES {
            Element::<P>::from_word(Tables::<P>::invert(a.word()))
        } else if Subfield::<P>::USED {
            Element::<P>::from_poly(Subfield::<P>::invert(a.poly()))
        } else if Plan::<P>::FROBENIUS_WINDOW > 0 {
            Element::<P>::from_poly(inverse::<P, Tabulated<P>>(a.poly()))
        } else {
            Self::ct_invert(a)
        }
    }

    /// The inverse of a nonzero `a` in constant time: `a^(2^n - 2)` along the field's chain, with
    /// squarings or the constant-time maps of the Frobenius powers.
    #[cfg_attr(debug_assertions, inline(never))]
    #[cfg_attr(not(debug_assertions), inline(always))]
    pub(crate) fn ct_invert(a: Element<P>) -> Element<P> {
        if Plan::<P>::SMALL {
            Element::<P>::from_word(inverse::<P, Words<P>>(a.word()))
        } else {
            Element::<P>::from_poly(inverse::<P, Squarings<P>>(a.poly()))
        }
    }
}

/// The inversion of a word of a field up to degree 16 in constant time: the constant-time maps of
/// the Frobenius powers and carry-less products.
struct Words<P>(PhantomData<P>);

impl<P: Polynomial> Inversion for Words<P> {
    type Value = u64;

    #[inline(always)]
    fn frobenius(x: u64, _: usize, power: usize) -> u64 {
        Field::<P>::word_frobenius(x, power)
    }

    #[inline(always)]
    fn mul(a: u64, b: u64) -> u64 {
        Field::<P>::word_mul(a, b)
    }
}

/// The inversion of a polynomial in constant time: squarings or the masked tables of the
/// Frobenius powers, and carry-less products.
struct Squarings<P>(PhantomData<P>);

impl<P: Polynomial> Inversion for Squarings<P> {
    type Value = Lanes<P>;

    #[inline(always)]
    fn frobenius(x: Lanes<P>, step: usize, power: usize) -> Lanes<P> {
        Frobenius::<P>::apply_ct(x, step, power)
    }

    #[inline(always)]
    fn mul(a: Lanes<P>, b: Lanes<P>) -> Lanes<P> {
        Field::<P>::lanes_mul(a, b)
    }
}

/// The default inversion of a polynomial of a field of 17 to 128 bits: the tables of the
/// Frobenius powers of its chain.
struct Tabulated<P>(PhantomData<P>);

impl<P: Polynomial> Inversion for Tabulated<P> {
    type Value = Lanes<P>;

    #[inline(always)]
    fn frobenius(x: Lanes<P>, step: usize, power: usize) -> Lanes<P> {
        Frobenius::<P>::apply(x, step, power)
    }

    #[inline(always)]
    fn mul(a: Lanes<P>, b: Lanes<P>) -> Lanes<P> {
        Field::<P>::lanes_mul(a, b)
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
    let mut x = gather_within(a);
    let half = N::BITS.div_ceil(2);
    each(
        5,
        #[inline(always)]
        |level| {
            let chunk = 32 << level;
            if chunk < half
                && let Some(&mask) = Masks::<N>::BLOCKS.get(level)
            {
                x = (x | (x >> chunk)) & mask;
            }
        },
    );
    x
}

/// The coefficients at the even positions of each word of `a`, moved to half their position
/// within the word.
#[inline(always)]
fn gather_within<N: clmul::Width>(a: Poly<N>) -> Poly<N> {
    let within = Masks::<N>::WITHIN;
    let mut x = a & within[0];
    each(
        5,
        #[inline(always)]
        |level| {
            if let Some(&mask) = within.get(level + 1) {
                x = (x | x.shr_within_words(1u32 << level)) & mask;
            }
        },
    );
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
