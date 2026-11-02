//! Tables of the Frobenius powers `a -> a^(2^k)` of the fields of one register, one per doubling of
//! the Itoh–Tsujii chain, behind `precomputed`: a power applies as one lookup per byte, in place of
//! `k` squarings. The lookups index the tables by the value, so the default inverse takes them and
//! `Secret` does not.

use core::marker::PhantomData;

use biguint::Uint;
use clmul::Poly;
use const_default::ConstDefault;
use generic_array::GenericArray;

use crate::element::Limbs;
use crate::plan::{Plan, Polynomial};

/// The byte positions of a table: the bytes of a register.
const POSITIONS: usize = 16;

/// The steps of a chain of a degree up to 128.
const STEPS: usize = 7;

/// The image of every byte value at every byte position of an element, as two words.
type Table = [[[u64; 2]; 256]; POSITIONS];

/// The Itoh–Tsujii chain of the field with the polynomial `P`.
struct Chain<P>(PhantomData<P>);

impl<P: Polynomial> Chain<P> {
    /// For each step that takes a table, the images of the basis elements `x^j` under the step's
    /// power, from one pass of squarings of all of them.
    const IMAGES: &'static [[u128; 128]; STEPS] = &images(
        Plan::<P>::INVERT_BY_TABLES,
        Plan::<P>::DEGREE,
        Plan::<P>::TAPS,
    );
}

/// The images of `x^j` under `a -> a^(2^k)` for the `k` of each tabulated step of the chain of the
/// field of degree `degree`, up to 128, with `taps`: all the basis elements square together, and
/// the images at each step's `k` are kept. Zero when `tabulated` is false.
#[expect(
    clippy::indexing_slicing,
    reason = "const evaluation, with every index below its array's length"
)]
const fn images(tabulated: bool, degree: usize, taps: &[usize]) -> [[u128; 128]; STEPS] {
    let mut kept = [[0; 128]; STEPS];
    if !tabulated || degree > 128 {
        return kept;
    }
    let mut powers = [0u128; 128];
    let mut bit = 0;
    while bit < degree {
        powers[bit] = 1 << bit;
        bit += 1;
    }
    let bits = usize::BITS as usize - (degree - 1).leading_zeros() as usize;
    let mut step = 0;
    let mut k = 0;
    while step + 1 < bits && step < STEPS {
        let target = chain_k(degree, step);
        while k < target {
            bit = 0;
            while bit < degree {
                powers[bit] = square_mod(powers[bit], degree, taps);
                bit += 1;
            }
            k += 1;
        }
        kept[step] = powers;
        step += 1;
    }
    kept
}

/// `a^2 mod x^degree + r` for `a` below `x^degree`, a degree up to 128: the square spreads the
/// bits, and the part at and above `x^degree` folds down by `r` until none is left.
const fn square_mod(a: u128, degree: usize, taps: &[usize]) -> u128 {
    let (mut low, mut high) = (spread(a as u64), spread((a >> 64) as u64));
    loop {
        let (top_low, top_high) = shift_right(low, high, degree);
        if top_low == 0 && top_high == 0 {
            return low;
        }
        if degree < 128 {
            low &= (1 << degree) - 1;
        }
        high = 0;
        let mut rest = taps;
        while let [tap, tail @ ..] = rest {
            let (add_low, add_high) = shift_left(top_low, top_high, *tap);
            low ^= add_low;
            high ^= add_high;
            rest = tail;
        }
    }
}

/// Bit `i` of `word` at bit `2i`.
const fn spread(word: u64) -> u128 {
    let mut value = word as u128;
    value = (value | (value << 32)) & 0x0000_0000_FFFF_FFFF_0000_0000_FFFF_FFFF;
    value = (value | (value << 16)) & 0x0000_FFFF_0000_FFFF_0000_FFFF_0000_FFFF;
    value = (value | (value << 8)) & 0x00FF_00FF_00FF_00FF_00FF_00FF_00FF_00FF;
    value = (value | (value << 4)) & 0x0F0F_0F0F_0F0F_0F0F_0F0F_0F0F_0F0F_0F0F;
    value = (value | (value << 2)) & 0x3333_3333_3333_3333_3333_3333_3333_3333;
    (value | (value << 1)) & 0x5555_5555_5555_5555_5555_5555_5555_5555
}

/// `(low + high 2^128) >> shift` for `shift` up to 128.
const fn shift_right(low: u128, high: u128, shift: usize) -> (u128, u128) {
    if shift == 0 {
        (low, high)
    } else if shift < 128 {
        ((low >> shift) | (high << (128 - shift)), high >> shift)
    } else {
        (high, 0)
    }
}

/// `(low + high 2^128) << shift`, for `shift` below 128.
const fn shift_left(low: u128, high: u128, shift: usize) -> (u128, u128) {
    if shift == 0 {
        (low, high)
    } else {
        (low << shift, (high << shift) | (low >> (128 - shift)))
    }
}

/// The doubling at step `S` of the Itoh–Tsujii chain of the field with the polynomial `P`.
pub(crate) struct ChainStep<P, const S: usize>(PhantomData<P>);

impl<P: Polynomial, const S: usize> ChainStep<P, S> {
    /// The step doubles `k` to `2k`: `x^(2^k - 1)` becomes `x^(2^(2k) - 1)` through `x^(2^k)`.
    pub(crate) const K: usize = chain_k(Plan::<P>::DEGREE, S);
    /// Whether the step exists and its power takes a table: `k` from 3 on, where one lookup per byte
    /// costs less than `k` squarings.
    pub(crate) const TABULATED: bool =
        Plan::<P>::INVERT_BY_TABLES && S + 1 < Plan::<P>::CHAIN_BITS && Self::K >= 3;
    /// The table of `a -> a^(2^k)`, empty unless the step is tabulated.
    const TABLE: &'static Table = &table(Self::TABULATED, Plan::<P>::DEGREE, Chain::<P>::IMAGES, S);

    /// `a^(2^k)`: the images of the bytes of `a`, summed.
    #[inline(always)]
    pub(crate) fn apply(a: Poly<P::Degree>) -> Poly<P::Degree> {
        let limbs = a.to_uint().into_limbs();
        let mut sums = [Poly::<P::Degree>::ZERO; 4];
        macro_rules! positions {
            ($($position:literal)*) => {$(
                if $position < Plan::<P>::DEGREE.div_ceil(8) {
                    let limb = limbs.get($position / 8).copied().unwrap_or(0);
                    let byte = ((limb >> (8 * ($position % 8))) & 0xFF) as usize;
                    if let Some(row) = Self::TABLE.get($position)
                        && let Some(&words) = row.get(byte)
                        && let Some(sum) = sums.get_mut($position % 4)
                    {
                        *sum ^= polynomial::<P>(words);
                    }
                }
            )*};
        }
        positions!(0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15);
        let [first, second, third, fourth] = sums;
        (first ^ second) ^ (third ^ fourth)
    }
}

/// The `k` at step `step` of the Itoh–Tsujii chain of degree `degree`: it starts at 1, and each
/// step doubles it and adds the next bit of `n - 1`, from the top.
const fn chain_k(degree: usize, step: usize) -> usize {
    let exponent = degree - 1;
    let bits = usize::BITS as usize - exponent.leading_zeros() as usize;
    let mut k = 1;
    let mut index = 0;
    while index < step && index + 1 < bits {
        k = 2 * k + ((exponent >> (bits - 2 - index)) & 1);
        index += 1;
    }
    k
}

/// The table of a power when `tabulated`, from the images of the basis elements `x^j`: their
/// sums per byte value. Zero otherwise.
#[expect(
    clippy::indexing_slicing,
    reason = "const evaluation, with every index below its array's length"
)]
const fn table(
    tabulated: bool,
    degree: usize,
    images: &[[u128; 128]; STEPS],
    step: usize,
) -> Table {
    let mut table = [[[0u64; 2]; 256]; POSITIONS];
    if !tabulated || step >= STEPS {
        return table;
    }
    let images = &images[step];
    // Each byte value is a smaller one plus its lowest bit.
    let mut position = 0;
    while position < POSITIONS && 8 * position < degree {
        let mut value: usize = 1;
        while value < 256 {
            let low = value & value.wrapping_neg();
            let bit = 8 * position + low.trailing_zeros() as usize;
            let image = if bit < degree { images[bit] } else { 0 };
            let rest = table[position][value ^ low];
            table[position][value] = [rest[0] ^ image as u64, rest[1] ^ (image >> 64) as u64];
            value += 1;
        }
        position += 1;
    }
    table
}

/// The polynomial of the field with the polynomial `P` whose low words are `words`.
#[inline(always)]
fn polynomial<P: Polynomial>(words: [u64; 2]) -> Poly<P::Degree> {
    let mut limbs = GenericArray::<u64, Limbs<P>>::DEFAULT;
    for (slot, word) in limbs.iter_mut().zip(words) {
        *slot = word;
    }
    Poly::from_uint(Uint::from_generic_limbs(limbs))
}
