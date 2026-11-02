//! Products, divisions and modular products at the widths where Karatsuba and divide-and-conquer
//! division run, against `ruint`: odd and even splits at one to three Karatsuba levels, unbalanced
//! products by blocks with and without a shorter last block, and divisions by full and partial
//! divide-and-conquer blocks.

use core::ops::Add;

use biguint::{Bits, Modulus, Uint, WideningMul};
use generic_array::GenericArray;
use generic_array::sequence::GenericSequence;
use typenum::{Sum, U64, U128, U1024, U2048, U4096, U8192};

type U1088 = Sum<U1024, typenum::U64>;
type U1984 = Sum<U1024, typenum::U960>;
type U2112 = Sum<U2048, typenum::U64>;
type U3072 = Sum<U2048, U1024>;
type U4160 = Sum<U4096, typenum::U64>;

/// A `ruint` width that holds every product here.
type Wide = ruint::Uint<16640, 260>;

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

/// Values whose limbs run the carries of every add, subtract and correction to their ends, and
/// random values of random lengths.
fn samples<N: Bits>(seed: u64) -> Vec<Uint<N>> {
    let max = !Uint::<N>::ZERO;
    fn generate<N: Bits>(limb: impl FnMut(usize) -> u64) -> Uint<N> {
        Uint::from_generic_limbs(GenericArray::generate(limb))
    }
    let mut values = vec![
        max,
        max >> 1usize,
        Uint::ONE << (N::BITS - 1),
        (Uint::<N>::ONE << (N::BITS - 1)) + Uint::<N>::ONE,
        // Halves that differ only at the bottom, so the Karatsuba differences are tiny.
        generate::<N>(|index| if index == 0 { 1 } else { u64::MAX }),
        // All ones in the lower half, zeros above.
        generate::<N>(|index| if index < N::LIMBS / 2 { u64::MAX } else { 0 }),
        // Zeros in the lower half, ones above.
        generate::<N>(|index| if index < N::LIMBS / 2 { 0 } else { u64::MAX }),
        generate::<N>(|index| if index % 2 == 0 { u64::MAX } else { 0 }),
        generate::<N>(|_| 0x8000_0000_0000_0001),
    ];
    let mut rng = Rng(seed ^ N::BITS as u64);
    for _ in 0..7 {
        let full = generate::<N>(|_| rng.next_u64());
        values.push(full >> (rng.next_u64() as usize % (N::BITS / 4)));
    }
    values
}

fn wide<N: Bits>(value: Uint<N>) -> Wide {
    Wide::from_limbs(core::array::from_fn(|index| {
        value.as_limbs().get(index).copied().unwrap_or(0)
    }))
}

/// `a * b` exactly and modulo `2^N`, for every pair of samples.
fn check_products<N, M>()
where
    N: Bits + Add<M>,
    M: Bits,
    Sum<N, M>: Bits,
{
    let rhs_values = samples::<M>(2);
    for a in samples::<N>(1) {
        for &b in &rhs_values {
            let exact = wide(a).wrapping_mul(wide(b));
            assert_eq!(wide(a.widening_mul(b)), exact, "{a:?} * {b:?}");
            let wrapped = exact & ((Wide::from(1u8) << N::BITS) - Wide::from(1u8));
            assert_eq!(wide(a * b), wrapped, "{a:?} * {b:?} mod 2^{}", N::BITS);
        }
    }
}

/// `a * b mod m` for every pair of operand samples and every modulus sample.
fn check_mul_mod<N, M, Q>()
where
    N: Bits + Add<M>,
    M: Bits,
    Q: Bits,
    Sum<N, M>: Bits,
{
    let rhs_values = samples::<M>(4);
    let operands = samples::<N>(3);
    for modulus in samples::<Q>(5) {
        let wide_modulus = wide(modulus);
        for &a in &operands {
            for &b in &rhs_values {
                let expected = wide(a).mul_mod(wide(b), wide_modulus);
                let product = a.mul_mod(b, modulus).map(wide);
                assert_eq!(product, Some(expected), "{a:?} * {b:?} mod {modulus:?}");
            }
        }
    }
}

/// `(a / d, a % d)` for every dividend sample and every divisor sample.
fn check_div_rem<N: Bits, M: Bits>() {
    let divisors = samples::<M>(7);
    for a in samples::<N>(6) {
        for &divisor in &divisors {
            let expected = (!divisor.is_zero()).then(|| wide(a).div_rem(wide(divisor)));
            let result = a.checked_div_rem(divisor).map(|(q, r)| (wide(q), wide(r)));
            assert_eq!(result, expected, "{a:?} / {divisor:?}");
        }
    }
}

#[test]
fn karatsuba_products_match_reference() {
    // One level, odd and even.
    check_products::<U1024, U1024>();
    check_products::<U1088, U1088>();
    // Two levels; odd top and odd halves.
    check_products::<U1984, U1984>();
    check_products::<U2048, U2048>();
    check_products::<U2112, U2112>();
    // Three levels.
    check_products::<U4096, U4096>();
    check_products::<U4160, U4160>();
    check_products::<U8192, U8192>();
    // Blocks: whole blocks only, and a shorter last block.
    check_products::<U4096, U1024>();
    check_products::<U4096, U3072>();
    check_products::<U1088, U3072>();
}

#[test]
fn divide_and_conquer_mul_mod_matches_reference() {
    // Whole blocks after a one-limb first block.
    check_mul_mod::<U2048, U2048, U2048>();
    check_mul_mod::<U4096, U4096, U4096>();
    check_mul_mod::<U4160, U4160, U4160>();
    // A first block of 33 quotient limbs below the 64-limb divisor.
    check_mul_mod::<U4096, U2048, U4096>();
    // Three whole blocks after a one-limb first block.
    check_mul_mod::<U4096, U4096, U2048>();
    // A 48-limb divisor, whose halves divide by schoolbook.
    check_mul_mod::<U3072, U3072, U3072>();
}

/// Products and reductions of the residues of every odd and even modulus sample.
fn check_modulus<N: Bits>()
where
    Uint<N>: Copy,
{
    let operands: Vec<Uint<N>> = samples::<N>(8).into_iter().take(4).collect();
    for modulus in samples::<N>(9).into_iter().take(4) {
        let Some(context) = Modulus::new(modulus) else {
            continue;
        };
        let wide_modulus = wide(modulus);
        context.scope(|ring| {
            for &a in &operands {
                let residue = ring.reduce(a);
                assert_eq!(wide(residue.get()), wide(a).reduce_mod(wide_modulus));
                for &b in &operands {
                    let expected = wide(a).mul_mod(wide(b), wide_modulus);
                    assert_eq!(
                        wide((residue * ring.reduce(b)).get()),
                        expected,
                        "{a:?} * {b:?}"
                    );
                }
            }
        });
    }
}

#[test]
fn modulus_context_at_large_widths_matches_reference() {
    // Rows past the unrolled length and Karatsuba products; divide-and-conquer reduction.
    check_modulus::<U1088>();
    check_modulus::<U2048>();
}

#[test]
fn mixed_width_division_matches_reference() {
    // A one-limb first block, then a divide-and-conquer block.
    check_div_rem::<U4096, U2048>();
    check_div_rem::<U4160, U2112>();
    // Schoolbook below the threshold, and the one- and two-limb divisors.
    check_div_rem::<U4096, U1024>();
    check_div_rem::<U4096, U128>();
    check_div_rem::<U4096, U64>();
    // A divisor wider than the dividend: the quotient is zero.
    check_div_rem::<U2048, U4096>();
}
