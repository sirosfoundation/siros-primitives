//! The default inverse of a field `L` of degree 128 through its subfield `K` of degree 64: with
//! `s = a^(2^64)`, the norm `a s` lies in `K`, and `a^-1 = s (a s)^-1`. The norm maps to the field
//! of [`Poly64`], which inverts it with products of one word, and its inverse maps back. Builds with
//! `precomputed` take this inverse.
//!
//! Both maps come from one tower in each field: `u_1^2 + u_1 = 1` and
//! `u_(k+1)^2 + u_(k+1) = u_1 ... u_k`, each a GF(2)-linear equation in `u_(k+1)`. Each equation
//! is irreducible over the field of the levels below it, whose trace of `u_1 ... u_k` is 1, so
//! `u_6` generates the subfield of degree 64, and `u_k -> u_k` extends to an isomorphism whichever
//! roots the two towers take. The maps send the powers of `u_6` of one field to those of the other.

use core::marker::PhantomData;

use biguint::Uint;
use clmul::{Lane, Poly};
use const_default::ConstDefault;
use generic_array::GenericArray;
use typenum::U128;

use crate::arith::{Element, Field, Lanes};
use crate::element::Limbs;
use crate::frobenius::{add_lane_rows, add_word_rows, images, square_mod};
use crate::plan::{Plan, Polynomial, TIER, Tier};
use crate::representation::Storage;
use crate::standard::Poly64;


/// The degree of `L`.
const DEGREE: usize = 128;

/// The degree of `K`.
const HALF: usize = 64;

/// The levels of the tower up to degree 64.
const LEVELS: usize = 6;

/// Byte tables of `L` to `L`: the Frobenius power `2^64`.
type LaneBytes = [[Lane<U128>; 256]; 16];
/// Byte tables of `L` to `K`.
type DownBytes = [[u64; 256]; 16];
/// Byte tables of `K` to `L`.
type UpBytes = [[Lane<U128>; 256]; 8];

/// The inverse through the subfield of the field with the polynomial `P`.
pub(crate) struct Subfield<P>(PhantomData<P>);

impl<P: Polynomial> Subfield<P> {
    /// Whether the field's default inverse runs through its subfield: degree 128, with the byte
    /// tables of `precomputed`. Its 128 KiB of tables and those of [`Poly64`] exceed the default
    /// budget.
    pub(crate) const USED: bool = Plan::<P>::DEGREE == DEGREE && matches!(TIER, Tier::Full);

    /// The images of `x^j` under the maps `L -> K` and `K -> L`.
    const MAPS: Maps = if Self::USED {
        maps(Plan::<P>::TAPS, Plan::<Poly64>::TAPS)
    } else {
        Maps::ZERO
    };

    const FROBENIUS: &'static LaneBytes = &lane_table::<16, 256>(
        &if Self::USED {
            images(DEGREE, Plan::<P>::TAPS, HALF)
        } else {
            [0; 128]
        },
        DEGREE,
    );
    const DOWN: &'static DownBytes = &word_table::<16, 256>(&Self::MAPS.down, DEGREE);
    const UP: &'static UpBytes = &lane_table::<8, 256>(&Self::MAPS.up, HALF);

    /// The inverse of a nonzero `a`: `s (a s)^-1` with `s = a^(2^64)`, the norm inverted in `K`.
    #[inline]
    pub(crate) fn invert(a: Lanes<P>) -> Lanes<P> {
        let s = Self::frobenius(a);
        let norm = Field::<P>::lanes_mul(a, s);
        let inverse = Field::<Poly64>::invert(Element::<Poly64>::from_word(Self::down(norm)));
        Field::<P>::lanes_mul(s, Self::up(inverse.word()))
    }

    /// `a^(2^64)`.
    #[inline(always)]
    fn frobenius(a: Lanes<P>) -> Lanes<P> {
        let [low, high] = words::<P>(a);
        let mut sums = [Poly::<U128>::ZERO; 4];
        let (low_rows, high_rows) = Self::FROBENIUS.split_at(8);
        add_lane_rows(&mut sums, low_rows, low, 8);
        add_lane_rows(&mut sums, high_rows, high, 8);
        lane_sum::<P>(sums)
    }

    /// The element of `K` in the field of [`Poly64`] that `a` of `K` in `L` denotes.
    #[inline(always)]
    fn down(a: Lanes<P>) -> u64 {
        let [low, high] = words::<P>(a);
        let mut sums = [0u64; 4];
        let (low_rows, high_rows) = Self::DOWN.split_at(8);
        add_word_rows(&mut sums, low_rows, low, 8);
        add_word_rows(&mut sums, high_rows, high, 8);
        let [first, second, third, fourth] = sums;
        (first ^ second) ^ (third ^ fourth)
    }

    /// The element of `L` that `a` of the field of [`Poly64`] denotes.
    #[inline(always)]
    fn up(a: u64) -> Lanes<P> {
        let mut sums = [Poly::<U128>::ZERO; 4];
        add_lane_rows(&mut sums, Self::UP, a, 8);
        lane_sum::<P>(sums)
    }
}

/// The sum of four partial sums of images, as a polynomial of the field with the polynomial `P`.
#[inline(always)]
fn lane_sum<P: Polynomial>(sums: [Poly<U128>; 4]) -> Lanes<P> {
    let [first, second, third, fourth] = sums;
    let [low, high] = ((first ^ second) ^ (third ^ fourth))
        .to_uint()
        .into_limbs()
        .into_array();
    polynomial::<P>([low, high])
}


/// The two low words of `a`.
#[inline(always)]
fn words<P: Polynomial>(a: Lanes<P>) -> [u64; 2] {
    let limbs = a.to_uint().into_limbs();
    [
        limbs.first().copied().unwrap_or(0),
        limbs.get(1).copied().unwrap_or(0),
    ]
}

/// The polynomial of the field with the polynomial `P` whose low words are `words`.
#[inline(always)]
fn polynomial<P: Polynomial>(words: [u64; 2]) -> Lanes<P> {
    let mut limbs = GenericArray::<u64, Limbs<P>>::DEFAULT;
    for (slot, word) in limbs.iter_mut().zip(words) {
        *slot = word;
    }
    Poly::from_uint(Uint::from_generic_limbs(limbs))
}

/// The images of the maps between the subfield of degree 64 of `L` and the field of `K`.
struct Maps {
    /// `down[j]`: the image in `K` of `x^j` of `L`, for a map that is the isomorphism on the
    /// subfield.
    down: [u128; 128],
    /// `up[j]`: the image in `L` of `x^j` of `K`.
    up: [u128; 128],
}

impl Maps {
    const ZERO: Self = Self {
        down: [0; 128],
        up: [0; 128],
    };
}

/// A basis in echelon form of the span of vectors of GF(2)^128, each row with the sum of the
/// inserted vectors it stands for.
struct Echelon {
    /// The row with its highest bit at each position.
    rows: [u128; 128],
    /// The inserted vectors that each row sums, as bits.
    sums: [u128; 128],
    /// The positions that hold a row.
    pivots: u128,
}

impl Echelon {
    const fn new() -> Self {
        Self {
            rows: [0; 128],
            sums: [0; 128],
            pivots: 0,
        }
    }

    /// `v` less the rows that its bits select from the top down, and the inserted vectors that
    /// those rows sum. A map of `v` that is GF(2)-linear.
    #[expect(
        clippy::indexing_slicing,
        reason = "const evaluation, with every index below 128"
    )]
    const fn reduce(&self, v: u128) -> (u128, u128) {
        let (mut rest, mut sum) = (v, 0);
        let mut bit = 128;
        while bit > 0 {
            bit -= 1;
            if (rest >> bit) & 1 == 1 && (self.pivots >> bit) & 1 == 1 {
                rest ^= self.rows[bit];
                sum ^= self.sums[bit];
            }
        }
        (rest, sum)
    }

    /// Inserts `v`, which stands for the inserted vectors of `sum`, unless the span holds it.
    #[expect(
        clippy::indexing_slicing,
        reason = "const evaluation, with every index below 128"
    )]
    const fn insert(&mut self, v: u128, sum: u128) {
        let (rest, reduced) = self.reduce(v);
        if rest != 0 {
            let bit = 127 - rest.leading_zeros() as usize;
            self.rows[bit] = rest;
            self.sums[bit] = sum ^ reduced;
            self.pivots |= 1 << bit;
        }
    }
}

/// `r` of the polynomial of `taps`.
const fn r_bits(taps: &[usize]) -> u128 {
    let mut r = 0;
    let mut rest = taps;
    while let [tap, tail @ ..] = rest {
        r |= 1 << *tap;
        rest = tail;
    }
    r
}

/// `a b mod x^degree + r`, for `a` and `b` below `x^degree`, a degree up to 128.
const fn mul_mod(a: u128, b: u128, degree: usize, taps: &[usize]) -> u128 {
    let r = r_bits(taps);
    let mask = if degree >= 128 {
        u128::MAX
    } else {
        (1 << degree) - 1
    };
    let mut product = 0u128;
    let mut bit = degree;
    while bit > 0 {
        bit -= 1;
        let top = (product >> (degree - 1)) & 1;
        product = (product << 1) & mask;
        if top == 1 {
            product ^= r;
        }
        if (b >> bit) & 1 == 1 {
            product ^= a;
        }
    }
    product
}

/// `u_6` of the tower of the field of `degree` and `taps`, and its powers below 64.
#[expect(
    clippy::indexing_slicing,
    reason = "const evaluation, with every index below 128"
)]
const fn tower_powers(degree: usize, taps: &[usize]) -> [u128; 64] {
    // The columns of `t -> t^2 + t`.
    let mut columns = Echelon::new();
    let mut bit = 0;
    while bit < degree {
        let x = 1u128 << bit;
        columns.insert(square_mod(x, degree, taps) ^ x, x);
        bit += 1;
    }
    let (mut product, mut level) = (1u128, 0);
    let mut u = 0u128;
    while level < LEVELS {
        let (rest, root) = columns.reduce(product);
        assert!(
            rest == 0,
            "the polynomials of a field and of its subfield must be irreducible"
        );
        u = root;
        product = mul_mod(product, u, degree, taps);
        level += 1;
    }
    let mut powers = [0u128; 64];
    powers[0] = 1;
    let mut index = 1;
    while index < HALF {
        powers[index] = mul_mod(powers[index - 1], u, degree, taps);
        index += 1;
    }
    powers
}

/// The maps between the subfield of degree 64 of the field of degree 128 with `taps` and the field
/// of degree 64 with `half_taps`.
#[expect(
    clippy::indexing_slicing,
    reason = "const evaluation, with every index below 128"
)]
const fn maps(taps: &[usize], half_taps: &[usize]) -> Maps {
    let large = tower_powers(DEGREE, taps);
    let small = tower_powers(HALF, half_taps);
    // `x^j` of `K` as a sum of powers of its `u_6`, mapped to the same sum of the powers in `L`.
    let mut basis = Echelon::new();
    let mut index = 0;
    while index < HALF {
        basis.insert(small[index], 1 << index);
        index += 1;
    }
    let mut maps = Maps::ZERO;
    let mut image = Echelon::new();
    let mut bit = 0;
    while bit < HALF {
        let (rest, sum) = basis.reduce(1 << bit);
        assert!(
            rest == 0,
            "the polynomials of a field and of its subfield must be irreducible"
        );
        let mut up = 0;
        index = 0;
        while index < HALF {
            if (sum >> index) & 1 == 1 {
                up ^= large[index];
            }
            index += 1;
        }
        maps.up[bit] = up;
        image.insert(up, 1 << bit);
        bit += 1;
    }
    bit = 0;
    while bit < DEGREE {
        maps.down[bit] = image.reduce(1 << bit).1;
        bit += 1;
    }
    maps
}

/// The table of the map of `images` on inputs of `bits` bits, one or two words: for each window
/// position, the sum of the images of the bits of every window value. The positions of each word
/// fill half of the table when the input has two words.
#[expect(
    clippy::indexing_slicing,
    reason = "const evaluation, with every index below its array's length"
)]
const fn sums<const POSITIONS: usize, const ENTRIES: usize>(
    images: &[u128; 128],
    bits: usize,
) -> [[u128; ENTRIES]; POSITIONS] {
    let mut table = [[0; ENTRIES]; POSITIONS];
    let words = bits.div_ceil(64);
    let per_word = POSITIONS / words;
    let window = ENTRIES.trailing_zeros() as usize;
    let mut position = 0;
    while position < POSITIONS {
        let mut value: usize = 1;
        while value < ENTRIES {
            let low = value & value.wrapping_neg();
            let offset = window * (position % per_word) + low.trailing_zeros() as usize;
            let bit = 64 * (position / per_word) + offset;
            let image = if offset < 64 && bit < bits {
                images[bit]
            } else {
                0
            };
            table[position][value] = table[position][value ^ low] ^ image;
            value += 1;
        }
        position += 1;
    }
    table
}

/// [`sums`] with images in vector registers.
#[expect(
    clippy::indexing_slicing,
    reason = "const evaluation, with every index below its array's length"
)]
const fn lane_table<const POSITIONS: usize, const ENTRIES: usize>(
    images: &[u128; 128],
    bits: usize,
) -> [[Lane<U128>; ENTRIES]; POSITIONS] {
    let sums = sums::<POSITIONS, ENTRIES>(images, bits);
    let mut table = [[Lane::<U128>::ZERO; ENTRIES]; POSITIONS];
    let mut position = 0;
    while position < POSITIONS {
        let mut value = 0;
        while value < ENTRIES {
            table[position][value] =
                Lane::from_poly(Poly::<U128>::from_uint(Uint::from_u128(sums[position][value])));
            value += 1;
        }
        position += 1;
    }
    table
}

/// [`sums`] with images of one word.
#[expect(
    clippy::indexing_slicing,
    reason = "const evaluation, with every index below its array's length"
)]
const fn word_table<const POSITIONS: usize, const ENTRIES: usize>(
    images: &[u128; 128],
    bits: usize,
) -> [[u64; ENTRIES]; POSITIONS] {
    let sums = sums::<POSITIONS, ENTRIES>(images, bits);
    let mut table = [[0; ENTRIES]; POSITIONS];
    let mut position = 0;
    while position < POSITIONS {
        let mut value = 0;
        while value < ENTRIES {
            table[position][value] = sums[position][value] as u64;
            value += 1;
        }
        position += 1;
    }
    table
}

