//! One adapter per crate: the function that the crate runs for each operation over each field.
//!
//! Each crate uses its own public function for an operation, in the row of its timing class as
//! its source shows it: variable time when a branch, a loop bound or a memory address depends on
//! the values; constant time otherwise. A crate without a function of a class, or without the
//! field, gets an absent cell with the reason. Elements enter and leave each crate in its own
//! representation; the conversions run before and after the counted batch.

use bench_compare::{Cells, Limbs, Row, Visitor};
use biguint::generic_array::GenericArray;
use biguint::typenum::{U31, U64, U254, U256, U381, U521};
use biguint::{Bits, Uint};
use core_algebra::{AlgebraicField, SerializableField};
use prime_field::{Fp, Modulus, Secret};
use runtime_algebra::field::RuntimeSerializableField;

use super::{
    BABY_BEAR, BLS12_381_BASE, BN254_SCALAR, Field, GOLDILOCKS, Kind, MERSENNE31, Op, P256_BASE,
    P256_SCALAR, P521_BASE, SECP256K1_BASE, Timing,
};

/// The cells of one crate in one row.
type Crate<V> = Cells<Op, Field, V>;

/// Offers the cells of every crate in `row`.
pub fn offer<V: Visitor<Op, Field>>(row: Row<Op, Field, V>) -> Row<Op, Field, V> {
    row.lib("prime-field", this_crate)
        .lib("ark-ff", ark)
        .lib("crypto-bigint", crypto_bigint)
        .lib("k256/p256", rust_crypto)
        .lib("halo2curves", halo2curves)
        .lib("plonky3", plonky3)
        .lib("winter-math", winter_math)
        .lib("runtime-algebra", runtime_algebra)
        .lib("compile-algebra", compile_algebra)
}

/// The variable-time row of `kind`.
const fn vartime(kind: Kind) -> Op {
    Op {
        kind,
        timing: Timing::Variable,
    }
}

/// The constant-time row of `kind`.
const fn constant(kind: Kind) -> Op {
    Op {
        kind,
        timing: Timing::Constant,
    }
}

/// The reason for an absent variable-time cell of a crate whose function runs in constant time.
const NO_VARIABLE_TIME: &str = "no variable-time function";
/// The reason for an absent constant-time cell of a crate whose function runs in variable time.
const NO_CONSTANT_TIME: &str = "no constant-time function";
/// The reasons for operations that a crate lacks in both classes.
const NO_SQUARE: &str = "no squaring function";
const NO_SQRT: &str = "no square root";

/// An operand type built from canonical limbs.
trait FromLimbs {
    fn from_limbs(limbs: &[u64]) -> Self;
}

/// A result type read back as canonical limbs.
trait ToLimbs {
    /// The limbs of the result; `None` when the result holds no value.
    fn to_limbs(&self) -> Option<Limbs>;
}

impl<T: ToLimbs> ToLimbs for Option<T> {
    fn to_limbs(&self) -> Option<Limbs> {
        self.as_ref().and_then(ToLimbs::to_limbs)
    }
}

impl<T: ToLimbs + Clone> ToLimbs for subtle::CtOption<T> {
    fn to_limbs(&self) -> Option<Limbs> {
        self.clone()
            .into_option()
            .and_then(|value| value.to_limbs())
    }
}

/// Offers cases whose results read back through [`ToLimbs`].
trait Offer<V> {
    fn case<I, R: ToLimbs>(
        self,
        op: Op,
        function: &'static str,
        prepare: impl Fn(&[Limbs]) -> I,
        run: impl Fn(&I) -> R + Copy,
    ) -> Self;

    /// Records the same reason for every operation.
    fn none(self, reason: &'static str) -> Self;

    /// Records `reason` for both timings of `kind`.
    fn neither(self, kind: Kind, reason: &'static str) -> Self;
}

impl<V: Visitor<Op, Field>> Offer<V> for Crate<V> {
    fn case<I, R: ToLimbs>(
        self,
        op: Op,
        function: &'static str,
        prepare: impl Fn(&[Limbs]) -> I,
        run: impl Fn(&I) -> R + Copy,
    ) -> Self {
        self.api(op, function, prepare, run, ToLimbs::to_limbs)
    }

    fn none(self, reason: &'static str) -> Self {
        Op::ALL
            .into_iter()
            .fold(self, |cells, op| cells.absent(op, reason))
    }

    fn neither(self, kind: Kind, reason: &'static str) -> Self {
        self.absent(vartime(kind), reason)
            .absent(constant(kind), reason)
    }
}

/// The two operands of a set.
fn pair<T: FromLimbs>(set: &[Limbs]) -> (T, T) {
    match set {
        [a, b] => (T::from_limbs(a), T::from_limbs(b)),
        _ => panic!("{} operands where two were drawn", set.len()),
    }
}

/// The operand of a set.
fn one<T: FromLimbs>(set: &[Limbs]) -> T {
    match set {
        [a] => T::from_limbs(a),
        _ => panic!("{} operands where one was drawn", set.len()),
    }
}

/// `limbs` as an array of exactly `L` limbs.
fn array<const L: usize>(limbs: &[u64]) -> [u64; L] {
    <[u64; L]>::try_from(limbs)
        .unwrap_or_else(|_| panic!("{} limbs for a {L}-limb element", limbs.len()))
}

/// The little-endian bytes of `limbs`.
fn le_bytes(limbs: &[u64]) -> Vec<u8> {
    limbs.iter().flat_map(|limb| limb.to_le_bytes()).collect()
}

/// The limbs of little-endian `bytes`.
fn from_le_bytes(bytes: &[u8]) -> Limbs {
    let mut padded = bytes.to_vec();
    padded.resize(bytes.len().div_ceil(8) * 8, 0);
    padded
        .chunks(8)
        .map(|chunk| u64::from_le_bytes(array8(chunk)))
        .collect()
}

fn array8(chunk: &[u8]) -> [u8; 8] {
    <[u8; 8]>::try_from(chunk).expect("eight bytes")
}

/// Offers add, sub, mul and square of `$t` in the row `$row` of their timing class, and records
/// `$reason` in the other row.
macro_rules! ops {
    ($cells:expr, $t:ty, $row:ident, $other:ident, $reason:expr,
     add: $add:expr, sub: $sub:expr, mul: $mul:expr, square: $square:expr $(,)?) => {
        $cells
            .case($row(Kind::Add), "+", pair::<$t>, $add)
            .case($row(Kind::Sub), "-", pair::<$t>, $sub)
            .case($row(Kind::Mul), "*", pair::<$t>, $mul)
            .case($row(Kind::Square), "square", one::<$t>, $square)
            .absent($other(Kind::Add), $reason)
            .absent($other(Kind::Sub), $reason)
            .absent($other(Kind::Mul), $reason)
            .absent($other(Kind::Square), $reason)
    };
}

// ---------------------------------------------------------------------------------------------
// prime-field

/// Declares one `Modulus` marker per field for `prime-field`.
macro_rules! moduli {
    ($($name:ident: $width:ty = $limbs:expr;)*) => {
        $(
            #[derive(Clone, Copy)]
            struct $name;

            impl Modulus for $name {
                type Width = $width;
                const MODULUS: Uint<$width> = Uint::from_limbs($limbs);
            }
        )*
    };
}

moduli! {
    Mersenne31: U31 = MERSENNE31;
    BabyBear: U31 = BABY_BEAR;
    Goldilocks: U64 = GOLDILOCKS;
    Bn254Scalar: U254 = BN254_SCALAR;
    Secp256k1Base: U256 = SECP256K1_BASE;
    P256Base: U256 = P256_BASE;
    P256Scalar: U256 = P256_SCALAR;
    Bls12381Base: U381 = BLS12_381_BASE;
    P521Base: U521 = P521_BASE;
}

impl<M: Modulus> FromLimbs for Fp<M> {
    fn from_limbs(limbs: &[u64]) -> Self {
        let array = GenericArray::<u64, <M::Width as Bits>::Limbs>::try_from_slice(limbs)
            .unwrap_or_else(|_| panic!("{} limbs for the field", limbs.len()));
        Self::from_uint(Uint::from_generic_limbs(*array)).expect("a canonical element")
    }
}

impl<M: Modulus> ToLimbs for Fp<M> {
    fn to_limbs(&self) -> Option<Limbs> {
        Some(self.to_uint().as_limbs().to_vec())
    }
}

impl<M: Modulus> FromLimbs for Secret<M> {
    fn from_limbs(limbs: &[u64]) -> Self {
        Self::new(Fp::from_limbs(limbs))
    }
}

impl<M: Modulus> ToLimbs for Secret<M> {
    fn to_limbs(&self) -> Option<Limbs> {
        self.reveal().to_limbs()
    }
}

/// `prime-field`, the crate under test.
fn this_crate<V: Visitor<Op, Field>>(cells: Crate<V>) -> Crate<V> {
    match cells.key().field.as_str() {
        "Mersenne-31" => ours::<Mersenne31, V>(cells),
        "BabyBear" => ours::<BabyBear, V>(cells),
        "Goldilocks" => ours::<Goldilocks, V>(cells),
        "BN254 scalar" => ours::<Bn254Scalar, V>(cells),
        "secp256k1 base" => ours::<Secp256k1Base, V>(cells),
        "P-256 base" => ours::<P256Base, V>(cells),
        "P-256 scalar" => ours::<P256Scalar, V>(cells),
        "BLS12-381 base" => ours::<Bls12381Base, V>(cells),
        "P-521 base" => ours::<P521Base, V>(cells),
        field => panic!("no field {field}"),
    }
}

/// `Fp` in the variable-time rows, `Secret` in the constant-time rows.
fn ours<M: Modulus, V: Visitor<Op, Field>>(cells: Crate<V>) -> Crate<V> {
    cells
        .case(vartime(Kind::Add), "+", pair::<Fp<M>>, |&(a, b)| a + b)
        .case(vartime(Kind::Sub), "-", pair::<Fp<M>>, |&(a, b)| a - b)
        .case(vartime(Kind::Mul), "*", pair::<Fp<M>>, |&(a, b)| a * b)
        .case(vartime(Kind::Square), "square", one::<Fp<M>>, |a| {
            a.square()
        })
        .case(vartime(Kind::Invert), "invert", one::<Fp<M>>, |a| {
            a.invert()
        })
        .case(vartime(Kind::Sqrt), "sqrt", one::<Fp<M>>, |a| a.sqrt())
        .case(
            constant(Kind::Add),
            "Secret +",
            pair::<Secret<M>>,
            |&(a, b)| a + b,
        )
        .case(
            constant(Kind::Sub),
            "Secret -",
            pair::<Secret<M>>,
            |&(a, b)| a - b,
        )
        .case(
            constant(Kind::Mul),
            "Secret *",
            pair::<Secret<M>>,
            |&(a, b)| a * b,
        )
        .case(
            constant(Kind::Square),
            "Secret::square",
            one::<Secret<M>>,
            |a| a.square(),
        )
        .case(
            constant(Kind::Invert),
            "Secret::invert",
            one::<Secret<M>>,
            |a| a.invert(),
        )
        .case(
            constant(Kind::Sqrt),
            "Secret::sqrt",
            one::<Secret<M>>,
            |a| a.sqrt(),
        )
}

// ---------------------------------------------------------------------------------------------
// ark-ff

/// `ark-ff` with the arkworks curve crates.
fn ark<V: Visitor<Op, Field>>(cells: Crate<V>) -> Crate<V> {
    match cells.key().field.as_str() {
        "BN254 scalar" => ark_field::<ark_bn254::Fr, V>(cells),
        "secp256k1 base" => ark_field::<ark_secp256k1::Fq, V>(cells),
        "P-256 base" => ark_field::<ark_secp256r1::Fq, V>(cells),
        "P-256 scalar" => ark_field::<ark_secp256r1::Fr, V>(cells),
        "BLS12-381 base" => ark_field::<ark_bls12_381::Fq, V>(cells),
        _ => cells.none("no arkworks crate declares the field"),
    }
}

/// An arkworks field: every operation branches on the value (`+` and `*` before their final
/// subtraction, `-` on its borrow, `inverse` and `sqrt` throughout).
fn ark_field<F: ark_ff::PrimeField, V: Visitor<Op, Field>>(cells: Crate<V>) -> Crate<V> {
    use ark_ff::BigInteger;
    let element = |limbs: &[u64]| F::from_le_bytes_mod_order(&le_bytes(limbs));
    let two = |set: &[Limbs]| match set {
        [a, b] => (element(a), element(b)),
        _ => panic!("{} operands where two were drawn", set.len()),
    };
    let single = |set: &[Limbs]| match set {
        [a] => element(a),
        _ => panic!("{} operands where one was drawn", set.len()),
    };
    let read = |e: &F| Some(from_le_bytes(&e.into_bigint().to_bytes_le()));
    let read_option = |e: &Option<F>| e.as_ref().and_then(read);
    let mut cells = cells
        .api(vartime(Kind::Add), "+", two, |&(a, b)| a + b, read)
        .api(vartime(Kind::Sub), "-", two, |&(a, b)| a - b, read)
        .api(vartime(Kind::Mul), "*", two, |&(a, b)| a * b, read)
        .api(
            vartime(Kind::Square),
            "square",
            single,
            |a| a.square(),
            read,
        )
        .api(
            vartime(Kind::Invert),
            "inverse",
            single,
            |a| a.inverse(),
            read_option,
        )
        .api(
            vartime(Kind::Sqrt),
            "sqrt",
            single,
            |a| a.sqrt(),
            read_option,
        );
    for kind in KINDS {
        cells = cells.absent(constant(kind), NO_CONSTANT_TIME);
    }
    cells
}

/// Every kind of operation.
const KINDS: [Kind; 6] = [
    Kind::Add,
    Kind::Sub,
    Kind::Mul,
    Kind::Square,
    Kind::Invert,
    Kind::Sqrt,
];

// ---------------------------------------------------------------------------------------------
// crypto-bigint

mod cb {
    use crypto_bigint::{U64, U256, U384, U576, const_prime_monty_params};

    const_prime_monty_params!(Mersenne31, U64, "000000007fffffff", 7);
    const_prime_monty_params!(BabyBear, U64, "0000000078000001", 31);
    const_prime_monty_params!(Goldilocks, U64, "ffffffff00000001", 7);
    const_prime_monty_params!(
        Bn254Scalar,
        U256,
        "30644e72e131a029b85045b68181585d2833e84879b9709143e1f593f0000001",
        5
    );
    const_prime_monty_params!(
        Secp256k1Base,
        U256,
        "fffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2f",
        3
    );
    const_prime_monty_params!(
        P256Base,
        U256,
        "ffffffff00000001000000000000000000000000ffffffffffffffffffffffff",
        2
    );
    const_prime_monty_params!(
        P256Scalar,
        U256,
        "ffffffff00000000ffffffffffffffffbce6faada7179e84f3b9cac2fc632551",
        11
    );
    const_prime_monty_params!(
        Bls12381Base,
        U384,
        "1a0111ea397fe69a4b1ba7b6434bacd764774b84f38512bf6730d2a0f6b0f6241eabfffeb153ffffb9feffffffffaaab",
        2
    );
    const_prime_monty_params!(
        P521Base,
        U576,
        "00000000000001ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
        3
    );
}

impl<MOD: crypto_bigint::modular::ConstMontyParams<L>, const L: usize> FromLimbs
    for crypto_bigint::modular::ConstMontyForm<MOD, L>
{
    fn from_limbs(limbs: &[u64]) -> Self {
        let mut words = [0u64; L];
        words[..limbs.len()].copy_from_slice(limbs);
        Self::new(&crypto_bigint::Uint::from_words(words))
    }
}

impl<MOD: crypto_bigint::modular::ConstMontyParams<L>, const L: usize> ToLimbs
    for crypto_bigint::modular::ConstMontyForm<MOD, L>
{
    fn to_limbs(&self) -> Option<Limbs> {
        Some(self.retrieve().to_words().to_vec())
    }
}

impl<T: ToLimbs + Clone> ToLimbs for crypto_bigint::CtOption<T> {
    fn to_limbs(&self) -> Option<Limbs> {
        self.clone()
            .into_option()
            .and_then(|value| value.to_limbs())
    }
}

/// `crypto-bigint`'s `ConstMontyForm`.
fn crypto_bigint<V: Visitor<Op, Field>>(cells: Crate<V>) -> Crate<V> {
    match cells.key().field.as_str() {
        "Mersenne-31" => monty::<cb::Mersenne31, 1, V>(cells),
        "BabyBear" => monty::<cb::BabyBear, 1, V>(cells),
        "Goldilocks" => monty::<cb::Goldilocks, 1, V>(cells),
        "BN254 scalar" => monty::<cb::Bn254Scalar, 4, V>(cells),
        "secp256k1 base" => monty::<cb::Secp256k1Base, 4, V>(cells),
        "P-256 base" => monty::<cb::P256Base, 4, V>(cells),
        "P-256 scalar" => monty::<cb::P256Scalar, 4, V>(cells),
        "BLS12-381 base" => monty::<cb::Bls12381Base, 6, V>(cells),
        "P-521 base" => monty::<cb::P521Base, 9, V>(cells),
        field => panic!("no field {field}"),
    }
}

/// `ConstMontyForm`: masks and constant-time selections throughout, except `invert_vartime`.
/// `sqrt` raises to a public exponent by `pow_vartime`, whose time depends on the exponent
/// alone, then selects in constant time.
fn monty<
    MOD: crypto_bigint::modular::ConstPrimeMontyParams<L>,
    const L: usize,
    V: Visitor<Op, Field>,
>(
    cells: Crate<V>,
) -> Crate<V> {
    type Form<MOD, const L: usize> = crypto_bigint::modular::ConstMontyForm<MOD, L>;
    ops!(cells, Form<MOD, L>, constant, vartime, NO_VARIABLE_TIME,
        add: |(a, b)| a + b,
        sub: |(a, b)| a - b,
        mul: |(a, b)| a * b,
        square: |a| a.square(),
    )
    .case(constant(Kind::Invert), "invert", one::<Form<MOD, L>>, |a| {
        a.invert()
    })
    .case(
        vartime(Kind::Invert),
        "invert_vartime",
        one::<Form<MOD, L>>,
        |a| a.invert_vartime(),
    )
    .case(constant(Kind::Sqrt), "sqrt", one::<Form<MOD, L>>, |a| {
        a.sqrt()
    })
    .absent(vartime(Kind::Sqrt), NO_VARIABLE_TIME)
}

// ---------------------------------------------------------------------------------------------
// k256 and p256

/// The big-endian 32-byte encoding of four limbs.
fn be_bytes_32(limbs: &[u64]) -> [u8; 32] {
    let mut bytes = le_bytes(limbs);
    bytes.reverse();
    <[u8; 32]>::try_from(bytes).expect("32 bytes")
}

/// The limbs of 32 big-endian bytes.
fn from_be_bytes(bytes: &[u8]) -> Limbs {
    let mut bytes = bytes.to_vec();
    bytes.reverse();
    from_le_bytes(&bytes)
}

impl FromLimbs for k256::FieldElement {
    fn from_limbs(limbs: &[u64]) -> Self {
        Self::from_bytes(&be_bytes_32(limbs).into())
            .into_option()
            .expect("a canonical element")
    }
}

impl ToLimbs for k256::FieldElement {
    fn to_limbs(&self) -> Option<Limbs> {
        Some(from_be_bytes(&self.to_bytes()))
    }
}

impl FromLimbs for p256::FieldElement {
    fn from_limbs(limbs: &[u64]) -> Self {
        Self::from_bytes(&be_bytes_32(limbs).into())
            .into_option()
            .expect("a canonical element")
    }
}

impl ToLimbs for p256::FieldElement {
    fn to_limbs(&self) -> Option<Limbs> {
        Some(from_be_bytes(&self.to_bytes()))
    }
}

impl FromLimbs for p256::Scalar {
    fn from_limbs(limbs: &[u64]) -> Self {
        <Self as p256::elliptic_curve::ff::PrimeField>::from_repr(be_bytes_32(limbs).into())
            .into_option()
            .expect("a canonical element")
    }
}

impl ToLimbs for p256::Scalar {
    fn to_limbs(&self) -> Option<Limbs> {
        Some(from_be_bytes(&self.to_bytes()))
    }
}

/// The RustCrypto curve crates: `k256` for secp256k1, `p256` for P-256. Their field arithmetic
/// selects by masks and `subtle`; `k256`'s `+` and `-` add limbs without a carry and leave the
/// reduction to a later `normalize`. `p256`'s scalar adds `invert_vartime`, Stein's algorithm.
fn rust_crypto<V: Visitor<Op, Field>>(cells: Crate<V>) -> Crate<V> {
    match cells.key().field.as_str() {
        "secp256k1 base" => ops!(cells, k256::FieldElement, constant, vartime, NO_VARIABLE_TIME,
            add: |&(a, b)| a + b,
            sub: |&(a, b)| a - b,
            mul: |&(a, b)| a * b,
            square: |a| a.square(),
        )
        .case(
            constant(Kind::Invert),
            "invert",
            one::<k256::FieldElement>,
            |a| a.invert(),
        )
        .absent(vartime(Kind::Invert), NO_VARIABLE_TIME)
        .case(
            constant(Kind::Sqrt),
            "sqrt",
            one::<k256::FieldElement>,
            |a| a.sqrt(),
        )
        .absent(vartime(Kind::Sqrt), NO_VARIABLE_TIME),
        "P-256 base" => ops!(cells, p256::FieldElement, constant, vartime, NO_VARIABLE_TIME,
            add: |&(a, b)| a + b,
            sub: |&(a, b)| a - b,
            mul: |&(a, b)| a * b,
            square: |a| a.square(),
        )
        .case(
            constant(Kind::Invert),
            "invert",
            one::<p256::FieldElement>,
            |a| a.invert(),
        )
        .absent(vartime(Kind::Invert), NO_VARIABLE_TIME)
        .case(
            constant(Kind::Sqrt),
            "sqrt",
            one::<p256::FieldElement>,
            |a| a.sqrt(),
        )
        .absent(vartime(Kind::Sqrt), NO_VARIABLE_TIME),
        "P-256 scalar" => {
            use p256::elliptic_curve::ff::Field;
            use p256::elliptic_curve::ops::Invert;
            ops!(cells, p256::Scalar, constant, vartime, NO_VARIABLE_TIME,
                add: |&(a, b)| a + b,
                sub: |&(a, b)| a - b,
                mul: |&(a, b)| a * b,
                square: |a| a.square(),
            )
            .case(constant(Kind::Invert), "invert", one::<p256::Scalar>, |a| {
                p256::Scalar::invert(a)
            })
            .case(
                vartime(Kind::Invert),
                "invert_vartime",
                one::<p256::Scalar>,
                Invert::invert_vartime,
            )
            .case(
                constant(Kind::Sqrt),
                "sqrt",
                one::<p256::Scalar>,
                Field::sqrt,
            )
            .absent(vartime(Kind::Sqrt), NO_VARIABLE_TIME)
        }
        "P-521 base" => cells.none("p521 keeps its field element private"),
        _ => cells.none("no RustCrypto crate declares the field"),
    }
}

// ---------------------------------------------------------------------------------------------
// halo2curves

/// The limb conversions of a `halo2curves` field with `$limbs` limbs and the byte order of its
/// `to_bytes`.
macro_rules! halo2_field {
    ($t:ty, $limbs:literal, $read:ident) => {
        impl FromLimbs for $t {
            fn from_limbs(limbs: &[u64]) -> Self {
                Self::from_raw(array::<$limbs>(limbs))
            }
        }

        impl ToLimbs for $t {
            fn to_limbs(&self) -> Option<Limbs> {
                Some($read(&self.to_bytes()))
            }
        }
    };
}

halo2_field!(halo2curves::bn256::Fr, 4, from_le_bytes);
halo2_field!(halo2curves::secp256k1::Fp, 4, from_le_bytes);
halo2_field!(halo2curves::secp256r1::Fp, 4, from_le_bytes);
halo2_field!(halo2curves::secp256r1::Fq, 4, from_le_bytes);
halo2_field!(halo2curves::bls12381::Fq, 6, from_be_bytes);

/// `halo2curves`.
fn halo2curves<V: Visitor<Op, Field>>(cells: Crate<V>) -> Crate<V> {
    match cells.key().field.as_str() {
        "BN254 scalar" => ff_field::<halo2curves::bn256::Fr, V>(cells),
        "secp256k1 base" => ff_field::<halo2curves::secp256k1::Fp, V>(cells),
        "P-256 base" => ff_field::<halo2curves::secp256r1::Fp, V>(cells),
        "P-256 scalar" => ff_field::<halo2curves::secp256r1::Fq, V>(cells),
        "BLS12-381 base" => ff_field::<halo2curves::bls12381::Fq, V>(cells),
        _ => cells.none("halo2curves declares no such field"),
    }
}

/// A field of halo2curves through the `ff::Field` trait: masks in `+`, `-` and `*`, and the
/// constant-time Tonelli–Shanks or exponentiation of `ff` in `sqrt`. Its `invert` runs the
/// variable-time division steps of halo2curves' `BYInverter`, which branch on the trailing
/// zeros of `g`.
fn ff_field<F: halo2curves::ff::Field + FromLimbs + ToLimbs, V: Visitor<Op, Field>>(
    cells: Crate<V>,
) -> Crate<V> {
    ops!(cells, F, constant, vartime, NO_VARIABLE_TIME,
        add: |&(a, b)| a + b,
        sub: |&(a, b)| a - b,
        mul: |&(a, b)| a * b,
        square: |a| a.square(),
    )
    .case(vartime(Kind::Invert), "invert", one::<F>, |a| a.invert())
    .absent(constant(Kind::Invert), NO_CONSTANT_TIME)
    .case(constant(Kind::Sqrt), "sqrt", one::<F>, |a| a.sqrt())
    .absent(vartime(Kind::Sqrt), NO_VARIABLE_TIME)
}

// ---------------------------------------------------------------------------------------------
// plonky3

/// The limb conversions of a Plonky3 field whose `new` takes `$int`.
macro_rules! p3_field {
    ($t:ty, $int:ty) => {
        impl FromLimbs for $t {
            fn from_limbs(limbs: &[u64]) -> Self {
                let [value] = array::<1>(limbs);
                Self::new(<$int>::try_from(value).expect("a canonical element"))
            }
        }

        impl ToLimbs for $t {
            fn to_limbs(&self) -> Option<Limbs> {
                use p3_field::PrimeField64;
                Some(vec![self.as_canonical_u64()])
            }
        }
    };
}

p3_field!(p3_mersenne_31::Mersenne31, u32);
p3_field!(p3_baby_bear::BabyBear, u32);
p3_field!(p3_goldilocks::Goldilocks, u64);

/// Plonky3: `p3-mersenne-31`, `p3-baby-bear` and `p3-goldilocks`.
fn plonky3<V: Visitor<Op, Field>>(cells: Crate<V>) -> Crate<V> {
    use p3_field::Field as _;
    match cells.key().field.as_str() {
        "Mersenne-31" => p3::<p3_mersenne_31::Mersenne31, V>(cells, true, |a| a.try_sqrt()),
        "BabyBear" => p3::<p3_baby_bear::BabyBear, V>(cells, false, |a| a.try_sqrt()),
        "Goldilocks" => p3::<p3_goldilocks::Goldilocks, V>(cells, false, |a| a.try_sqrt()),
        _ => cells.none("Plonky3 declares no such field"),
    }
}

/// A Plonky3 field: `+`, `*`, `square`, `try_inverse` and `try_sqrt` decide their corrections and
/// steps with `if` on the value; Mersenne-31's `-` is arithmetic alone, BabyBear's and
/// Goldilocks' `-` branch on the borrow. `subtraction_constant` tells the arithmetic `-`; `sqrt` is the
/// field's own `try_sqrt`, which BabyBear specializes in an inherent method.
fn p3<F: p3_field::Field + FromLimbs + ToLimbs, V: Visitor<Op, Field>>(
    cells: Crate<V>,
    subtraction_constant: bool,
    sqrt: impl Fn(&F) -> Option<F> + Copy,
) -> Crate<V> {
    let (sub_row, sub_other, sub_reason) = if subtraction_constant {
        (constant(Kind::Sub), vartime(Kind::Sub), NO_VARIABLE_TIME)
    } else {
        (vartime(Kind::Sub), constant(Kind::Sub), NO_CONSTANT_TIME)
    };
    let mut cells = cells
        .case(vartime(Kind::Add), "+", pair::<F>, |&(a, b)| a + b)
        .case(sub_row, "-", pair::<F>, |&(a, b)| a - b)
        .absent(sub_other, sub_reason)
        .case(vartime(Kind::Mul), "*", pair::<F>, |&(a, b)| a * b)
        .case(vartime(Kind::Square), "square", one::<F>, |a| a.square())
        .case(vartime(Kind::Invert), "try_inverse", one::<F>, |a| {
            a.try_inverse()
        })
        .case(vartime(Kind::Sqrt), "try_sqrt", one::<F>, move |a| sqrt(a));
    for kind in [Kind::Add, Kind::Mul, Kind::Square, Kind::Invert, Kind::Sqrt] {
        cells = cells.absent(constant(kind), NO_CONSTANT_TIME);
    }
    cells
}

// ---------------------------------------------------------------------------------------------
// winter-math

impl FromLimbs for winter_math::fields::f64::BaseElement {
    fn from_limbs(limbs: &[u64]) -> Self {
        let [value] = array::<1>(limbs);
        Self::new(value)
    }
}

impl ToLimbs for winter_math::fields::f64::BaseElement {
    fn to_limbs(&self) -> Option<Limbs> {
        Some(vec![self.as_int()])
    }
}

/// `winter-math`'s `f64` field: masks in `+`, `-` and `*`, a fixed addition chain in `inv`.
fn winter_math<V: Visitor<Op, Field>>(cells: Crate<V>) -> Crate<V> {
    use winter_math::FieldElement;
    use winter_math::fields::f64::BaseElement;
    match cells.key().field.as_str() {
        "Goldilocks" => ops!(cells, BaseElement, constant, vartime, NO_VARIABLE_TIME,
            add: |&(a, b)| a + b,
            sub: |&(a, b)| a - b,
            mul: |&(a, b)| a * b,
            square: |a| a.square(),
        )
        .case(constant(Kind::Invert), "inv", one::<BaseElement>, |a| {
            a.inv()
        })
        .absent(vartime(Kind::Invert), NO_VARIABLE_TIME)
        .neither(Kind::Sqrt, NO_SQRT),
        _ => cells.none("winter-math declares no such field"),
    }
}

// ---------------------------------------------------------------------------------------------
// longfellow-zk runtime-algebra

/// `runtime-algebra`: `P256Field` and `Q256Field` for P-256, `FpGenericField` elsewhere.
fn runtime_algebra<V: Visitor<Op, Field>>(cells: Crate<V>) -> Crate<V> {
    use runtime_algebra::fp_generic::FpGenericField;
    match cells.key().field.as_str() {
        "Mersenne-31" => {
            runtime::<1, _, V>(FpGenericField::<1, 1, 3>::new_generic(MERSENNE31), cells)
        }
        "BabyBear" => runtime::<1, _, V>(FpGenericField::<1, 1, 3>::new_generic(BABY_BEAR), cells),
        "Goldilocks" => {
            runtime::<1, _, V>(FpGenericField::<1, 1, 3>::new_generic(GOLDILOCKS), cells)
        }
        "BN254 scalar" => {
            runtime::<4, _, V>(FpGenericField::<4, 4, 9>::new_generic(BN254_SCALAR), cells)
        }
        "secp256k1 base" => runtime::<4, _, V>(
            FpGenericField::<4, 4, 9>::new_generic(SECP256K1_BASE),
            cells,
        ),
        "P-256 base" => runtime::<4, _, V>(runtime_algebra::p256::P256Field::new(), cells),
        "P-256 scalar" => runtime::<4, _, V>(runtime_algebra::Q256Field::new(), cells),
        "BLS12-381 base" => runtime::<6, _, V>(
            FpGenericField::<6, 6, 13>::new_generic(BLS12_381_BASE),
            cells,
        ),
        "P-521 base" => {
            runtime::<9, _, V>(FpGenericField::<9, 9, 19>::new_generic(P521_BASE), cells)
        }
        field => panic!("no field {field}"),
    }
}

/// The operations of a longfellow-zk runtime field `F` over `W` words: `addf`, `subf` and `mulf`
/// select by `cmov` in inline assembly; `invert` runs the binary extended Euclidean algorithm,
/// which branches on the parity of the value.
fn runtime<const W: usize, F: RuntimeSerializableField<W>, V: Visitor<Op, Field>>(
    field: F,
    cells: Crate<V>,
) -> Crate<V> {
    let element = |limbs: &[u64]| {
        field
            .words64_to_element(&array::<W>(limbs))
            .expect("a canonical element")
    };
    let two = |set: &[Limbs]| match set {
        [a, b] => (element(a), element(b)),
        _ => panic!("{} operands where two were drawn", set.len()),
    };
    let single = |set: &[Limbs]| match set {
        [a] => element(a),
        _ => panic!("{} operands where one was drawn", set.len()),
    };
    let read = |e: &F::E| Some(field.to_words64(e).to_vec());
    cells
        .api(
            constant(Kind::Add),
            "addf",
            two,
            |(a, b)| field.addf(a, b),
            read,
        )
        .api(
            constant(Kind::Sub),
            "subf",
            two,
            |(a, b)| field.subf(a, b),
            read,
        )
        .api(
            constant(Kind::Mul),
            "mulf",
            two,
            |(a, b)| field.mulf(a, b),
            read,
        )
        .absent(vartime(Kind::Add), NO_VARIABLE_TIME)
        .absent(vartime(Kind::Sub), NO_VARIABLE_TIME)
        .absent(vartime(Kind::Mul), NO_VARIABLE_TIME)
        .neither(Kind::Square, NO_SQUARE)
        .api(
            vartime(Kind::Invert),
            "invert",
            single,
            |a| field.invert(a),
            read,
        )
        .absent(constant(Kind::Invert), NO_CONSTANT_TIME)
        .neither(Kind::Sqrt, NO_SQRT)
}

// ---------------------------------------------------------------------------------------------
// longfellow-zk compile-algebra

/// `compile-algebra`: `FpField` on `num-bigint`, for P-256 base and scalar.
fn compile_algebra<V: Visitor<Op, Field>>(cells: Crate<V>) -> Crate<V> {
    match cells.key().field.as_str() {
        "P-256 base" => compile(compile_algebra::p256::P256Field::new(), cells),
        "P-256 scalar" => compile(compile_algebra::q256::Q256Field::new(), cells),
        _ => cells.none("compile-algebra declares P-256 base and scalar only"),
    }
}

/// `FpField` on `num-bigint`, whose arithmetic takes time that depends on the values.
fn compile<T: 'static, V: Visitor<Op, Field>>(
    field: compile_algebra::fp::FpField<T>,
    cells: Crate<V>,
) -> Crate<V> {
    let element = |limbs: &[u64]| {
        field
            .bytes_to_element(&le_bytes(limbs))
            .expect("a canonical element")
    };
    let two = |set: &[Limbs]| match set {
        [a, b] => (element(a), element(b)),
        _ => panic!("{} operands where two were drawn", set.len()),
    };
    let single = |set: &[Limbs]| match set {
        [a] => element(a),
        _ => panic!("{} operands where one was drawn", set.len()),
    };
    let read = |e: &compile_algebra::fp::Elt<T>| {
        let mut value = from_le_bytes(&field.value(e).to_bytes_le());
        value.resize(4, 0);
        Some(value)
    };
    cells
        .api(
            vartime(Kind::Add),
            "addf",
            two,
            |(a, b)| field.addf(a, b),
            read,
        )
        .api(
            vartime(Kind::Sub),
            "subf",
            two,
            |(a, b)| field.subf(a, b),
            read,
        )
        .api(
            vartime(Kind::Mul),
            "mulf",
            two,
            |(a, b)| field.mulf(a, b),
            read,
        )
        .api(
            vartime(Kind::Invert),
            "invert",
            single,
            |a| field.invert(a),
            read,
        )
        .absent(constant(Kind::Add), NO_CONSTANT_TIME)
        .absent(constant(Kind::Sub), NO_CONSTANT_TIME)
        .absent(constant(Kind::Mul), NO_CONSTANT_TIME)
        .absent(constant(Kind::Invert), NO_CONSTANT_TIME)
        .neither(Kind::Square, NO_SQUARE)
        .neither(Kind::Sqrt, NO_SQRT)
}
