//! Tables of the Frobenius powers `x -> x^(2^k)` that the default inverse of a field of 17 to 128
//! bits applies along its addition chain. Squaring is GF(2)-linear, so a power applies as the sum of
//! the images of the windows of `x`: one lookup per window of 3, 4 or 8 bits. With `precomputed`
//! every power of the chain has a byte table. The default build has the tables of 3 or 4 bits that
//! cost least within its budget, and composes the other powers from them and squarings.
//! `not-precomputed` squares. The lookups index the tables by the value, so `ConstantTime` squares.

use core::marker::PhantomData;

use biguint::Uint;
use clmul::{Lane, Poly};
use const_default::ConstDefault;
use generic_array::GenericArray;
use typenum::U128;

use crate::arith::Field;
use crate::chain::{Chain, Chains, MAX_STEPS};
use crate::element::Limbs;
use crate::plan::{Plan, Polynomial, TABLE_BUDGET, TIER, Tier};

/// The tables of a field at most.
const MAX_TABLES: usize = 8;

/// The cost of one squaring, in tenths of a cycle of the chain of inverses on Zen 3: 14 cycles.
const SQUARING_COST: usize = 140;

/// The cost of one table application to a value of `degree` bits, in tenths of a cycle: about
/// 2 cycles per lookup of a 3-bit window, 2.5 per nibble and 3 per byte, which the moves of the
/// windows into general registers, their indices and the sums take.
const fn apply_cost(window: usize, degree: usize) -> usize {
    let words = if degree <= 64 { 1 } else { 2 };
    let lookups = words * 64usize.div_ceil(window);
    lookups
        * match window {
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

/// The table plan of the field with the polynomial `P`.
pub(crate) struct Frobenius<P>(PhantomData<P>);

impl<P: Polynomial> Frobenius<P> {
    /// The plan of the field's chain: byte tables of every power with `precomputed`; by default,
    /// the cheaper of the plans of 3-bit and 4-bit tables within the budget.
    pub(crate) const POWERS: Powers = {
        let chain = &Chains::<P>::CHAIN;
        let degree = Plan::<P>::DEGREE;
        if degree <= 16 || degree > 128 {
            powers(chain, 0, degree, 0)
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

    /// `x^(2^power)` for step `step` of the chain: the step's table applications, then squarings.
    /// A `step` past the chain squares `power` times.
    #[inline(always)]
    pub(crate) fn apply(x: Poly<P::Degree>, step: usize, power: usize) -> Poly<P::Degree> {
        let mut x = x;
        let (counts, squarings) = match (
            Self::POWERS.applications.get(step),
            Self::POWERS.squarings.get(step),
        ) {
            (Some(counts), Some(&squarings)) if Self::POWERS.window > 0 => (*counts, squarings),
            _ => ([0; MAX_TABLES], power),
        };
        macro_rules! tables {
            ($($table:literal)*) => {$(
                if let Some(&count) = counts.get($table) {
                    let mut applied = 0;
                    while applied < count {
                        x = Table::<P, $table>::apply(x);
                        applied += 1;
                    }
                }
            )*};
        }
        tables!(0 1 2 3 4 5 6 7);
        let mut squared = 0;
        while squared < squarings {
            x = Field::<P>::run_square(x);
            squared += 1;
        }
        x
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
        cost += squarings * SQUARING_COST;
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

/// The bytes of one table of `window` bits for a degree up to 128.
const fn table_bytes(window: usize, degree: usize) -> usize {
    if window == 0 {
        return 0;
    }
    let words = if degree <= 64 { 1 } else { 2 };
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
        !Self::WORD && Self::WINDOW == 3,
        Plan::<P>::DEGREE,
        Plan::<P>::TAPS,
        Self::POWER,
    );
    const LANE_NIBBLES: &'static LaneNibbles = &lane_table::<32, 16>(
        !Self::WORD && Self::WINDOW == 4,
        Plan::<P>::DEGREE,
        Plan::<P>::TAPS,
        Self::POWER,
    );
    const LANE_BYTES: &'static LaneBytes = &lane_table::<16, 256>(
        !Self::WORD && Self::WINDOW == 8,
        Plan::<P>::DEGREE,
        Plan::<P>::TAPS,
        Self::POWER,
    );

    /// `x^(2^POWER)`: the images of the windows of `x`, summed.
    #[inline(always)]
    fn apply(x: Poly<P::Degree>) -> Poly<P::Degree> {
        let limbs = x.to_uint().into_limbs();
        let low = limbs.first().copied().unwrap_or(0);
        let high = limbs.get(1).copied().unwrap_or(0);
        if Self::WORD {
            let sum = match Self::WINDOW {
                3 => word_sum(Self::WORD_TRIPLES, low, 3),
                4 => word_sum(Self::WORD_NIBBLES, low, 4),
                _ => word_sum(Self::WORD_BYTES, low, 8),
            };
            polynomial::<P>([sum, 0])
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
fn add_word_rows<const ENTRIES: usize>(
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
fn add_lane_rows<const ENTRIES: usize>(
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
fn polynomial<P: Polynomial>(words: [u64; 2]) -> Poly<P::Degree> {
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
const fn images(degree: usize, taps: &[usize], power: usize) -> [u128; 128] {
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
