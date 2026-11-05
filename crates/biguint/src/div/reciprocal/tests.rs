//! The division steps against exact division. On 8-bit words, where every boundary case is
//! reachable, Algorithms 4 and 6 run on every input and Algorithm 5 on every divisor. On 64-bit
//! limbs, the reciprocal runs on both ends of every seed class, and the steps run end to end
//! against `u128` and `ruint`.

extern crate std;

use std::vec::Vec;

use super::{
    NormalizedLimb, Reciprocal, Reciprocal3By2, divided_reciprocal, join, newton_reciprocal,
};

mod byte {
    division_steps!(u8, u16, widening);

    const fn widening(a: u8, b: u8) -> u16 {
        a as u16 * b as u16
    }
}

type U192 = ruint::Uint<192, 3>;

/// SplitMix64.
struct Rng(u64);

impl Rng {
    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }
}

/// `floor((2^16 - 1) / d) - 2^8`, the reciprocal of a normalized byte.
fn byte_reciprocal(divisor: u8) -> u8 {
    (u16::MAX / u16::from(divisor) - 256) as u8
}

#[test]
fn byte_division_by_one_word_is_exact_on_every_input() {
    for divisor in 128u8..=255 {
        let reciprocal = byte_reciprocal(divisor);
        for high in 0..divisor {
            for low in 0..=255u8 {
                let dividend = u16::from(high) << 8 | u16::from(low);
                let expected = (
                    (dividend / u16::from(divisor)) as u8,
                    (dividend % u16::from(divisor)) as u8,
                );
                let actual = byte::div_2_by_1(high, low, divisor, reciprocal);
                assert_eq!(actual, expected, "{dividend:#x} / {divisor:#x}");
                let masked = byte::div_2_by_1_masked(high, low, divisor, reciprocal);
                assert_eq!(masked, expected, "masked {dividend:#x} / {divisor:#x}");
            }
        }
    }
}

#[test]
fn byte_reciprocal_of_two_words_is_exact_on_every_divisor() {
    for high in 128u8..=255 {
        for low in 0..=255u8 {
            let divisor = u32::from(high) << 8 | u32::from(low);
            let expected = ((1u32 << 24) - 1) / divisor - 256;
            let actual = byte::reciprocal_3_by_2(high, low, byte_reciprocal(high));
            assert_eq!(u32::from(actual), expected, "d = {divisor:#x}");
        }
    }
}

#[test]
fn byte_division_by_two_words_is_exact_on_every_divisor() {
    let mut rng = Rng(4);
    for high in 128u8..=255 {
        let reciprocal = byte_reciprocal(high);
        for low in 0..=255u8 {
            let divisor = byte::join(high, low);
            let reciprocal = byte::reciprocal_3_by_2(high, low, reciprocal);
            let divisor_wide = u32::from(divisor);
            // Dividends `q * d + r` for quotients and remainders at their edges and at random.
            let random = rng.next_u64();
            let quotients = [0, 1, 2, 127, 128, 254, 255, (random & 0xff) as u32];
            let remainders = [0, 1, divisor_wide / 2, divisor_wide - 1];
            let random_remainder = (random >> 8) as u32 % divisor_wide;
            for quotient in quotients {
                for remainder in remainders.into_iter().chain([random_remainder]) {
                    let dividend = quotient * divisor_wide + remainder;
                    let top = (dividend >> 8) as u16;
                    let actual = byte::div_3_by_2(top, dividend as u8, divisor, reciprocal);
                    let expected = (quotient as u8, remainder as u16);
                    assert_eq!(actual, expected, "{dividend:#x} / {divisor:#x}");
                }
            }
        }
    }
}

/// Normalized limbs: each seed class at both ends and at a random point, the extremes of the
/// range, and random limbs near its bottom.
fn normalized_limbs() -> Vec<NormalizedLimb> {
    let mut rng = Rng(1);
    let mut limbs = Vec::new();
    for class in 256u64..512 {
        let low = class << 55;
        limbs.extend([low, low | ((1 << 55) - 1), low | (rng.next_u64() >> 9)]);
    }
    limbs.extend([1 << 63, (1 << 63) + 1, u64::MAX, u64::MAX - 1]);
    limbs.extend((0..256).map(|_| (1 << 63) | (rng.next_u64() >> 40)));
    limbs.into_iter().map(NormalizedLimb::top_set).collect()
}

#[test]
fn reciprocal_word_is_exact() {
    for divisor in normalized_limbs() {
        let expected = (u128::MAX / u128::from(divisor.0) - (1 << 64)) as u64;
        assert_eq!(
            divided_reciprocal(divisor),
            expected,
            "d = {:#x}",
            divisor.0
        );
        assert_eq!(newton_reciprocal(divisor), expected, "d = {:#x}", divisor.0);
    }
}

#[test]
fn limb_division_by_one_limb_is_exact() {
    let mut rng = Rng(2);
    for divisor in normalized_limbs() {
        let d = divisor.0;
        let reciprocal = Reciprocal::new(divisor);
        for quotient in [0, 1, u64::MAX, rng.next_u64()] {
            for remainder in [0, 1, d - 1, rng.next_u64() % d] {
                let dividend = u128::from(quotient) * u128::from(d) + u128::from(remainder);
                let actual = reciprocal.div_rem((dividend >> 64) as u64, dividend as u64);
                assert_eq!(actual, (quotient, remainder), "{dividend:#x} / {d:#x}");
            }
        }
    }
}

#[test]
fn limb_division_by_two_limbs_is_exact() {
    let mut rng = Rng(3);
    for high in normalized_limbs() {
        for low in [0, 1, u64::MAX, rng.next_u64()] {
            let divisor = Reciprocal3By2::new(high, low);
            let d = U192::from_limbs([low, high.0, 0]);
            for quotient in [0, 1, u64::MAX, rng.next_u64()] {
                let quotient = U192::from_limbs([quotient, 0, 0]);
                let random_remainder = U192::from_limbs([rng.next_u64(), rng.next_u64(), 0]) % d;
                for remainder in [
                    U192::ZERO,
                    U192::from(1u8),
                    d - U192::from(1u8),
                    random_remainder,
                ] {
                    let dividend = (quotient * d + remainder).into_limbs();
                    let [next, top_low, top_high] = dividend;
                    let remainder = remainder.into_limbs();
                    let expected = (quotient.into_limbs()[0], join(remainder[1], remainder[0]));
                    let actual = divisor.div_rem(join(top_high, top_low), next);
                    assert_eq!(actual, expected, "{dividend:x?} / {d:#x}");
                }
            }
        }
    }
}
