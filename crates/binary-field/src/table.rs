//! The constants of a field that its operations read: `sqrt(x)`, the constant-time maps of the
//! Frobenius powers of the fields up to degree 16, and the lookup tables of their default
//! operations, which const evaluation generates on first use. Each table has one entry per
//! element, in the narrowest integer that holds one; the plan says which a build carries.

use core::marker::PhantomData;

use biguint::Uint;
use clmul::Poly;
use const_default::ConstDefault;
use generic_array::GenericArray;

use crate::arith::Lanes;
use crate::element::Limbs;
use crate::kernel::{copy, head, set_bit, spread};
use crate::linear::{Blocks, apply, blocks};
use crate::plan::{Plan, Polynomial, Wide};
use crate::reduce::const_reduce;

/// The entries of a table indexed by an element of a field of degree up to 8.
const BYTES: usize = 1 << 8;
/// The entries of a table indexed by an element of a field of degree up to 16, or by two elements
/// of a field of degree up to 8.
const WORDS: usize = 1 << 16;

/// A map of 16 bits to 16 bits by bytes: the images of the low byte, then of the high byte.
type ByteMap = [[u16; BYTES]; 2];

/// The constants and tables of the field with the polynomial `P`.
pub(crate) struct Tables<P>(PhantomData<P>);

impl<P: Polynomial> Tables<P> {
    /// `sqrt(x) = x^(2^(n-1))`.
    const SQRT_X: Poly<P::Degree> = Poly::from_uint(Uint::from_generic_limbs(sqrt_x::<P>()));

    /// `sqrt(x)`.
    #[inline(always)]
    pub(crate) fn sqrt_x() -> Lanes<P> {
        Self::SQRT_X
    }

    /// Whether the element tables hold bytes.
    const BYTE_ELEMENTS: bool = Plan::<P>::ELEMENT_TABLES && Plan::<P>::DEGREE <= 8;
    /// Whether the element tables hold 16-bit words.
    const WORD_ELEMENTS: bool = Plan::<P>::ELEMENT_TABLES && Plan::<P>::DEGREE > 8;

    /// The product of every pair of elements of a field of degree up to 8, `a b` at `256 b + a`.
    const PRODUCTS: &'static [u8; WORDS] = &products(
        Plan::<P>::MUL_BY_PRODUCTS,
        Plan::<P>::DEGREE,
        Plan::<P>::TAPS,
    );
    /// The logarithm of every nonzero element of a field of degree up to 8 to a generator.
    const LOGARITHMS: &'static [u8; BYTES] = &logarithms(
        Plan::<P>::MUL_BY_LOGARITHMS,
        Plan::<P>::DEGREE,
        Plan::<P>::TAPS,
    );
    /// The powers of the generator, twice over, so that a sum of two logarithms indexes it.
    const EXPONENTIALS: &'static [u8; 2 * BYTES] = &exponentials(
        Plan::<P>::MUL_BY_LOGARITHMS,
        Plan::<P>::DEGREE,
        Plan::<P>::TAPS,
    );

    /// The square of every element, for degrees up to 8.
    const BYTE_SQUARES: &'static [u8; BYTES] = &narrowed(&linear_table(
        Self::BYTE_ELEMENTS,
        Plan::<P>::DEGREE,
        Plan::<P>::TAPS,
        1,
    ));
    /// The square root of every element, for degrees up to 8.
    const BYTE_ROOTS: &'static [u8; BYTES] = &narrowed(&linear_table(
        Self::BYTE_ELEMENTS,
        Plan::<P>::DEGREE,
        Plan::<P>::TAPS,
        Plan::<P>::DEGREE - 1,
    ));
    /// The inverse of every element, for degrees up to 8.
    const BYTE_INVERSES: &'static [u8; BYTES] = &narrowed(&inverses(
        Self::BYTE_ELEMENTS,
        Plan::<P>::DEGREE,
        Plan::<P>::TAPS,
    ));

    /// The square of every element, for degrees 9 to 16.
    const WORD_SQUARES: &'static [u16; WORDS] =
        &linear_table(Self::WORD_ELEMENTS, Plan::<P>::DEGREE, Plan::<P>::TAPS, 1);
    /// The square root of every element, for degrees 9 to 16.
    const WORD_ROOTS: &'static [u16; WORDS] = &linear_table(
        Self::WORD_ELEMENTS,
        Plan::<P>::DEGREE,
        Plan::<P>::TAPS,
        Plan::<P>::DEGREE - 1,
    );
    /// The inverse of every element, for degrees 9 to 16.
    const WORD_INVERSES: &'static [u16; WORDS] =
        &inverses(Self::WORD_ELEMENTS, Plan::<P>::DEGREE, Plan::<P>::TAPS);

    /// The square by bytes, for degrees 9 to 16.
    const SQUARE_BYTES: &'static ByteMap =
        &byte_map(Plan::<P>::BYTE_MAPS, Plan::<P>::DEGREE, Plan::<P>::TAPS, 1);
    /// The square root by bytes, for degrees 9 to 16.
    const ROOT_BYTES: &'static ByteMap = &byte_map(
        Plan::<P>::BYTE_MAPS,
        Plan::<P>::DEGREE,
        Plan::<P>::TAPS,
        Plan::<P>::DEGREE - 1,
    );

    /// The Frobenius maps `a -> a^(2^k)` for `k` up to 16, of the fields up to degree 16: the
    /// constants of the constant-time operations, which read every entry for every value.
    const FROBENIUS: &'static [Blocks; 17] = &frobenius_maps(Plan::<P>::DEGREE, Plan::<P>::TAPS);

    /// `a b` by the table of products, for degrees up to 8: the index `256 b + a` takes `b` into
    /// the row, so a factor that a loop keeps fixes the row.
    #[inline(always)]
    pub(crate) fn mul(a: u64, b: u64) -> u64 {
        Self::PRODUCTS
            .get(((b << 8) | a) as usize)
            .copied()
            .map_or(0, u64::from)
    }

    /// `a b` by logarithms, for degrees up to 8: the power of the sum of the logarithms, `0` when
    /// either factor is.
    #[inline(always)]
    pub(crate) fn log_mul(a: u64, b: u64) -> u64 {
        let log = |value: u64| {
            Self::LOGARITHMS
                .get(value as usize)
                .copied()
                .map_or(0, usize::from)
        };
        let power = Self::EXPONENTIALS
            .get(log(a) + log(b))
            .copied()
            .map_or(0, u64::from);
        if a == 0 || b == 0 { 0 } else { power }
    }

    /// `a^2` by the table of squares.
    #[inline(always)]
    pub(crate) fn square(a: u64) -> u64 {
        if Plan::<P>::DEGREE <= 8 {
            Self::BYTE_SQUARES
                .get(a as usize)
                .copied()
                .map_or(0, u64::from)
        } else {
            Self::WORD_SQUARES
                .get(a as usize)
                .copied()
                .map_or(0, u64::from)
        }
    }

    /// The square root by the table of square roots.
    #[inline(always)]
    pub(crate) fn sqrt(a: u64) -> u64 {
        if Plan::<P>::DEGREE <= 8 {
            Self::BYTE_ROOTS
                .get(a as usize)
                .copied()
                .map_or(0, u64::from)
        } else {
            Self::WORD_ROOTS
                .get(a as usize)
                .copied()
                .map_or(0, u64::from)
        }
    }

    /// The inverse, `0` for `0`, by the table of inverses.
    #[inline(always)]
    pub(crate) fn invert(a: u64) -> u64 {
        if Plan::<P>::DEGREE <= 8 {
            Self::BYTE_INVERSES
                .get(a as usize)
                .copied()
                .map_or(0, u64::from)
        } else {
            Self::WORD_INVERSES
                .get(a as usize)
                .copied()
                .map_or(0, u64::from)
        }
    }

    /// `a^2` by the images of the two bytes of `a`, for degrees 9 to 16.
    #[inline(always)]
    pub(crate) fn square_by_bytes(a: u64) -> u64 {
        map_by_bytes(Self::SQUARE_BYTES, a)
    }

    /// The square root by the images of the two bytes of `a`, for degrees 9 to 16.
    #[inline(always)]
    pub(crate) fn sqrt_by_bytes(a: u64) -> u64 {
        map_by_bytes(Self::ROOT_BYTES, a)
    }

    /// `a^(2^k)` for `k` up to 16, by the constant-time linear map of the Frobenius power. Degrees
    /// up to 16.
    #[inline(always)]
    pub(crate) fn frobenius(a: u64, k: usize) -> u64 {
        match Self::FROBENIUS.get(k) {
            Some(map) => apply(map, a, Plan::<P>::DEGREE),
            None => 0,
        }
    }
}

/// The image of `a`, below `2^16`, under the map `bytes`.
#[inline(always)]
fn map_by_bytes(bytes: &ByteMap, a: u64) -> u64 {
    let [low, high] = bytes;
    let image = |row: &[u16; BYTES], byte: u64| row.get(byte as usize).copied().unwrap_or(0);
    u64::from(image(low, a & 0xFF) ^ image(high, (a >> 8) & 0xFF))
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

/// The product of every pair of elements of a field of degree up to 8, `a b` at `256 b + a`, by
/// the logarithms along the powers of a generator `g`: `a b = g^((log a + log b) mod q)`. Zero
/// unless `used`.
#[expect(
    clippy::indexing_slicing,
    reason = "const evaluation, with every index below its table's length"
)]
const fn products(used: bool, degree: usize, taps: &[usize]) -> [u8; WORDS] {
    let mut table = [0; WORDS];
    if !used || degree > 8 {
        return table;
    }
    let order = (1 << degree) - 1;
    let logs = logarithms(true, degree, taps);
    let exps = exponentials(true, degree, taps);
    let mut a = 1;
    while a <= order {
        let mut b = 1;
        while b <= order {
            table[(b << 8) | a] = exps[(logs[a] as usize + logs[b] as usize) % order];
            b += 1;
        }
        a += 1;
    }
    table
}

/// The logarithm of every nonzero element of a field of degree up to 8 to a generator, `0` at
/// `0`. Zero unless `used`.
#[expect(
    clippy::indexing_slicing,
    reason = "const evaluation, with every index below its table's length"
)]
const fn logarithms(used: bool, degree: usize, taps: &[usize]) -> [u8; BYTES] {
    let mut table = [0; BYTES];
    if !used || degree > 8 {
        return table;
    }
    let order = (1 << degree) - 1;
    let generator = generator(degree, taps);
    let mut power = 1;
    let mut index = 0;
    while index < order {
        table[power as usize] = index as u8;
        power = small_mul(power, generator, degree, taps);
        index += 1;
    }
    table
}

/// `g^i` at `i` for every `i` below `2 (2^n - 1)`, for a field of degree up to 8 and its
/// generator `g`. Zero unless `used`.
#[expect(
    clippy::indexing_slicing,
    reason = "const evaluation, with every index below its table's length"
)]
const fn exponentials(used: bool, degree: usize, taps: &[usize]) -> [u8; 2 * BYTES] {
    let mut table = [0; 2 * BYTES];
    if !used || degree > 8 {
        return table;
    }
    let order = (1 << degree) - 1;
    let generator = generator(degree, taps);
    let mut power = 1;
    let mut index = 0;
    while index < 2 * order {
        table[index] = power as u8;
        power = small_mul(power, generator, degree, taps);
        index += 1;
    }
    table
}

/// The inverse of every element of a field of degree up to 16, `0` for `0`: `g^(q - i)` for
/// `g^i`, along the powers of a generator `g`. Zero unless `used`.
#[expect(
    clippy::indexing_slicing,
    reason = "const evaluation, with every index below its table's length"
)]
const fn inverses(used: bool, degree: usize, taps: &[usize]) -> [u16; WORDS] {
    let mut table = [0; WORDS];
    if !used || degree > 16 {
        return table;
    }
    let order = (1 << degree) - 1;
    let generator = generator(degree, taps);
    let mut powers = [0u16; WORDS];
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

/// The first `BYTES` entries of `table`, in bytes: the entries of a field of degree up to 8.
#[expect(
    clippy::indexing_slicing,
    reason = "const evaluation, with every index below its table's length"
)]
const fn narrowed(table: &[u16; WORDS]) -> [u8; BYTES] {
    let mut bytes = [0; BYTES];
    let mut index = 0;
    while index < BYTES {
        bytes[index] = table[index] as u8;
        index += 1;
    }
    bytes
}

/// The images of `x^j` under `a -> a^(2^k)` in a field of degree up to 16.
#[expect(
    clippy::indexing_slicing,
    reason = "const evaluation, with every index below its array's length"
)]
const fn images(degree: usize, taps: &[usize], k: usize) -> [u64; 16] {
    let mut images = [0u64; 16];
    let mut index = 0;
    while index < degree && index < 16 {
        let mut power = 1 << index;
        let mut step = 0;
        while step < k {
            power = small_mul(power, power, degree, taps);
            step += 1;
        }
        images[index] = power;
        index += 1;
    }
    images
}

/// The blocks of `a -> a^(2^k)` for `k` up to 16, of a field of degree up to 16; zero above.
#[expect(
    clippy::indexing_slicing,
    reason = "const evaluation, with every index below its array's length"
)]
const fn frobenius_maps(degree: usize, taps: &[usize]) -> [Blocks; 17] {
    let mut maps = [[[0; 2]; 2]; 17];
    if degree > 16 {
        return maps;
    }
    let mut k = 0;
    while k <= 16 {
        maps[k] = blocks(&images(degree, taps, k));
        k += 1;
    }
    maps
}

/// The image of every element of a field of degree up to 16 under `a -> a^(2^k)`: each element is
/// a smaller one plus its lowest basis element. Zero unless `used`.
#[expect(
    clippy::indexing_slicing,
    reason = "const evaluation, with every index below its table's length"
)]
const fn linear_table(used: bool, degree: usize, taps: &[usize], k: usize) -> [u16; WORDS] {
    let mut table = [0; WORDS];
    if !used || degree > 16 {
        return table;
    }
    let images = images(degree, taps, k);
    let mut value: usize = 1;
    while value < 1 << degree {
        let low = value & value.wrapping_neg();
        table[value] = table[value ^ low] ^ images[low.trailing_zeros() as usize] as u16;
        value += 1;
    }
    table
}

/// The map `a -> a^(2^k)` of a field of degree 9 to 16 by bytes: the image of every value of the
/// low byte, then of the high byte. Zero unless `used`.
#[expect(
    clippy::indexing_slicing,
    reason = "const evaluation, with every index below its table's length"
)]
const fn byte_map(used: bool, degree: usize, taps: &[usize], k: usize) -> ByteMap {
    let mut map = [[0; BYTES]; 2];
    if !used || degree > 16 {
        return map;
    }
    let images = images(degree, taps, k);
    let mut half = 0;
    while half < 2 {
        let mut value: usize = 1;
        while value < BYTES {
            let low = value & value.wrapping_neg();
            let bit = 8 * half + low.trailing_zeros() as usize;
            let image = if bit < degree { images[bit] as u16 } else { 0 };
            map[half][value] = map[half][value ^ low] ^ image;
            value += 1;
        }
        half += 1;
    }
    map
}
