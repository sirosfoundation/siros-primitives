//! The compile-time plan of a field: every constant that the defining polynomial determines, and
//! every choice of algorithm, as associated consts of [`Plan<P>`]. Each is a `const fn` of the
//! degree and the taps, so the operations monomorphize to straight-line code per polynomial.

use core::marker::PhantomData;
use core::ops::{Add, Div};

use clmul::Width;
use typenum::{Quot, Sum, U1, U2, U64, Unsigned};

use crate::representation::Representation;

/// The defining polynomial `f(x) = x^n + r(x)` of a field `GF(2^n) = GF(2)[x] / (f(x))`.
///
/// `f` must be irreducible over GF(2); the crate does not check that. `Degree` is `n` as a
/// `typenum` unsigned integer, and [`TAPS`](Self::TAPS) lists the exponents of the terms of `r`.
///
/// ```
/// use binary_field::typenum::U163;
/// use binary_field::{Gf2, Polynomial, Pow};
///
/// /// `x^163 + x^7 + x^6 + x^3 + 1`, the polynomial of the NIST curve B-163.
/// pub struct B163;
///
/// impl Polynomial for B163 {
///     type Degree = U163;
///     const TAPS: &'static [usize] = &[7, 6, 3, 0];
/// }
///
/// pub type Gf2p163 = Gf2<B163>;
/// let x = Gf2p163::from_u64(2).unwrap();
/// assert_eq!(x.pow(163u32).into_limbs()[0], 0b1100_1001);
/// ```
pub trait Polynomial {
    /// The degree `n`.
    type Degree: Representation + Halve + Add<Self::Degree, Output: Width> + Add<U64, Output: Width>;

    /// The exponents of the terms of `r(x) = f(x) - x^n`: distinct, each below `n`, and including
    /// `0`. Any use of the field asserts these conditions at compile time.
    const TAPS: &'static [usize];
}

/// The width of a carry-less product of two elements, `2n` bits.
pub(crate) type Wide<P> = Sum<<P as Polynomial>::Degree, <P as Polynomial>::Degree>;

/// A degree `n` with the width `ceil(n / 2)` of the odd half of a square root, and the width of its
/// product with an element. Every `typenum` unsigned integer that is a [`Width`] implements it.
pub trait Halve: Width {
    /// `ceil(n / 2)`.
    type Half: Width + Add<Self, Output: Width>;
}

impl<N> Halve for N
where
    N: Width + Add<U1>,
    Sum<N, U1>: Div<U2>,
    Quot<Sum<N, U1>, U2>: Width + Add<N, Output: Width>,
{
    type Half = Quot<Sum<N, U1>, U2>;
}

/// The odd half of a square root of the field with the polynomial `P`, `ceil(n / 2)` bits.
pub(crate) type Half<P> = <<P as Polynomial>::Degree as Halve>::Half;

/// The plan of the field with the polynomial `P`.
pub(crate) struct Plan<P>(PhantomData<P>);

/// How a product folds back below `x^n`.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Shape {
    /// The product fits one 64-bit word: `n` up to 32.
    Word,
    /// Two words, folded by shifts per tap with the overflow added first: `n` from 33 to 64.
    Words,
    /// One register for `n = 128`: with hardware word products and `d` below 64,
    /// `a0 b + a1 (x^64 b mod f)` by six word products and one fold; otherwise Karatsuba's three
    /// products and folds by shifts.
    Lane,
    /// Any other degree: a whole product, then rounds of folds.
    Lanes,
}

/// Degrees up to this have their square root of `x` computed at compile time; the const evaluator
/// refuses longer squaring chains.
const SQRT_CONSTANT_LIMIT: usize = 1024;

/// The lookup tables of the build.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Tier {
    /// `not-precomputed`: no table; every operation computes.
    None,
    /// The default: at most [`BUDGET`] bytes of tables per field.
    Budget,
    /// `precomputed`: the fastest tables, of any size.
    Full,
}

/// The tables of the build. `not-precomputed` wins over `precomputed`: Cargo unifies the features
/// of a build, so either feature set by any dependent switches the whole build.
pub(crate) const TIER: Tier = if cfg!(feature = "not-precomputed") {
    Tier::None
} else if cfg!(feature = "precomputed") {
    Tier::Full
} else {
    Tier::Budget
};

/// Whether `clmul`'s nibble maps take one byte shuffle: PSHUFB with SSSE3, TBL on the PMULL
/// backend of `aarch64`. Elsewhere a map takes masks per bit, slower than the parity gathers on a
/// general register.
const BYTE_SHUFFLE: bool = cfg!(any(
    all(target_arch = "x86_64", target_feature = "ssse3"),
    all(
        target_arch = "aarch64",
        target_endian = "little",
        target_feature = "aes"
    )
));

/// The largest degree whose inverse applies tables of Frobenius powers. On WebAssembly a squaring
/// of 129 to 256 bits takes about 58 cycles, and the 254 squarings of the inverse of GF(2^256)
/// took Chrome two thirds of its 23,000 cycles; tables of four words per image replace them.
/// Elsewhere 128.
pub(crate) const TABLE_DEGREE: usize = if cfg!(target_arch = "wasm32") {
    256
} else {
    128
};

/// The bytes of constants a field may hold without `precomputed`.
pub(crate) const BUDGET: usize = 16 * 1024;

/// The bytes of the Frobenius tables of a field without `precomputed`: the budget less 1 KiB for
/// the field's other constants.
pub(crate) const TABLE_BUDGET: usize = BUDGET - 1024;

impl<P: Polynomial> Plan<P> {
    /// The degree `n`.
    pub(crate) const DEGREE: usize = P::Degree::USIZE;
    /// The limbs of an element.
    pub(crate) const LIMBS: usize = <P::Degree as biguint::Bits>::LIMBS;
    /// The taps, validated.
    pub(crate) const TAPS: &'static [usize] = validated(P::TAPS, Self::DEGREE);
    /// The degree `d` of `r`.
    pub(crate) const R_DEGREE: usize = max_tap(Self::TAPS);
    /// `r` as one word, when `d` is below 64.
    pub(crate) const R_WORD: u64 = r_word(Self::TAPS);

    /// The elements live in a general register: a byte up to degree 8, a 16-bit word up to 16.
    pub(crate) const SMALL: bool = Self::DEGREE <= 16;
    /// The default product indexes the table of every product: `precomputed`, degree up to 8.
    pub(crate) const MUL_BY_PRODUCTS: bool = matches!(TIER, Tier::Full) && Self::DEGREE <= 8;
    /// The default product adds logarithms: the default tables up to degree 8, and `precomputed`
    /// from degree 9 to 16 without hardware word products.
    pub(crate) const MUL_BY_LOGARITHMS: bool = match TIER {
        Tier::Budget => Self::DEGREE <= 8,
        Tier::Full => Self::DEGREE > 8 && Self::SMALL && !clmul::HARDWARE,
        Tier::None => false,
    };
    /// The default product of degrees 9 to 16 with hardware word products folds its part at and
    /// above `x^n` by a table of each of its two bytes, 1 KiB in all.
    pub(crate) const MUL_BY_FOLDS: bool =
        !matches!(TIER, Tier::None) && Self::DEGREE > 8 && Self::SMALL && clmul::HARDWARE;
    /// The default square, square root and inverse index a table of every element: `precomputed`
    /// up to degree 16, the default tables up to degree 8.
    pub(crate) const ELEMENT_TABLES: bool = match TIER {
        Tier::Full => Self::SMALL,
        Tier::Budget => Self::DEGREE <= 8,
        Tier::None => false,
    };
    /// The default square and square root of degrees 9 to 16 add the images of the two bytes of
    /// the element: the default tables.
    pub(crate) const BYTE_MAPS: bool =
        matches!(TIER, Tier::Budget) && Self::SMALL && Self::DEGREE > 8;
    /// The window in bits of the tables of the Frobenius powers of the default inverse of a field
    /// of 17 to [`TABLE_DEGREE`] bits: bytes with `precomputed` up to 128 bits and nibbles above,
    /// nibbles by default up to 128 bits, none without tables.
    pub(crate) const FROBENIUS_WINDOW: usize = if Self::DEGREE <= 16 || Self::DEGREE > TABLE_DEGREE
    {
        0
    } else {
        match TIER {
            Tier::Full if Self::DEGREE > 128 => 4,
            Tier::Full => 8,
            Tier::Budget if Self::DEGREE > 128 => 0,
            Tier::Budget => 4,
            Tier::None => 0,
        }
    };

    /// The folds by `r` multiply it carry-lessly: the word products are hardware, and `d` is below
    /// 64. Otherwise each fold adds one shifted copy per tap.
    pub(crate) const FOLD_BY_PRODUCTS: bool = clmul::HARDWARE && Self::R_DEGREE < 64;
    /// The overflow of `h r` for `h` below `x^n` folds once without spilling again: `2d <= n`.
    pub(crate) const ONE_SPILL: bool = 2 * Self::R_DEGREE <= Self::DEGREE;
    /// The rounds of folds that bring a product of two elements below `x^n`.
    pub(crate) const ROUNDS: usize = rounds(Self::DEGREE, Self::R_DEGREE);
    /// How a product folds back.
    pub(crate) const SHAPE: Shape = shape(Self::DEGREE, Self::R_DEGREE, Self::ONE_SPILL);

    /// The constant-time linear maps of a field up to degree 8 (its squares, square roots and
    /// Frobenius powers) and the fold of its product with hardware word products are nibble maps
    /// in a vector register: the build has a byte shuffle, PSHUFB or TBL.
    pub(crate) const NIBBLE_MAPS: bool = BYTE_SHUFFLE && Self::DEGREE <= 8;

    /// A product of degree 33 to 64 folds its part `h` at and above `x^n` as `(h r mod x^n) + t r`,
    /// with `t r` the image of the top nibble of `h` under a nibble map: `d` is at most 4, so only
    /// that nibble overflows past `x^n`.
    pub(crate) const SPILL_BY_NIBBLES: bool =
        matches!(Self::SHAPE, Shape::Words) && Self::R_DEGREE <= 4;

    /// The bits of `n - 1`, the exponent of the Itoh–Tsujii chain `a^(2^(n-1) - 1)`.
    pub(crate) const CHAIN_BITS: usize =
        usize::BITS as usize - (Self::DEGREE - 1).leading_zeros() as usize;

    /// The square root splits the even and odd coefficients and multiplies by the constant
    /// `sqrt(x)`; otherwise it squares `n - 1` times.
    pub(crate) const SQRT_BY_SPLIT: bool = Self::DEGREE <= SQRT_CONSTANT_LIMIT;

    /// The square root of `x^128 + x^7 + x^2 + x + 1` multiplies by `sqrt(x)` through shifts, on
    /// WebAssembly, whose two word products take 96 integer products: Chrome 41.0 cycles against
    /// 154.4.
    pub(crate) const SQRT_BY_SHIFTS: bool = cfg!(target_arch = "wasm32")
        && Self::DEGREE == 128
        && Self::R_DEGREE == 7
        && Self::R_WORD == 0x87;
}

/// The fold shape of a degree.
const fn shape(degree: usize, r_degree: usize, one_spill: bool) -> Shape {
    if degree <= 32 && one_spill {
        Shape::Word
    } else if degree > 32 && degree <= 64 && one_spill {
        Shape::Words
    } else if degree == 128 && r_degree < 64 {
        Shape::Lane
    } else {
        Shape::Lanes
    }
}

/// The rounds of folds of a product of two elements: each round moves the bits at and above
/// `x^n` down by `n` times `r`, which puts its top at `top - n + d`.
const fn rounds(degree: usize, r_degree: usize) -> usize {
    let mut top = 2 * degree - 2;
    let mut count = 0;
    while top >= degree {
        top = top - degree + r_degree;
        count += 1;
    }
    count
}

/// `taps`, after asserting that they are distinct, below `degree`, and include `0`.
const fn validated(taps: &'static [usize], degree: usize) -> &'static [usize] {
    assert!(degree >= 2, "a field needs a degree of at least 2");
    let mut has_constant = false;
    let mut rest = taps;
    while let [tap, tail @ ..] = rest {
        assert!(*tap < degree, "every tap must be below the degree");
        has_constant |= *tap == 0;
        let mut others = tail;
        while let [other, others_tail @ ..] = others {
            assert!(*other != *tap, "the taps must be distinct");
            others = others_tail;
        }
        rest = tail;
    }
    assert!(
        has_constant,
        "an irreducible polynomial has the constant term 1: the taps must include 0"
    );
    taps
}

const fn max_tap(taps: &[usize]) -> usize {
    let mut max = 0;
    let mut rest = taps;
    while let [tap, tail @ ..] = rest {
        if *tap > max {
            max = *tap;
        }
        rest = tail;
    }
    max
}

/// `r` in one word: the taps at and above 64 are dropped, so only a plan with `d < 64` reads it.
const fn r_word(taps: &[usize]) -> u64 {
    let mut word = 0;
    let mut rest = taps;
    while let [tap, tail @ ..] = rest {
        if *tap < 64 {
            word |= 1 << *tap;
        }
        rest = tail;
    }
    word
}
