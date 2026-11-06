//! Inverses of a field `L` through a subfield `K`: with `r` the product of the conjugates of `a`
//! other than `a`, the norm `a r` lies in `K`, and `a^-1 = r (a r)^-1`. The norm maps to the field
//! of the degree of `K` with a polynomial of its own, which inverts it, and its inverse maps back.
//! Builds with `precomputed` take the default inverse at degree 128 through [`Poly64`], with
//! `r = a^(2^64)`. On WebAssembly they also take the default inverse at degree 64 through
//! [`Poly16`], the constant-time one at degree 128 through [`Poly64`], and both at degree 256
//! through [`Gcm`].
//!
//! Both maps come from one tower in each field: `u_1^2 + u_1 = 1` and
//! `u_(k+1)^2 + u_(k+1) = u_1 ... u_k`, each a GF(2)-linear equation in `u_(k+1)`. Each equation
//! is irreducible over the field of the levels below it, whose trace of `u_1 ... u_k` is 1, so
//! `u_j` generates the subfield of degree `2^j`, and `u_k -> u_k` extends to an isomorphism
//! whichever roots the two towers take. The maps send the powers of `u_j` of one field to those of
//! the other, for `K` of degree `2^j`.

use core::marker::PhantomData;

use clmul::{Lane, Poly};
use typenum::{U16, U64, U128};

use crate::arith::{Element, Field, Lanes};
use crate::frobenius::{
    add_lane_rows, add_quad_rows, add_word_rows, pair_masked_sum, polynomial, quad_images,
    quad_masked_sum, quad_mul_mod, quad_square_mod, shifts, word_masked_sum, xor_words,
};
use crate::plan::{Plan, Polynomial, TIER, Tier};
use crate::representation::Storage;
use crate::standard::{Gcm, Poly16, Poly64};

/// A vector of GF(2)^256, or a polynomial below `x^256`, in four words.
type Vector = [u64; 4];

/// Byte tables of `L` to `L` at degree 128: the Frobenius power `2^64`.
type LaneBytes = [[Lane<U128>; 256]; 16];
/// Byte tables of `L` to `K` at degree 128.
type DownBytes = [[u64; 256]; 16];
/// Byte tables of `K` to `L` at degree 128.
type UpBytes = [[Lane<U128>; 256]; 8];

/// Byte tables of a map of `L` of degree 64.
type WordBytes = [[u64; 256]; 8];

/// Byte tables of `L` to `L` at degree 256: the Frobenius power `2^128`.
type QuadBytes = [[Vector; 256]; 32];
/// Byte tables of `L` to `K` at degree 256.
type QuadDownBytes = [[Lane<U128>; 256]; 32];
/// Byte tables of `K` to `L` at degree 256.
type QuadUpBytes = [[Vector; 256]; 16];

/// The inverse through the subfield of the field with the polynomial `P`.
pub(crate) struct Subfield<P>(PhantomData<P>);

impl<P: Polynomial> Subfield<P> {
    /// Whether the field's default inverse runs through its subfield of degree 64: degree 128,
    /// with the byte tables of `precomputed`. Its 128 KiB of tables and those of [`Poly64`] exceed
    /// the default budget.
    pub(crate) const USED: bool = Plan::<P>::DEGREE == 128 && matches!(TIER, Tier::Full);

    /// Whether both inverses of the field run through its subfield of degree 128: degree 256 on
    /// WebAssembly with `precomputed`: the chain of the degree takes 14 products of four words, 9,900
    /// of the 13,600 cycles of its default inverse in Chrome; this inverse takes 2. The default
    /// inverse takes 512 KiB of byte tables; the constant-time one 16 KiB of the images of single
    /// bits, which it adds under masks.
    pub(crate) const QUAD_USED: bool = cfg!(target_arch = "wasm32")
        && !clmul::HARDWARE
        && Plan::<P>::DEGREE == 256
        && matches!(TIER, Tier::Full);

    /// Whether the constant-time inverse of the field runs through its subfield of degree 64:
    /// degree 128 on WebAssembly with `precomputed`, by masked sums of the images of single bits,
    /// 4 KiB, and the constant-time inverse of [`Poly64`]: Chrome took 2,864 cycles against 4,547
    /// along the chain of the degree.
    pub(crate) const LANE_CT_USED: bool = cfg!(target_arch = "wasm32")
        && !clmul::HARDWARE
        && Plan::<P>::DEGREE == 128
        && matches!(TIER, Tier::Full);

    /// Whether the default inverse of the field runs through its subfield of degree 16: degree 64
    /// on WebAssembly with `precomputed`, whose products of one word take 64 cycles in Chrome. The
    /// norm `a r` with `r = a^(2^16 + 2^32 + 2^48)` takes four products and two Frobenius powers,
    /// and [`Poly16`] inverts it by its table; 52 KiB of byte tables. Chrome took 375 cycles against
    /// 796 along the chain of the degree, of ten products.
    pub(crate) const WORD_USED: bool = cfg!(target_arch = "wasm32")
        && !clmul::HARDWARE
        && Plan::<P>::DEGREE == 64
        && matches!(TIER, Tier::Full);

    /// The images of `x^j` under the maps `L -> K` and `K -> L`.
    const MAPS: Maps = if Self::USED {
        maps(128, Plan::<P>::TAPS, 64, Plan::<Poly64>::TAPS)
    } else if Self::QUAD_USED {
        maps(256, Plan::<P>::TAPS, 128, Plan::<Gcm>::TAPS)
    } else if Self::WORD_USED {
        maps(64, Plan::<P>::TAPS, 16, Plan::<Poly16>::TAPS)
    } else {
        Maps::ZERO
    };

    /// The byte tables of the Frobenius powers `2^16` and `2^32` at degree 64.
    const WORD_FROBENIUS: &'static [WordBytes; 2] = &[
        word_table::<8, 256>(
            Self::WORD_USED,
            &quad_images(Self::WORD_USED, 64, Plan::<P>::TAPS, 16),
            64,
        ),
        word_table::<8, 256>(
            Self::WORD_USED,
            &quad_images(Self::WORD_USED, 64, Plan::<P>::TAPS, 32),
            64,
        ),
    ];
    const WORD_DOWN: &'static WordBytes =
        &word_table::<8, 256>(Self::WORD_USED, &Self::MAPS.down, 64);
    const WORD_UP: &'static [[u64; 256]; 2] =
        &word_table::<2, 256>(Self::WORD_USED, &Self::MAPS.up, 16);

    /// The images of `x^j` under the Frobenius power `2^64` at degree 128, for the masked sums of
    /// the constant-time inverse.
    const LANE_FROBENIUS_BITS: &'static [[u64; 2]; 128] =
        &pairs(Self::LANE_CT_USED, Self::FROBENIUS_IMAGES);
    /// The images of `x^j` of `L` in `K` at degree 128.
    const LANE_DOWN_BITS: &'static [u64; 128] = &singles(Self::LANE_CT_USED, &Self::MAPS.down);
    /// The images of `x^j` of `K` in `L` at degree 128.
    const LANE_UP_BITS: &'static [[u64; 2]; 64] = &pairs(Self::LANE_CT_USED, &Self::MAPS.up);

    /// The images of `x^j` under the Frobenius power `2^m`.
    const FROBENIUS_IMAGES: &'static [Vector; 256] = &quad_images(
        Self::USED || Self::QUAD_USED,
        Plan::<P>::DEGREE,
        Plan::<P>::TAPS,
        Plan::<P>::DEGREE / 2,
    );

    const FROBENIUS: &'static LaneBytes =
        &lane_table::<16, 256>(Self::USED, Self::FROBENIUS_IMAGES, 128);
    const DOWN: &'static DownBytes = &word_table::<16, 256>(Self::USED, &Self::MAPS.down, 128);
    const UP: &'static UpBytes = &lane_table::<8, 256>(Self::USED, &Self::MAPS.up, 64);

    const QUAD_FROBENIUS: &'static QuadBytes =
        &sums::<32, 256>(Self::QUAD_USED, Self::FROBENIUS_IMAGES, 256);
    const QUAD_DOWN: &'static QuadDownBytes =
        &lane_table::<32, 256>(Self::QUAD_USED, &Self::MAPS.down, 256);
    const QUAD_UP: &'static QuadUpBytes = &sums::<16, 256>(Self::QUAD_USED, &Self::MAPS.up, 128);

    /// The images of `x^j` of `L` in `K` at degree 256, for the masked sums of the constant-time
    /// inverse.
    const QUAD_DOWN_BITS: &'static [[u64; 2]; 256] = &pairs(Self::QUAD_USED, &Self::MAPS.down);
    /// The images of `x^j` of `K` in `L` at degree 256.
    const QUAD_UP_BITS: &'static [Vector; 128] = &Self::MAPS.up;

    /// The inverse of a nonzero `a` at degree 128: `s (a s)^-1` with `s = a^(2^64)`, the norm
    /// inverted in `K`.
    #[inline]
    pub(crate) fn invert(a: Lanes<P>) -> Lanes<P> {
        let s = Self::frobenius(a);
        let norm = Field::<P>::lanes_mul(a, s);
        let norm = <Element<Poly64> as Storage<U64>>::from_word(Self::down(norm));
        let inverse = <Element<Poly64> as Storage<U64>>::word(Field::<Poly64>::invert(norm));
        Field::<P>::lanes_mul(s, Self::up(inverse))
    }

    /// [`Self::invert`] in constant time: the maps add the images of the set bits under masks,
    /// and [`Poly64`] inverts in constant time. No address and no branch depends on `a`.
    #[inline]
    pub(crate) fn ct_invert_lane(a: Lanes<P>) -> Lanes<P> {
        let s = polynomial::<P, 2>(pair_masked_sum(
            Self::LANE_FROBENIUS_BITS,
            &words::<P, 2>(a),
        ));
        let norm = word_masked_sum(
            Self::LANE_DOWN_BITS,
            &words::<P, 2>(Field::<P>::lanes_mul(a, s)),
        );
        let inverse = <Element<Poly64> as Storage<U64>>::word(Field::<Poly64>::ct_invert(
            <Element<Poly64> as Storage<U64>>::from_word(norm),
        ));
        Field::<P>::lanes_mul(
            s,
            polynomial::<P, 2>(pair_masked_sum(Self::LANE_UP_BITS, &[inverse])),
        )
    }

    /// The inverse of a nonzero `a` at degree 64: `r (a r)^-1` with
    /// `r = a^(2^16 + 2^32 + 2^48) = t (a t)^(2^32)` for `t = a^(2^16)`, the norm `a r` inverted
    /// in the subfield of degree 16.
    #[inline]
    pub(crate) fn invert_word(a: Lanes<P>) -> Lanes<P> {
        let [sixteen, thirty_two] = Self::WORD_FROBENIUS;
        let t = Self::word_map(sixteen, a);
        let r = Field::<P>::lanes_mul(t, Self::word_map(thirty_two, Field::<P>::lanes_mul(a, t)));
        let [norm] = words::<P, 1>(Field::<P>::lanes_mul(a, r));
        let norm =
            <Element<Poly16> as Storage<U16>>::from_word(Self::word_sum(Self::WORD_DOWN, norm));
        let inverse = <Element<Poly16> as Storage<U16>>::word(Field::<Poly16>::invert(norm));
        Field::<P>::lanes_mul(
            r,
            polynomial::<P, 1>([Self::word_sum(Self::WORD_UP, inverse)]),
        )
    }

    /// The image of `a` of degree 64 under the linear map of the byte tables `rows`.
    #[inline(always)]
    fn word_map(rows: &WordBytes, a: Lanes<P>) -> Lanes<P> {
        let [word] = words::<P, 1>(a);
        polynomial::<P, 1>([Self::word_sum(rows, word)])
    }

    /// The sum of the rows of the byte tables `rows` that the bytes of `word` select.
    #[inline(always)]
    fn word_sum<const POSITIONS: usize>(rows: &[[u64; 256]; POSITIONS], word: u64) -> u64 {
        let mut sums = [0u64; 4];
        add_word_rows(&mut sums, rows, word, 8);
        let [first, second, third, fourth] = sums;
        (first ^ second) ^ (third ^ fourth)
    }

    /// `a^(2^64)`.
    #[inline(always)]
    fn frobenius(a: Lanes<P>) -> Lanes<P> {
        let [low, high] = words::<P, 2>(a);
        let mut sums = [Poly::<U128>::ZERO; 4];
        let (low_rows, high_rows) = Self::FROBENIUS.split_at(8);
        add_lane_rows(&mut sums, low_rows, low, 8);
        add_lane_rows(&mut sums, high_rows, high, 8);
        lane_sum::<P>(sums)
    }

    /// The element of `K` in the field of [`Poly64`] that `a` of `K` in `L` denotes.
    #[inline(always)]
    fn down(a: Lanes<P>) -> u64 {
        let [low, high] = words::<P, 2>(a);
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

    /// The inverse of a nonzero `a` at degree 256: `s (a s)^-1` with `s = a^(2^128)`, the norm
    /// inverted in `K` by the default inverse of [`Gcm`].
    #[inline]
    pub(crate) fn invert_quad(a: Lanes<P>) -> Lanes<P> {
        let s = Self::quad_frobenius(a);
        let norm = <Element<Gcm> as Storage<U128>>::from_poly(Self::quad_down(
            Field::<P>::lanes_mul(a, s),
        ));
        let inverse = <Element<Gcm> as Storage<U128>>::poly(Field::<Gcm>::invert(norm));
        Field::<P>::lanes_mul(s, Self::quad_up(inverse))
    }

    /// [`Self::invert_quad`] in constant time: the maps add the images of the set bits under
    /// masks, and [`Gcm`] inverts in constant time. No address and no branch depends on `a`.
    #[inline]
    pub(crate) fn ct_invert_quad(a: Lanes<P>) -> Lanes<P> {
        let s = polynomial::<P, 4>(quad_masked_sum(Self::FROBENIUS_IMAGES, &words::<P, 4>(a)));
        let norm = words::<P, 4>(Field::<P>::lanes_mul(a, s));
        let norm = <Element<Gcm> as Storage<U128>>::from_poly(polynomial::<Gcm, 2>(
            pair_masked_sum(Self::QUAD_DOWN_BITS, &norm),
        ));
        let inverse = <Element<Gcm> as Storage<U128>>::poly(Field::<Gcm>::ct_invert(norm));
        let inverse = words::<Gcm, 2>(inverse);
        Field::<P>::lanes_mul(
            s,
            polynomial::<P, 4>(quad_masked_sum(Self::QUAD_UP_BITS, &inverse)),
        )
    }

    /// `a^(2^128)` at degree 256.
    #[inline(always)]
    fn quad_frobenius(a: Lanes<P>) -> Lanes<P> {
        let mut sums = [[0u64; 4]; 2];
        for (rows, word) in Self::QUAD_FROBENIUS.chunks(8).zip(words::<P, 4>(a)) {
            add_quad_rows(&mut sums, rows, word, 8);
        }
        let [first, second] = sums;
        polynomial::<P, 4>(xor_words(first, second))
    }

    /// The element of `K` in the field of [`Gcm`] that `a` of `K` in `L` denotes, at degree 256.
    #[inline(always)]
    fn quad_down(a: Lanes<P>) -> Poly<U128> {
        let mut sums = [Poly::<U128>::ZERO; 4];
        for (rows, word) in Self::QUAD_DOWN.chunks(8).zip(words::<P, 4>(a)) {
            add_lane_rows(&mut sums, rows, word, 8);
        }
        let [first, second, third, fourth] = sums;
        (first ^ second) ^ (third ^ fourth)
    }

    /// The element of `L` that `a` of the field of [`Gcm`] denotes, at degree 256.
    #[inline(always)]
    fn quad_up(a: Poly<U128>) -> Lanes<P> {
        let mut sums = [[0u64; 4]; 2];
        for (rows, word) in Self::QUAD_UP.chunks(8).zip(words::<Gcm, 2>(a)) {
            add_quad_rows(&mut sums, rows, word, 8);
        }
        let [first, second] = sums;
        polynomial::<P, 4>(xor_words(first, second))
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
    polynomial::<P, 2>([low, high])
}

/// The `WORDS` low words of `a`.
#[inline(always)]
fn words<P: Polynomial, const WORDS: usize>(a: Lanes<P>) -> [u64; WORDS] {
    let limbs = a.to_uint().into_limbs();
    core::array::from_fn(|index| limbs.get(index).copied().unwrap_or(0))
}

/// The images of the maps between the subfield of degree `m` of `L` and the field of `K`.
struct Maps {
    /// `down[j]`: the image in `K` of `x^j` of `L`, for a map that is the isomorphism on the
    /// subfield.
    down: [Vector; 256],
    /// `up[j]`: the image in `L` of `x^j` of `K`.
    up: [Vector; 128],
}

impl Maps {
    const ZERO: Self = Self {
        down: [[0; 4]; 256],
        up: [[0; 4]; 128],
    };
}

/// `a + b`.
const fn xor(a: Vector, b: Vector) -> Vector {
    [a[0] ^ b[0], a[1] ^ b[1], a[2] ^ b[2], a[3] ^ b[3]]
}

/// Bit `bit` of `v`.
#[expect(
    clippy::indexing_slicing,
    reason = "const evaluation, with every bit below 256"
)]
const fn bit_of(v: &Vector, bit: usize) -> bool {
    (v[bit / 64] >> (bit % 64)) & 1 == 1
}

/// `x^bit`.
#[expect(
    clippy::indexing_slicing,
    reason = "const evaluation, with every bit below 256"
)]
const fn unit(bit: usize) -> Vector {
    let mut v = [0; 4];
    v[bit / 64] = 1 << (bit % 64);
    v
}

/// Whether `v` is zero.
const fn is_zero(v: &Vector) -> bool {
    (v[0] | v[1] | v[2] | v[3]) == 0
}

/// The position of the highest set bit of the nonzero `v`.
#[expect(
    clippy::indexing_slicing,
    reason = "const evaluation, with every index below 4 for a nonzero `v`"
)]
const fn top_bit(v: &Vector) -> usize {
    let mut word = 3;
    while v[word] == 0 {
        word -= 1;
    }
    64 * word + 63 - v[word].leading_zeros() as usize
}

/// A basis in echelon form of the span of vectors of GF(2)^256, each row with the sum of the
/// inserted vectors it stands for.
struct Echelon {
    /// The row with its highest bit at each position.
    rows: [Vector; 256],
    /// The inserted vectors that each row sums, as bits.
    sums: [Vector; 256],
    /// The positions that hold a row.
    pivots: Vector,
}

impl Echelon {
    const fn new() -> Self {
        Self {
            rows: [[0; 4]; 256],
            sums: [[0; 4]; 256],
            pivots: [0; 4],
        }
    }

    /// `v` less the rows that its bits select from the top down, and the inserted vectors that
    /// those rows sum. A map of `v` that is GF(2)-linear.
    #[expect(
        clippy::indexing_slicing,
        reason = "const evaluation, with every index below 256"
    )]
    const fn reduce(&self, v: Vector) -> (Vector, Vector) {
        let (mut rest, mut sum) = (v, [0; 4]);
        let mut bit = 256;
        while bit > 0 {
            bit -= 1;
            if bit_of(&rest, bit) && bit_of(&self.pivots, bit) {
                rest = xor(rest, self.rows[bit]);
                sum = xor(sum, self.sums[bit]);
            }
        }
        (rest, sum)
    }

    /// Inserts `v`, which stands for the inserted vectors of `sum`, unless the span holds it.
    #[expect(
        clippy::indexing_slicing,
        reason = "const evaluation, with every index below 256"
    )]
    const fn insert(&mut self, v: Vector, sum: Vector) {
        let (rest, reduced) = self.reduce(v);
        if !is_zero(&rest) {
            let bit = top_bit(&rest);
            self.rows[bit] = rest;
            self.sums[bit] = xor(sum, reduced);
            self.pivots = xor(self.pivots, unit(bit));
        }
    }
}

/// `u_levels` of the tower of the field of `degree` and `taps`, and its powers below
/// `2^levels`.
#[expect(
    clippy::indexing_slicing,
    reason = "const evaluation, with every index below 128"
)]
const fn tower_powers(degree: usize, taps: &[usize], levels: usize) -> [Vector; 128] {
    // The columns of `t -> t^2 + t`.
    let mut columns = Echelon::new();
    let mut bit = 0;
    while bit < degree {
        let x = unit(bit);
        columns.insert(xor(quad_square_mod(x, degree, taps), x), x);
        bit += 1;
    }
    let (mut product, mut level) = (unit(0), 0);
    let mut u = [0; 4];
    while level < levels {
        let (rest, root) = columns.reduce(product);
        assert!(
            is_zero(&rest),
            "the polynomials of a field and of its subfield must be irreducible"
        );
        u = root;
        product = quad_mul_mod(&shifts(product), u, degree, taps);
        level += 1;
    }
    let mut powers = [[0; 4]; 128];
    powers[0] = unit(0);
    let multiples = shifts(u);
    let mut index = 1;
    while index < 1 << levels {
        powers[index] = quad_mul_mod(&multiples, powers[index - 1], degree, taps);
        index += 1;
    }
    powers
}

/// The maps between the subfield of degree `sub` of the field of `degree`, up to 256, with `taps`
/// and the field of degree `sub`, a power of two up to 128, with `sub_taps`.
#[expect(
    clippy::indexing_slicing,
    reason = "const evaluation, with every index below 256"
)]
const fn maps(degree: usize, taps: &[usize], sub: usize, sub_taps: &[usize]) -> Maps {
    let levels = sub.trailing_zeros() as usize;
    let large = tower_powers(degree, taps, levels);
    let small = tower_powers(sub, sub_taps, levels);
    // `x^j` of `K` as a sum of powers of its `u_j`, mapped to the same sum of the powers in `L`.
    let mut basis = Echelon::new();
    let mut index = 0;
    while index < sub {
        basis.insert(small[index], unit(index));
        index += 1;
    }
    let mut maps = Maps::ZERO;
    let mut image = Echelon::new();
    let mut bit = 0;
    while bit < sub {
        let (rest, sum) = basis.reduce(unit(bit));
        assert!(
            is_zero(&rest),
            "the polynomials of a field and of its subfield must be irreducible"
        );
        let mut up = [0; 4];
        index = 0;
        while index < sub {
            if bit_of(&sum, index) {
                up = xor(up, large[index]);
            }
            index += 1;
        }
        maps.up[bit] = up;
        image.insert(up, unit(bit));
        bit += 1;
    }
    bit = 0;
    while bit < degree {
        maps.down[bit] = image.reduce(unit(bit)).1;
        bit += 1;
    }
    maps
}

/// The table of the map of `images` on inputs of `bits` bits, up to four words: for each window
/// position, the sum of the images of the bits of every window value. The positions of each word
/// fill an equal share of the table. Zero unless `used`.
#[expect(
    clippy::indexing_slicing,
    reason = "const evaluation, with every index below its array's length"
)]
const fn sums<const POSITIONS: usize, const ENTRIES: usize>(
    used: bool,
    images: &[Vector],
    bits: usize,
) -> [[Vector; ENTRIES]; POSITIONS] {
    let mut table = [[[0; 4]; ENTRIES]; POSITIONS];
    if !used {
        return table;
    }
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
                [0; 4]
            };
            table[position][value] = xor(table[position][value ^ low], image);
            value += 1;
        }
        position += 1;
    }
    table
}

/// [`sums`] with images of two words in vector registers.
#[expect(
    clippy::indexing_slicing,
    reason = "const evaluation, with every index below its array's length"
)]
const fn lane_table<const POSITIONS: usize, const ENTRIES: usize>(
    used: bool,
    images: &[Vector],
    bits: usize,
) -> [[Lane<U128>; ENTRIES]; POSITIONS] {
    let sums = sums::<POSITIONS, ENTRIES>(used, images, bits);
    let mut table = [[Lane::<U128>::ZERO; ENTRIES]; POSITIONS];
    let mut position = 0;
    while position < POSITIONS {
        let mut value = 0;
        while value < ENTRIES {
            let [low, high, _, _] = sums[position][value];
            table[position][value] = Lane::from_poly(Poly::<U128>::from_uint(
                biguint::Uint::from_u128(low as u128 | ((high as u128) << 64)),
            ));
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
    used: bool,
    images: &[Vector],
    bits: usize,
) -> [[u64; ENTRIES]; POSITIONS] {
    let sums = sums::<POSITIONS, ENTRIES>(used, images, bits);
    let mut table = [[0; ENTRIES]; POSITIONS];
    let mut position = 0;
    while position < POSITIONS {
        let mut value = 0;
        while value < ENTRIES {
            table[position][value] = sums[position][value][0];
            value += 1;
        }
        position += 1;
    }
    table
}

/// The two low words of each of the first `N` of `images`; zero unless `used`.
#[expect(
    clippy::indexing_slicing,
    reason = "const evaluation, with every index below `N`, at most the length of `images`"
)]
const fn pairs<const N: usize>(used: bool, images: &[Vector]) -> [[u64; 2]; N] {
    let mut pairs = [[0; 2]; N];
    if !used {
        return pairs;
    }
    let mut bit = 0;
    while bit < N {
        pairs[bit] = [images[bit][0], images[bit][1]];
        bit += 1;
    }
    pairs
}

/// The low word of each of the first `N` of `images`; zero unless `used`.
#[expect(
    clippy::indexing_slicing,
    reason = "const evaluation, with every index below `N`, at most the length of `images`"
)]
const fn singles<const N: usize>(used: bool, images: &[Vector]) -> [u64; N] {
    let mut singles = [0; N];
    if !used {
        return singles;
    }
    let mut bit = 0;
    while bit < N {
        singles[bit] = images[bit][0];
        bit += 1;
    }
    singles
}
