//! The compile-time plan of a field: every constant that the defining polynomial determines, and
//! every choice of algorithm, as associated consts of [`Plan<P>`]. Each is a `const fn` of the
//! degree and the taps, so the operations monomorphize to straight-line code per polynomial.

use core::marker::PhantomData;
use core::ops::{Add, Mul};

use biguint::Bits;
use typenum::{U4, Unsigned};

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
/// assert_eq!(x.pow(163u32).as_limbs()[0], 0b1100_1001);
/// ```
pub trait Polynomial {
    /// The degree `n`.
    type Degree: Bits + Add<Self::Degree, Output: Bits> + Mul<U4, Output: Bits>;

    /// The exponents of the terms of `r(x) = f(x) - x^n`: distinct, each below `n`, and including
    /// `0`. Any use of the field asserts these conditions at compile time.
    const TAPS: &'static [usize];
}

/// The width of a carry-less product of two elements, `2n` bits.
pub(crate) type Wide<P> = <<P as Polynomial>::Degree as Add<<P as Polynomial>::Degree>>::Output;

/// The width of the scratch of a Karatsuba product, `4n` bits.
pub(crate) type Scratch<P> = <<P as Polynomial>::Degree as Mul<U4>>::Output;

/// The plan of the field with the polynomial `P`.
pub(crate) struct Plan<P>(PhantomData<P>);

/// The instructions that one 64-bit carry-less product costs, with the moves around it.
const CLMUL_COST: usize = if clmul::HARDWARE { 6 } else { 120 };

/// The instructions that one limb of a shifted XOR costs.
const SHIFTED_XOR_COST: usize = 4;

/// The instructions that one limb of a bit spread costs.
const SPREAD_COST: usize = 24;

/// Degrees up to this have their square root of `x` computed at compile time; the const evaluator
/// refuses longer squaring chains.
const SQRT_CONSTANT_LIMIT: usize = 1024;

impl<P: Polynomial> Plan<P> {
    /// The degree `n`.
    pub(crate) const DEGREE: usize = P::Degree::USIZE;
    /// The limbs of an element.
    pub(crate) const LIMBS: usize = P::Degree::LIMBS;
    /// The taps, validated.
    pub(crate) const TAPS: &'static [usize] = validated(P::TAPS, Self::DEGREE);
    /// The degree of `r`.
    pub(crate) const R_DEGREE: usize = max_tap(Self::TAPS);
    /// `r` as one word, when it fits.
    pub(crate) const R_WORD: u64 = r_word(Self::TAPS);
    /// The number of taps above `0`.
    pub(crate) const SHIFTING_TAPS: usize = Self::TAPS.len() - 1;

    /// The reduction folds the overflow of each product by carry-less products with `r`; otherwise
    /// it shifts and adds the overflow once per tap.
    pub(crate) const FOLD_BY_CLMUL: bool = Self::R_DEGREE < 64
        && Self::DEGREE >= 64
        && reduction_cost(Self::DEGREE, Self::R_DEGREE, true, Self::SHIFTING_TAPS)
            < reduction_cost(Self::DEGREE, Self::R_DEGREE, false, Self::SHIFTING_TAPS);
    /// The number of fold steps of the chosen reduction.
    pub(crate) const STEPS: usize = steps(Self::DEGREE, Self::R_DEGREE, Self::FOLD_BY_CLMUL);
    /// The number of fold steps of the shift-and-add reduction.
    pub(crate) const SHIFT_STEPS: usize = steps(Self::DEGREE, Self::R_DEGREE, false);

    /// Squares are carry-less products of each limb with itself; otherwise bit spreads.
    pub(crate) const SQUARE_BY_CLMUL: bool = clmul::HARDWARE;
    /// Products of this many limbs and more use Karatsuba.
    pub(crate) const KARATSUBA_THRESHOLD: usize = if clmul::HARDWARE { 8 } else { 2 };

    /// The inverse is `a^(2^n - 2)` by Itoh and Tsujii; otherwise the extended Euclidean algorithm
    /// over GF(2)[x].
    pub(crate) const INVERT_BY_ITOH_TSUJII: bool = itoh_tsujii_cost(
        Self::DEGREE,
        Self::LIMBS,
        Self::SQUARE_BY_CLMUL,
        Self::KARATSUBA_THRESHOLD,
        reduction_cost(
            Self::DEGREE,
            Self::R_DEGREE,
            Self::FOLD_BY_CLMUL,
            Self::SHIFTING_TAPS,
        ),
    ) <= euclid_cost(Self::DEGREE, Self::LIMBS);
    /// The bits of `n - 1`, the exponent of the Itoh–Tsujii chain `a^(2^(n-1) - 1)`.
    pub(crate) const CHAIN_BITS: usize =
        usize::BITS as usize - (Self::DEGREE - 1).leading_zeros() as usize;

    /// The square root splits the even and odd coefficients and multiplies by the constant
    /// `sqrt(x)`; otherwise it squares `n - 1` times.
    pub(crate) const SQRT_BY_SPLIT: bool = Self::DEGREE <= SQRT_CONSTANT_LIMIT;

    /// The bits `lo..hi` that the fold step `index` of the chosen reduction moves down by `n`.
    #[inline(always)]
    pub(crate) const fn step(index: usize) -> (usize, usize) {
        step(Self::DEGREE, Self::R_DEGREE, Self::FOLD_BY_CLMUL, index)
    }

    /// The bits `lo..hi` that the fold step `index` of the shift-and-add reduction moves down.
    #[inline(always)]
    pub(crate) const fn shift_step(index: usize) -> (usize, usize) {
        step(Self::DEGREE, Self::R_DEGREE, false, index)
    }
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

/// `r` in one word: the taps at and above 64 are dropped, so only a plan with `R_DEGREE < 64`
/// reads it.
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

/// The fold step `index`: the bits `lo..hi` of the accumulator that it multiplies by `r` and adds
/// at `lo - n`. The steps run from the top of the product down. A chunked plan takes at most 64
/// bits per step, aligned to `n`, so that each step is one word product; a plan that is not
/// chunked takes the whole overflow at once.
const fn step(degree: usize, r_degree: usize, chunked: bool, index: usize) -> (usize, usize) {
    let mut top = 2 * degree - 1;
    let mut current = 0;
    loop {
        assert!(top > degree, "no fold step at this index");
        let lo = step_low(degree, top, chunked);
        if current == index {
            return (lo, top);
        }
        top = next_top(degree, r_degree, lo, top);
        current += 1;
    }
}

/// The low bit of the step that folds the bits below `top`.
const fn step_low(degree: usize, top: usize, chunked: bool) -> usize {
    if chunked && top - degree > 64 {
        degree + 64 * ((top - degree - 1) / 64)
    } else {
        degree
    }
}

/// The top of the accumulator after the step `lo..top`: the bits below `lo` that it left, or the
/// bits that its products with `r` reach.
const fn next_top(degree: usize, r_degree: usize, lo: usize, top: usize) -> usize {
    let landed = top - degree + r_degree;
    if lo > landed { lo } else { landed }
}

/// The number of fold steps of the reduction.
const fn steps(degree: usize, r_degree: usize, chunked: bool) -> usize {
    let mut top = 2 * degree - 1;
    let mut count = 0;
    while top > degree {
        top = next_top(degree, r_degree, step_low(degree, top, chunked), top);
        count += 1;
    }
    count
}

/// The estimated instructions of one reduction.
const fn reduction_cost(
    degree: usize,
    r_degree: usize,
    by_clmul: bool,
    shifting_taps: usize,
) -> usize {
    let mut cost = 0;
    let mut index = 0;
    let count = steps(degree, r_degree, by_clmul);
    while index < count {
        let (lo, hi) = step(degree, r_degree, by_clmul, index);
        let limbs = (hi - lo).div_ceil(64);
        // Extracting and clearing the overflow.
        cost += limbs * SHIFTED_XOR_COST + 2;
        cost += if by_clmul {
            limbs * CLMUL_COST + (limbs + 1) * 2
        } else {
            shifting_taps * (limbs + 1) * SHIFTED_XOR_COST + limbs
        };
        index += 1;
    }
    cost
}

/// The estimated instructions of one product of `limbs` limbs, before reduction.
const fn product_cost(limbs: usize, karatsuba_threshold: usize) -> usize {
    if limbs >= karatsuba_threshold && limbs > 1 {
        let half = limbs.div_ceil(2);
        3 * product_cost(half, karatsuba_threshold) + 8 * half
    } else {
        limbs * limbs * CLMUL_COST + 2 * limbs * limbs
    }
}

/// The estimated instructions of an inversion by Itoh and Tsujii.
const fn itoh_tsujii_cost(
    degree: usize,
    limbs: usize,
    square_by_clmul: bool,
    karatsuba_threshold: usize,
    reduction: usize,
) -> usize {
    let square = if square_by_clmul {
        limbs * CLMUL_COST
    } else {
        limbs * SPREAD_COST
    } + reduction;
    let exponent = degree - 1;
    let multiplications =
        (usize::BITS - exponent.leading_zeros()) as usize + exponent.count_ones() as usize - 2;
    (degree - 2) * square + multiplications * (product_cost(limbs, karatsuba_threshold) + reduction)
}

/// The estimated instructions of an inversion by the extended Euclidean algorithm: about `2n`
/// iterations, each two bit lengths and two shifted additions over the limbs, with a mispredicted
/// branch.
const fn euclid_cost(degree: usize, limbs: usize) -> usize {
    2 * degree * ((limbs + 1) * (2 * SHIFTED_XOR_COST + 3) + 12)
}
