//! Every product against references that share no code with the crate: the bit-by-bit polynomial
//! product, the shift-and-add product in GF(2^128), values derived by hand and test case 2 of the
//! GCM specification.

use clmul::{
    Gf2p128Accumulator, clmul_into, clmul64, const_clmul_into, const_clmul64, gf2p128_mul,
};

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

    fn next_u128(&mut self) -> u128 {
        u128::from(self.next_u64()) | (u128::from(self.next_u64()) << 64)
    }
}

const EDGES: [u64; 6] = [
    0,
    1,
    1 << 63,
    u64::MAX,
    0xaaaa_aaaa_aaaa_aaaa,
    0x5555_5555_5555_5555,
];

/// `a * b mod x^(64 * len)`, bit by bit: the XOR of `a * x^k` over the set bits `k` of `b`.
fn reference(a: &[u64], b: &[u64], len: usize) -> Vec<u64> {
    let mut product = vec![0; len.max(a.len() + b.len())];
    for k in 0..64 * b.len() {
        if (b[k / 64] >> (k % 64)) & 1 == 1 {
            for (index, &limb) in a.iter().enumerate() {
                product[index + k / 64] ^= limb << (k % 64);
                if k % 64 != 0 {
                    product[index + k / 64 + 1] ^= limb >> (64 - k % 64);
                }
            }
        }
    }
    product.truncate(len);
    product
}

/// `a * b` in GF(2^128) by shift and add: the multiple of `a` doubles and reduces once per bit of
/// `b`.
fn reference_gf2p128(a: u128, b: u128) -> u128 {
    let (mut multiple, mut product) = (a, 0);
    for bit in 0..128 {
        if (b >> bit) & 1 == 1 {
            product ^= multiple;
        }
        multiple = (multiple << 1) ^ ((multiple >> 127) * 0x87);
    }
    product
}

fn limbs(value: u128) -> [u64; 2] {
    [value as u64, (value >> 64) as u64]
}

/// `clmul64` and `const_clmul64` on the pairs of edge words, of single bits and of random words.
#[test]
fn word_products_match_the_bit_by_bit_product() {
    let mut rng = Rng(1);
    let edges = EDGES.into_iter().flat_map(|a| EDGES.map(|b| (a, b)));
    let single_bits = (0..64).flat_map(|i| (0..64).map(move |j| (1 << i, 1 << j)));
    let random: Vec<_> = (0..2048)
        .map(|_| (rng.next_u64(), rng.next_u64()))
        .collect();
    for (a, b) in edges.chain(single_bits).chain(random) {
        let product = reference(&[a], &[b], 2);
        let expected = u128::from(product[0]) | (u128::from(product[1]) << 64);
        assert_eq!(clmul64(a, b), expected, "{a:#x} * {b:#x}");
        assert_eq!(const_clmul64(a, b), expected, "const {a:#x} * {b:#x}");
    }
}

/// `clmul_into` and `const_clmul_into` for operands of up to four limbs, into outputs that truncate
/// the product, hold it exactly and exceed it, each prefilled with ones.
#[test]
fn limb_products_match_the_bit_by_bit_product() {
    let mut rng = Rng(2);
    let mut operand = |len: usize, kind: usize| -> Vec<u64> {
        (0..len)
            .map(|index| match kind {
                0 => u64::MAX,
                1 if index + 1 == len => 1 << 63,
                1 => 0,
                2 => 0xaaaa_aaaa_aaaa_aaaa,
                _ => rng.next_u64(),
            })
            .collect()
    };
    for a_len in 0..=4_usize {
        for b_len in 0..=4 {
            let full = a_len + b_len;
            for len in [1, full / 2, full.saturating_sub(1), full, full + 1] {
                for kind in 0..8 {
                    let (a, b) = (operand(a_len, kind), operand(b_len, kind));
                    let expected = reference(&a, &b, len);
                    let mut out = vec![u64::MAX; len];
                    clmul_into(&mut out, &a, &b);
                    assert_eq!(out, expected, "{a:x?} * {b:x?} in {len} limbs");
                    out.fill(u64::MAX);
                    const_clmul_into(&mut out, &a, &b);
                    assert_eq!(out, expected, "const {a:x?} * {b:x?} in {len} limbs");
                }
            }
        }
    }
}

/// Products in `const` items equal the same products at run time.
#[test]
fn const_evaluation_matches_run_time() {
    const A: [u64; 2] = [0x0123_4567_89ab_cdef, 0xfedc_ba98_7654_3210];
    const B: [u64; 3] = [
        0x243f_6a88_85a3_08d3,
        0x1319_8a2e_0370_7344,
        0xa409_3822_299f_31d0,
    ];
    const WORD: u128 = const_clmul64(A[0], B[0]);
    const LIMBS: [u64; 5] = {
        let mut out = [0; 5];
        const_clmul_into(&mut out, &A, &B);
        out
    };
    let mut run_time = [0; 5];
    clmul_into(&mut run_time, &A, &B);
    assert_eq!(WORD, clmul64(A[0], B[0]));
    assert_eq!(LIMBS, run_time);

    #[cfg(feature = "biguint")]
    {
        use biguint::Uint;
        use biguint::typenum::{Sum, U100, U129};
        use clmul::{CarrylessMul, const_carryless_mul, const_widening_carryless_mul};

        const X: Uint<U100> = Uint::from_limbs(A);
        const Y: Uint<U129> = Uint::from_limbs(B);
        const TRUNCATED: Uint<U100> = const_carryless_mul(X, Y);
        const WIDE: Uint<Sum<U100, U129>> = const_widening_carryless_mul(X, Y);
        assert_eq!(TRUNCATED, X.carryless_mul(Y));
        assert_eq!(WIDE, X.widening_carryless_mul(Y));
    }
}

/// GF(2^128) products from outside the crate: values derived by hand from
/// `x^128 = x^7 + x^2 + x + 1`, and test case 2 of the GCM specification (McGrew and Viega).
#[test]
fn gf2p128_products_match_known_answers() {
    // (lhs, rhs, product), bit `i` the coefficient of `x^i`.
    let by_hand: [(u128, u128, u128); 4] = [
        // x^127 * x = x^7 + x^2 + x + 1.
        (1 << 127, 1 << 1, 0x87),
        // x^64 * x^64 = x^7 + x^2 + x + 1.
        (1 << 64, 1 << 64, 0x87),
        // x^127 * x^127 = x^126 * x^128 = x^127 + x^126 + x^12 + x^6 + x^5 + x^2 + x + 1.
        (1 << 127, 1 << 127, (0b11 << 126) | 0x1067),
        // The unit.
        (
            1,
            0x0123_4567_89ab_cdef_fedc_ba98_7654_3210,
            0x0123_4567_89ab_cdef_fedc_ba98_7654_3210,
        ),
    ];
    // Test case 2: K = 0^128, IV = 0^96, P = 0^128. GHASH computes X1 = C * H, then
    // (X1 + len(A) || len(C)) * H. The blocks are big-endian with the coefficient of x^0 in the top
    // bit, so an element is the block's bit reversal.
    const H: u128 = 0x66e9_4bd4_ef8a_2c3b_884c_fa59_ca34_2b2e;
    const C: u128 = 0x0388_dace_60b6_a392_f328_c2b9_71b2_fe78;
    const X1: u128 = 0x5e2e_c746_9170_6288_2c85_b068_5353_deb7;
    const LENGTHS: u128 = 0x80;
    const GHASH: u128 = 0xf38c_bb1a_d692_23dc_c345_7ae5_b6b0_f885;
    let gcm = [(C, H, X1), (X1 ^ LENGTHS, H, GHASH)].map(|(lhs, rhs, product)| {
        (
            lhs.reverse_bits(),
            rhs.reverse_bits(),
            product.reverse_bits(),
        )
    });
    for (lhs, rhs, product) in by_hand.into_iter().chain(gcm) {
        assert_eq!(
            gf2p128_mul(limbs(lhs), limbs(rhs)),
            limbs(product),
            "{lhs:#x} * {rhs:#x}"
        );
    }
}

/// `gf2p128_mul` on edge and random pairs, and a seeded `Gf2p128Accumulator` over all of them.
#[test]
fn gf2p128_products_and_sums_match_shift_and_add() {
    let mut rng = Rng(3);
    let edges = [0, 1, 1 << 64, 1 << 127, u128::MAX, u128::MAX / 3];
    let pairs = edges.into_iter().flat_map(|a| edges.map(|b| (a, b)));
    let random: Vec<_> = (0..1024)
        .map(|_| (rng.next_u128(), rng.next_u128()))
        .collect();
    let seed = rng.next_u128();
    let mut sum = Gf2p128Accumulator::from_seed(limbs(seed));
    let mut expected = seed;
    for (lhs, rhs) in pairs.chain(random) {
        let product = reference_gf2p128(lhs, rhs);
        assert_eq!(
            gf2p128_mul(limbs(lhs), limbs(rhs)),
            limbs(product),
            "{lhs:#x} * {rhs:#x}"
        );
        sum.accumulate(limbs(lhs), limbs(rhs));
        expected ^= product;
    }
    assert_eq!(sum.finalize(), limbs(expected));
}

#[cfg(feature = "biguint")]
mod uint {
    use core::ops::Add;

    use biguint::generic_array::GenericArray;
    use biguint::generic_array::sequence::GenericSequence;
    use biguint::typenum::{Sum, U1, U8, U16, U63, U64, U65, U100, U128, U129, U192, U233, U571};
    use biguint::{Bits, Uint};
    use clmul::{CarrylessMul, Poly, Width, const_carryless_mul, const_widening_carryless_mul};

    use super::{Rng, reference};

    /// Zero, one, all ones, the top bit alone, and random values.
    fn samples<N: Bits>(rng: &mut Rng) -> Vec<Uint<N>> {
        let mut values = vec![
            Uint::ZERO,
            Uint::ONE,
            !Uint::<N>::ZERO,
            Uint::ONE << (N::BITS - 1),
        ];
        values.extend(
            (0..6).map(|_| Uint::from_generic_limbs(GenericArray::generate(|_| rng.next_u64()))),
        );
        values
    }

    /// `a * b mod x^P` by the bit-by-bit reference.
    fn expected<P: Bits>(a: &[u64], b: &[u64]) -> Uint<P> {
        let limbs = reference(a, b, P::LIMBS);
        Uint::from_generic_limbs(GenericArray::generate(|index| limbs[index]))
    }

    fn check<N, M>()
    where
        N: Bits + Add<M>,
        M: Bits,
        Sum<N, M>: Bits,
    {
        let mut rng = Rng(((N::BITS as u64) << 32) | M::BITS as u64);
        let rhs_values = samples::<M>(&mut rng);
        for a in samples::<N>(&mut rng) {
            for &b in &rhs_values {
                let truncated = expected::<N>(a.as_limbs(), b.as_limbs());
                let wide = expected::<Sum<N, M>>(a.as_limbs(), b.as_limbs());
                assert_eq!(a.carryless_mul(b), truncated, "{a:?} * {b:?}");
                assert_eq!(const_carryless_mul(a, b), truncated, "const {a:?} * {b:?}");
                assert_eq!(a.widening_carryless_mul(b), wide, "{a:?} * {b:?}");
                assert_eq!(
                    const_widening_carryless_mul(a, b),
                    wide,
                    "const {a:?} * {b:?}"
                );
            }
        }
    }

    /// The trait and the const functions at widths of one bit, of partial top limbs and of whole
    /// limbs, with operands of equal and of different lengths.
    #[test]
    fn uint_products_match_the_bit_by_bit_product() {
        check::<U1, U1>();
        check::<U63, U65>();
        check::<U64, U64>();
        check::<U100, U129>();
        check::<U128, U128>();
        check::<U192, U64>();
    }

    /// `Poly` against `Uint` and the bit-by-bit product: conversions, bitwise operators, shifts,
    /// resizing, products and squares, at widths of one word, of partial and whole registers and of
    /// several registers.
    fn check_poly<N, M>()
    where
        N: Width + Add<M> + Add<N>,
        M: Width,
        Sum<N, M>: Width,
        Sum<N, N>: Width,
    {
        let mut rng = Rng(((N::BITS as u64) << 40) | ((M::BITS as u64) << 20) | 7);
        let rhs_values = samples::<M>(&mut rng);
        let lhs_values = samples::<N>(&mut rng);
        let shifts = [0, 1, 7, 63, 64, 65, 127, 128, 129, N::BITS - 1, N::BITS];
        for a in &lhs_values {
            let p = Poly::from_uint(*a);
            assert_eq!(p.to_uint(), *a);
            assert_eq!(Poly::<N>::from_le_bytes(&a.to_le_bytes()), Ok(p));
            assert_eq!(Poly::<N>::from_be_bytes(&p.to_be_bytes()), Ok(p));
            assert_eq!(p.is_zero(), a.is_zero());
            for &bits in &shifts {
                assert_eq!((p << bits).to_uint(), *a << bits, "{a:?} << {bits}");
                assert_eq!((p >> bits).to_uint(), *a >> bits, "{a:?} >> {bits}");
            }
            for bits in [1, 7, 32, 63] {
                let words = |shift: fn(u64, u32) -> u64| -> Uint<N> {
                    Uint::from_generic_limbs(GenericArray::generate(|index| {
                        shift(a.as_limbs()[index], bits)
                    }))
                };
                assert_eq!(
                    p.shl_within_words(bits).to_uint(),
                    words(|word, bits| word << bits)
                );
                assert_eq!(
                    p.shr_within_words(bits).to_uint(),
                    words(|word, bits| word >> bits)
                );
            }
            assert_eq!(p.resize::<M>().to_uint(), a.resize::<M>());
            assert_eq!(
                p.widening_square().to_uint(),
                expected::<Sum<N, N>>(a.as_limbs(), a.as_limbs())
            );
            for b in &lhs_values {
                let q = Poly::from_uint(*b);
                assert_eq!((p ^ q).to_uint(), *a ^ *b);
                assert_eq!((p & q).to_uint(), *a & *b);
                assert_eq!((p | q).to_uint(), *a | *b);
            }
            for b in &rhs_values {
                let q = Poly::from_uint(*b);
                assert_eq!(
                    p.widening_mul(q).to_uint(),
                    expected::<Sum<N, M>>(a.as_limbs(), b.as_limbs()),
                    "{a:?} * {b:?}"
                );
                for i in 0..N::LIMBS {
                    for j in 0..M::LIMBS {
                        let (x, y) = (a.as_limbs()[i], b.as_limbs()[j]);
                        assert_eq!(
                            p.clmul_words(i, q, j).to_uint(),
                            expected::<U128>(&[x], &[y]),
                            "word {i} of {a:?} * word {j} of {b:?}"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn polys_match_uint_and_the_bit_by_bit_product() {
        check_poly::<U1, U1>();
        check_poly::<U8, U16>();
        check_poly::<U63, U65>();
        check_poly::<U64, U64>();
        check_poly::<U100, U129>();
        check_poly::<U128, U128>();
        check_poly::<U192, U64>();
        check_poly::<U233, U233>();
        check_poly::<U571, U571>();
    }
}
