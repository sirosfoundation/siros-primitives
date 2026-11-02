//! The constants of a field that its operations read: `sqrt(x)`, and the tables of the default
//! operations of the fields up to degree 16, which const evaluation generates on first use.

use core::marker::PhantomData;

use biguint::Uint;
use clmul::Poly;
use const_default::ConstDefault;
use generic_array::GenericArray;

use crate::arith::Lanes;
use crate::element::Limbs;
use crate::kernel::{copy, head, set_bit, spread};
use crate::plan::{Plan, Polynomial, Wide};
use crate::reduce::const_reduce;

/// The entries of a table indexed by an element of a field of degree up to 16.
const ELEMENTS: usize = 1 << 16;

/// The constants and tables of the field with the polynomial `P`.
pub(crate) struct Tables<P>(PhantomData<P>);

impl<P: Polynomial> Tables<P> {
    /// `sqrt(x) = x^(2^(n-1))`.
    const SQRT_X: Poly<P::Degree> = Poly::from_uint(Uint::from_generic_limbs(sqrt_x::<P>()));
    /// `sqrt(x)` in one word, for degrees up to 64.
    pub(crate) const SQRT_X_WORD: u64 = word(sqrt_x::<P>().as_slice());

    /// `sqrt(x)`.
    #[inline(always)]
    pub(crate) fn sqrt_x() -> Lanes<P> {
        Self::SQRT_X
    }

    /// The product of every pair of elements of a field of degree up to 8, `a b` at `256 b + a`.
    const PRODUCTS: &'static [u8; ELEMENTS] = &products(Plan::<P>::DEGREE, Plan::<P>::TAPS);

    /// The squares of the bytes, and of the bytes times `x^8`.
    const SQUARES: &'static [[u16; 256]; 2] =
        &linear_tables(Plan::<P>::DEGREE, Plan::<P>::TAPS, false);
    /// The square roots of the bytes, and of the bytes times `x^8`.
    const ROOTS: &'static [[u16; 256]; 2] =
        &linear_tables(Plan::<P>::DEGREE, Plan::<P>::TAPS, true);
    /// The inverse of every element, `0` for `0`.
    const INVERSES: &'static [u16; ELEMENTS] = &inverses(Plan::<P>::DEGREE, Plan::<P>::TAPS);

    /// `a b` by the table of products, for degrees up to 8: the index `256 b + a` takes `b` into
    /// the row, so a factor that a loop keeps fixes the row.
    #[inline(always)]
    pub(crate) fn mul(a: u64, b: u64) -> u64 {
        Self::PRODUCTS
            .get(((b << 8) | a) as usize)
            .copied()
            .map_or(0, u64::from)
    }

    /// `a^2` by the table of squares: one lookup per byte.
    #[inline(always)]
    pub(crate) fn square(a: u64) -> u64 {
        linear(Self::SQUARES, a)
    }

    /// The square root by the table of square roots: one lookup per byte.
    #[inline(always)]
    pub(crate) fn sqrt(a: u64) -> u64 {
        linear(Self::ROOTS, a)
    }

    /// The inverse, `0` for `0`, by the table of inverses.
    #[inline(always)]
    pub(crate) fn invert(a: u64) -> u64 {
        Self::INVERSES.get(a as usize).copied().map_or(0, u64::from)
    }
}

/// The image of `a` under a linear map given by the images of the bytes, of the low byte alone
/// for degrees up to 8.
#[inline(always)]
fn linear(tables: &'static [[u16; 256]; 2], a: u64) -> u64 {
    let [low, high] = tables;
    let image = |table: &'static [u16; 256], byte: u64| -> u64 {
        table.get(byte as usize).copied().map_or(0, u64::from)
    };
    if a >> 8 == 0 {
        image(low, a)
    } else {
        image(low, a & 0xFF) ^ image(high, a >> 8)
    }
}

/// `sqrt(x) = x^(2^(n-1))`, by `n - 1` squarings, each a bit spread and a shift-and-add
/// reduction. Degrees above the plan's limit skip it, and their square roots square at runtime.
const fn sqrt_x<P: Polynomial>() -> GenericArray<u64, Limbs<P>> {
    let mut power = GenericArray::<u64, Limbs<P>>::DEFAULT;
    if !Plan::<P>::SQRT_BY_SPLIT {
        return power;
    }
    set_bit(power.as_mut_slice(), 1);
    let mut wide = GenericArray::<u64, <Wide<P> as biguint::Bits>::Limbs>::DEFAULT;
    let mut scratch = GenericArray::<u64, <Wide<P> as biguint::Bits>::Limbs>::DEFAULT;
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

/// The low limb of `limbs`.
const fn word(limbs: &[u64]) -> u64 {
    match limbs {
        [low, ..] => *low,
        [] => 0,
    }
}

/// `a b` in a field of degree up to 16, bit by bit: the multiple of `a` doubles and reduces once
/// per bit of `b`.
const fn small_mul(a: u64, b: u64, degree: usize, taps: &[usize]) -> u64 {
    let r = r_bits(taps);
    let top = 1 << (degree - 1);
    let mask = (1 << degree) - 1;
    let (mut multiple, mut product, mut bit) = (a, 0, 0);
    while bit < degree {
        if (b >> bit) & 1 == 1 {
            product ^= multiple;
        }
        multiple = if multiple & top == 0 {
            (multiple << 1) & mask
        } else {
            ((multiple << 1) & mask) ^ r
        };
        bit += 1;
    }
    product
}

/// `r` as a word, for degrees up to 16.
const fn r_bits(taps: &[usize]) -> u64 {
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

/// `a^exponent` in a field of degree up to 16.
const fn small_pow(a: u64, exponent: u64, degree: usize, taps: &[usize]) -> u64 {
    let (mut result, mut base, mut rest) = (1, a, exponent);
    while rest > 0 {
        if rest & 1 == 1 {
            result = small_mul(result, base, degree, taps);
        }
        base = small_mul(base, base, degree, taps);
        rest >>= 1;
    }
    result
}

/// The product of every pair of elements of a field of degree up to 8, `a b` at `256 b + a`, by
/// the logarithms along the powers of a generator `g`: `a b = g^((log a + log b) mod q)`. Empty
/// above degree 8.
#[expect(
    clippy::indexing_slicing,
    reason = "const evaluation, with every index below its table's length"
)]
const fn products(degree: usize, taps: &[usize]) -> [u8; ELEMENTS] {
    let mut table = [0; ELEMENTS];
    if degree > 8 {
        return table;
    }
    let order = (1 << degree) - 1;
    let generator = generator(degree, taps);
    let (mut log, mut exp) = ([0usize; 256], [0u8; 256]);
    let mut power = 1;
    let mut index = 0;
    while index < order {
        log[power as usize] = index;
        exp[index] = power as u8;
        power = small_mul(power, generator, degree, taps);
        index += 1;
    }
    let mut a = 1;
    while a <= order {
        let mut b = 1;
        while b <= order {
            table[(b << 8) | a] = exp[(log[a] + log[b]) % order];
            b += 1;
        }
        a += 1;
    }
    table
}

/// A generator of the multiplicative group of a field of degree up to 16: the first element
/// whose powers by `q / p` differ from 1 for every prime `p` dividing `q = 2^n - 1`.
const fn generator(degree: usize, taps: &[usize]) -> u64 {
    let order = (1u64 << degree) - 1;
    let mut candidate = 2;
    while candidate <= order {
        let mut generates = true;
        let mut rest = order;
        let mut prime = 2;
        while rest > 1 {
            if rest.is_multiple_of(prime) {
                if small_pow(candidate, order / prime, degree, taps) == 1 {
                    generates = false;
                }
                while rest.is_multiple_of(prime) {
                    rest /= prime;
                }
            }
            prime += 1;
        }
        if generates {
            return candidate;
        }
        candidate += 1;
    }
    1
}

/// The inverse of every element of a field of degree up to 16, `0` for `0`: `g^(q - i)` for
/// `g^i`, along the powers of a generator `g`. Empty above degree 16.
#[expect(
    clippy::indexing_slicing,
    reason = "const evaluation, with every index below its table's length"
)]
const fn inverses(degree: usize, taps: &[usize]) -> [u16; ELEMENTS] {
    let mut table = [0; ELEMENTS];
    if degree > 16 {
        return table;
    }
    let order = (1 << degree) - 1;
    let generator = generator(degree, taps);
    let mut powers = [0u16; ELEMENTS];
    let mut power = 1;
    let mut index = 0;
    while index < order {
        powers[index] = power as u16;
        power = small_mul(power, generator, degree, taps);
        index += 1;
    }
    index = 0;
    while index < order {
        table[powers[index] as usize] = powers[(order - index) % order];
        index += 1;
    }
    table
}

/// The tables of a linear map of a field of degree up to 16: the images of the bytes, and of the
/// bytes times `x^8`. The map is squaring, or the square root when `root` holds.
#[expect(
    clippy::indexing_slicing,
    reason = "const evaluation, with every index below its table's length"
)]
const fn linear_tables(degree: usize, taps: &[usize], root: bool) -> [[u16; 256]; 2] {
    let mut tables = [[0; 256]; 2];
    if degree > 16 {
        return tables;
    }
    // The image of each basis element `x^i`: its square, or its root `x^(i 2^(n-1))`.
    let mut images = [0u64; 16];
    let mut index = 0;
    while index < degree {
        let basis = 1 << index;
        images[index] = if root {
            let mut power = basis;
            let mut step = 1;
            while step < degree {
                power = small_mul(power, power, degree, taps);
                step += 1;
            }
            power
        } else {
            small_mul(basis, basis, degree, taps)
        };
        index += 1;
    }
    let mut half = 0;
    while half < 2 {
        let mut byte: usize = 1;
        while byte < 256 {
            let low = byte & byte.wrapping_neg();
            let bit = low.trailing_zeros() as usize + 8 * half;
            let image = if bit < degree { images[bit] } else { 0 };
            tables[half][byte] = tables[half][byte ^ low] ^ image as u16;
            byte += 1;
        }
        half += 1;
    }
    tables
}
