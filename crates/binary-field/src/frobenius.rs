//! Tables of the Frobenius powers `x -> x^(2^k)` that the default inverse of a field of 17 to
//! [`TABLE_DEGREE`] bits applies along its addition chain. Squaring is GF(2)-linear, so a power applies as the sum of
//! the images of the windows of `x`: one lookup per window of 3, 4 or 8 bits. With `precomputed`
//! every power of the chain has a byte table, above 128 bits a nibble table. The default build has the tables of 3 or 4 bits that
//! cost least within its budget, and composes the other powers from them and squarings.
//! `not-precomputed` squares. The lookups index the tables by the value. Without hardware word
//! products, `ConstantTime` sums the images of the single bits of `x` under masks instead, from
//! tables that it plans the same way; with them, it squares.

use core::marker::PhantomData;

use biguint::Uint;
use clmul::{Lane, Poly};
use const_default::ConstDefault;
use generic_array::GenericArray;
use typenum::U128;

use crate::arith::Field;
use crate::chain::{Chain, Chains, MAX_STEPS};
use crate::element::Limbs;
use crate::plan::{Plan, Polynomial, TABLE_BUDGET, TABLE_DEGREE, TIER, Tier};

/// The tables of a field at most.
const MAX_TABLES: usize = 8;

/// The cost of one squaring of `degree` bits, in tenths of a cycle of the chain of inverses: 14
/// cycles on Zen 3 up to two words; 58 cycles in WebAssembly for four words, the one target with
/// tables of four words.
const fn squaring_cost(degree: usize) -> usize {
    if words(degree) == 4 { 580 } else { 140 }
}

/// The words of an image in the tables of a field of `degree` bits: 1, 2 or 4.
const fn words(degree: usize) -> usize {
    if degree <= 64 {
        1
    } else if degree <= 128 {
        2
    } else {
        4
    }
}

/// The cost of one table application to a value of `degree` bits, in tenths of a cycle: about
/// 2 cycles per lookup of a 3-bit window, 2.5 per nibble and 3 per byte, which the moves of the
/// windows into general registers, their indices and the sums take.
const fn apply_cost(window: usize, degree: usize) -> usize {
    let words = words(degree);
    let lookups = words * 64usize.div_ceil(window);
    if words == 4 {
        // WebAssembly cycles against its squarings of four words, [`squaring_cost`]: four loads
        // or four masked words and four sums per lookup.
        return lookups * if window == 1 { 30 } else { 40 };
    }
    lookups
        * match window {
            // A masked image per bit, in WebAssembly cycles against the squarings there, which
            // take about twice those of Zen 3.
            1 => 8,
            3 => 21,
            4 => 25,
            _ => 30,
        }
}

/// How the inverse of a field computes the Frobenius power of each step of its chain.
#[derive(Clone, Copy)]
pub(crate) struct Powers {
    /// The window of the tables in bits: 3, 4, 8, or 0 without tables.
    window: usize,
    /// The tabulated powers, in decreasing order.
    tabulated: [usize; MAX_TABLES],
    /// The number of tabulated powers.
    tables: usize,
    /// For each step, the applications of each table.
    applications: [[usize; MAX_TABLES]; MAX_STEPS],
    /// For each step, the squarings after the tables.
    squarings: [usize; MAX_STEPS],
    /// The cost of the chain's Frobenius powers, in quarter squarings.
    cost: usize,
}

/// Applies each table `$table::<P, T>` to `$x` `$counts[T]` times.
macro_rules! tables {
    ($x:ident, $counts:ident, $table:ident) => {
        tables!(@ $x, $counts, $table; 0 1 2 3 4 5 6 7)
    };
    (@ $x:ident, $counts:ident, $table:ident; $($index:literal)*) => {$(
        if let Some(&count) = $counts.get($index) {
            let mut applied = 0;
            while applied < count {
                $x = $table::<P, $index>::apply($x);
                applied += 1;
            }
        }
    )*};
}

/// The table plan of the field with the polynomial `P`.
pub(crate) struct Frobenius<P>(PhantomData<P>);

impl<P: Polynomial> Frobenius<P> {
    /// The plan of the field's chain: byte tables of every power with `precomputed`; by default,
    /// the cheaper of the plans of 3-bit and 4-bit tables within the budget.
    pub(crate) const POWERS: Powers = {
        let chain = &Chains::<P>::CHAIN;
        let degree = Plan::<P>::DEGREE;
        if degree <= 16 || degree > TABLE_DEGREE {
            powers(chain, 0, degree, 0)
        } else if degree > 128 {
            // The byte tables of four words take 256 KiB a power; the nibble tables 32 KiB, above
            // the default budget.
            match TIER {
                Tier::Full => powers(chain, 4, degree, usize::MAX),
                Tier::Budget | Tier::None => powers(chain, 0, degree, 0),
            }
        } else {
            match TIER {
                Tier::Full => powers(chain, 8, degree, usize::MAX),
                Tier::Budget => {
                    let nibbles = powers(chain, 4, degree, TABLE_BUDGET);
                    let triples = powers(chain, 3, degree, TABLE_BUDGET);
                    if triples.cost < nibbles.cost {
                        triples
                    } else {
                        nibbles
                    }
                }
                Tier::None => powers(chain, 0, degree, 0),
            }
        }
    };

    /// The plan of the constant-time inverse: tables of the images of single bits, within the
    /// budget that [`Self::POWERS`] leaves, for builds without hardware word products.
    pub(crate) const CT_POWERS: Powers = {
        let chain = &Chains::<P>::CHAIN;
        let degree = Plan::<P>::DEGREE;
        if degree <= 16 || degree > TABLE_DEGREE || clmul::HARDWARE {
            powers(chain, 0, degree, 0)
        } else if degree > 128 {
            // The powers of the nibble tables, whose images [`QuadImages`] computes once for both.
            let mut plan = Self::POWERS;
            if plan.window > 0 {
                plan.window = 1;
            }
            plan
        } else {
            let used = Self::POWERS.tables * table_bytes(Self::POWERS.window, degree);
            match TIER {
                Tier::Full => powers(chain, 1, degree, usize::MAX),
                Tier::Budget => powers(chain, 1, degree, TABLE_BUDGET.saturating_sub(used)),
                Tier::None => powers(chain, 0, degree, 0),
            }
        }
    };

    /// `x^(2^power)` for step `step` of the chain: the step's table applications, then squarings.
    /// A `step` past the chain squares `power` times.
    #[inline(always)]
    pub(crate) fn apply(x: Poly<P::Degree>, step: usize, power: usize) -> Poly<P::Degree> {
        let mut x = x;
        let (counts, squarings) = Self::POWERS.step(step, power);
        tables!(x, counts, Table);
        Self::square(x, squarings)
    }

    /// [`Self::apply`] in constant time, with the tables of [`Self::CT_POWERS`].
    #[inline(always)]
    pub(crate) fn apply_ct(x: Poly<P::Degree>, step: usize, power: usize) -> Poly<P::Degree> {
        if Self::CT_POWERS.window == 0 {
            return Self::square(x, power);
        }
        let mut x = x;
        let (counts, squarings) = Self::CT_POWERS.step(step, power);
        tables!(x, counts, BitTable);
        Self::square(x, squarings)
    }

    /// `x^(2^squarings)` by squarings.
    #[inline(always)]
    fn square(x: Poly<P::Degree>, squarings: usize) -> Poly<P::Degree> {
        let mut x = x;
        let mut squared = 0;
        while squared < squarings {
            x = Field::<P>::run_square(x);
            squared += 1;
        }
        x
    }
}

impl Powers {
    /// The table applications and the squarings of step `step` of `power`; `power` squarings
    /// past the chain or without tables.
    #[inline(always)]
    fn step(&self, step: usize, power: usize) -> ([usize; MAX_TABLES], usize) {
        match (self.applications.get(step), self.squarings.get(step)) {
            (Some(counts), Some(&squarings)) if self.window > 0 => (*counts, squarings),
            _ => ([0; MAX_TABLES], power),
        }
    }
}

/// The plan of `chain` with tables of `window` bits in a field of degree `degree`: the set of its
/// distinct powers of the least cost within `budget` bytes. A window of 0 plans no table.
#[expect(
    clippy::indexing_slicing,
    reason = "const evaluation, with every index below its array's length"
)]
const fn powers(chain: &Chain, window: usize, degree: usize, budget: usize) -> Powers {
    let mut plan = Powers {
        window,
        tabulated: [0; MAX_TABLES],
        tables: 0,
        applications: [[0; MAX_TABLES]; MAX_STEPS],
        squarings: [0; MAX_STEPS],
        cost: 0,
    };
    let Some(len) = chain.len else {
        return plan;
    };
    // The distinct powers of 2 or more, in decreasing order.
    let mut distinct = [0; MAX_STEPS];
    let mut count = 0;
    let mut step = 0;
    while step < len {
        let power = chain.steps[step].power;
        let mut seen = power < 2;
        let mut index = 0;
        while index < count {
            if distinct[index] == power {
                seen = true;
            }
            index += 1;
        }
        if !seen {
            let mut position = count;
            while position > 0 && distinct[position - 1] < power {
                distinct[position] = distinct[position - 1];
                position -= 1;
            }
            distinct[position] = power;
            count += 1;
        }
        step += 1;
    }
    if window == 0 {
        count = 0;
    }
    if count > MAX_TABLES {
        count = MAX_TABLES;
    }
    let size = table_bytes(window, degree);
    let mut best = 0usize;
    let mut best_cost = usize::MAX;
    let mut subset = 0usize;
    while subset < 1 << count {
        let mut chosen = [0; MAX_TABLES];
        let mut tables = 0;
        let mut index = 0;
        while index < count {
            if (subset >> index) & 1 == 1 {
                chosen[tables] = distinct[index];
                tables += 1;
            }
            index += 1;
        }
        if tables * size <= budget {
            let cost = chain_cost(chain, len, &chosen, tables, window, degree);
            if cost < best_cost {
                best_cost = cost;
                best = subset;
            }
        }
        subset += 1;
    }
    let mut index = 0;
    while index < count {
        if (best >> index) & 1 == 1 {
            plan.tabulated[plan.tables] = distinct[index];
            plan.tables += 1;
        }
        index += 1;
    }
    step = 0;
    while step < len {
        let (counts, squarings) = decompose(chain.steps[step].power, &plan.tabulated, plan.tables);
        plan.applications[step] = counts;
        plan.squarings[step] = squarings;
        step += 1;
    }
    plan.cost = best_cost;
    plan
}

/// The cost of the Frobenius powers of `chain` with the tables of `chosen`.
#[expect(
    clippy::indexing_slicing,
    reason = "const evaluation, with every index below its array's length"
)]
const fn chain_cost(
    chain: &Chain,
    len: usize,
    chosen: &[usize; MAX_TABLES],
    tables: usize,
    window: usize,
    degree: usize,
) -> usize {
    let mut cost = 0;
    let mut step = 0;
    while step < len {
        let (counts, squarings) = decompose(chain.steps[step].power, chosen, tables);
        let mut index = 0;
        while index < tables {
            cost += counts[index] * apply_cost(window, degree);
            index += 1;
        }
        cost += squarings * squaring_cost(degree);
        step += 1;
    }
    cost
}

/// `power` as applications of the tables of `tabulated`, the largest first, and the squarings
/// that remain.
#[expect(
    clippy::indexing_slicing,
    reason = "const evaluation, with every index below its array's length"
)]
const fn decompose(
    power: usize,
    tabulated: &[usize; MAX_TABLES],
    tables: usize,
) -> ([usize; MAX_TABLES], usize) {
    let mut counts = [0; MAX_TABLES];
    let mut rest = power;
    let mut index = 0;
    while index < tables {
        if let Some(count) = rest.checked_div(tabulated[index]) {
            counts[index] = count;
            rest -= count * tabulated[index];
        }
        index += 1;
    }
    (counts, rest)
}

/// The bytes of one table of `window` bits for a degree up to 256.
const fn table_bytes(window: usize, degree: usize) -> usize {
    if window == 0 {
        return 0;
    }
    let words = words(degree);
    if window == 1 {
        // One image per bit.
        return words * 64 * 8 * words;
    }
    words * 64usize.div_ceil(window) * (1 << window) * 8 * words
}

/// The table of the tabulated power `T` of the field with the polynomial `P`, in the type of its
/// window and width; the other five types hold nothing.
struct Table<P, const T: usize>(PhantomData<P>);

/// Tables of 3-bit windows of fields of one word: 22 windows, the last of one bit.
type WordTriples = [[u64; 8]; 22];
/// Nibble tables of fields of one word.
type WordNibbles = [[u64; 16]; 16];
/// Byte tables of fields of one word.
type WordBytes = [[u64; 256]; 8];
/// Tables of 3-bit windows of fields of two words: 22 windows per word.
type LaneTriples = [[Lane<U128>; 8]; 44];
/// Nibble tables of fields of two words.
type LaneNibbles = [[Lane<U128>; 16]; 32];
/// Byte tables of fields of two words.
type LaneBytes = [[Lane<U128>; 256]; 16];
/// Nibble tables of fields of three or four words: 16 windows per word.
type QuadNibbles = [[[u64; 4]; 16]; 64];

impl<P: Polynomial, const T: usize> Table<P, T> {
    /// The power, or 0 past the tabulated ones.
    const POWER: usize = {
        let plan = Frobenius::<P>::POWERS;
        if T < plan.tables {
            match plan.tabulated.split_at(T).1 {
                [power, ..] => *power,
                [] => 0,
            }
        } else {
            0
        }
    };
    /// Whether the table holds one word per entry.
    const WORD: bool = Plan::<P>::DEGREE <= 64;
    /// Whether the table holds four words per entry.
    const QUAD: bool = Plan::<P>::DEGREE > 128;
    /// The window of the field's tables.
    const WINDOW: usize = Frobenius::<P>::POWERS.window;

    const WORD_TRIPLES: &'static WordTriples = &word_table::<22, 8>(
        Self::WORD && Self::WINDOW == 3,
        Plan::<P>::DEGREE,
        Plan::<P>::TAPS,
        Self::POWER,
    );
    const WORD_NIBBLES: &'static WordNibbles = &word_table::<16, 16>(
        Self::WORD && Self::WINDOW == 4,
        Plan::<P>::DEGREE,
        Plan::<P>::TAPS,
        Self::POWER,
    );
    const WORD_BYTES: &'static WordBytes = &word_table::<8, 256>(
        Self::WORD && Self::WINDOW == 8,
        Plan::<P>::DEGREE,
        Plan::<P>::TAPS,
        Self::POWER,
    );
    const LANE_TRIPLES: &'static LaneTriples = &lane_table::<44, 8>(
        !Self::WORD && !Self::QUAD && Self::WINDOW == 3,
        Plan::<P>::DEGREE,
        Plan::<P>::TAPS,
        Self::POWER,
    );
    const LANE_NIBBLES: &'static LaneNibbles = &lane_table::<32, 16>(
        !Self::WORD && !Self::QUAD && Self::WINDOW == 4,
        Plan::<P>::DEGREE,
        Plan::<P>::TAPS,
        Self::POWER,
    );
    const LANE_BYTES: &'static LaneBytes = &lane_table::<16, 256>(
        !Self::WORD && !Self::QUAD && Self::WINDOW == 8,
        Plan::<P>::DEGREE,
        Plan::<P>::TAPS,
        Self::POWER,
    );
    const QUAD_NIBBLES: &'static QuadNibbles = &quad_table(
        Self::QUAD && Self::WINDOW == 4,
        Plan::<P>::DEGREE,
        QuadImages::<P, T>::IMAGES,
    );

    /// `x^(2^POWER)`: the images of the windows of `x`, summed.
    #[inline(always)]
    fn apply(x: Poly<P::Degree>) -> Poly<P::Degree> {
        let limbs = x.to_uint().into_limbs();
        if Self::QUAD {
            return polynomial::<P, 4>(quad_sum(Self::QUAD_NIBBLES, &limbs));
        }
        let low = limbs.first().copied().unwrap_or(0);
        let high = limbs.get(1).copied().unwrap_or(0);
        if Self::WORD {
            let sum = match Self::WINDOW {
                3 => word_sum(Self::WORD_TRIPLES, low, 3),
                4 => word_sum(Self::WORD_NIBBLES, low, 4),
                _ => word_sum(Self::WORD_BYTES, low, 8),
            };
            polynomial::<P, 2>([sum, 0])
        } else {
            let sum = match Self::WINDOW {
                3 => lane_sum(Self::LANE_TRIPLES, low, high, 3),
                4 => lane_sum(Self::LANE_NIBBLES, low, high, 4),
                _ => lane_sum(Self::LANE_BYTES, low, high, 8),
            };
            sum.into_poly().resize()
        }
    }
}

/// The images of the bits of an element under the tabulated power `T` of a field of three or four
/// words, which [`Table`] and [`BitTable`] share: the two plans tabulate the same powers there.
struct QuadImages<P, const T: usize>(PhantomData<P>);

impl<P: Polynomial, const T: usize> QuadImages<P, T> {
    /// The images; zero for the other fields and past the tabulated powers.
    const IMAGES: &'static [[u64; 4]; 256] = &quad_images(
        Plan::<P>::DEGREE > 128,
        Plan::<P>::DEGREE,
        Plan::<P>::TAPS,
        Table::<P, T>::POWER,
    );
}

/// The table of the tabulated power `T` of [`Frobenius::CT_POWERS`]: the image of each bit of an
/// element, which [`Self::apply`] adds under the mask of the bit.
struct BitTable<P, const T: usize>(PhantomData<P>);

impl<P: Polynomial, const T: usize> BitTable<P, T> {
    /// The power, or 0 past the tabulated ones.
    const POWER: usize = {
        let plan = Frobenius::<P>::CT_POWERS;
        if T < plan.tables {
            match plan.tabulated.split_at(T).1 {
                [power, ..] => *power,
                [] => 0,
            }
        } else {
            0
        }
    };
    /// Whether an image is one word.
    const WORD: bool = Plan::<P>::DEGREE <= 64;
    /// The images of the bits of a field of one word; zero for the other fields.
    const WORD_IMAGES: &'static [u64; 64] =
        &word_images(Self::WORD, Plan::<P>::DEGREE, Plan::<P>::TAPS, Self::POWER);
    /// The images of the bits of a field of two words; zero for the other fields.
    const LANE_IMAGES: &'static [u128; 128] = &if Self::WORD || Self::QUAD || Self::POWER == 0 {
        [0; 128]
    } else {
        images(Plan::<P>::DEGREE, Plan::<P>::TAPS, Self::POWER)
    };
    /// Whether an image is four words.
    const QUAD: bool = Plan::<P>::DEGREE > 128;
    /// The images of the bits of a field of three or four words; zero for the other fields.
    const QUAD_IMAGES: &'static [[u64; 4]; 256] = QuadImages::<P, T>::IMAGES;

    /// `x^(2^POWER)`: the sum of the images of the set bits of `x`, each under the mask of its
    /// bit, in four sums. No address and no branch depends on `x`.
    #[inline(always)]
    fn apply(x: Poly<P::Degree>) -> Poly<P::Degree> {
        let limbs = x.to_uint().into_limbs();
        if Self::QUAD {
            return polynomial::<P, 4>(quad_masked_sum(Self::QUAD_IMAGES, &limbs));
        }
        let low = limbs.first().copied().unwrap_or(0);
        let high = limbs.get(1).copied().unwrap_or(0);
        if Self::WORD {
            let mask = |bits: u64, bit: u32| ((bits >> bit) & 1).wrapping_neg();
            let [mut first, mut second, mut third, mut fourth] = [0u64; 4];
            let (chunks, _) = Self::WORD_IMAGES.as_chunks::<4>();
            for (index, [a, b, c, d]) in chunks.iter().enumerate() {
                let bits = low >> (4 * index);
                first ^= a & mask(bits, 0);
                second ^= b & mask(bits, 1);
                third ^= c & mask(bits, 2);
                fourth ^= d & mask(bits, 3);
            }
            polynomial::<P, 2>([(first ^ second) ^ (third ^ fourth), 0])
        } else {
            let mask = |bits: u64, bit: u32| ((bits >> bit) & 1).wrapping_neg();
            let mut sums = [[0u64; 2]; 4];
            let (chunks, _) = Self::LANE_IMAGES.as_chunks::<4>();
            for (index, images) in chunks.iter().enumerate() {
                let bits = if index < 16 { low } else { high } >> (4 * (index % 16));
                for (bit, (sum, image)) in sums.iter_mut().zip(images).enumerate() {
                    let mask = mask(bits, bit as u32);
                    let [low_sum, high_sum] = sum;
                    *low_sum ^= (*image as u64) & mask;
                    *high_sum ^= ((*image >> 64) as u64) & mask;
                }
            }
            let [first, second, third, fourth] = sums;
            polynomial::<P, 2>([
                (first[0] ^ second[0]) ^ (third[0] ^ fourth[0]),
                (first[1] ^ second[1]) ^ (third[1] ^ fourth[1]),
            ])
        }
    }
}

/// The images of the bits of an element of a field of one word, as [`images`]; zero unless
/// `used`.
#[expect(
    clippy::indexing_slicing,
    reason = "const evaluation, with every index below its array's length"
)]
const fn word_images(used: bool, degree: usize, taps: &[usize], power: usize) -> [u64; 64] {
    let mut words = [0; 64];
    if !used || power == 0 {
        return words;
    }
    let images = images(degree, taps, power);
    let mut bit = 0;
    while bit < 64 {
        words[bit] = images[bit] as u64;
        bit += 1;
    }
    words
}

/// The sum of the rows of `table` that the windows of `word` select.
#[inline(always)]
fn word_sum<const POSITIONS: usize, const ENTRIES: usize>(
    table: &[[u64; ENTRIES]; POSITIONS],
    word: u64,
    window: u32,
) -> u64 {
    let mut sums = [0u64; 4];
    add_word_rows(&mut sums, table, word, window);
    let [first, second, third, fourth] = sums;
    (first ^ second) ^ (third ^ fourth)
}

/// Adds the images that the windows of `word` select in `rows` to four sums, a row to each in
/// turn: four rows at a time, so that the sums stay in registers when the loop stays rolled.
#[inline(always)]
pub(crate) fn add_word_rows<const ENTRIES: usize>(
    sums: &mut [u64; 4],
    rows: &[[u64; ENTRIES]],
    word: u64,
    window: u32,
) {
    let mask = (ENTRIES as u64) - 1;
    let image = |row: &[u64; ENTRIES], position: usize| {
        let index = ((word >> (window * position as u32)) & mask) as usize;
        row.get(index).copied().unwrap_or(0)
    };
    let [first, second, third, fourth] = sums;
    let (chunks, remainder) = rows.as_chunks::<4>();
    let mut position = 0;
    for [a, b, c, d] in chunks {
        *first ^= image(a, position);
        *second ^= image(b, position + 1);
        *third ^= image(c, position + 2);
        *fourth ^= image(d, position + 3);
        position += 4;
    }
    for row in remainder {
        *first ^= image(row, position);
        position += 1;
    }
}

/// The sum of the rows of `table` that the windows of `low + high x^64` select: the first half of
/// the rows windows `low`, the second half `high`.
#[inline(always)]
fn lane_sum<const POSITIONS: usize, const ENTRIES: usize>(
    table: &[[Lane<U128>; ENTRIES]; POSITIONS],
    low: u64,
    high: u64,
    window: u32,
) -> Lane<U128> {
    let mut sums = [Poly::<U128>::ZERO; 4];
    let (low_rows, high_rows) = table.split_at(POSITIONS / 2);
    add_lane_rows(&mut sums, low_rows, low, window);
    add_lane_rows(&mut sums, high_rows, high, window);
    let [first, second, third, fourth] = sums;
    Lane::from_poly((first ^ second) ^ (third ^ fourth))
}

/// Adds the images that the windows of `word` select in `rows` to four sums, as `add_word_rows`.
#[inline(always)]
pub(crate) fn add_lane_rows<const ENTRIES: usize>(
    sums: &mut [Poly<U128>; 4],
    rows: &[[Lane<U128>; ENTRIES]],
    word: u64,
    window: u32,
) {
    let mask = (ENTRIES as u64) - 1;
    let image = |row: &[Lane<U128>; ENTRIES], position: usize| {
        let index = ((word >> (window * position as u32)) & mask) as usize;
        row.get(index)
            .map_or(Poly::<U128>::ZERO, |&lane| lane.into_poly())
    };
    let [first, second, third, fourth] = sums;
    let (chunks, remainder) = rows.as_chunks::<4>();
    let mut position = 0;
    for [a, b, c, d] in chunks {
        *first ^= image(a, position);
        *second ^= image(b, position + 1);
        *third ^= image(c, position + 2);
        *fourth ^= image(d, position + 3);
        position += 4;
    }
    for row in remainder {
        *first ^= image(row, position);
        position += 1;
    }
}

/// The polynomial of the field with the polynomial `P` whose low words are `words`.
#[inline(always)]
fn polynomial<P: Polynomial, const WORDS: usize>(words: [u64; WORDS]) -> Poly<P::Degree> {
    let mut limbs = GenericArray::<u64, Limbs<P>>::DEFAULT;
    for (slot, word) in limbs.iter_mut().zip(words) {
        *slot = word;
    }
    Poly::from_uint(Uint::from_generic_limbs(limbs))
}

/// The images of the basis elements `x^j` under `a -> a^(2^power)` in the field of degree
/// `degree`, up to 128, with `taps`.
#[expect(
    clippy::indexing_slicing,
    reason = "const evaluation, with every index below its array's length"
)]
pub(crate) const fn images(degree: usize, taps: &[usize], power: usize) -> [u128; 128] {
    let mut images = [0u128; 128];
    let mut bit = 0;
    while bit < degree && bit < 128 {
        images[bit] = 1 << bit;
        bit += 1;
    }
    let mut squaring = 0;
    while squaring < power {
        bit = 0;
        while bit < degree && bit < 128 {
            images[bit] = square_mod(images[bit], degree, taps);
            bit += 1;
        }
        squaring += 1;
    }
    images
}

/// The sum of the rows of `table` that the nibbles of `limbs`, up to four words, select. Two sums
/// alternate, so that consecutive lookups add in parallel.
#[inline(always)]
fn quad_sum(table: &QuadNibbles, limbs: &[u64]) -> [u64; 4] {
    let mut sums = [[0u64; 4]; 2];
    for (position, row) in table.iter().enumerate() {
        let word = limbs.get(position / 16).copied().unwrap_or(0);
        let index = ((word >> (4 * (position % 16))) & 15) as usize;
        if let (Some(image), Some(sum)) = (row.get(index), sums.get_mut(position % 2)) {
            for (slot, &limb) in sum.iter_mut().zip(image) {
                *slot ^= limb;
            }
        }
    }
    let [first, second] = sums;
    xor_words(first, second)
}

/// `a + b` word by word.
#[inline(always)]
fn xor_words(a: [u64; 4], b: [u64; 4]) -> [u64; 4] {
    let ([a0, a1, a2, a3], [b0, b1, b2, b3]) = (a, b);
    [a0 ^ b0, a1 ^ b1, a2 ^ b2, a3 ^ b3]
}

/// The sum of the images of the set bits of `limbs`, up to four words, each image under the mask
/// of its bit, in four sums. No address and no branch depends on `limbs`.
#[inline(always)]
fn quad_masked_sum(images: &[[u64; 4]; 256], limbs: &[u64]) -> [u64; 4] {
    let mut sums = [[0u64; 4]; 4];
    let (chunks, _) = images.as_chunks::<4>();
    for (index, group) in chunks.iter().enumerate() {
        let bits = limbs.get(index / 16).copied().unwrap_or(0) >> (4 * (index % 16));
        for (bit, (sum, image)) in sums.iter_mut().zip(group).enumerate() {
            let mask = ((bits >> bit) & 1).wrapping_neg();
            // Spelled out: a loop over the words would let LLVM unswitch it on the mask, a branch
            // on the bit.
            let [first, second, third, fourth] = sum;
            let [a, b, c, d] = *image;
            *first ^= a & mask;
            *second ^= b & mask;
            *third ^= c & mask;
            *fourth ^= d & mask;
        }
    }
    let [first, second, third, fourth] = sums;
    xor_words(xor_words(first, second), xor_words(third, fourth))
}

/// The images of the basis elements `x^j` under `a -> a^(2^power)` in the field of degree
/// `degree`, from 129 to 256, with `taps`, as four words each; zero unless `used`.
#[expect(
    clippy::indexing_slicing,
    reason = "const evaluation, with every index below its array's length"
)]
const fn quad_images(used: bool, degree: usize, taps: &[usize], power: usize) -> [[u64; 4]; 256] {
    let mut images = [[0u64; 4]; 256];
    if !used || power == 0 {
        return images;
    }
    // The image of `x^j` is `y^j` with `y = x^(2^power)`: the square of the image of `x^(j / 2)`
    // for an even `j`, and the product of the image of `x^(j - 1)` and `y` for an odd one.
    let mut y = [0u64; 4];
    y[0] = 2;
    let mut squaring = 0;
    while squaring < power {
        y = quad_square_mod(y, degree, taps);
        squaring += 1;
    }
    let shifted = shifts(y);
    images[0][0] = 1;
    let mut bit = 1;
    while bit < degree {
        images[bit] = if bit % 2 == 0 {
            quad_square_mod(images[bit / 2], degree, taps)
        } else {
            quad_mul_mod(&shifted, images[bit - 1], degree, taps)
        };
        bit += 1;
    }
    images
}

/// The carry-less products of `a` and every byte, each on five words. Constant evaluation runs a
/// product by them a byte of the multiplier at a time.
#[expect(
    clippy::indexing_slicing,
    reason = "const evaluation, with every index below its array's length"
)]
const fn shifts(a: [u64; 4]) -> [[u64; 5]; 256] {
    let mut multiples = [[0u64; 5]; 256];
    let mut value: usize = 1;
    while value < 256 {
        // Each value is a smaller one plus its lowest bit.
        let low = value & value.wrapping_neg();
        let shift = low.trailing_zeros();
        let mut word = 0;
        while word < 5 {
            let shifted = if word < 4 { a[word] << shift } else { 0 };
            let carried = if word > 0 && shift > 0 {
                a[word - 1] >> (64 - shift)
            } else {
                0
            };
            multiples[value][word] = multiples[value ^ low][word] ^ shifted ^ carried;
            word += 1;
        }
        value += 1;
    }
    multiples
}

/// `a b mod x^degree + r` for `a` and `b` below `x^degree`, a degree from 129 to 256, with the
/// [`shifts`] of `a`: the carry-less product by the bytes of `b`.
#[expect(
    clippy::indexing_slicing,
    reason = "const evaluation, with every index below its array's length"
)]
const fn quad_mul_mod(
    multiples: &[[u64; 5]; 256],
    b: [u64; 4],
    degree: usize,
    taps: &[usize],
) -> [u64; 4] {
    let mut product = [0u64; 8];
    let mut byte = 0;
    while byte < 32 {
        let multiple = &multiples[((b[byte / 8] >> (8 * (byte % 8))) & 0xFF) as usize];
        let (word, shift) = (byte / 8, 8 * (byte % 8));
        let mut index = 0;
        while index < 5 {
            product[word + index] ^= multiple[index] << shift;
            // The top word of a multiple holds 7 bits, which a shift keeps within the word.
            if shift > 0 && word + index + 1 < 8 {
                product[word + index + 1] ^= multiple[index] >> (64 - shift);
            }
            index += 1;
        }
        byte += 1;
    }
    quad_reduce(product, degree, taps)
}

/// The nibble table of the power whose bit `images` are given, in fields of three or four words, as
/// `word_table`; zero unless `used`.
#[expect(
    clippy::indexing_slicing,
    reason = "const evaluation, with every index below its array's length"
)]
const fn quad_table(used: bool, degree: usize, images: &[[u64; 4]; 256]) -> QuadNibbles {
    let mut table = [[[0u64; 4]; 16]; 64];
    if !used {
        return table;
    }
    let mut position = 0;
    while position < 64 {
        let mut value: usize = 1;
        while value < 16 {
            let low = value & value.wrapping_neg();
            let bit = 4 * position + low.trailing_zeros() as usize;
            let mut word = 0;
            while word < 4 {
                let image = if bit < degree { images[bit][word] } else { 0 };
                table[position][value][word] = table[position][value ^ low][word] ^ image;
                word += 1;
            }
            value += 1;
        }
        position += 1;
    }
    table
}

/// `a^2 mod x^degree + r` for `a` below `x^degree`, a degree from 129 to 256, as [`square_mod`]
/// on eight words.
#[expect(
    clippy::indexing_slicing,
    reason = "const evaluation, with every index below its array's length"
)]
const fn quad_square_mod(a: [u64; 4], degree: usize, taps: &[usize]) -> [u64; 4] {
    let mut value = [0u64; 8];
    let mut word = 0;
    while word < 4 {
        let spread = spread(a[word]);
        value[2 * word] = spread as u64;
        value[2 * word + 1] = (spread >> 64) as u64;
        word += 1;
    }
    quad_reduce(value, degree, taps)
}

/// `value mod x^degree + r` for `value` of up to 512 bits, a degree from 129 to 256: the part at
/// and above `x^degree` folds down by `r` until none is left.
#[expect(
    clippy::indexing_slicing,
    reason = "const evaluation, with every index below its array's length"
)]
const fn quad_reduce(value: [u64; 8], degree: usize, taps: &[usize]) -> [u64; 4] {
    let mut value = value;
    loop {
        // The part at and above `x^degree` folds down by `r`.
        let top = shift_words(&value, degree as isize);
        let mut any = false;
        let mut index = 0;
        while index < 8 {
            any |= top[index] != 0;
            index += 1;
        }
        if !any {
            return [value[0], value[1], value[2], value[3]];
        }
        index = 0;
        while index < 8 {
            let low = index * 64;
            value[index] &= if low + 64 <= degree {
                u64::MAX
            } else if low >= degree {
                0
            } else {
                (1 << (degree - low)) - 1
            };
            index += 1;
        }
        let mut rest = taps;
        while let [tap, tail @ ..] = rest {
            let added = shift_words(&top, -(*tap as isize));
            index = 0;
            while index < 8 {
                value[index] ^= added[index];
                index += 1;
            }
            rest = tail;
        }
    }
}

/// `value >> shift` for a positive `shift`, `value << -shift` for a negative one, on eight words.
#[expect(
    clippy::indexing_slicing,
    reason = "const evaluation, with every index below its array's length"
)]
const fn shift_words(value: &[u64; 8], shift: isize) -> [u64; 8] {
    let mut out = [0u64; 8];
    let amount = shift.unsigned_abs();
    let (words, offset) = (amount / 64, (amount % 64) as u32);
    let mut index = 0;
    while index < 8 {
        // Bit `b` of `out` is bit `b + shift` of `value`.
        let word = if shift >= 0 {
            index as isize + words as isize
        } else {
            index as isize - words as isize
        };
        out[index] = if offset == 0 {
            word_at(value, word)
        } else if shift >= 0 {
            (word_at(value, word) >> offset) | (word_at(value, word + 1) << (64 - offset))
        } else {
            (word_at(value, word) << offset) | (word_at(value, word - 1) >> (64 - offset))
        };
        index += 1;
    }
    out
}

/// Word `word` of `value`, zero outside it.
#[expect(
    clippy::indexing_slicing,
    reason = "const evaluation, with the index checked against the length"
)]
const fn word_at(value: &[u64; 8], word: isize) -> u64 {
    if word >= 0 && word < 8 {
        value[word as usize]
    } else {
        0
    }
}

/// The table of a power in fields of one word: for each window position, the sum of the images of
/// the bits of every window value. Zero unless `used`.
#[expect(
    clippy::indexing_slicing,
    reason = "const evaluation, with every index below its array's length"
)]
const fn word_table<const POSITIONS: usize, const ENTRIES: usize>(
    used: bool,
    degree: usize,
    taps: &[usize],
    power: usize,
) -> [[u64; ENTRIES]; POSITIONS] {
    let mut table = [[0; ENTRIES]; POSITIONS];
    if !used || power == 0 {
        return table;
    }
    let images = images(degree, taps, power);
    let window = ENTRIES.trailing_zeros() as usize;
    let mut position = 0;
    while position < POSITIONS {
        // Each value is a smaller one plus its lowest bit.
        let mut value: usize = 1;
        while value < ENTRIES {
            let low = value & value.wrapping_neg();
            let bit = window * position + low.trailing_zeros() as usize;
            let image = if bit < degree { images[bit] as u64 } else { 0 };
            table[position][value] = table[position][value ^ low] ^ image;
            value += 1;
        }
        position += 1;
    }
    table
}

/// The table of a power in fields of two words, as `word_table`.
#[expect(
    clippy::indexing_slicing,
    reason = "const evaluation, with every index below its array's length"
)]
const fn lane_table<const POSITIONS: usize, const ENTRIES: usize>(
    used: bool,
    degree: usize,
    taps: &[usize],
    power: usize,
) -> [[Lane<U128>; ENTRIES]; POSITIONS] {
    let zero = Lane::<U128>::ZERO;
    let mut table = [[zero; ENTRIES]; POSITIONS];
    if !used || power == 0 {
        return table;
    }
    let images = images(degree, taps, power);
    let window = ENTRIES.trailing_zeros() as usize;
    let mut position = 0;
    while position < POSITIONS {
        let mut sums = [0u128; ENTRIES];
        let mut value: usize = 1;
        while value < ENTRIES {
            let low = value & value.wrapping_neg();
            // Each word has its own windows; a window that would cross a word ends at it.
            let offset = window * (position % (POSITIONS / 2)) + low.trailing_zeros() as usize;
            let bit = 64 * (position / (POSITIONS / 2)) + offset;
            let image = if offset < 64 && bit < degree {
                images[bit]
            } else {
                0
            };
            sums[value] = sums[value ^ low] ^ image;
            table[position][value] =
                Lane::from_poly(Poly::<U128>::from_uint(Uint::from_u128(sums[value])));
            value += 1;
        }
        position += 1;
    }
    table
}

/// `a^2 mod x^degree + r` for `a` below `x^degree`, a degree up to 128: the square spreads the
/// bits, and the part at and above `x^degree` folds down by `r` until none is left.
pub(crate) const fn square_mod(a: u128, degree: usize, taps: &[usize]) -> u128 {
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
