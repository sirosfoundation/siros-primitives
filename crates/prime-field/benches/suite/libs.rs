//! One adapter per crate: the function that the crate runs for each operation over each field.
//!
//! Each crate uses its own public function for an operation. A crate without one, or without the
//! field, gets an absent cell with the reason. Elements enter and leave each crate in its own
//! representation; the conversions run before and after the counted batch.

use bench_compare::{Barrier, Cells, Chain, Gpr, Limbs, Row, Visitor};
use biguint::generic_array::GenericArray;
use biguint::typenum::{U31, U64, U254, U256, U381, U521};
use biguint::{Bits, Uint};
use core_algebra::{AlgebraicField, SerializableField};
use prime_field::{Fp, Modulus};
use runtime_algebra::field::RuntimeSerializableField;

use super::{
    BABY_BEAR, BLS12_381_BASE, BN254_SCALAR, Field, GOLDILOCKS, MERSENNE31, Op, P256_BASE,
    P256_SCALAR, P521_BASE, SECP256K1_BASE,
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

    /// Offers the case with its chain form, `x = op(x, y)`, whose results pass through `barrier`.
    fn chained<I: Chain<R>, R: ToLimbs, B: Barrier>(
        self,
        op: Op,
        function: &'static str,
        prepare: impl Fn(&[Limbs]) -> I,
        run: impl Fn(&I) -> R + Copy,
        barrier: B,
    ) -> Self;

    /// Records the same reason for every operation.
    fn none(self, reason: &'static str) -> Self;
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

    fn chained<I: Chain<R>, R: ToLimbs, B: Barrier>(
        self,
        op: Op,
        function: &'static str,
        prepare: impl Fn(&[Limbs]) -> I,
        run: impl Fn(&I) -> R + Copy,
        barrier: B,
    ) -> Self {
        self.chain(op, function, prepare, run, ToLimbs::to_limbs, barrier)
    }

    fn none(self, reason: &'static str) -> Self {
        Op::ALL
            .into_iter()
            .fold(self, |cells, op| cells.absent(op, reason))
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

/// Offers add, sub, mul and square of `$t` through the given closures, with their chain forms
/// through `$barrier`.
macro_rules! ops {
    (
        $cells:expr, $t:ty, $barrier:expr,
        add: $add:expr, sub: $sub:expr, mul: $mul:expr, square: $square:expr $(,)?
    ) => {
        $cells
            .chained(Op::Add, "+", pair::<$t>, $add, $barrier)
            .chained(Op::Sub, "-", pair::<$t>, $sub, $barrier)
            .chained(Op::Mul, "*", pair::<$t>, $mul, $barrier)
            .chained(Op::Square, "square", one::<$t>, $square, $barrier)
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

fn ours<M: Modulus, V: Visitor<Op, Field>>(cells: Crate<V>) -> Crate<V> {
    ops!(cells, Fp<M>, Gpr,
        add: |&(a, b)| a + b,
        sub: |&(a, b)| a - b,
        mul: |&(a, b)| a * b,
        square: |&a| a.square(),
    )
    .case(Op::Invert, "invert", one::<Fp<M>>, |&a| a.invert())
    .case(Op::InvertVartime, "invert_vartime", one::<Fp<M>>, |&a| {
        a.invert_vartime()
    })
    .case(Op::Sqrt, "sqrt", one::<Fp<M>>, |&a| a.sqrt())
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
    cells
        .chain(Op::Add, "+", two, |&(a, b)| a + b, read, Gpr)
        .chain(Op::Sub, "-", two, |&(a, b)| a - b, read, Gpr)
        .chain(Op::Mul, "*", two, |&(a, b)| a * b, read, Gpr)
        .chain(Op::Square, "square", single, |a| a.square(), read, Gpr)
        .absent(Op::Invert, VARTIME_INVERT)
        .api(
            Op::InvertVartime,
            "inverse",
            single,
            |a| a.inverse(),
            read_option,
        )
        .api(Op::Sqrt, "sqrt", single, |a| a.sqrt(), read_option)
}

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

fn monty<
    MOD: crypto_bigint::modular::ConstPrimeMontyParams<L>,
    const L: usize,
    V: Visitor<Op, Field>,
>(
    cells: Crate<V>,
) -> Crate<V> {
    type Form<MOD, const L: usize> = crypto_bigint::modular::ConstMontyForm<MOD, L>;
    ops!(cells, Form<MOD, L>, Gpr,
        add: |(a, b)| a + b,
        sub: |(a, b)| a - b,
        mul: |(a, b)| a * b,
        square: |a| a.square(),
    )
    .case(Op::Invert, "invert", one::<Form<MOD, L>>, |a| a.invert())
    .case(
        Op::InvertVartime,
        "invert_vartime",
        one::<Form<MOD, L>>,
        |a| a.invert_vartime(),
    )
    .case(Op::Sqrt, "sqrt", one::<Form<MOD, L>>, |a| a.sqrt())
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

/// The reason for an absent variable-time inverse.
const NO_VARTIME_INVERSE: &str = "no variable-time inverse";

/// The RustCrypto curve crates: `k256` for secp256k1, `p256` for P-256.
fn rust_crypto<V: Visitor<Op, Field>>(cells: Crate<V>) -> Crate<V> {
    match cells.key().field.as_str() {
        "secp256k1 base" => ops!(cells, k256::FieldElement, Gpr,
            add: |&(a, b)| a + b,
            sub: |&(a, b)| a - b,
            mul: |&(a, b)| a * b,
            square: |a| a.square(),
        )
        .case(Op::Invert, "invert", one::<k256::FieldElement>, |a| {
            a.invert()
        })
        .absent(Op::InvertVartime, NO_VARTIME_INVERSE)
        .case(Op::Sqrt, "sqrt", one::<k256::FieldElement>, |a| a.sqrt()),
        "P-256 base" => ops!(cells, p256::FieldElement, Gpr,
            add: |&(a, b)| a + b,
            sub: |&(a, b)| a - b,
            mul: |&(a, b)| a * b,
            square: |a| a.square(),
        )
        .case(Op::Invert, "invert", one::<p256::FieldElement>, |a| {
            a.invert()
        })
        .absent(Op::InvertVartime, NO_VARTIME_INVERSE)
        .case(Op::Sqrt, "sqrt", one::<p256::FieldElement>, |a| a.sqrt()),
        "P-256 scalar" => {
            use p256::elliptic_curve::ff::Field;
            ops!(cells, p256::Scalar, Gpr,
                add: |&(a, b)| a + b,
                sub: |&(a, b)| a - b,
                mul: |&(a, b)| a * b,
                square: |a| a.square(),
            )
            .case(Op::Invert, "invert", one::<p256::Scalar>, |a| a.invert())
            .absent(Op::InvertVartime, NO_VARTIME_INVERSE)
            .case(Op::Sqrt, "sqrt", one::<p256::Scalar>, Field::sqrt)
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

/// A field of halo2curves through the `ff::Field` trait. Its `invert` runs the variable-time
/// division steps of halo2curves' `BYInverter`, which branch on the trailing zeros of `g`.
fn ff_field<F: halo2curves::ff::Field + FromLimbs + ToLimbs, V: Visitor<Op, Field>>(
    cells: Crate<V>,
) -> Crate<V> {
    ops!(cells, F, Gpr,
        add: |&(a, b)| a + b,
        sub: |&(a, b)| a - b,
        mul: |&(a, b)| a * b,
        square: |a| a.square(),
    )
    .absent(Op::Invert, VARTIME_INVERT)
    .case(Op::InvertVartime, "invert", one::<F>, |a| a.invert())
    .case(Op::Sqrt, "sqrt", one::<F>, |a| a.sqrt())
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
    match cells.key().field.as_str() {
        "Mersenne-31" => p3::<p3_mersenne_31::Mersenne31, V>(cells),
        "BabyBear" => p3::<p3_baby_bear::BabyBear, V>(cells),
        "Goldilocks" => p3::<p3_goldilocks::Goldilocks, V>(cells),
        _ => cells.none("Plonky3 declares no such field"),
    }
}

fn p3<F: p3_field::Field + FromLimbs + ToLimbs, V: Visitor<Op, Field>>(
    cells: Crate<V>,
) -> Crate<V> {
    ops!(cells, F, Gpr,
        add: |&(a, b)| a + b,
        sub: |&(a, b)| a - b,
        mul: |&(a, b)| a * b,
        square: |a| a.square(),
    )
    .case(Op::Invert, "try_inverse", one::<F>, |a| a.try_inverse())
    .absent(Op::InvertVartime, NO_VARTIME_INVERSE)
    .case(Op::Sqrt, "tonelli_shanks", one::<F>, |&a| {
        p3_field::tonelli_shanks(a)
    })
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

/// `winter-math`'s `f64` field.
fn winter_math<V: Visitor<Op, Field>>(cells: Crate<V>) -> Crate<V> {
    use winter_math::FieldElement;
    use winter_math::fields::f64::BaseElement;
    match cells.key().field.as_str() {
        "Goldilocks" => ops!(cells, BaseElement, Gpr,
            add: |&(a, b)| a + b,
            sub: |&(a, b)| a - b,
            mul: |&(a, b)| a * b,
            square: |a| a.square(),
        )
        .case(Op::Invert, "inv", one::<BaseElement>, |a| a.inv())
        .absent(Op::InvertVartime, NO_VARTIME_INVERSE)
        .absent(Op::Sqrt, "no square root"),
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

/// The reasons for the absent cells of the longfellow-zk fields.
const NO_SQUARE: &str = "no squaring function";
const NO_SQRT: &str = "no square root";
const VARTIME_INVERT: &str = "the inverse runs in variable time";

/// The operations of a longfellow-zk runtime field `F` over `W` words.
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
        .chain(Op::Add, "addf", two, |(a, b)| field.addf(a, b), read, Gpr)
        .chain(Op::Sub, "subf", two, |(a, b)| field.subf(a, b), read, Gpr)
        .chain(Op::Mul, "mulf", two, |(a, b)| field.mulf(a, b), read, Gpr)
        .absent(Op::Square, NO_SQUARE)
        .absent(Op::Invert, VARTIME_INVERT)
        .api(
            Op::InvertVartime,
            "invert",
            single,
            |a| field.invert(a),
            read,
        )
        .absent(Op::Sqrt, NO_SQRT)
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
        .chain(Op::Add, "addf", two, |(a, b)| field.addf(a, b), read, Gpr)
        .chain(Op::Sub, "subf", two, |(a, b)| field.subf(a, b), read, Gpr)
        .chain(Op::Mul, "mulf", two, |(a, b)| field.mulf(a, b), read, Gpr)
        .absent(Op::Square, NO_SQUARE)
        .absent(Op::Invert, "the inverse is a variable-time `modpow`")
        .api(
            Op::InvertVartime,
            "invert",
            single,
            |a| field.invert(a),
            read,
        )
        .absent(Op::Sqrt, NO_SQRT)
}
