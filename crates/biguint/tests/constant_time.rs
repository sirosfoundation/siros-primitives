//! Every operation of `ConstantTime<Uint<N>>` and `ConstantTimeResidue` against the variable-time
//! `Uint` and `Residue` paths, at 64, 256, 521 and 4096 bits, under odd and even moduli and
//! divisors: one table of operations, run on edge and random operands.

use core::ops::Add;

use biguint::typenum::{U64, U256, U521, U4096};
use biguint::{Bits, ConstantTime, Modulus, NonZeroUint, Pow, Uint};
use generic_array::GenericArray;
use generic_array::sequence::GenericSequence;
use typenum::Sum;

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

fn generate<N: Bits>(limb: impl FnMut(usize) -> u64) -> Uint<N> {
    Uint::from_generic_limbs(GenericArray::generate(limb))
}

/// Zero, one, all ones, the top bit, and two random values, one of them shortened.
fn operands<N: Bits>(rng: &mut Rng) -> Vec<Uint<N>> {
    let full = generate::<N>(|_| rng.next_u64());
    let short = generate::<N>(|_| rng.next_u64()) >> (rng.next_u64() as usize % N::BITS);
    vec![
        Uint::ZERO,
        Uint::ONE,
        !Uint::ZERO,
        Uint::ONE << (N::BITS - 1),
        full,
        short,
        // Over the divisor `2^127 + 2^64 - 1`: a quotient limb that the top divisor limb
        // overestimates by two, and a window whose top limb equals the divisor's.
        generate::<N>(|index| {
            [0, u64::MAX, (1 << 63) - 1]
                .get(index)
                .copied()
                .unwrap_or(0)
        }),
        generate::<N>(|index| [5, u64::MAX - 1, 1 << 63].get(index).copied().unwrap_or(0)),
    ]
}

/// Odd and even divisors: all ones and its even neighbour, one-limb divisors, random ones of full
/// and partial length, and two-limb ones whose top limb is `2^63`.
fn divisors<N: Bits>(rng: &mut Rng) -> Vec<NonZeroUint<N>> {
    let max = !Uint::<N>::ZERO;
    let random = generate::<N>(|_| rng.next_u64());
    let partial = random >> (N::BITS / 2);
    [
        max,
        max - Uint::<N>::ONE,
        Uint::from_u64(3),
        Uint::from_u64(10),
        random | Uint::<N>::ONE,
        random & !Uint::<N>::ONE,
        partial | Uint::<N>::ONE,
        (partial & !Uint::<N>::ONE) | Uint::<N>::from_u64(2),
        generate::<N>(|index| [u64::MAX, 1 << 63].get(index).copied().unwrap_or(0)),
        generate::<N>(|index| [u64::MAX - 1, 1 << 63].get(index).copied().unwrap_or(0)),
    ]
    .into_iter()
    .filter_map(NonZeroUint::new)
    .collect()
}

/// The operations on two operands and a divisor: the constant-time result, revealed, against the
/// variable-time one.
type Case<N> = (
    &'static str,
    fn(Uint<N>, Uint<N>, NonZeroUint<N>) -> Uint<N>,
    fn(Uint<N>, Uint<N>, NonZeroUint<N>) -> Uint<N>,
);

/// The operations modulo `2^N`, which ignore the divisor.
fn integer_cases<N: Bits>() -> Vec<Case<N>> {
    vec![
        (
            "a + b",
            |a, b, _| (ConstantTime::new(a) + ConstantTime::new(b)).reveal(),
            |a, b, _| a + b,
        ),
        (
            "a + public",
            |a, b, _| (ConstantTime::new(a) + b).reveal(),
            |a, b, _| a + b,
        ),
        (
            "public + a",
            |a, b, _| (a + ConstantTime::new(b)).reveal(),
            |a, b, _| a + b,
        ),
        (
            "a - b",
            |a, b, _| (ConstantTime::new(a) - ConstantTime::new(b)).reveal(),
            |a, b, _| a - b,
        ),
        (
            "a * b",
            |a, b, _| (ConstantTime::new(a) * ConstantTime::new(b)).reveal(),
            |a, b, _| a * b,
        ),
        (
            "a * public",
            |a, b, _| (ConstantTime::new(a) * b).reveal(),
            |a, b, _| a * b,
        ),
        (
            "public * a",
            |a, b, _| (a * ConstantTime::new(b)).reveal(),
            |a, b, _| a * b,
        ),
        (
            "a *= b",
            |a, b, _| {
                let mut x = ConstantTime::new(a);
                x *= ConstantTime::new(b);
                x *= b;
                x.reveal()
            },
            |a, b, _| a * b * b,
        ),
        (
            "a * b of 64 bits",
            |a, b, _| (ConstantTime::new(a) * ConstantTime::new(b.resize::<U64>())).reveal(),
            |a, b, _| a * b.resize::<U64>(),
        ),
        (
            "a & b",
            |a, b, _| (ConstantTime::new(a) & ConstantTime::new(b)).reveal(),
            |a, b, _| a & b,
        ),
        (
            "a | b",
            |a, b, _| (ConstantTime::new(a) | b).reveal(),
            |a, b, _| a | b,
        ),
        (
            "a ^ b",
            |a, b, _| (a ^ ConstantTime::new(b)).reveal(),
            |a, b, _| a ^ b,
        ),
        (
            "!a",
            |a, _, _| (!ConstantTime::new(a)).reveal(),
            |a, _, _| !a,
        ),
        (
            "a << 67",
            |a, _, _| (ConstantTime::new(a) << 67u32).reveal(),
            |a, _, _| a << 67u32,
        ),
        (
            "a >> 5",
            |a, _, _| (ConstantTime::new(a) >> 5u32).reveal(),
            |a, _, _| a >> 5u32,
        ),
        (
            "a <<= N - 1, a >>= 1",
            |a, _, _| {
                let mut x = ConstantTime::new(a);
                x <<= N::BITS - 1;
                x >>= 1usize;
                x.reveal()
            },
            |a, _, _| (a << (N::BITS - 1)) >> 1usize,
        ),
        (
            "a.square()",
            |a, _, _| ConstantTime::new(a).square().reveal(),
            |a, _, _| a * a,
        ),
        (
            "a.double()",
            |a, _, _| ConstantTime::new(a).double().reveal(),
            |a, _, _| a + a,
        ),
        (
            "a.pow(b of 64 bits)",
            |a, b, _| {
                ConstantTime::new(a)
                    .pow(ConstantTime::new(b.resize::<U64>()))
                    .reveal()
            },
            |a, b, _| Pow::pow(a, b.resize::<U64>()),
        ),
    ]
}

/// The operations by a modulus or a divisor.
fn modular_cases<N>() -> Vec<Case<N>>
where
    N: Bits + Add<N> + Add<U64>,
    Sum<N, N>: Bits,
    Sum<N, U64>: Bits,
{
    vec![
        (
            "a.add_mod(b, m)",
            |a, b, m| {
                ConstantTime::new(a)
                    .add_mod(ConstantTime::new(b), m)
                    .reveal()
            },
            |a, b, m| a.add_mod(b, m),
        ),
        (
            "a.sub_mod(b, m)",
            |a, b, m| {
                ConstantTime::new(a)
                    .sub_mod(ConstantTime::new(b), m)
                    .reveal()
            },
            |a, b, m| a.sub_mod(b, m),
        ),
        (
            "a.neg_mod(m)",
            |a, _, m| ConstantTime::new(a).neg_mod(m).reveal(),
            |a, _, m| a.neg_mod(m),
        ),
        (
            "a.mul_mod(b, m)",
            |a, b, m| {
                ConstantTime::new(a)
                    .mul_mod(ConstantTime::new(b), m)
                    .reveal()
            },
            |a, b, m| a.mul_mod(b, m),
        ),
        (
            "a.mul_mod(b of 64 bits, m)",
            |a, b, m| {
                ConstantTime::new(a)
                    .mul_mod(ConstantTime::new(b.resize::<U64>()), m)
                    .reveal()
            },
            |a, b, m| a.mul_mod(b.resize::<U64>(), m),
        ),
        (
            "a / m",
            |a, _, m| (ConstantTime::new(a) / m).reveal(),
            |a, _, m| a / m,
        ),
        (
            "a % m",
            |a, _, m| (ConstantTime::new(a) % m).reveal(),
            |a, _, m| a % m,
        ),
        (
            "a.div_rem(m).0 + a.div_rem(m).1",
            |a, _, m| {
                let (q, r) = ConstantTime::new(a).div_rem(m);
                (q + r).reveal()
            },
            |a, _, m| {
                let (q, r) = a.div_rem(m);
                q + r
            },
        ),
        (
            "a % (m of 64 bits)",
            |a, _, m| match NonZeroUint::new(m.get().resize::<U64>()) {
                Some(m) => (ConstantTime::new(a) % m).reveal().resize(),
                None => Uint::ZERO,
            },
            |a, _, m| match NonZeroUint::new(m.get().resize::<U64>()) {
                Some(m) => (a % m).resize(),
                None => Uint::ZERO,
            },
        ),
    ]
}

/// The operations of `ConstantTimeResidue` against those of `Residue`, under one modulus.
fn check_residues<N: Bits>(pairs: &[(Uint<N>, Uint<N>)], modulus: NonZeroUint<N>)
where
    Uint<N>: Copy,
{
    let context = Modulus::from(modulus);
    context.scope(|ring| {
        for &(a, b) in pairs {
            let (x, public_x) = (
                ring.reduce_constant_time(ConstantTime::new(a)),
                ring.reduce(a),
            );
            assert_eq!(
                x.get().reveal(),
                public_x.get(),
                "reduce {a:?} mod {modulus:?}"
            );
            assert_eq!((-x).get().reveal(), (-public_x).get(), "-{a:?}");
            assert_eq!(
                x.square().get().reveal(),
                public_x.square().get(),
                "{a:?}^2"
            );
            let exponent = a.resize::<U64>();
            assert_eq!(
                x.pow(ConstantTime::new(exponent)).get().reveal(),
                public_x.pow(exponent).get(),
                "{a:?}^{exponent:?} mod {modulus:?}"
            );
            let (y, public_y) = (
                ring.reduce_constant_time(ConstantTime::new(b)),
                ring.reduce(b),
            );
            let results = [
                ((x + y).get(), public_x + public_y),
                ((x - public_y).get(), public_x - public_y),
                ((public_x * y).get(), public_x * public_y),
                ((x * y).get(), public_x * public_y),
            ];
            for (wrapped, public) in results {
                assert_eq!(
                    wrapped.reveal(),
                    public.get(),
                    "{a:?}, {b:?} mod {modulus:?}"
                );
            }
        }
    });
}

fn check_width<N>()
where
    N: Bits + Add<N> + Add<U64>,
    Sum<N, N>: Bits,
    Sum<N, U64>: Bits,
    Uint<N>: Copy,
{
    let mut rng = Rng(N::BITS as u64);
    let values = operands::<N>(&mut rng);
    let divisors = divisors::<N>(&mut rng);
    // Each operand meets the next one: every edge value meets another edge value.
    let pairs: Vec<_> = values
        .iter()
        .copied()
        .zip(values.iter().copied().cycle().skip(1))
        .collect();
    let check = |cases: &[Case<N>], m: NonZeroUint<N>| {
        for &(a, b) in &pairs {
            for (name, wrapped, public) in cases {
                assert_eq!(
                    wrapped(a, b, m),
                    public(a, b, m),
                    "{name} at {} bits, a = {a:?}, b = {b:?}, m = {m:?}",
                    N::BITS
                );
            }
        }
    };
    if let Some(&m) = divisors.first() {
        check(&integer_cases(), m);
    }
    let modular = modular_cases();
    for &m in &divisors {
        check(&modular, m);
        check_residues(&pairs, m);
    }
}

#[test]
fn constant_time_operations_match_the_variable_time_paths() {
    check_width::<U64>();
    check_width::<U256>();
    check_width::<U521>();
    check_width::<U4096>();
}
