//! The constants of a field that its operations read: `sqrt(x)`, the constant-time maps of the
//! Frobenius powers of the fields up to degree 16, and the lookup tables of their default
//! operations, which const evaluation generates on first use. Each table has one entry per
//! element, in the narrowest integer that holds one; the plan says which a build carries.

use core::marker::PhantomData;

use biguint::Uint;
use clmul::{NibbleMap, Poly};
use const_default::ConstDefault;
use generic_array::GenericArray;
use typenum::{U8, U128};

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

    /// The fold `t r` of the overflow `t` of `h r` past `x^n`, for the top nibble of `h`, below
    /// `x^n`, at `x^(n-4)`: `t = r x^(k-4)` rounded down for bit `k` of the nibble. Read when the
    /// plan's `SPILL_BY_NIBBLES` holds, where `d` is at most 4.
    pub(crate) const SPILL: NibbleMap = NibbleMap::from_images(spill_images(Plan::<P>::R_WORD));

    /// [`SPILL`](Self::SPILL) for a square, indexed by the top two bits of the element: they square
    /// to bits 0 and 2 of the top nibble of `h`.
    pub(crate) const SQUARE_SPILL: NibbleMap = {
        let [low, _, high, _] = spill_images(Plan::<P>::R_WORD);
        NibbleMap::from_images([low, high, 0, 0])
    };

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
    /// Whether the product adds logarithms of bytes.
    const BYTE_LOGARITHMS: bool = Plan::<P>::MUL_BY_LOGARITHMS && Plan::<P>::DEGREE <= 8;
    /// Whether the product adds logarithms of 16-bit words.
    const WORD_LOGARITHMS: bool = Plan::<P>::MUL_BY_LOGARITHMS && Plan::<P>::DEGREE > 8;

    /// The logarithm of every nonzero element of a field of degree up to 8 to a generator.
    const LOGARITHMS: &'static [u8; BYTES] = &narrowed(&logarithms::<BYTES>(
        Self::BYTE_LOGARITHMS,
        Plan::<P>::DEGREE,
        Plan::<P>::TAPS,
    ));
    /// The powers of the generator, twice over, so that a sum of two logarithms indexes it.
    const EXPONENTIALS: &'static [u8; 2 * BYTES] = &narrowed(&exponentials::<{ 2 * BYTES }>(
        Self::BYTE_LOGARITHMS,
        Plan::<P>::DEGREE,
        Plan::<P>::TAPS,
    ));
    /// The logarithm of every nonzero element of a field of degree 9 to 16.
    const WORD_LOGARITHM_TABLE: &'static [u16; WORDS] =
        &logarithms::<WORDS>(Self::WORD_LOGARITHMS, Plan::<P>::DEGREE, Plan::<P>::TAPS);
    /// The powers of the generator of a field of degree 9 to 16, twice over.
    const WORD_EXPONENTIALS: &'static [u16; 2 * WORDS] =
        &exponentials::<{ 2 * WORDS }>(Self::WORD_LOGARITHMS, Plan::<P>::DEGREE, Plan::<P>::TAPS);

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

    /// The composite form of the field, for even degrees from 10 to 16.
    const COMPOSITE: &'static Composite = &composite(
        Plan::<P>::MUL_BY_SUBFIELD_PRODUCTS,
        Plan::<P>::MUL_BY_SUBFIELD_LOGARITHMS,
        Plan::<P>::DEGREE,
        Plan::<P>::TAPS,
    );

    /// `u v x^(8 k)` reduced, at `u + 256 v`, for `k` from 0 to 2.
    const SPLIT: &'static [[u16; WORDS]; 3] = &split_tables(
        Plan::<P>::MUL_BY_SUBFIELD_PRODUCTS,
        Plan::<P>::DEGREE,
        Plan::<P>::TAPS,
    );

    /// `t v x^(4 k)` reduced, at `256 t + v`, for nibbles `t`, bytes `v` and `k` from 0 to 5.
    const NIBBLE_SPLIT: &'static [[u16; 16 * BYTES]; 6] = &nibble_split_tables(
        Plan::<P>::MUL_BY_SUBFIELD_PRODUCTS,
        Plan::<P>::DEGREE,
        Plan::<P>::TAPS,
    );

    /// The fold of the part of a product at and above `x^n` by bytes, for degrees 9 to 16.
    const FOLDS: &'static ByteMap = &fold_map(
        Plan::<P>::MUL_BY_FOLDS || Plan::<P>::MUL_BY_SUBFIELD_PRODUCTS,
        Plan::<P>::DEGREE,
        Plan::<P>::TAPS,
    );

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

    /// `p mod f` for a product `p` of two elements of a field of degree 9 to 16: the low `n` bits
    /// plus the folds of the two bytes above them.
    #[inline(always)]
    pub(crate) fn fold(product: u64) -> u64 {
        let degree = Plan::<P>::DEGREE;
        let high = product >> degree;
        let [low_folds, high_folds] = Self::FOLDS;
        let fold = |folds: &[u16; BYTES], byte: u64| {
            folds
                .get((byte & 0xFF) as usize)
                .copied()
                .map_or(0, u64::from)
        };
        (product & ((1 << degree) - 1)) ^ fold(low_folds, high) ^ fold(high_folds, high >> 8)
    }

    /// `a b` by logarithms, for degrees up to 16: the power of the sum of the logarithms, `0` when
    /// either factor is.
    #[inline(always)]
    pub(crate) fn log_mul(a: u64, b: u64) -> u64 {
        let power = if Plan::<P>::DEGREE <= 8 {
            let log = |value: u64| {
                Self::LOGARITHMS
                    .get(value as usize)
                    .copied()
                    .map_or(0, usize::from)
            };
            Self::EXPONENTIALS
                .get(log(a) + log(b))
                .copied()
                .map_or(0, u64::from)
        } else {
            let log = |value: u64| {
                Self::WORD_LOGARITHM_TABLE
                    .get(value as usize)
                    .copied()
                    .map_or(0, usize::from)
            };
            Self::WORD_EXPONENTIALS
                .get(log(a) + log(b))
                .copied()
                .map_or(0, u64::from)
        };
        if a == 0 || b == 0 { 0 } else { power }
    }

    /// `a b = a0 b0 + (a0 b1 + a1 b0) x^8 + a1 b1 x^16` for the bytes of `a` and `b`, each term from a
    /// table of the products of two bytes times `x^(8 k)`, reduced.
    #[inline(always)]
    pub(crate) fn split_mul(a: u64, b: u64) -> u64 {
        let [low, middle, high] = Self::SPLIT;
        let entry = |table: &[u16; WORDS], u: u64, v: u64| {
            table
                .get((((v & 0xFF) << 8) | (u & 0xFF)) as usize)
                .copied()
                .map_or(0, u64::from)
        };
        entry(low, a, b)
            ^ entry(middle, a, b >> 8)
            ^ entry(middle, a >> 8, b)
            ^ entry(high, a >> 8, b >> 8)
    }

    /// `a b` as the sum over the nibbles `a_i` of `a` and the bytes `b_j` of `b` of the reduced
    /// `a_i b_j x^(4 i + 8 j)`, each from the table of its shift.
    #[inline(always)]
    pub(crate) fn nibble_split_mul(a: u64, b: u64) -> u64 {
        let tables = Self::NIBBLE_SPLIT;
        let entry = |k: usize, nibble: u64, byte: u64| {
            tables
                .get(k)
                .and_then(|table| table.get((((nibble & 0xF) << 8) | (byte & 0xFF)) as usize))
                .copied()
                .map_or(0, u64::from)
        };
        let (b0, b1) = (b, b >> 8);
        (entry(0, a, b0) ^ entry(1, a >> 4, b0))
            ^ (entry(2, a >> 8, b0) ^ entry(3, a >> 12, b0))
            ^ ((entry(2, a, b1) ^ entry(3, a >> 4, b1))
                ^ (entry(4, a >> 8, b1) ^ entry(5, a >> 12, b1)))
    }

    /// `a b` from the carry-less products of bytes, by Karatsuba's three, folded by bytes.
    #[inline(always)]
    pub(crate) fn byte_product_mul(a: u64, b: u64) -> u64 {
        let [products, _, _] = Self::SPLIT;
        let entry = |u: u64, v: u64| {
            products
                .get((((v & 0xFF) << 8) | (u & 0xFF)) as usize)
                .copied()
                .map_or(0, u64::from)
        };
        let low = entry(a, b);
        let high = entry(a >> 8, b >> 8);
        let middle = entry(a ^ (a >> 8), b ^ (b >> 8)) ^ low ^ high;
        Self::fold(low ^ (middle << 8) ^ (high << 16))
    }

    /// `a b` in the composite form, for even degrees from 10 to 16: the words of `a` and `b` split
    /// into their subfield coordinates `a0 + a1 w` and `b0 + b1 w`, and with `w^2 = w + l`,
    /// `a b = (a0 b0 + l a1 b1) + ((a0 + a1)(b0 + b1) + a0 b0) w`. The three subfield products
    /// index the tables of every product, or add subfield logarithms.
    #[inline(always)]
    pub(crate) fn composite_mul(a: u64, b: u64) -> u64 {
        let form = Self::COMPOSITE;
        let half = Plan::<P>::DEGREE / 2;
        let mask = (1 << half) - 1;
        let (a, b) = (map_by_bytes(&form.into, a), map_by_bytes(&form.into, b));
        let (a0, a1, b0, b1) = (a & mask, a >> half, b & mask, b >> half);
        let low = Self::subfield_mul(a0, b0, false);
        let middle = Self::subfield_mul(a0 ^ a1, b0 ^ b1, false);
        let high = Self::subfield_mul(a1, b1, true);
        map_by_bytes(&form.out, (low ^ high) | ((middle ^ low) << half))
    }

    /// The subfield product `u v`, times `l` when `scaled`.
    #[inline(always)]
    fn subfield_mul(u: u64, v: u64, scaled: bool) -> u64 {
        let form = Self::COMPOSITE;
        if Plan::<P>::MUL_BY_SUBFIELD_PRODUCTS {
            let table = if scaled { &form.scaled } else { &form.products };
            return table
                .get(((v << 8) | u) as usize)
                .copied()
                .map_or(0, u64::from);
        }
        let log = |value: u64| {
            form.logarithms
                .get(value as usize)
                .copied()
                .map_or(0, usize::from)
        };
        let shift = if scaled { usize::from(form.scale) } else { 0 };
        let power = form
            .exponentials
            .get(log(u) + log(v) + shift)
            .copied()
            .map_or(0, u64::from);
        if u == 0 || v == 0 { 0 } else { power }
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

    /// The Frobenius maps `a -> a^(2^k)` for `k` up to 16 of a field up to degree 8 as nibble
    /// maps of the low and the high nibble.
    const FROBENIUS_NIBBLES: [[NibbleMap; 2]; 17] =
        frobenius_nibbles(Plan::<P>::DEGREE, Plan::<P>::TAPS);

    /// The fold `h x^n mod f` of the part `h`, below `x^(n-1)`, of a product of two elements of a
    /// field up to degree 8, as nibble maps of the low and the high nibble of `h`.
    const FOLD_NIBBLES: [NibbleMap; 2] = fold_nibbles(Plan::<P>::DEGREE, Plan::<P>::TAPS);

    /// `a^(2^k)` for `k` up to 16, by the constant-time linear map of the Frobenius power. Degrees
    /// up to 16; up to 8, two nibble maps in a vector register where the plan says so.
    #[inline(always)]
    pub(crate) fn frobenius(a: u64, k: usize) -> u64 {
        if Plan::<P>::NIBBLE_MAPS
            && let Some(&[low, high]) = Self::FROBENIUS_NIBBLES.get(k)
        {
            let byte = Poly::<U8>::from_uint(Uint::from_u64(a));
            return word(byte.map_low_nibbles(low) ^ byte.map_high_nibbles(high));
        }
        match Self::FROBENIUS.get(k) {
            Some(map) => apply(map, a, Plan::<P>::DEGREE),
            None => 0,
        }
    }

    /// `p mod f` for a product `p` of two elements of a field up to degree 8, in the low word of a
    /// register: the low `n` bits plus the nibble maps of the part at and above `x^n`.
    #[inline(always)]
    pub(crate) fn fold_byte(product: Poly<U128>) -> u64 {
        let [low, high] = Self::FOLD_NIBBLES;
        let degree = Plan::<P>::DEGREE;
        let part = (product >> degree).resize::<U8>();
        let folded =
            product.resize::<U8>() ^ part.map_low_nibbles(low) ^ part.map_high_nibbles(high);
        word(folded) & ((1 << degree) - 1)
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

/// The polynomial of `byte` as a word.
#[inline(always)]
fn word(byte: Poly<U8>) -> u64 {
    byte.to_uint().as_limbs().first().copied().unwrap_or(0)
}

/// The nibble maps of the low and the high nibble of a byte under the map that sends `x^j` to
/// `images[j]`, for `j` below 8.
const fn nibble_pair(images: &[u64; 16]) -> [NibbleMap; 2] {
    let [i0, i1, i2, i3, i4, i5, i6, i7, ..] = *images;
    [
        NibbleMap::from_images([i0 as u8, i1 as u8, i2 as u8, i3 as u8]),
        NibbleMap::from_images([i4 as u8, i5 as u8, i6 as u8, i7 as u8]),
    ]
}

/// The nibble maps of `a -> a^(2^k)` for `k` up to 16, of a field up to degree 8; zero above.
#[expect(
    clippy::indexing_slicing,
    reason = "const evaluation, with every index below its array's length"
)]
const fn frobenius_nibbles(degree: usize, taps: &[usize]) -> [[NibbleMap; 2]; 17] {
    let zero = NibbleMap::from_images([0; 4]);
    let mut maps = [[zero; 2]; 17];
    if degree > 8 {
        return maps;
    }
    let mut k = 0;
    while k <= 16 {
        maps[k] = nibble_pair(&images(degree, taps, k));
        k += 1;
    }
    maps
}

/// The nibble maps of `h -> h x^n mod f` for `h` below `x^8`, of a field up to degree 8; zero
/// above.
#[expect(
    clippy::indexing_slicing,
    reason = "const evaluation, with every index below its array's length"
)]
const fn fold_nibbles(degree: usize, taps: &[usize]) -> [NibbleMap; 2] {
    let mut images = [0u64; 16];
    if degree > 8 {
        return nibble_pair(&images);
    }
    let r = r_bits(taps);
    let mut j = 0;
    while j < 8 {
        images[j] = small_mul(r, 1 << j, degree, taps);
        j += 1;
    }
    nibble_pair(&images)
}

/// The images of the bits `k` of the top nibble of `h` under `h -> t r`: `t = (r << k) >> 4`, the
/// carry-less product of `t` and `r` in a byte for `r` below `x^5`.
const fn spill_images(r: u64) -> [u8; 4] {
    let mut images = [0u8; 4];
    let mut k = 0;
    while k < 4 {
        let overflow = ((r << k) >> 4) & 0xF;
        let mut product = 0u64;
        let mut bit = 0;
        while bit < 4 {
            if (overflow >> bit) & 1 == 1 {
                product ^= r << bit;
            }
            bit += 1;
        }
        if let Some(slot) = images.as_mut_slice().split_at_mut(k).1.first_mut() {
            *slot = product as u8;
        }
        k += 1;
    }
    images
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
    let logs = logarithms::<BYTES>(true, degree, taps);
    let exps = exponentials::<{ 2 * BYTES }>(true, degree, taps);
    let mut a = 1;
    while a <= order {
        let mut b = 1;
        while b <= order {
            table[(b << 8) | a] = exps[(logs[a] as usize + logs[b] as usize) % order] as u8;
            b += 1;
        }
        a += 1;
    }
    table
}

/// The logarithm of every nonzero element of a field of degree up to 16 to a generator, `0` at
/// `0`, in a table of `LEN` entries, `2^n` at least. Zero unless `used`.
#[expect(
    clippy::indexing_slicing,
    reason = "const evaluation, with every index below its table's length"
)]
const fn logarithms<const LEN: usize>(used: bool, degree: usize, taps: &[usize]) -> [u16; LEN] {
    let mut table = [0; LEN];
    if !used || degree > 16 || LEN < 1 << degree {
        return table;
    }
    let order = (1 << degree) - 1;
    let generator = generator(degree, taps);
    let mut power = 1;
    let mut index = 0;
    while index < order {
        table[power as usize] = index as u16;
        power = small_mul(power, generator, degree, taps);
        index += 1;
    }
    table
}

/// `g^i` at `i` for every `i` below `2 (2^n - 1)`, for a field of degree up to 16 and its
/// generator `g`, in a table of `LEN` entries, `2^(n+1)` at least. Zero unless `used`.
#[expect(
    clippy::indexing_slicing,
    reason = "const evaluation, with every index below its table's length"
)]
const fn exponentials<const LEN: usize>(used: bool, degree: usize, taps: &[usize]) -> [u16; LEN] {
    let mut table = [0; LEN];
    if !used || degree > 16 || LEN < 2 << degree {
        return table;
    }
    let order = (1 << degree) - 1;
    let generator = generator(degree, taps);
    let mut power = 1;
    let mut index = 0;
    while index < order {
        table[index] = power as u16;
        table[index + order] = power as u16;
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

/// The first `OUT` entries of `table`, in bytes: the entries of a field of degree up to 8.
#[expect(
    clippy::indexing_slicing,
    reason = "const evaluation, with every index below its table's length"
)]
const fn narrowed<const IN: usize, const OUT: usize>(table: &[u16; IN]) -> [u8; OUT] {
    let mut bytes = [0; OUT];
    let mut index = 0;
    while index < OUT && index < IN {
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

/// `map[c][v] = v x^(n + 8 c) mod f` for degrees 9 to 16: the fold by bytes of the part of a product
/// at and above `x^n`, of at most 15 bits. Zero unless `used`.
#[expect(
    clippy::indexing_slicing,
    reason = "const evaluation, with every index below its array's length"
)]
const fn fold_map(used: bool, degree: usize, taps: &[usize]) -> ByteMap {
    let mut map = [[0; BYTES]; 2];
    if !used || degree > 16 {
        return map;
    }
    let mut r = 0u32;
    let mut rest = taps;
    while let [tap, tail @ ..] = rest {
        r |= 1 << *tap;
        rest = tail;
    }
    // `x^(n + j) mod f`, from `x^n = r`.
    let mut images = [0u16; 16];
    let mut power = r;
    let mut j = 0;
    while j < 16 {
        images[j] = power as u16;
        power <<= 1;
        if (power >> degree) & 1 == 1 {
            power ^= (1 << degree) | r;
        }
        j += 1;
    }
    let mut half = 0;
    while half < 2 {
        let mut value: usize = 1;
        while value < BYTES {
            let low = value & value.wrapping_neg();
            let bit = 8 * half + low.trailing_zeros() as usize;
            map[half][value] = map[half][value ^ low] ^ images[bit];
            value += 1;
        }
        half += 1;
    }
    map
}

/// A field of even degree `n` from 10 to 16 as the quadratic extension `GF(2^h)[w]/(w^2 + w + l)`
/// of its subfield of degree `h = n / 2`: the element `a0 + a1 w` is the word `a0 + 2^h a1` of the
/// subfield coordinates of `a0` and `a1` in the basis `1, c, ..., c^(h-1)`, `c` a generator of the
/// subfield. The maps into and out of the composite words are linear, by bytes.
pub(crate) struct Composite {
    /// The map of an element to its composite word.
    into: ByteMap,
    /// The map of a composite word to its element.
    out: ByteMap,
    /// The subfield product `u v` at `u + 256 v`.
    products: [u8; WORDS],
    /// The subfield product `l u v` at `u + 256 v`.
    scaled: [u8; WORDS],
    /// The subfield logarithm of every nonzero coordinate word to `c`.
    logarithms: [u8; BYTES],
    /// The powers of `c` as coordinate words, three times over: the sum of two logarithms and the
    /// logarithm of `l` indexes it.
    exponentials: [u8; 3 * BYTES],
    /// The logarithm of `l`.
    scale: u8,
}

/// The [`Composite`] form of a field of even degree from 10 to 16, with the product tables when
/// `products` holds and the subfield logarithms when `logarithms` holds. Zero unless one holds.
#[expect(
    clippy::indexing_slicing,
    reason = "const evaluation, with every index below its table's length"
)]
const fn composite(products: bool, logarithms: bool, degree: usize, taps: &[usize]) -> Composite {
    let mut form = Composite {
        into: [[0; BYTES]; 2],
        out: [[0; BYTES]; 2],
        products: [0; WORDS],
        scaled: [0; WORDS],
        logarithms: [0; BYTES],
        exponentials: [0; 3 * BYTES],
        scale: 0,
    };
    if !(products || logarithms) || degree > 16 || degree < 10 || degree % 2 == 1 {
        return form;
    }
    let half = degree / 2;
    let order = (1u64 << degree) - 1;
    let sub_order = (1u64 << half) - 1;
    // A generator of the subfield: a generator of the field to the power `q / (2^h - 1)`.
    let c = small_pow(generator(degree, taps), order / sub_order, degree, taps);
    // `w = v / t` for an element `v` outside the subfield and the trace `t = v + v^(2^h)` of its
    // conjugates: `w^2 + w = l` with `l = v v^(2^h) / t^2`, in the subfield.
    let mut v = 2;
    while small_pow(v, 1 << half, degree, taps) == v {
        v += 1;
    }
    let conjugate = small_pow(v, 1 << half, degree, taps);
    let inverse_trace = small_pow(v ^ conjugate, order - 1, degree, taps);
    let w = small_mul(v, inverse_trace, degree, taps);
    let l = small_mul(
        small_mul(v, conjugate, degree, taps),
        small_mul(inverse_trace, inverse_trace, degree, taps),
        degree,
        taps,
    );
    // The elements of the bits of a composite word: `c^j`, then `c^j w`.
    let mut images = [0u64; 16];
    let mut power = 1;
    let mut j = 0;
    while j < half {
        images[j] = power;
        images[j + half] = small_mul(power, w, degree, taps);
        power = small_mul(power, c, degree, taps);
        j += 1;
    }
    form.out = bit_byte_map(&images, degree);
    // The inverse map, through the image of every composite word.
    let mut words = [0u16; WORDS];
    let mut word: usize = 0;
    while word < 1 << degree {
        let element = form.out[0][word & 0xFF] ^ form.out[1][word >> 8];
        words[element as usize] = word as u16;
        word += 1;
    }
    let mut into = [0u64; 16];
    j = 0;
    while j < degree {
        into[j] = words[1 << j] as u64;
        j += 1;
    }
    form.into = bit_byte_map(&into, degree);
    // The subfield logarithms of the coordinate words.
    power = 1;
    let mut index = 0;
    while index < sub_order {
        let coordinates = (words[power as usize] as u64 & sub_order) as u8;
        form.logarithms[coordinates as usize] = index as u8;
        form.exponentials[index as usize] = coordinates;
        form.exponentials[(index + sub_order) as usize] = coordinates;
        form.exponentials[(index + 2 * sub_order) as usize] = coordinates;
        power = small_mul(power, c, degree, taps);
        index += 1;
    }
    form.scale = form.logarithms[(words[l as usize] as u64 & sub_order) as usize];
    if products {
        let mut a: usize = 1;
        while a <= sub_order as usize {
            let mut b: usize = 1;
            while b <= sub_order as usize {
                let sum = form.logarithms[a] as usize + form.logarithms[b] as usize;
                form.products[(b << 8) | a] = form.exponentials[sum];
                form.scaled[(b << 8) | a] = form.exponentials[sum + form.scale as usize];
                b += 1;
            }
            a += 1;
        }
    }
    form
}

/// `u v x^(8 k)` reduced, at `u + 256 v`, for `k` from 0 to 2 and bytes `u` and `v`, in a field of
/// degree 9 to 16. Zero unless `used`.
#[expect(
    clippy::indexing_slicing,
    reason = "const evaluation, with every index below its table's length"
)]
const fn split_tables(used: bool, degree: usize, taps: &[usize]) -> [[u16; WORDS]; 3] {
    let mut tables = [[0; WORDS]; 3];
    if !used || degree > 16 || degree < 9 {
        return tables;
    }
    let mut k = 0;
    while k < 3 {
        let shift = small_pow(2, 8 * k as u64, degree, taps);
        let mut v: usize = 0;
        while v < BYTES {
            // `u v x^(8 k)` is linear in `u`: the images of its bits are `v x^(8 k + i)`, and each
            // `u` adds its lowest bit's image to a smaller one.
            let mut images = [0u64; 8];
            let mut power = small_mul(v as u64 & ((1 << degree) - 1), shift, degree, taps);
            let mut bit = 0;
            while bit < 8 {
                images[bit] = power;
                power = small_mul(power, 2, degree, taps);
                bit += 1;
            }
            let mut u: usize = 1;
            while u < BYTES {
                let low = u & u.wrapping_neg();
                let image = images[low.trailing_zeros() as usize];
                tables[k][(v << 8) | u] = tables[k][(v << 8) | (u ^ low)] ^ image as u16;
                u += 1;
            }
            v += 1;
        }
        k += 1;
    }
    tables
}

/// `t v x^(4 k)` reduced, at `256 t + v`, for nibbles `t`, bytes `v` and `k` from 0 to 5, in a field
/// of degree 9 to 16. Zero unless `used`.
#[expect(
    clippy::indexing_slicing,
    reason = "const evaluation, with every index below its table's length"
)]
const fn nibble_split_tables(used: bool, degree: usize, taps: &[usize]) -> [[u16; 16 * BYTES]; 6] {
    let mut tables = [[0; 16 * BYTES]; 6];
    if !used || degree > 16 || degree < 9 {
        return tables;
    }
    let mut k = 0;
    while k < 6 {
        let shift = small_pow(2, 4 * k as u64, degree, taps);
        let mut t: usize = 0;
        while t < 16 {
            let scaled = small_mul(t as u64, shift, degree, taps);
            // Linear in `v`: the images of its bits are `t x^(4 k + i)`.
            let mut images = [0u64; 8];
            let mut power = scaled;
            let mut bit = 0;
            while bit < 8 {
                images[bit] = power;
                power = small_mul(power, 2, degree, taps);
                bit += 1;
            }
            let mut v: usize = 1;
            while v < BYTES {
                let low = v & v.wrapping_neg();
                let image = images[low.trailing_zeros() as usize];
                tables[k][(t << 8) | v] = tables[k][(t << 8) | (v ^ low)] ^ image as u16;
                v += 1;
            }
            t += 1;
        }
        k += 1;
    }
    tables
}

/// The linear map of 16-bit words that takes bit `j` to `images[j]`, for the bits below `degree`,
/// by bytes.
#[expect(
    clippy::indexing_slicing,
    reason = "const evaluation, with every index below its table's length"
)]
const fn bit_byte_map(images: &[u64; 16], degree: usize) -> ByteMap {
    let mut map = [[0; BYTES]; 2];
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
