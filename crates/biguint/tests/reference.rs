//! Every operation against `ruint`, an independent fixed-width implementation. The widths cover
//! one bit, partial top limbs, exact limb multiples, the 521-bit case of nine limbs, and for
//! arithmetic the 1000-bit case of 16 limbs.

use core::ops::Add;

use biguint::{Bits, Modulus, NonZeroUint, NotReduced, OutOfRange, Pow, Uint, WideningMul};
use generic_array::GenericArray;
use generic_array::sequence::GenericSequence;
use typenum::{
    Sum, U1, U7, U8, U28, U63, U64, U65, U100, U128, U129, U130, U192, U256, U320, U384, U512,
    U521, U1000, U1024,
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
        $check::<U320, 320, 5>();
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
    // The 16-limb sum that WebAssembly runs by 128-bit words, with a partial top limb.
    check::<U1000, 1000, 16>();
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
                    let product = a.checked_mul_mod(b, modulus).map(reference::<N, B, L>);
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
                    let product = a.checked_mul_mod(b, modulus).map(wide);
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
        let values: Vec<Uint<N>> = samples::<N>().into_iter().rev().take(8).collect();
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
            assert_eq!(a.is_zero(), ra.is_zero(), "{B}: {a:?}");
            assert_eq!(NonZeroUint::new(a).is_some(), !ra.is_zero(), "{B}: {a:?}");
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
                if let Ok(signed) = i64::try_from(shift) {
                    assert_eq!(reference(a << signed), ra.wrapping_shl(shift));
                    assert!(
                        (a >> (-signed - 1)).is_zero(),
                        "{B}: {a:?} >> {}",
                        -signed - 1
                    );
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
                assert_eq!(a == b, ra == rb, "{B}: {a:?} == {b:?}");
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

#[test]
fn nonzero_operations_match_reference() {
    fn check<N, const B: usize, const L: usize>()
    where
        N: Bits + Add<N>,
        Sum<N, N>: Bits,
    {
        let values: Vec<NonZeroUint<N>> = samples::<N>()
            .into_iter()
            .filter_map(NonZeroUint::new)
            .collect();
        let nonzero = |value: Option<NonZeroUint<N>>| value.map(|value| reference(value.get()));
        for &a in &values {
            let ra = reference::<N, B, L>(a.get());
            assert_eq!(a.leading_zeros(), ra.leading_zeros(), "{B}: {a:?}");
            assert_eq!(a.trailing_zeros(), ra.trailing_zeros(), "{B}: {a:?}");
            assert_eq!(a.ilog2(), ra.bit_len() - 1, "{B}: {a:?}");
            assert_eq!(a.is_power_of_two(), ra.is_power_of_two(), "{B}: {a:?}");
            for exponent in [0, 1, 2, 3, 7, 64, 521] {
                let expected = (0..exponent).try_fold(Reference::<B, L>::from(1u8), |power, _| {
                    power.checked_mul(ra)
                });
                assert_eq!(
                    nonzero(a.checked_pow(exponent)),
                    expected,
                    "{B}: {a:?}^{exponent}"
                );
            }
            for &b in &values {
                let rb = reference::<N, B, L>(b.get());
                assert_eq!(reference((a | b).get()), ra | rb, "{B}: {a:?} | {b:?}");
                assert_eq!(
                    reference((a | b.get()).get()),
                    ra | rb,
                    "{B}: {a:?} | {b:?}"
                );
                let sum = ra.saturating_add(rb);
                assert_eq!(reference(a.saturating_add(b.get()).get()), sum, "{B}: +");
                assert_eq!(
                    nonzero(a.checked_add(b.get())),
                    ra.checked_add(rb),
                    "{B}: +"
                );
                let product = ra.saturating_mul(rb);
                assert_eq!(reference(a.saturating_mul(b).get()), product, "{B}: *");
                assert_eq!(nonzero(a.checked_mul(b)), ra.checked_mul(rb), "{B}: *");
            }
        }
        assert_eq!(NonZeroUint::<N>::try_from(Uint::ZERO), Err(biguint::IsZero));
    }
    for_each_width!(check);
}

#[test]
fn num_traits_match_reference() {
    use num_traits::ops::overflowing::{OverflowingAdd, OverflowingMul, OverflowingSub};
    use num_traits::{
        Bounded, CheckedAdd, CheckedMul, CheckedNeg, CheckedShl, CheckedShr, CheckedSub, ConstOne,
        ConstZero, FromBytes, FromPrimitive, MulAdd, NumCast, SaturatingAdd, SaturatingMul,
        SaturatingSub, ToBytes, ToPrimitive, WrappingAdd, WrappingMul, WrappingNeg, WrappingShl,
        WrappingShr, WrappingSub,
    };

    /// A trait method on both types, its output in the reference's terms.
    type Binary<N, const B: usize, const L: usize> = (
        &'static str,
        fn(Uint<N>, Uint<N>) -> String,
        fn(Reference<B, L>, Reference<B, L>) -> String,
    );
    type Unary<N, const B: usize, const L: usize> = (
        &'static str,
        fn(Uint<N>) -> String,
        fn(Reference<B, L>) -> String,
    );

    fn r<N: Bits, const B: usize, const L: usize>(value: Uint<N>) -> Reference<B, L> {
        reference(value)
    }

    fn check<N: Bits, const B: usize, const L: usize>() {
        let binary: [Binary<N, B, L>; 13] = [
            (
                "checked_add",
                |a, b| format!("{:?}", CheckedAdd::checked_add(&a, &b).map(r::<N, B, L>)),
                |a, b| format!("{:?}", CheckedAdd::checked_add(&a, &b)),
            ),
            (
                "checked_sub",
                |a, b| format!("{:?}", CheckedSub::checked_sub(&a, &b).map(r::<N, B, L>)),
                |a, b| format!("{:?}", CheckedSub::checked_sub(&a, &b)),
            ),
            (
                "checked_mul",
                |a, b| format!("{:?}", CheckedMul::checked_mul(&a, &b).map(r::<N, B, L>)),
                |a, b| format!("{:?}", CheckedMul::checked_mul(&a, &b)),
            ),
            (
                "wrapping_add",
                |a, b| format!("{:?}", r::<N, B, L>(WrappingAdd::wrapping_add(&a, &b))),
                |a, b| format!("{:?}", WrappingAdd::wrapping_add(&a, &b)),
            ),
            (
                "wrapping_sub",
                |a, b| format!("{:?}", r::<N, B, L>(WrappingSub::wrapping_sub(&a, &b))),
                |a, b| format!("{:?}", WrappingSub::wrapping_sub(&a, &b)),
            ),
            (
                "wrapping_mul",
                |a, b| format!("{:?}", r::<N, B, L>(WrappingMul::wrapping_mul(&a, &b))),
                |a, b| format!("{:?}", WrappingMul::wrapping_mul(&a, &b)),
            ),
            (
                "saturating_add",
                |a, b| format!("{:?}", r::<N, B, L>(SaturatingAdd::saturating_add(&a, &b))),
                |a, b| format!("{:?}", SaturatingAdd::saturating_add(&a, &b)),
            ),
            (
                "saturating_sub",
                |a, b| format!("{:?}", r::<N, B, L>(SaturatingSub::saturating_sub(&a, &b))),
                |a, b| format!("{:?}", SaturatingSub::saturating_sub(&a, &b)),
            ),
            (
                "saturating_mul",
                |a, b| format!("{:?}", r::<N, B, L>(SaturatingMul::saturating_mul(&a, &b))),
                |a, b| format!("{:?}", SaturatingMul::saturating_mul(&a, &b)),
            ),
            (
                "overflowing_add",
                |a, b| {
                    let (sum, overflow) = OverflowingAdd::overflowing_add(&a, &b);
                    format!("{:?}", (r::<N, B, L>(sum), overflow))
                },
                |a, b| format!("{:?}", OverflowingAdd::overflowing_add(&a, &b)),
            ),
            (
                "overflowing_sub",
                |a, b| {
                    let (difference, overflow) = OverflowingSub::overflowing_sub(&a, &b);
                    format!("{:?}", (r::<N, B, L>(difference), overflow))
                },
                |a, b| format!("{:?}", OverflowingSub::overflowing_sub(&a, &b)),
            ),
            (
                "overflowing_mul",
                |a, b| {
                    let (product, overflow) = OverflowingMul::overflowing_mul(&a, &b);
                    format!("{:?}", (r::<N, B, L>(product), overflow))
                },
                |a, b| format!("{:?}", OverflowingMul::overflowing_mul(&a, &b)),
            ),
            (
                "mul_add",
                |a, b| format!("{:?}", r::<N, B, L>(MulAdd::mul_add(a, b, a ^ b))),
                |a, b| format!("{:?}", MulAdd::mul_add(a, b, a ^ b)),
            ),
        ];
        let unary: [Unary<N, B, L>; 12] = [
            (
                "checked_neg",
                |a| format!("{:?}", CheckedNeg::checked_neg(&a).map(r::<N, B, L>)),
                |a| format!("{:?}", CheckedNeg::checked_neg(&a)),
            ),
            (
                "wrapping_neg",
                |a| format!("{:?}", r::<N, B, L>(WrappingNeg::wrapping_neg(&a))),
                |a| format!("{:?}", WrappingNeg::wrapping_neg(&a)),
            ),
            (
                "pow",
                |a| {
                    let powers = EXPONENTS.map(|e| r::<N, B, L>(num_traits::Pow::pow(a, e)));
                    format!("{powers:?}")
                },
                // ruint's `Pow` takes an exponent of its own width.
                |a| {
                    let powers = EXPONENTS.map(|e| {
                        (0..e).fold(<Reference<B, L> as num_traits::One>::one(), |power, _| {
                            WrappingMul::wrapping_mul(&power, &a)
                        })
                    });
                    format!("{powers:?}")
                },
            ),
            (
                "to_u64",
                |a| format!("{:?}", ToPrimitive::to_u64(&a)),
                |a| format!("{:?}", ToPrimitive::to_u64(&a)),
            ),
            (
                "to_i64",
                |a| format!("{:?}", ToPrimitive::to_i64(&a)),
                |a| format!("{:?}", ToPrimitive::to_i64(&a)),
            ),
            (
                "to_u128",
                |a| format!("{:?}", ToPrimitive::to_u128(&a)),
                |a| format!("{:?}", ToPrimitive::to_u128(&a)),
            ),
            (
                "to_i128",
                |a| format!("{:?}", ToPrimitive::to_i128(&a)),
                |a| format!("{:?}", ToPrimitive::to_i128(&a)),
            ),
            (
                "to_f32",
                |a| format!("{:?}", ToPrimitive::to_f32(&a).map(f32::to_bits)),
                |a| format!("{:?}", ToPrimitive::to_f32(&a).map(f32::to_bits)),
            ),
            (
                "to_f64",
                |a| format!("{:?}", ToPrimitive::to_f64(&a).map(f64::to_bits)),
                |a| format!("{:?}", ToPrimitive::to_f64(&a).map(f64::to_bits)),
            ),
            (
                "to_bytes",
                |a| {
                    let (le, be) = (ToBytes::to_le_bytes(&a), ToBytes::to_be_bytes(&a));
                    format!("{:?}", (le.as_slice(), be.as_slice()))
                },
                |a| {
                    let (le, be) = (ToBytes::to_le_bytes(&a), ToBytes::to_be_bytes(&a));
                    format!("{:?}", (le.as_slice(), be.as_slice()))
                },
            ),
            (
                "from_bytes",
                |a| {
                    let le = <Uint<N> as FromBytes>::from_le_bytes(&a.to_le_bytes());
                    let be = <Uint<N> as FromBytes>::from_be_bytes(&a.to_be_bytes());
                    format!("{:?}", (r::<N, B, L>(le), r::<N, B, L>(be)))
                },
                |a| {
                    let le = <Reference<B, L> as FromBytes>::from_le_bytes(&a.to_le_bytes_vec());
                    let be = <Reference<B, L> as FromBytes>::from_be_bytes(&a.to_be_bytes_vec());
                    format!("{:?}", (le, be))
                },
            ),
            (
                "checked_shl, checked_shr, wrapping_shl, wrapping_shr",
                |a| {
                    let shifts: Vec<_> = shift_amounts(B)
                        .into_iter()
                        .filter_map(|shift| u32::try_from(shift).ok())
                        .map(|shift| {
                            (
                                CheckedShl::checked_shl(&a, shift).map(r::<N, B, L>),
                                CheckedShr::checked_shr(&a, shift).map(r::<N, B, L>),
                                r::<N, B, L>(WrappingShl::wrapping_shl(&a, shift)),
                                r::<N, B, L>(WrappingShr::wrapping_shr(&a, shift)),
                            )
                        })
                        .collect();
                    format!("{shifts:?}")
                },
                // ruint's shift traits refuse lost bits. The traits' contract refuses `rhs >= B`
                // and reduces `rhs` modulo `B`, which ruint's operators then shift by.
                |a| {
                    let shifts: Vec<_> = shift_amounts(B)
                        .into_iter()
                        .filter(|&shift| u32::try_from(shift).is_ok())
                        .map(|shift| {
                            (
                                (shift < B).then(|| a << shift),
                                (shift < B).then(|| a >> shift),
                                a << (shift % B),
                                a >> (shift % B),
                            )
                        })
                        .collect();
                    format!("{shifts:?}")
                },
            ),
        ];

        let mut values = samples::<N>();
        // Factors whose bit lengths sum to `B + 1`, with products below and above `2^B`.
        for shift in [1, B / 3, B / 2, B - 1] {
            values.push(Uint::MAX >> shift);
            values.push((Uint::ONE << shift) + Uint::<N>::ONE);
        }
        for &a in &values {
            let ra = reference::<N, B, L>(a);
            for (name, ours, theirs) in &unary {
                assert_eq!(ours(a), theirs(ra), "{B}: {name}({a:?})");
            }
            for &b in &values {
                let rb = reference::<N, B, L>(b);
                for (name, ours, theirs) in &binary {
                    assert_eq!(ours(a, b), theirs(ra, rb), "{B}: {name}({a:?}, {b:?})");
                }
            }
        }

        assert_eq!(
            [
                <Uint<N> as Bounded>::min_value(),
                <Uint<N> as Bounded>::max_value(),
                <Uint<N> as num_traits::One>::one(),
                <Uint<N> as ConstOne>::ONE,
                <Uint<N> as ConstZero>::ZERO,
            ]
            .map(r::<N, B, L>),
            [
                <Reference<B, L> as Bounded>::min_value(),
                <Reference<B, L> as Bounded>::max_value(),
                <Reference<B, L> as num_traits::One>::one(),
                <Reference<B, L> as ConstOne>::ONE,
                <Reference<B, L> as ConstZero>::ZERO,
            ],
            "{B}: constants"
        );

        let power = |exponent: usize| 2f64.powi(exponent as i32);
        let floats = [
            0.0,
            -0.0,
            -0.5,
            -1.0,
            0.75,
            2.9,
            1e30,
            f64::NAN,
            f64::INFINITY,
            f64::NEG_INFINITY,
            f64::MAX,
            power(B - 1),
            power(B),
            power(B) * (1.0 - f64::EPSILON / 2.0),
        ];
        let sampled = values
            .iter()
            .map(|&value| <f64 as From<_>>::from(reference::<N, B, L>(value)));
        for float in floats.into_iter().chain(sampled) {
            assert_eq!(
                <Uint<N> as FromPrimitive>::from_f64(float).map(r::<N, B, L>),
                <Reference<B, L> as FromPrimitive>::from_f64(float),
                "{B}: from_f64({float})"
            );
            assert_eq!(
                <Uint<N> as NumCast>::from(float).map(r::<N, B, L>),
                <Reference<B, L> as NumCast>::from(float),
                "{B}: NumCast::from({float})"
            );
        }
        let integers = [
            0,
            1,
            -1,
            127,
            128,
            <i128 as From<i64>>::from(i64::MIN),
            <i128 as From<u64>>::from(u64::MAX),
            i128::MAX,
            i128::MIN,
        ];
        for integer in integers {
            let unsigned = integer.cast_unsigned();
            let (low, signed_low) = (unsigned as u64, integer as i64);
            let ours = (
                <Uint<N> as FromPrimitive>::from_u64(low).map(r::<N, B, L>),
                <Uint<N> as FromPrimitive>::from_i64(signed_low).map(r::<N, B, L>),
                <Uint<N> as FromPrimitive>::from_u128(unsigned).map(r::<N, B, L>),
                <Uint<N> as FromPrimitive>::from_i128(integer).map(r::<N, B, L>),
                <Uint<N> as NumCast>::from(integer).map(r::<N, B, L>),
            );
            let theirs = (
                <Reference<B, L> as FromPrimitive>::from_u64(low),
                <Reference<B, L> as FromPrimitive>::from_i64(signed_low),
                <Reference<B, L> as FromPrimitive>::from_u128(unsigned),
                <Reference<B, L> as FromPrimitive>::from_i128(integer),
                <Reference<B, L> as NumCast>::from(integer),
            );
            assert_eq!(ours, theirs, "{B}: from {integer}");
        }
    }

    const EXPONENTS: [u32; 7] = [0, 1, 2, 3, 7, 64, 521];

    check::<U1, 1, 1>();
    check::<U7, 7, 1>();
    check::<U64, 64, 1>();
    check::<U129, 129, 3>();
    check::<U256, 256, 4>();
    check::<U521, 521, 9>();
}
