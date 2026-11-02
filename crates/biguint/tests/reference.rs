//! Every operation against `ruint`, an independent fixed-width implementation. The widths cover
//! one bit, partial top limbs, exact limb multiples, and the 521-bit case of nine limbs.

use core::ops::Add;

use biguint::{Bits, Modulus, NonZeroUint, NotReduced, OutOfRange, Pow, Uint, WideningMul};
use generic_array::GenericArray;
use generic_array::sequence::GenericSequence;
use typenum::{
    Sum, U1, U7, U8, U28, U63, U64, U65, U100, U128, U129, U130, U192, U256, U384, U512, U521,
    U1024,
};

type Reference<const B: usize, const L: usize> = ruint::Uint<B, L>;

/// 1152 bits: twice the widest width here, with room.
type U1152 = Sum<U1024, U128>;

/// Calls `$check::<N, B, L>()` for every width `B` with `L` limbs.
macro_rules! for_each_width {
    ($check:ident) => {
        $check::<U1, 1, 1>();
        $check::<U7, 7, 1>();
        $check::<U8, 8, 1>();
        $check::<U63, 63, 1>();
        $check::<U64, 64, 1>();
        $check::<U65, 65, 2>();
        $check::<U100, 100, 2>();
        $check::<U128, 128, 2>();
        $check::<U129, 129, 3>();
        $check::<U192, 192, 3>();
        $check::<U256, 256, 4>();
        $check::<U384, 384, 6>();
        $check::<U512, 512, 8>();
        $check::<U521, 521, 9>();
    };
}

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

    /// A random value with a random bit length.
    fn uint<N: Bits>(&mut self) -> Uint<N> {
        let full = Uint::<N>::from_generic_limbs(GenericArray::generate(|_| self.next_u64()));
        full >> (self.next_u64() as usize % N::BITS)
    }
}

fn repeated<N: Bits>(limb: u64) -> Uint<N> {
    Uint::from_generic_limbs(GenericArray::generate(|_| limb))
}

fn limbs<N: Bits>(values: &[u64]) -> Uint<N> {
    Uint::from_generic_limbs(GenericArray::generate(|index| {
        values.get(index).copied().unwrap_or(0)
    }))
}

/// Edge values and random values of width `N`.
fn samples<N: Bits>() -> Vec<Uint<N>> {
    let max = !Uint::<N>::ZERO;
    let mut values = vec![
        Uint::ZERO,
        Uint::ONE,
        Uint::from_u64(2),
        max,
        max - Uint::<N>::ONE,
        max >> 1usize,
        Uint::ONE << (N::BITS - 1),
        repeated(0xaaaa_aaaa_aaaa_aaaa),
        repeated(0x5555_5555_5555_5555),
    ];
    for bit in [31, 32, 63, 64, 65, 127, 128] {
        if bit < N::BITS {
            values.push(Uint::ONE << bit);
            values.push((Uint::ONE << bit) - Uint::<N>::ONE);
        }
    }
    if N::BITS >= 256 {
        // Dividing the first by the second, the 3-by-2 step overestimates the first quotient limb
        // (Knuth's add-back, step D6), and the second quotient limb is `2^64 - 1`.
        values.push(limbs(&[11, 3, 7, 1 << 63]));
        values.push(limbs(&[9, 7, 1 << 63]));
    }
    let mut rng = Rng(N::BITS as u64);
    values.extend((0..24).map(|_| rng.uint::<N>()));
    values
}

fn reference<N: Bits, const B: usize, const L: usize>(value: Uint<N>) -> Reference<B, L> {
    Reference::from_limbs(core::array::from_fn(|index| value.as_limbs()[index]))
}

fn shift_amounts(bits: usize) -> [usize; 11] {
    [
        0,
        1,
        31,
        63,
        64,
        65,
        bits - 1,
        bits,
        bits + 1,
        2 * bits + 3,
        usize::MAX,
    ]
}

#[test]
fn arithmetic_matches_reference() {
    fn check<N: Bits, const B: usize, const L: usize>() {
        let values = samples::<N>();
        for &a in &values {
            let ra = reference::<N, B, L>(a);
            for &b in &values {
                let rb = reference::<N, B, L>(b);
                assert_eq!(reference(a + b), ra.wrapping_add(rb), "{B}: {a:?} + {b:?}");
                assert_eq!(reference(a - b), ra.wrapping_sub(rb), "{B}: {a:?} - {b:?}");
                assert_eq!(reference(a * b), ra.wrapping_mul(rb), "{B}: {a:?} * {b:?}");
                for carry in [false, true] {
                    let carry_in = Reference::<B, L>::from_limbs(core::array::from_fn(|index| {
                        u64::from(carry && index == 0)
                    }));
                    let (sum, add_low) = ra.overflowing_add(rb);
                    let (sum, add_high) = sum.overflowing_add(carry_in);
                    let (out, carry_out) = a.carrying_add(b, carry);
                    assert_eq!((reference(out), carry_out), (sum, add_low | add_high));

                    let (difference, sub_low) = ra.overflowing_sub(rb);
                    let (difference, sub_high) = difference.overflowing_sub(carry_in);
                    let (out, borrow_out) = a.borrowing_sub(b, carry);
                    assert_eq!(
                        (reference(out), borrow_out),
                        (difference, sub_low | sub_high)
                    );
                }
            }
        }
        for (&a, &b) in values.iter().zip(values.iter().rev()).take(6) {
            let expected = reference::<N, B, L>(a).wrapping_pow(reference(b));
            assert_eq!(reference(a.pow(b)), expected, "{B}: {a:?} ^ {b:?}");
        }
    }
    for_each_width!(check);
}

#[test]
fn widening_mul_matches_reference() {
    /// `N`-bit times `M`-bit into `R = N + M` bits; `L`, `K` and `S` are the limb counts.
    fn check<
        N,
        M,
        const B: usize,
        const L: usize,
        const C: usize,
        const K: usize,
        const R: usize,
        const S: usize,
    >()
    where
        N: Bits + Add<M>,
        M: Bits,
        Sum<N, M>: Bits,
    {
        let rhs_values = samples::<M>();
        for a in samples::<N>() {
            for &b in &rhs_values {
                let expected: Reference<R, S> =
                    reference::<N, B, L>(a).widening_mul(reference::<M, C, K>(b));
                assert_eq!(
                    reference(a.widening_mul(b)),
                    expected,
                    "{B}x{C}: {a:?} {b:?}"
                );
            }
        }
    }
    check::<U1, U7, 1, 1, 7, 1, 8, 1>();
    check::<U64, U64, 64, 1, 64, 1, 128, 2>();
    check::<U65, U63, 65, 2, 63, 1, 128, 2>();
    check::<U100, U28, 100, 2, 28, 1, 128, 2>();
    check::<U129, U192, 129, 3, 192, 3, 321, 6>();
    check::<U256, U256, 256, 4, 256, 4, 512, 8>();
    check::<U521, U521, 521, 9, 521, 9, 1042, 17>();
}

#[test]
fn division_matches_reference() {
    fn check<N: Bits, const B: usize, const L: usize>() {
        let values = samples::<N>();
        for &a in &values {
            for &b in &values {
                let Some((quotient, remainder)) = a.checked_div_rem(b) else {
                    assert!(b.is_zero(), "{B}: {a:?} / {b:?} refused");
                    continue;
                };
                let expected = reference::<N, B, L>(a).div_rem(reference(b));
                assert_eq!(
                    (reference(quotient), reference(remainder)),
                    expected,
                    "{B}: {a:?} / {b:?}"
                );
                let divisor = NonZeroUint::new(b).expect("a nonzero divisor");
                assert_eq!(
                    (a / divisor, a % divisor),
                    (quotient, remainder),
                    "{B}: operators"
                );
            }
        }
    }
    for_each_width!(check);
}

/// The moduli: the edges of the range and random values.
fn moduli<N: Bits>() -> Vec<Uint<N>> {
    let values = samples::<N>();
    let mut moduli: Vec<Uint<N>> = values.iter().copied().take(7).collect();
    moduli.extend(values.iter().copied().rev().take(5));
    moduli
}

#[test]
fn mul_mod_matches_reference() {
    fn check<N, const B: usize, const L: usize>()
    where
        N: Bits + Add<N>,
        Sum<N, N>: Bits,
    {
        let operands: Vec<Uint<N>> = samples::<N>().into_iter().rev().take(16).collect();
        for modulus in moduli::<N>() {
            let rm = reference::<N, B, L>(modulus);
            for &a in &operands {
                for &b in &operands {
                    let expected = (!modulus.is_zero())
                        .then(|| reference::<N, B, L>(a).mul_mod(reference(b), rm));
                    let product = a.mul_mod(b, modulus).map(reference::<N, B, L>);
                    assert_eq!(product, expected, "{B}: {a:?} * {b:?} mod {modulus:?}");
                }
            }
        }
    }

    /// Operands and modulus of three different widths, against `ruint` at a width that holds all.
    fn check_mixed<N, M, Q>()
    where
        N: Bits + Add<M>,
        M: Bits,
        Q: Bits,
        Sum<N, M>: Bits,
    {
        type Wide = ruint::Uint<1152, 18>;
        fn wide<W: Bits>(value: Uint<W>) -> Wide {
            Wide::from_limbs(core::array::from_fn(|index| {
                value.as_limbs().get(index).copied().unwrap_or(0)
            }))
        }
        let rhs_values: Vec<Uint<M>> = samples::<M>().into_iter().rev().take(12).collect();
        for modulus in moduli::<Q>() {
            for a in samples::<N>().into_iter().rev().take(12) {
                for &b in &rhs_values {
                    let expected =
                        (!modulus.is_zero()).then(|| wide(a).mul_mod(wide(b), wide(modulus)));
                    let product = a.mul_mod(b, modulus).map(wide);
                    assert_eq!(product, expected, "{a:?} * {b:?} mod {modulus:?}");
                }
            }
        }
    }

    for_each_width!(check);
    check_mixed::<U64, U64, U128>();
    check_mixed::<U256, U128, U192>();
    check_mixed::<U521, U521, U64>();
    check_mixed::<U100, U28, U100>();
    check_mixed::<U512, U512, U521>();
    check_mixed::<U8, U1, U7>();
}

#[test]
fn modular_sums_match_reference() {
    /// Operands above and below each modulus, and its largest residues.
    fn check<N: Bits, const B: usize, const L: usize>() {
        let samples: Vec<Uint<N>> = samples::<N>().into_iter().rev().take(16).collect();
        for modulus in moduli::<N>() {
            let Some(divisor) = NonZeroUint::new(modulus) else {
                continue;
            };
            let rm = reference::<N, B, L>(modulus);
            let mut operands = samples.clone();
            operands.extend([modulus - Uint::<N>::ONE, modulus >> 1usize]);
            for &a in &operands {
                let ra = reference::<N, B, L>(a).reduce_mod(rm);
                assert_eq!(
                    reference(a.neg_mod(divisor)),
                    (rm - ra).reduce_mod(rm),
                    "{B}: -{a:?} mod {modulus:?}"
                );
                for &b in &operands {
                    let rb = reference::<N, B, L>(b).reduce_mod(rm);
                    assert_eq!(
                        reference(a.add_mod(b, divisor)),
                        ra.add_mod(rb, rm),
                        "{B}: {a:?} + {b:?} mod {modulus:?}"
                    );
                    assert_eq!(
                        reference(a.sub_mod(b, divisor)),
                        ra.add_mod(rm - rb, rm),
                        "{B}: {a:?} - {b:?} mod {modulus:?}"
                    );
                }
            }
        }
    }
    for_each_width!(check);
}

#[test]
fn modulus_context_matches_reference() {
    /// Residues of odd and even moduli: reduction of narrower, equal and wider values, the
    /// residue check, the ring operations and the way back out.
    fn check<N: Bits, const B: usize, const L: usize>()
    where
        Uint<N>: Copy,
    {
        type Wide = ruint::Uint<1152, 18>;
        fn wide<W: Bits>(value: Uint<W>) -> Wide {
            Wide::from_limbs(core::array::from_fn(|index| {
                value.as_limbs().get(index).copied().unwrap_or(0)
            }))
        }
        let values: Vec<Uint<N>> = samples::<N>().into_iter().rev().take(10).collect();
        let exponent = Uint::<U130>::from_limbs([0x9e37_79b9_7f4a_7c15, 0xbf58_476d_1ce4_e5b9, 3]);
        for modulus in moduli::<N>() {
            let Some(context) = Modulus::new(modulus) else {
                continue;
            };
            let rm = reference::<N, B, L>(modulus);
            context.scope(|ring| {
                for &x in &values {
                    let rx = reference::<N, B, L>(x).reduce_mod(rm);
                    let a = ring.reduce(x);
                    assert_eq!(reference(a.get()), rx, "{B}: {x:?} mod {modulus:?}");
                    let accepted = ring.residue(x).map(|residue| reference(residue.get()));
                    let expected = if x < modulus { Ok(rx) } else { Err(NotReduced) };
                    assert_eq!(accepted, expected, "{B}: residue {x:?} of {modulus:?}");
                    // A value twice as wide, and a 64-bit one.
                    let double = (x.resize::<U1152>() << N::BITS) | values[0].resize::<U1152>();
                    let expected = (wide(x) << N::BITS | wide(values[0])).reduce_mod(wide(modulus));
                    assert_eq!(
                        wide(ring.reduce(double).get()),
                        expected,
                        "{B}: wide reduce"
                    );
                    let low = Uint::<U64>::from_u64(x.as_limbs()[0]);
                    let expected = wide(low).reduce_mod(wide(modulus));
                    assert_eq!(wide(ring.reduce(low).get()), expected, "{B}: narrow reduce");
                    assert_eq!(
                        reference((-a).get()),
                        (rm - rx).reduce_mod(rm),
                        "{B}: -{x:?}"
                    );
                    assert_eq!(
                        reference(a.square().get()),
                        rx.mul_mod(rx, rm),
                        "{B}: {x:?}^2"
                    );
                    assert_eq!(
                        wide(a.pow(exponent).get()),
                        wide(x).pow_mod(wide(exponent), wide(modulus)),
                        "{B}: {x:?}^e mod {modulus:?}"
                    );
                    for &y in &values {
                        let ry = reference::<N, B, L>(y).reduce_mod(rm);
                        let b = ring.reduce(y);
                        assert_eq!(reference((a + b).get()), rx.add_mod(ry, rm), "{B}: +");
                        assert_eq!(reference((a - b).get()), rx.add_mod(rm - ry, rm), "{B}: -");
                        assert_eq!(reference((a * b).get()), rx.mul_mod(ry, rm), "{B}: *");
                    }
                }
            });
        }
    }
    for_each_width!(check);
}

#[test]
fn bit_operations_match_reference() {
    fn check<N: Bits, const B: usize, const L: usize>() {
        let values = samples::<N>();
        for &a in &values {
            let ra = reference::<N, B, L>(a);
            assert_eq!(reference(!a), !ra);
            assert_eq!(a.bit_len(), ra.bit_len(), "{B}: {a:?}");
            assert_eq!(a.leading_zeros(), ra.leading_zeros(), "{B}: {a:?}");
            assert_eq!(a.trailing_zeros(), ra.trailing_zeros(), "{B}: {a:?}");
            assert_eq!(a.count_ones(), ra.count_ones(), "{B}: {a:?}");
            assert_eq!(a.count_zeros(), ra.count_zeros(), "{B}: {a:?}");
            for index in [0, 1, 63, 64, B - 1, B, B + 1, 4096] {
                assert_eq!(a.bit_at(index), ra.bit(index), "{B}: bit {index} of {a:?}");
            }
            for shift in shift_amounts(B) {
                let amount = Uint::<U8>::from_u64(shift as u64);
                let fits_amount = shift < 256;
                assert_eq!(
                    reference(a << shift),
                    ra.wrapping_shl(shift),
                    "{B}: {a:?} << {shift}"
                );
                assert_eq!(
                    reference(a >> shift),
                    ra.wrapping_shr(shift),
                    "{B}: {a:?} >> {shift}"
                );
                if fits_amount {
                    assert_eq!(reference(a << amount), ra.wrapping_shl(shift));
                    assert_eq!(reference(a >> amount), ra.wrapping_shr(shift));
                }
                assert_eq!(reference(a.rotate_left(shift)), ra.rotate_left(shift));
                assert_eq!(reference(a.rotate_right(shift)), ra.rotate_right(shift));
            }
            for &b in &values {
                let rb = reference::<N, B, L>(b);
                assert_eq!(reference(a & b), ra & rb);
                assert_eq!(reference(a | b), ra | rb);
                assert_eq!(reference(a ^ b), ra ^ rb);
                assert_eq!(a.cmp(&b), ra.cmp(&rb), "{B}: {a:?} cmp {b:?}");
                let expected_shift = usize::try_from(rb).ok().filter(|shift| *shift < B);
                let expected_shl = expected_shift.map_or(Reference::ZERO, |shift| ra << shift);
                assert_eq!(reference(a << b), expected_shl, "{B}: {a:?} << {b:?}");
            }
        }
    }
    for_each_width!(check);
}

#[test]
fn encodings_and_conversions_match_reference() {
    fn check<N: Bits, const B: usize, const L: usize>() {
        for a in samples::<N>() {
            let ra = reference::<N, B, L>(a);
            let expected_le: Vec<u8> = (0..B.div_ceil(8)).map(|index| ra.byte(index)).collect();
            let le = a.to_le_bytes();
            let be = a.to_be_bytes();
            assert_eq!(le.as_slice(), expected_le, "{B}: {a:?}");
            assert!(be.iter().eq(expected_le.iter().rev()), "{B}: {a:?}");
            assert_eq!(Uint::<N>::from_le_bytes(&le), Ok(a));
            assert_eq!(Uint::<N>::from_be_bytes(&be), Ok(a));

            assert_eq!(u64::try_from(a).ok(), u64::try_from(ra).ok(), "{B}: {a:?}");
            assert_eq!(
                u128::try_from(a).ok(),
                u128::try_from(ra).ok(),
                "{B}: {a:?}"
            );
            let narrowed = a.try_resize::<U64>().map(reference::<U64, 64, 1>);
            let expected_narrowed = u64::try_from(ra)
                .ok()
                .map(|low| Reference::<64, 1>::from_limbs([low]))
                .ok_or(OutOfRange { width: 64 });
            assert_eq!(narrowed, expected_narrowed, "{B}: {a:?}");
            let resized = a.resize::<U100>();
            assert!((0..128).all(|index| resized.bit_at(index) == (index < 100 && ra.bit(index))));
            if let Ok(value) = u128::try_from(ra) {
                assert_eq!(format!("{a:?}"), format!("{value:#x}"));
            }
        }
        if !B.is_multiple_of(8) {
            let mut over = Uint::<N>::ZERO.to_le_bytes();
            if let Some(top) = over.last_mut() {
                *top = 1 << (B % 8);
            }
            assert_eq!(
                Uint::<N>::from_le_bytes(&over),
                Err(OutOfRange { width: B })
            );
        }
    }
    for_each_width!(check);

    assert_eq!(u64::try_from(Uint::<U64>::from(u64::MAX)), Ok(u64::MAX));
    assert_eq!(u128::try_from(Uint::<U128>::from(u128::MAX)), Ok(u128::MAX));
    assert_eq!(Uint::<U1>::from(true), Uint::ONE);
}

#[test]
fn swap_bytes_matches_reference() {
    fn check<N, const B: usize, const L: usize, const Y: usize>()
    where
        N: Bits + core::ops::Rem<U8, Output = typenum::U0>,
    {
        for a in samples::<N>() {
            let ra = reference::<N, B, L>(a);
            let expected = Reference::<B, L>::from_be_bytes::<Y>(ra.to_le_bytes::<Y>());
            assert_eq!(reference(a.swap_bytes()), expected, "{B}: {a:?}");
        }
    }
    check::<U8, 8, 1, 1>();
    check::<U64, 64, 1, 8>();
    check::<U128, 128, 2, 16>();
    check::<U192, 192, 3, 24>();
    check::<U256, 256, 4, 32>();
    check::<U512, 512, 8, 64>();
}
