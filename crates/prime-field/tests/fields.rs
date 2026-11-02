//! Every operation against `num-bigint` over a pseudo-random sweep and edges around `0`, `p / 2`
//! and `p`, through `Fp`, `Secret` and the `const_*` twins, for moduli of every representation:
//! below 2^64, Mersenne, pseudo-Mersenne, Montgomery with and without a spare bit, `p = 1 mod 4`
//! and `p = 3 mod 4`, one and nine limbs.

use biguint::typenum::{U7, U31, U64, U127, U254, U256, U381, U521};
use biguint::{Bits, Uint};
use num_bigint::BigUint;
use prime_field::{Fp, Modulus, NotCanonical, Secret};

/// Declares one modulus marker per line: the name, the width and the little-endian limbs.
macro_rules! moduli {
    ($($name:ident: $width:ty = [$($limb:expr),* $(,)?];)*) => {
        $(
            #[derive(Clone, Copy)]
            struct $name;

            impl Modulus for $name {
                type Width = $width;
                const MODULUS: Uint<$width> = Uint::from_limbs([$($limb),*]);
            }
        )*

        /// Runs the sweep on every modulus.
        fn each_modulus(sweep: impl Fn(&str, &BigUint, &dyn Fn(&mut Rng))) {
            $(sweep(stringify!($name), &big(<$name>::MODULUS.as_limbs()), &|rng| check::<$name>(rng));)*
        }
    };
}

moduli! {
    Seven: U7 = [7];
    Mersenne31: U31 = [0x7fff_ffff];
    BabyBear: U31 = [0x7800_0001];
    Goldilocks: U64 = [0xffff_ffff_0000_0001];
    Mersenne127: U127 = [0xffff_ffff_ffff_ffff, 0x7fff_ffff_ffff_ffff];
    Bn254Scalar: U254 = [
        0x43e1_f593_f000_0001,
        0x2833_e848_79b9_7091,
        0xb850_45b6_8181_585d,
        0x3064_4e72_e131_a029,
    ];
    Secp256k1Base: U256 = [
        0xffff_fffe_ffff_fc2f,
        0xffff_ffff_ffff_ffff,
        0xffff_ffff_ffff_ffff,
        0xffff_ffff_ffff_ffff,
    ];
    Secp256k1Scalar: U256 = [
        0xbfd2_5e8c_d036_4141,
        0xbaae_dce6_af48_a03b,
        0xffff_ffff_ffff_fffe,
        0xffff_ffff_ffff_ffff,
    ];
    P256Base: U256 = [
        0xffff_ffff_ffff_ffff,
        0x0000_0000_ffff_ffff,
        0x0000_0000_0000_0000,
        0xffff_ffff_0000_0001,
    ];
    P256Scalar: U256 = [
        0xf3b9_cac2_fc63_2551,
        0xbce6_faad_a717_9e84,
        0xffff_ffff_ffff_ffff,
        0xffff_ffff_0000_0000,
    ];
    Bls12381Base: U381 = [
        0xb9fe_ffff_ffff_aaab,
        0x1eab_fffe_b153_ffff,
        0x6730_d2a0_f6b0_f624,
        0x6477_4b84_f385_12bf,
        0x4b1b_a7b6_434b_acd7,
        0x1a01_11ea_397f_e69a,
    ];
    P521Base: U521 = [
        u64::MAX, u64::MAX, u64::MAX, u64::MAX, u64::MAX, u64::MAX, u64::MAX, u64::MAX,
        0x1ff,
    ];
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

    /// A uniform value below `bound`.
    fn below(&mut self, bound: &BigUint) -> BigUint {
        let bits = bound.bits();
        let limbs = bits.div_ceil(64) as usize;
        loop {
            let mut digits: Vec<u64> = (0..limbs).map(|_| self.next_u64()).collect();
            if let Some(top) = digits.last_mut() {
                *top &= u64::MAX >> (63 - (bits - 1) % 64);
            }
            let value = BigUint::from_slice(&to_u32s(&digits));
            if value < *bound {
                return value;
            }
        }
    }
}

fn to_u32s(limbs: &[u64]) -> Vec<u32> {
    limbs
        .iter()
        .flat_map(|limb| [*limb as u32, (*limb >> 32) as u32])
        .collect()
}

fn big(limbs: &[u64]) -> BigUint {
    BigUint::from_slice(&to_u32s(limbs))
}

fn uint<N: Bits>(value: &BigUint) -> Uint<N> {
    let mut limbs = biguint::generic_array::GenericArray::<u64, N::Limbs>::default();
    for (slot, digit) in limbs.iter_mut().zip(value.to_u64_digits()) {
        *slot = digit;
    }
    Uint::from_generic_limbs(limbs)
}

fn element<M: Modulus>(value: &BigUint) -> Fp<M> {
    Fp::from_uint(uint(value)).expect("a value below the modulus")
}

fn value<M: Modulus>(element: Fp<M>) -> BigUint {
    big(element.to_uint().as_limbs())
}

/// Asserts that every result of the operation `name` on `(a, b)` is `expected`.
fn agree<M: Modulus>(name: &str, a: &BigUint, b: &BigUint, expected: &BigUint, results: &[Fp<M>]) {
    for (index, result) in results.iter().enumerate() {
        assert_eq!(value(*result), *expected, "{name} #{index} of ({a}, {b})");
    }
}

/// Asserts that `root` squares to `a` exactly when `a` is a quadratic residue.
fn check_root<M: Modulus>(name: &str, a: &BigUint, residue: bool, root: Option<Fp<M>>) {
    match root {
        Some(root) => {
            assert!(residue, "{name} of the non-residue {a}");
            assert_eq!(value(root.square()), *a, "{name} squared");
        }
        None => assert!(!residue, "no {name} of the residue {a}"),
    }
}

/// The sweep on the field of `M`: 64 random pairs and every pair of edges. Each operation runs
/// through `Fp`, `Secret` (with a public `Fp` on either side) and the `const_*` twins. The edges
/// include `p - 1 + 0`, whose sum has the top limb of `p` and lies below it: the rare branch of
/// the variable-time reduction.
fn check<M: Modulus>(rng: &mut Rng) {
    let p = big(M::MODULUS.as_limbs());
    let half = &p >> 1u8;
    let edges = [
        BigUint::ZERO,
        BigUint::from(1u8),
        BigUint::from(2u8),
        half.clone(),
        &half + 1u8,
        &p - 2u8,
        &p - 1u8,
    ];
    let mut pairs: Vec<(BigUint, BigUint)> = edges
        .iter()
        .flat_map(|a| edges.iter().map(move |b| (a.clone(), b.clone())))
        .collect();
    pairs.extend((0..64).map(|_| (rng.below(&p), rng.below(&p))));
    for (a, b) in &pairs {
        let (x, y) = (element::<M>(a), element::<M>(b));
        let (s, t) = (Secret::new(x), Secret::new(y));
        let assigned = |mut target: Secret<M>, apply: &dyn Fn(&mut Secret<M>)| {
            apply(&mut target);
            target.reveal()
        };
        let sum = (a + b) % &p;
        let mut plain = x;
        plain += y;
        agree::<M>(
            "add",
            a,
            b,
            &sum,
            &[
                x + y,
                (s + t).reveal(),
                (s + y).reveal(),
                (x + t).reveal(),
                plain,
                assigned(s, &|target| *target += t),
                assigned(s, &|target| *target += y),
                x.const_add(y),
            ],
        );
        let difference = (a + &p - b) % &p;
        let mut plain = x;
        plain -= y;
        agree::<M>(
            "sub",
            a,
            b,
            &difference,
            &[
                x - y,
                (s - t).reveal(),
                (s - y).reveal(),
                (x - t).reveal(),
                plain,
                assigned(s, &|target| *target -= t),
                assigned(s, &|target| *target -= y),
                x.const_sub(y),
            ],
        );
        let negation = (&p - a) % &p;
        agree::<M>("neg", a, b, &negation, &[-x, (-s).reveal(), x.const_neg()]);
        let double = (a + a) % &p;
        agree::<M>(
            "double",
            a,
            b,
            &double,
            &[x.double(), s.double().reveal(), x.const_double()],
        );
        let product = a * b % &p;
        let mut plain = x;
        plain *= y;
        agree::<M>(
            "mul",
            a,
            b,
            &product,
            &[
                x * y,
                (s * t).reveal(),
                (s * y).reveal(),
                (x * t).reveal(),
                plain,
                assigned(s, &|target| *target *= t),
                assigned(s, &|target| *target *= y),
                x.const_mul(y),
            ],
        );
        let square = a * a % &p;
        agree::<M>(
            "square",
            a,
            b,
            &square,
            &[x.square(), s.square().reveal(), x.const_square()],
        );
        let power = a.modpow(b, &p);
        let exponent = uint::<M::Width>(b);
        agree::<M>(
            "pow",
            a,
            b,
            &power,
            &[x.pow(exponent), s.pow(exponent).reveal()],
        );
        let inverse = if *a == BigUint::ZERO {
            None
        } else {
            Some(a.modpow(&(&p - 2u8), &p))
        };
        assert_eq!(x.invert().map(value), inverse, "invert of {a}");
        assert_eq!(
            s.invert().map(|inverse| value(inverse.reveal())),
            inverse,
            "secret invert of {a}"
        );
        let residue = *a == BigUint::ZERO || a.modpow(&(&p >> 1u8), &p) == BigUint::from(1u8);
        check_root("sqrt", a, residue, x.sqrt());
        check_root("secret sqrt", a, residue, s.sqrt().map(Secret::reveal));
        assert_eq!(x == y, a == b, "eq");
        assert_eq!(x.is_zero(), *a == BigUint::ZERO, "is_zero");
        assert_eq!(Fp::<M>::from_le_bytes(&x.to_le_bytes()), Ok(x), "le bytes");
        assert_eq!(Fp::<M>::from_be_bytes(&x.to_be_bytes()), Ok(x), "be bytes");
    }
    assert_eq!(Fp::<M>::from_uint(M::MODULUS), Err(NotCanonical));
    assert_eq!(
        Fp::<M>::from_le_bytes(&M::MODULUS.to_le_bytes()),
        Err(NotCanonical)
    );
    let word = rng.next_u64();
    assert_eq!(
        value(Fp::<M>::from_u64(word)),
        BigUint::from(word) % &p,
        "from_u64"
    );
    assert_eq!(value(Fp::<M>::ONE), BigUint::from(1u8));
    assert_eq!(value(Fp::<M>::ZERO), BigUint::ZERO);
}

#[test]
fn every_operation_matches_num_bigint() {
    each_modulus(|name, p, sweep| {
        let mut rng = Rng(p.bits());
        eprintln!("{name}");
        sweep(&mut rng);
    });
}

/// The generators of secp256k1 and P-256 satisfy their curve equations, as the standards give
/// them (SEC 2, FIPS 186-4).
#[test]
fn generators_lie_on_their_curves() {
    type K = Fp<Secp256k1Base>;
    let x = K::from_be_bytes(
        &hex("79be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798").into(),
    )
    .expect("canonical");
    let y = K::from_be_bytes(
        &hex("483ada7726a3c4655da4fbfc0e1108a8fd17b448a68554199c47d08ffb10d4b8").into(),
    )
    .expect("canonical");
    assert_eq!(y.square(), x.square() * x + K::from_u64(7));

    type P = Fp<P256Base>;
    let x = P::from_be_bytes(
        &hex("6b17d1f2e12c4247f8bce6e563a440f277037d812deb33a0f4a13945d898c296").into(),
    )
    .expect("canonical");
    let y = P::from_be_bytes(
        &hex("4fe342e2fe1a7f9b8ee7eb4a7c0f9e162bce33576b315ececbb6406837bf51f5").into(),
    )
    .expect("canonical");
    let b = P::from_be_bytes(
        &hex("5ac635d8aa3a93e7b3ebbd55769886bc651d06b0cc53b0f63bce3c3e27d2604b").into(),
    )
    .expect("canonical");
    assert_eq!(y.square(), x.square() * x - x - x - x + b);
}

fn hex(text: &str) -> [u8; 32] {
    let mut bytes = [0u8; 32];
    for (slot, pair) in bytes.iter_mut().zip(text.as_bytes().chunks(2)) {
        *slot = u8::from_str_radix(std::str::from_utf8(pair).expect("ascii"), 16).expect("hex");
    }
    bytes
}
