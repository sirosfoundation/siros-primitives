//! Every operation against `num-bigint` over a pseudo-random sweep and edges around `0`, `p / 2`
//! and `p`, through `Fp`, `ConstantTime` and the `const_*` twins, for moduli of every
//! representation: below 2^64, Mersenne, pseudo-Mersenne, Montgomery with and without a spare
//! bit, `p = 1 mod 4` and `p = 3 mod 4`, one and nine limbs.

use biguint::typenum::{
    IsLess, True, U7, U30, U63, U64, U126, U127, U128, U253, U255, U256, U380, U520,
};
use biguint::{Bits, NonZero, NonZeroUint, Uint};
use num_bigint::BigUint;
use prime_field::standard::{
    BabyBear, Bls12381Base, Bn254Scalar, Goldilocks, Mersenne31, P256Base, P256Scalar, P521Base,
    Secp256k1Base, Tom256Base, Tom256Scalar,
};
use prime_field::{ConstantTime, Fp, Modulus, NotCanonical};

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
    };
}

moduli! {
    Seven: U7 = [7];
    Mersenne127: U127 = [0xffff_ffff_ffff_ffff, 0x7fff_ffff_ffff_ffff];
    Secp256k1Scalar: U256 = [
        0xbfd2_5e8c_d036_4141,
        0xbaae_dce6_af48_a03b,
        0xffff_ffff_ffff_fffe,
        0xffff_ffff_ffff_ffff,
    ];
}

/// Runs the sweep of `$check` on each modulus marker.
macro_rules! each_modulus {
    ($sweep:expr, $check:ident, $($name:ident),* $(,)?) => {
        $($sweep(stringify!($name), &big(<$name>::MODULUS.as_limbs()), &|rng: &mut Rng| $check::<$name>(rng));)*
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
/// through `Fp`, `ConstantTime` (with a public `Fp` on either side) and the `const_*` twins. The
/// edges include `p - 1 + 0`, whose sum has the top limb of `p` and lies below it: the rare branch
/// of the variable-time reduction.
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
        let (s, t) = (ConstantTime::new(x), ConstantTime::new(y));
        let assigned = |mut target: ConstantTime<Fp<M>>,
                        apply: &dyn Fn(&mut ConstantTime<Fp<M>>)| {
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
        let inverse = (*a != BigUint::ZERO).then(|| a.modpow(&(&p - 2u8), &p));
        assert_eq!(
            NonZero::new(x).map(|x| value(x.invert().get())),
            inverse,
            "invert of {a}"
        );
        assert_eq!(
            s.nonzero().map(|s| value(s.invert().reveal().get())),
            inverse,
            "secret invert of {a}"
        );
        let residue = *a == BigUint::ZERO || a.modpow(&(&p >> 1u8), &p) == BigUint::from(1u8);
        check_root("sqrt", a, residue, x.sqrt());
        check_root(
            "secret sqrt",
            a,
            residue,
            s.sqrt().map(ConstantTime::reveal),
        );
        assert_eq!(x == y, a == b, "eq");
        assert_eq!(x.is_zero(), *a == BigUint::ZERO, "is_zero");
        let integer = uint::<M::Width>(a);
        assert_eq!(x.to_le_bytes(), integer.to_le_bytes(), "to le bytes of {a}");
        assert_eq!(x.to_be_bytes(), integer.to_be_bytes(), "to be bytes of {a}");
        assert_eq!(
            Fp::<M>::from_le_bytes(&integer.to_le_bytes()),
            Ok(x),
            "from le bytes of {a}"
        );
        assert_eq!(
            Fp::<M>::from_be_bytes(&integer.to_be_bytes()),
            Ok(x),
            "from be bytes of {a}"
        );
        assert_eq!(Fp::<M>::try_from(integer), Ok(x), "try_from of {a}");
        assert_eq!(Uint::from(x), integer, "Uint from {a}");
    }
    assert_eq!(Fp::<M>::from_uint(M::MODULUS), Err(NotCanonical));
    assert_eq!(
        Fp::<M>::from_le_bytes(&M::MODULUS.to_le_bytes()),
        Err(NotCanonical)
    );
    // The words at the edges of the folds of a Mersenne-31 reduction: multiples of `2^31 - 1`
    // and of `2^31`, and the top of the range.
    for word in [
        0,
        1,
        0x7fff_ffff,
        0xffff_fffe,
        0x3fff_ffff_0000_0001,
        1 << 31,
        1 << 62,
        u64::MAX - 1,
        u64::MAX,
        rng.next_u64(),
    ] {
        let expected = BigUint::from(word) % &p;
        assert_eq!(
            value(Fp::<M>::from_u64(word)),
            expected,
            "from_u64 of {word}"
        );
        assert_eq!(value(Fp::<M>::from(word)), expected, "From<u64> of {word}");
        assert_eq!(
            value(Fp::<M>::ONE * Uint::<biguint::typenum::U64>::from_u64(word)),
            expected,
            "1 * {word}"
        );
    }
    assert_eq!(value(Fp::<M>::ONE), BigUint::from(1u8));
    assert_eq!(value(Fp::<M>::ZERO), BigUint::ZERO);
}

#[test]
fn every_operation_matches_num_bigint() {
    let sweep = |name: &str, p: &BigUint, sweep: &dyn Fn(&mut Rng)| {
        let mut rng = Rng(p.bits());
        eprintln!("{name}");
        sweep(&mut rng);
    };
    each_modulus!(
        sweep,
        check,
        Seven,
        Mersenne31,
        BabyBear,
        Goldilocks,
        Mersenne127,
        Bn254Scalar,
        Secp256k1Base,
        Secp256k1Scalar,
        P256Base,
        P256Scalar,
        Tom256Base,
        Bls12381Base,
        P521Base,
    );
}

/// Each modulus of `standard` against its definition or the modulus of an independent crate.
#[test]
fn standard_moduli_are_the_standards() {
    use halo2curves::ff::PrimeField;
    let power = |exponent: u32| BigUint::from(1u8) << exponent;
    let halo2 = |modulus: &str| {
        BigUint::parse_bytes(modulus.trim_start_matches("0x").as_bytes(), 16).expect("hex")
    };
    let table = [
        (big(Mersenne31::MODULUS.as_limbs()), power(31) - 1u8),
        (big(BabyBear::MODULUS.as_limbs()), power(27) * 15u8 + 1u8),
        (
            big(Goldilocks::MODULUS.as_limbs()),
            power(64) - power(32) + 1u8,
        ),
        (
            big(Bn254Scalar::MODULUS.as_limbs()),
            halo2(halo2curves::bn256::Fr::MODULUS),
        ),
        (
            big(Secp256k1Base::MODULUS.as_limbs()),
            power(256) - power(32) - 977u16,
        ),
        (
            big(P256Base::MODULUS.as_limbs()),
            power(256) - power(224) + power(192) + power(96) - 1u8,
        ),
        (
            big(P256Scalar::MODULUS.as_limbs()),
            halo2(halo2curves::secp256r1::Fq::MODULUS),
        ),
        (
            big(Tom256Base::MODULUS.as_limbs()),
            halo2(halo2curves::t256::Fp::MODULUS),
        ),
        (
            big(Tom256Scalar::MODULUS.as_limbs()),
            halo2(halo2curves::t256::Fq::MODULUS),
        ),
        (
            big(Bls12381Base::MODULUS.as_limbs()),
            halo2(halo2curves::bls12381::Fq::MODULUS),
        ),
        (big(P521Base::MODULUS.as_limbs()), power(521) - 1u8),
    ];
    for (index, (ours, expected)) in table.iter().enumerate() {
        assert_eq!(ours, expected, "modulus {index}");
    }
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

/// The width of the oracle's integers: twice the widest field and more.
type Oracle = biguint::typenum::U2048;

/// `x k`, `x + k` and `x - k` for one `k` of the width `K`, through `Fp`, the assigning form,
/// `ConstantTime` with `k` wrapped, and `ConstantTime` with `k` public: one row per operation.
fn integer_results<M: Modulus, K: Bits>(x: Fp<M>, k: Uint<K>) -> [[Fp<M>; 4]; 3] {
    let s = ConstantTime::new(x);
    let t = ConstantTime::new(k);
    let (mut product, mut sum, mut difference) = (x, x, x);
    product *= k;
    sum += k;
    difference -= k;
    [
        [x * k, product, (s * t).reveal(), (s * k).reveal()],
        [x + k, sum, (s + t).reveal(), (s + k).reveal()],
        [x - k, difference, (s - t).reveal(), (s - k).reveal()],
    ]
}

/// A uniform value of `bits` bits.
fn random_bits(rng: &mut Rng, bits: usize) -> BigUint {
    let digits: Vec<u64> = (0..bits.div_ceil(64)).map(|_| rng.next_u64()).collect();
    BigUint::from_slice(&to_u32s(&digits)) >> (64 * digits.len() - bits)
}

/// The integer operands of the field of `M` against `biguint`'s `mul_mod`, `add_mod` and
/// `sub_mod` on the canonical integers, for `k` in every width that holds it: 64 and 128 bits,
/// the field's width and the oracle's. `From<u64>` against `k mod p`.
fn check_integers<M: Modulus>(rng: &mut Rng) {
    let p = big(M::MODULUS.as_limbs());
    let width = <M::Width as Bits>::BITS;
    let modulus = NonZero::new(uint::<Oracle>(&p)).expect("a nonzero modulus");
    let mut ks = vec![
        BigUint::ZERO,
        BigUint::from(1u8),
        BigUint::from(2u8),
        &p - 1u8,
        p.clone(),
        &p + 1u8,
        BigUint::from(u64::MAX),
    ];
    for bits in [64, 128, width, 2 * width] {
        ks.extend((0..4).map(|_| random_bits(rng, bits)));
    }
    let xs = [BigUint::ZERO, BigUint::from(1u8), &p - 1u8, rng.below(&p)];
    for a in &xs {
        let x = element::<M>(a);
        let integer = uint::<Oracle>(a);
        for k in &ks {
            let wide = uint::<Oracle>(k);
            let reduced = wide % modulus;
            let expected = [
                integer.mul_mod(wide, modulus),
                integer.add_mod(reduced, modulus),
                integer.sub_mod(reduced, modulus),
            ]
            .map(|result| big(result.as_limbs()));
            let mut rows = vec![integer_results(x, wide)];
            if k.bits() <= 64 {
                let word = uint::<biguint::typenum::U64>(k);
                rows.push(integer_results(x, word));
                assert_eq!(
                    value(Fp::<M>::from(word.as_limbs()[0])),
                    big(reduced.as_limbs()),
                    "From<u64> of {k}"
                );
            }
            if k.bits() <= 128 {
                rows.push(integer_results(x, uint::<biguint::typenum::U128>(k)));
            }
            if k.bits() <= width as u64 {
                rows.push(integer_results(x, uint::<M::Width>(k)));
            }
            for row in &rows {
                for ((name, results), expected) in
                    ["mul", "add", "sub"].iter().zip(row).zip(&expected)
                {
                    agree::<M>(name, a, k, expected, results);
                }
            }
        }
    }
}

/// `x / k` through `Fp`, the assigning form, and `ConstantTime` with `k` wrapped and its
/// assigning form.
fn division_results<M: Modulus, K>(x: Fp<M>, k: NonZeroUint<K>) -> [Fp<M>; 4]
where
    K: Bits + IsLess<M::Width, Output = True>,
{
    let (s, t) = (ConstantTime::new(x), ConstantTime::new(k));
    let (mut quotient, mut wrapped) = (x, s);
    quotient /= k;
    wrapped /= t;
    [x / k, quotient, (s / t).reveal(), wrapped.reveal()]
}

/// The division of the field of `M` by `k` of `K` bits against `biguint`'s `mul_mod` by the
/// inverse `k^(p - 2) mod p`, which `biguint`'s `pow` computes: `k` at the edges of 32, 64 and
/// `K` bits, and random of 32, 64 and `K` bits.
fn check_division<M: Modulus, K>(rng: &mut Rng)
where
    K: Bits + IsLess<M::Width, Output = True>,
{
    let p = big(M::MODULUS.as_limbs());
    let bits = K::BITS;
    let oracle = NonZero::new(uint::<Oracle>(&p)).expect("a nonzero modulus");
    let field = biguint::Modulus::new(M::MODULUS).expect("a nonzero modulus");
    let exponent = uint::<M::Width>(&(&p - 2u8));
    let top = (BigUint::from(1u8) << bits) - 1u8;
    let mut ks: Vec<BigUint> = [1u64, 2, 3, u32::MAX.into(), 1 << 32, u64::MAX]
        .map(BigUint::from)
        .into_iter()
        .chain([BigUint::from(1u8) << 64, (BigUint::from(1u8) << 64) + 1u8])
        .chain([top.clone(), &top - 1u8, (&top >> 1) + 1u8])
        .collect();
    for width in [32, 64, bits] {
        ks.extend((0..4).map(|_| random_bits(rng, width.min(bits))));
    }
    ks.retain(|k| k.bits() <= bits as u64 && *k != BigUint::ZERO);
    let xs = [BigUint::ZERO, BigUint::from(1u8), &p - 1u8, rng.below(&p)];
    for k in &ks {
        let inverse = field.scope(|ring| ring.reduce(uint::<K>(k)).pow(exponent).get());
        let divisor = NonZero::new(uint::<K>(k)).expect("a nonzero divisor");
        for a in &xs {
            let expected = big(uint::<Oracle>(a)
                .mul_mod(inverse.resize::<Oracle>(), oracle)
                .as_limbs());
            agree::<M>(
                "div",
                a,
                k,
                &expected,
                &division_results(element::<M>(a), divisor),
            );
        }
    }
}

/// A sweep over one field.
type Check = fn(&mut Rng);

/// Each modulus that fills its width, with the widths of its divisors: 64 and 128 bits where
/// they fit, and one bit short of the width.
const DIVISIONS: [(&str, Check); 21] = [
    ("Mersenne31 / U30", check_division::<Mersenne31, U30>),
    ("BabyBear / U30", check_division::<BabyBear, U30>),
    ("Goldilocks / U63", check_division::<Goldilocks, U63>),
    ("Mersenne127 / U64", check_division::<Mersenne127, U64>),
    ("Mersenne127 / U126", check_division::<Mersenne127, U126>),
    ("Bn254Scalar / U64", check_division::<Bn254Scalar, U64>),
    ("Bn254Scalar / U128", check_division::<Bn254Scalar, U128>),
    ("Bn254Scalar / U253", check_division::<Bn254Scalar, U253>),
    ("Secp256k1Base / U64", check_division::<Secp256k1Base, U64>),
    (
        "Secp256k1Base / U255",
        check_division::<Secp256k1Base, U255>,
    ),
    (
        "Secp256k1Scalar / U64",
        check_division::<Secp256k1Scalar, U64>,
    ),
    (
        "Secp256k1Scalar / U255",
        check_division::<Secp256k1Scalar, U255>,
    ),
    ("P256Base / U64", check_division::<P256Base, U64>),
    ("P256Base / U128", check_division::<P256Base, U128>),
    ("P256Base / U255", check_division::<P256Base, U255>),
    ("P256Scalar / U255", check_division::<P256Scalar, U255>),
    ("Tom256Base / U64", check_division::<Tom256Base, U64>),
    ("Tom256Base / U255", check_division::<Tom256Base, U255>),
    ("Bls12381Base / U64", check_division::<Bls12381Base, U64>),
    ("Bls12381Base / U380", check_division::<Bls12381Base, U380>),
    ("P521Base / U520", check_division::<P521Base, U520>),
];

#[test]
fn integer_operands_match_biguint() {
    for (name, check) in DIVISIONS {
        eprintln!("{name}");
        check(&mut Rng(name.len() as u64));
    }
    let sweep = |name: &str, p: &BigUint, sweep: &dyn Fn(&mut Rng)| {
        let mut rng = Rng(p.bits() + 1);
        eprintln!("{name}");
        sweep(&mut rng);
    };
    each_modulus!(
        sweep,
        check_integers,
        Seven,
        Mersenne31,
        BabyBear,
        Goldilocks,
        Mersenne127,
        Bn254Scalar,
        Secp256k1Base,
        Secp256k1Scalar,
        P256Base,
        P256Scalar,
        Tom256Base,
        Bls12381Base,
        P521Base,
    );
}
