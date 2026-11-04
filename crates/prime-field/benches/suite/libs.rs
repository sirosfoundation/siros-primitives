//! One adapter per crate: the function that the crate runs for each operation over each field.
//!
//! Each crate uses its own public function for an operation, in the row of its timing class as
//! its source shows it: variable time when a branch, a loop bound or a memory address depends on
//! the values; constant time otherwise. A crate without a function of a class, or without the
//! field, gets an absent cell with the reason. Elements enter and leave each crate in its own
//! representation; the conversions run before and after the counted batch.

use bench_compare::{Cells, Chain, Limbs, Row, Selected, Visitor};
use biguint::generic_array::GenericArray;
use biguint::typenum::{IsLess, True, U30, U63, U64, U253, U255, U380, U520};
use biguint::{Bits, NonZero, Uint};
use core_algebra::{AlgebraicField, SerializableField};
use prime_field::standard::{
    BabyBear, Bls12381Base, Bn254Scalar, Goldilocks, Mersenne31, P256Base, P256Scalar, P521Base,
    Secp256k1Base, Tom256Base,
};
use prime_field::{ConstantTime, Fp, Modulus};
use runtime_algebra::field::RuntimeSerializableField;

use num_bigint::BigUint;

use super::{Field, Kind, KindOf, Op, Timing, big, ct, in_row, kind, limbs, spec, var};

/// The limbs of the modulus of `M`, zero-extended to `L` limbs.
const fn words<M: Modulus, const L: usize>() -> [u64; L] {
    let modulus = M::MODULUS;
    let limbs = modulus.as_limbs();
    let mut words = [0; L];
    let mut index = 0;
    while index < L && index < limbs.len() {
        words[index] = limbs[index];
        index += 1;
    }
    words
}

/// The cells of one crate in one row.
type Crate<V> = Cells<Op, Field, V>;

/// Offers the cells of every crate in `row`.
pub fn offer<V: Visitor<Op, Field>>(row: Row<Op, Field, V>) -> Row<Op, Field, V> {
    row.lib("prime-field", this_crate)
        .lib("prime-field × invert", this_crate_inverse)
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
        op: impl Selected<Op>,
        function: &'static str,
        prepare: impl Fn(&[Limbs]) -> I,
        run: impl Fn(&I) -> R + Copy,
    ) -> Self;

    /// Offers the case with its chain form, `x = op(x, y)`.
    fn chained<I: Chain<R>, R: ToLimbs>(
        self,
        op: impl Selected<Op>,
        function: &'static str,
        prepare: impl Fn(&[Limbs]) -> I,
        run: impl Fn(&I) -> R + Copy,
    ) -> Self;

    /// Records the same reason for every operation.
    fn none(self, reason: &'static str) -> Self;

    /// Records `reason` for both timings of `kind`.
    fn neither(self, kind: Kind, reason: &'static str) -> Self;

    /// Declares the deviation of the next case from its row's definition, when it has one.
    fn declare(self, deviation: Option<&'static str>) -> Self;
}

impl<V: Visitor<Op, Field>> Offer<V> for Crate<V> {
    fn case<I, R: ToLimbs>(
        self,
        op: impl Selected<Op>,
        function: &'static str,
        prepare: impl Fn(&[Limbs]) -> I,
        run: impl Fn(&I) -> R + Copy,
    ) -> Self {
        self.api(op, function, prepare, run, ToLimbs::to_limbs)
    }

    fn chained<I: Chain<R>, R: ToLimbs>(
        self,
        op: impl Selected<Op>,
        function: &'static str,
        prepare: impl Fn(&[Limbs]) -> I,
        run: impl Fn(&I) -> R + Copy,
    ) -> Self {
        self.chain(op, function, prepare, run, ToLimbs::to_limbs)
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

    fn declare(self, deviation: Option<&'static str>) -> Self {
        match deviation {
            Some(deviation) => self.deviating(deviation),
            None => self,
        }
    }
}

/// The deviation of a sum or a difference that leaves the reduction for later.
const SKIPPED_REDUCTION: &str = "skips the final reduction; the result is not canonical";

/// The deviation of a result that leaves the normalization for later.
const SKIPPED_NORMALIZATION: &str = "skips the final normalization; the result is not canonical";

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

/// The element and the integer of a set.
fn mixed<A: FromLimbs, B: FromLimbs>(set: &[Limbs]) -> (A, B) {
    match set {
        [a, b] => (A::from_limbs(a), B::from_limbs(b)),
        _ => panic!("{} operands where two were drawn", set.len()),
    }
}

impl FromLimbs for u64 {
    fn from_limbs(limbs: &[u64]) -> Self {
        let [value] = array::<1>(limbs);
        value
    }
}

impl<K: Bits> FromLimbs for Uint<K> {
    fn from_limbs(limbs: &[u64]) -> Self {
        let mut array = GenericArray::<u64, K::Limbs>::default();
        array[..limbs.len()].copy_from_slice(limbs);
        Self::from_generic_limbs(array)
    }
}

impl<K: Bits> FromLimbs for ConstantTime<Uint<K>> {
    fn from_limbs(limbs: &[u64]) -> Self {
        Self::new(<Uint<K> as FromLimbs>::from_limbs(limbs))
    }
}

impl<K: Bits> FromLimbs for NonZero<Uint<K>> {
    fn from_limbs(limbs: &[u64]) -> Self {
        Self::new(<Uint<K> as FromLimbs>::from_limbs(limbs)).expect("a nonzero divisor")
    }
}

impl<K: Bits> FromLimbs for ConstantTime<NonZero<Uint<K>>> {
    fn from_limbs(limbs: &[u64]) -> Self {
        Self::new(NonZero::from_limbs(limbs))
    }
}

/// The operations with an integer operand.
const INTEGER_KINDS: [Kind; 9] = [
    Kind::MulU64,
    Kind::MulUint,
    Kind::AddU64,
    Kind::AddUint,
    Kind::SubU64,
    Kind::SubUint,
    Kind::FromU64,
    Kind::DivU64,
    Kind::DivUint,
];

/// The reason for an absent division by a `k` of 64 bits over a field of 64 bits or fewer.
const WORD_REACHES_P: &str = "a k of 64 bits reaches p";

/// Offers a crate's `a * invert(from(k))` for `kind` in the row of `timing`, with its chain form:
/// [`integer_op`] with the product by the inverse.
#[expect(
    clippy::too_many_arguments,
    reason = "each argument is one part of the composed operation"
)]
fn division_op<K: KindOf, E, I: Copy, V: Visitor<Op, Field>>(
    cells: Crate<V>,
    kind: K,
    timing: Timing,
    function: &'static str,
    element: &impl Fn(&[u64]) -> E,
    integer: &impl Fn(&[u64]) -> I,
    from: impl Fn(I) -> E + Copy,
    invert: impl Fn(E) -> E + Copy,
    read: impl Fn(&E) -> Option<Limbs>,
) -> Crate<V>
where
    E: Copy + core::ops::Mul<Output = E>,
{
    integer_op(
        cells,
        kind,
        timing,
        function,
        element,
        integer,
        (from, move |a, k| a * invert(k)),
        read,
    )
}

/// The operations with an integer of the bit length of `p`.
const UINT_KINDS: [Kind; 3] = [Kind::MulUint, Kind::AddUint, Kind::SubUint];

/// The reason for an absent cell of an integer that may reach `p`.
const NO_REDUCING_CONVERSION: &str = "no conversion that reduces an integer at or above p";

/// Records `reason` for both timings of each of `kinds`.
fn absent_kinds<V: Visitor<Op, Field>>(
    mut cells: Crate<V>,
    kinds: &[Kind],
    reason: &'static str,
) -> Crate<V> {
    for &kind in kinds {
        cells = cells.neither(kind, reason);
    }
    cells
}

/// Records `reason` in the row of `timing` for each of `kinds`.
fn absent_in<V: Visitor<Op, Field>>(
    mut cells: Crate<V>,
    timing: Timing,
    kinds: &[Kind],
    reason: &'static str,
) -> Crate<V> {
    for &kind in kinds {
        cells = cells.absent(Op { kind, timing }, reason);
    }
    cells
}

/// Offers a crate's `op(a, from(k))` for `kind` in the row of `timing`, with its chain form:
/// `element` reads `a`, `integer` reads `k`, and `from` converts `k` inside the measured step.
#[expect(
    clippy::too_many_arguments,
    reason = "each argument is one part of the composed operation"
)]
fn integer_op<K: KindOf, E: Copy, I: Copy, V: Visitor<Op, Field>>(
    cells: Crate<V>,
    kind: K,
    timing: Timing,
    function: &'static str,
    element: &impl Fn(&[u64]) -> E,
    integer: &impl Fn(&[u64]) -> I,
    convert: (impl Fn(I) -> E + Copy, impl Fn(E, E) -> E + Copy),
    read: impl Fn(&E) -> Option<Limbs>,
) -> Crate<V> {
    let (from, op) = convert;
    cells.chain(
        in_row(kind, timing),
        function,
        |set| match set {
            [a, k] => (element(a), integer(k)),
            _ => panic!("{} operands where two were drawn", set.len()),
        },
        #[inline(always)]
        move |&(a, k)| op(a, from(k)),
        read,
    )
}

/// Offers a crate's `a * from(k)`, `a + from(k)` and `a - from(k)` for `k` of 64 bits, and
/// `from(k)`, in the row of `timing`, and records `reason` in the other row. `deviation` tells how
/// each kind's result deviates from its row's definition.
fn word_ops<E, V: Visitor<Op, Field>>(
    cells: Crate<V>,
    timing: Timing,
    element: &impl Fn(&[u64]) -> E,
    from: (&'static str, impl Fn(u64) -> E + Copy),
    read: impl Fn(&E) -> Option<Limbs> + Copy,
    deviation: impl Fn(Kind) -> Option<&'static str>,
) -> Crate<V>
where
    E: Copy + core::ops::Add<Output = E> + core::ops::Sub<Output = E> + core::ops::Mul<Output = E>,
{
    let (name, from) = from;
    let word = |limbs: &[u64]| u64::from_limbs(limbs);
    let names = word_names(name);
    let cells = integer_op(
        cells.declare(deviation(Kind::MulU64)),
        kind::MulU64,
        timing,
        names[0],
        element,
        &word,
        (from, |a, b| a * b),
        read,
    );
    let cells = integer_op(
        cells.declare(deviation(Kind::AddU64)),
        kind::AddU64,
        timing,
        names[1],
        element,
        &word,
        (from, |a, b| a + b),
        read,
    );
    let cells = integer_op(
        cells.declare(deviation(Kind::SubU64)),
        kind::SubU64,
        timing,
        names[2],
        element,
        &word,
        (from, |a, b| a - b),
        read,
    );
    let other = match timing {
        Timing::Variable => Timing::Constant,
        Timing::Constant => Timing::Variable,
    };
    let reason = match other {
        Timing::Variable => NO_VARIABLE_TIME,
        Timing::Constant => NO_CONSTANT_TIME,
    };
    let cells = cells.declare(deviation(Kind::FromU64)).api(
        in_row(kind::FromU64, timing),
        name,
        |set| match set {
            [k] => u64::from_limbs(k),
            _ => panic!("{} operands where one was drawn", set.len()),
        },
        #[inline(always)]
        move |&k| from(k),
        read,
    );
    absent_in(
        cells,
        other,
        &[Kind::MulU64, Kind::AddU64, Kind::SubU64, Kind::FromU64],
        reason,
    )
}

/// The functions of [`word_ops`]'s cells for the conversion `name`.
fn word_names(name: &'static str) -> [&'static str; 3] {
    match name {
        "From<u64>" => ["* From<u64>", "+ From<u64>", "- From<u64>"],
        "from_u64" => ["* from_u64", "+ from_u64", "- from_u64"],
        "from_int" => ["* from_int", "+ from_int", "- from_int"],
        "new" => ["* new", "+ new", "- new"],
        _ => panic!("no names for {name}"),
    }
}

/// The bits of `exponent` from its top set bit down.
fn bits_from_top(exponent: &BigUint) -> Vec<bool> {
    (0..exponent.bits())
        .rev()
        .map(|bit| exponent.bit(bit))
        .collect()
}

/// The exponents of the field `key` that compose an inverse, `p − 2`, and a square root,
/// `(p + 1) / 4`, the latter only for `p ≡ 3 (mod 4)`; each as its bits from the top.
fn exponents(key: &Field) -> (Vec<bool>, Option<Vec<bool>>) {
    let p = big(spec(key).modulus);
    let root = (&p % 4u32 == BigUint::from(3u32)).then(|| bits_from_top(&((&p + 1u32) >> 2)));
    (bits_from_top(&(p - 2u32)), root)
}

/// `a` to the power of `bits`, from the top set bit down, by square-and-multiply with `product`:
/// the sequence of products follows the public exponent alone, so it takes the time class of
/// `product`.
fn power<E>(a: &E, bits: &[bool], product: impl Fn(&E, &E) -> E) -> E
where
    E: Clone,
{
    bits.iter().skip(1).fold(a.clone(), |r, &set| {
        let square = product(&r, &r);
        if set { product(&square, a) } else { square }
    })
}

/// The reason for an absent square root where `p ≡ 1 (mod 4)`, which no single power yields.
const NO_SQRT_POWER: &str = "no square root; p ≡ 1 (mod 4) needs more than a power";

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

/// Offers add, sub, mul and square of `$t`, with their chain forms, in the row `$row` of their
/// timing class, and records `$reason` in the other row.
macro_rules! ops {
    ($cells:expr, $t:ty, $row:ident, $other:ident, $reason:expr,
     add: $add:expr, sub: $sub:expr, mul: $mul:expr, square: $square:expr $(,)?) => {
        $cells
            .chained($row(kind::Add), "+", pair::<$t>, $add)
            .chained($row(kind::Sub), "-", pair::<$t>, $sub)
            .chained($row(kind::Mul), "*", pair::<$t>, $mul)
            .chained($row(kind::Square), "square", one::<$t>, $square)
            .absent($other(Kind::Add), $reason)
            .absent($other(Kind::Sub), $reason)
            .absent($other(Kind::Mul), $reason)
            .absent($other(Kind::Square), $reason)
    };
}

// ---------------------------------------------------------------------------------------------
// prime-field

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

impl<M: Modulus> FromLimbs for ConstantTime<Fp<M>> {
    fn from_limbs(limbs: &[u64]) -> Self {
        Self::new(Fp::from_limbs(limbs))
    }
}

impl<M: Modulus> ToLimbs for ConstantTime<Fp<M>> {
    fn to_limbs(&self) -> Option<Limbs> {
        self.reveal().to_limbs()
    }
}

impl<M: Modulus> FromLimbs for NonZero<Fp<M>> {
    fn from_limbs(limbs: &[u64]) -> Self {
        Self::new(Fp::from_limbs(limbs)).expect("a nonzero operand")
    }
}

impl<M: Modulus> ToLimbs for NonZero<Fp<M>> {
    fn to_limbs(&self) -> Option<Limbs> {
        self.get().to_limbs()
    }
}

impl<M: Modulus> FromLimbs for ConstantTime<NonZero<Fp<M>>> {
    fn from_limbs(limbs: &[u64]) -> Self {
        Self::new(NonZero::from_limbs(limbs))
    }
}

impl<M: Modulus> ToLimbs for ConstantTime<NonZero<Fp<M>>> {
    fn to_limbs(&self) -> Option<Limbs> {
        self.reveal().to_limbs()
    }
}

/// `prime-field`, the crate under test.
fn this_crate<V: Visitor<Op, Field>>(cells: Crate<V>) -> Crate<V> {
    match cells.key().field.as_str() {
        "Mersenne-31" => wide_divisions::<Mersenne31, U30, V>(ours::<Mersenne31, V>(cells))
            .neither(Kind::DivU64, WORD_REACHES_P),
        "BabyBear" => wide_divisions::<BabyBear, U30, V>(ours::<BabyBear, V>(cells))
            .neither(Kind::DivU64, WORD_REACHES_P),
        "Goldilocks" => wide_divisions::<Goldilocks, U63, V>(ours::<Goldilocks, V>(cells))
            .neither(Kind::DivU64, WORD_REACHES_P),
        "BN254 scalar" => divisions::<Bn254Scalar, U253, V>(ours::<Bn254Scalar, V>(cells)),
        "secp256k1 base" => divisions::<Secp256k1Base, U255, V>(ours::<Secp256k1Base, V>(cells)),
        "P-256 base" => divisions::<P256Base, U255, V>(ours::<P256Base, V>(cells)),
        "P-256 scalar" => divisions::<P256Scalar, U255, V>(ours::<P256Scalar, V>(cells)),
        "Tom-256 base" => divisions::<Tom256Base, U255, V>(ours::<Tom256Base, V>(cells)),
        "BLS12-381 base" => divisions::<Bls12381Base, U380, V>(ours::<Bls12381Base, V>(cells)),
        "P-521 base" => divisions::<P521Base, U520, V>(ours::<P521Base, V>(cells)),
        field => panic!("no field {field}"),
    }
}

/// `Fp / NonZeroUint<K>` in the variable-time row of `kind` and `ConstantTime` with `k` wrapped in
/// its constant-time row.
fn division<M: Modulus, K, D: KindOf, V: Visitor<Op, Field>>(
    cells: Crate<V>,
    kind: D,
    names: [&'static str; 2],
) -> Crate<V>
where
    K: Bits + IsLess<M::Width, Output = True>,
{
    cells
        .chained(
            var(kind),
            names[0],
            mixed::<Fp<M>, NonZero<Uint<K>>>,
            #[inline(always)]
            |&(a, k)| a / k,
        )
        .chained(
            ct(kind),
            names[1],
            mixed::<ConstantTime<Fp<M>>, ConstantTime<NonZero<Uint<K>>>>,
            #[inline(always)]
            |&(a, k)| a / k,
        )
}

/// The division by a `k` of `K` bits, one short of the bit length of `p`.
fn wide_divisions<M: Modulus, K, V: Visitor<Op, Field>>(cells: Crate<V>) -> Crate<V>
where
    K: Bits + IsLess<M::Width, Output = True>,
{
    division::<M, K, _, V>(
        cells,
        kind::DivUint,
        ["/ NonZeroUint", "ConstantTime / ConstantTime<NonZeroUint>"],
    )
}

/// The divisions by a `k` of 64 bits and of `K` bits.
fn divisions<M: Modulus, K, V: Visitor<Op, Field>>(cells: Crate<V>) -> Crate<V>
where
    K: Bits + IsLess<M::Width, Output = True>,
    U64: IsLess<M::Width, Output = True>,
{
    let cells = division::<M, U64, _, V>(
        cells,
        kind::DivU64,
        [
            "/ NonZeroUint<U64>",
            "ConstantTime / ConstantTime<NonZeroUint<U64>>",
        ],
    );
    wide_divisions::<M, K, V>(cells)
}

/// `prime-field`'s division as the inverse of the element of `k` and a product.
fn this_crate_inverse<V: Visitor<Op, Field>>(cells: Crate<V>) -> Crate<V> {
    let cells = Op::ALL
        .into_iter()
        .filter(|op| !matches!(op.kind, Kind::DivU64 | Kind::DivUint))
        .fold(cells, |cells, op| {
            cells.absent(
                op,
                "the product by an inverse stands for the divisions alone",
            )
        });
    match cells.key().field.as_str() {
        "Mersenne-31" => {
            inverse_uint::<Mersenne31, U30, V>(cells.neither(Kind::DivU64, WORD_REACHES_P))
        }
        "BabyBear" => inverse_uint::<BabyBear, U30, V>(cells.neither(Kind::DivU64, WORD_REACHES_P)),
        "Goldilocks" => {
            inverse_uint::<Goldilocks, U63, V>(cells.neither(Kind::DivU64, WORD_REACHES_P))
        }
        "BN254 scalar" => {
            inverse_uint::<Bn254Scalar, U253, V>(inverse_word::<Bn254Scalar, V>(cells))
        }
        "secp256k1 base" => {
            inverse_uint::<Secp256k1Base, U255, V>(inverse_word::<Secp256k1Base, V>(cells))
        }
        "P-256 base" => inverse_uint::<P256Base, U255, V>(inverse_word::<P256Base, V>(cells)),
        "P-256 scalar" => inverse_uint::<P256Scalar, U255, V>(inverse_word::<P256Scalar, V>(cells)),
        "Tom-256 base" => inverse_uint::<Tom256Base, U255, V>(inverse_word::<Tom256Base, V>(cells)),
        "BLS12-381 base" => {
            inverse_uint::<Bls12381Base, U380, V>(inverse_word::<Bls12381Base, V>(cells))
        }
        "P-521 base" => inverse_uint::<P521Base, U520, V>(inverse_word::<P521Base, V>(cells)),
        field => panic!("no field {field}"),
    }
}

/// `a * NonZero(Fp::from(k)).invert()` for a `u64` `k`, and its constant-time form on a wrapped
/// `Uint<U64>`.
fn inverse_word<M: Modulus, V: Visitor<Op, Field>>(cells: Crate<V>) -> Crate<V> {
    cells
        .chained(
            var(kind::DivU64),
            "* NonZero(From<u64>).invert()",
            mixed::<Fp<M>, u64>,
            #[inline(always)]
            |&(a, k)| a * inverse(NonZero::new(Fp::<M>::from(k)).expect("k < p")),
        )
        .chained(
            ct(kind::DivU64),
            "ConstantTime * (ConstantTime + ConstantTime<Uint<U64>>).nonzero().invert()",
            mixed::<ConstantTime<Fp<M>>, ConstantTime<Uint<U64>>>,
            #[inline(always)]
            |&(a, k)| a * ct_inverse(k),
        )
}

/// `a * NonZero(Fp::from_uint(k)).invert()` for `k` of `K` bits, and its constant-time form.
fn inverse_uint<M: Modulus, K: Bits, V: Visitor<Op, Field>>(cells: Crate<V>) -> Crate<V> {
    cells
        .chained(
            var(kind::DivUint),
            "* NonZero(from_uint).invert()",
            mixed::<Fp<M>, Uint<K>>,
            #[inline(always)]
            |&(a, k)| {
                let element = Fp::<M>::from_uint(k.resize()).expect("k < p");
                a * inverse(NonZero::new(element).expect("a nonzero k"))
            },
        )
        .chained(
            ct(kind::DivUint),
            "ConstantTime * (ConstantTime + ConstantTime<Uint>).nonzero().invert()",
            mixed::<ConstantTime<Fp<M>>, ConstantTime<Uint<K>>>,
            #[inline(always)]
            |&(a, k)| a * ct_inverse(k),
        )
}

/// `k^-1`, as a call: inlined into each of the 40 cells of the composition, the inverses tripled
/// the build time of the bench. `prime-field`'s division calls its inverse alike.
#[inline(never)]
fn inverse<M: Modulus>(k: NonZero<Fp<M>>) -> Fp<M> {
    k.invert().get()
}

/// The constant-time [`inverse`].
#[inline(never)]
fn invert_ct<M: Modulus>(k: ConstantTime<NonZero<Fp<M>>>) -> ConstantTime<Fp<M>> {
    // `reveal` and `new` move the same bits.
    ConstantTime::new(k.invert().reveal().get())
}

/// The constant-time inverse of the element of the wrapped nonzero integer `k`.
#[inline(always)]
fn ct_inverse<M: Modulus, K: Bits>(k: ConstantTime<Uint<K>>) -> ConstantTime<Fp<M>> {
    invert_ct(
        (ConstantTime::new(Fp::<M>::ZERO) + k)
            .nonzero()
            .expect("a nonzero k"),
    )
}

/// `Fp` in the variable-time rows, `ConstantTime` in the constant-time rows. Each operation inlines
/// into the batch loop.
fn ours<M: Modulus, V: Visitor<Op, Field>>(cells: Crate<V>) -> Crate<V> {
    cells
        .chained(
            var(kind::Add),
            "+",
            pair::<Fp<M>>,
            #[inline(always)]
            |&(a, b)| a + b,
        )
        .chained(
            var(kind::Sub),
            "-",
            pair::<Fp<M>>,
            #[inline(always)]
            |&(a, b)| a - b,
        )
        .chained(
            var(kind::Mul),
            "*",
            pair::<Fp<M>>,
            #[inline(always)]
            |&(a, b)| a * b,
        )
        .chained(
            var(kind::Square),
            "square",
            one::<Fp<M>>,
            #[inline(always)]
            |a| a.square(),
        )
        .case(
            var(kind::Invert),
            "invert",
            one::<NonZero<Fp<M>>>,
            #[inline(always)]
            |a| a.invert(),
        )
        .case(
            var(kind::Sqrt),
            "sqrt",
            one::<Fp<M>>,
            #[inline(always)]
            |a| a.sqrt(),
        )
        .chained(
            ct(kind::Add),
            "ConstantTime +",
            pair::<ConstantTime<Fp<M>>>,
            #[inline(always)]
            |&(a, b)| a + b,
        )
        .chained(
            ct(kind::Sub),
            "ConstantTime -",
            pair::<ConstantTime<Fp<M>>>,
            #[inline(always)]
            |&(a, b)| a - b,
        )
        .chained(
            ct(kind::Mul),
            "ConstantTime *",
            pair::<ConstantTime<Fp<M>>>,
            #[inline(always)]
            |&(a, b)| a * b,
        )
        .chained(
            ct(kind::Square),
            "ConstantTime::square",
            one::<ConstantTime<Fp<M>>>,
            #[inline(always)]
            |a| a.square(),
        )
        .case(
            ct(kind::Invert),
            "ConstantTime::invert",
            one::<ConstantTime<NonZero<Fp<M>>>>,
            #[inline(always)]
            |a| a.invert(),
        )
        .case(
            ct(kind::Sqrt),
            "ConstantTime::sqrt",
            one::<ConstantTime<Fp<M>>>,
            #[inline(always)]
            |a| a.sqrt(),
        )
        .chained(
            var(kind::MulU64),
            "* Uint<U64>",
            mixed::<Fp<M>, Uint<U64>>,
            #[inline(always)]
            |&(a, k)| a * k,
        )
        .chained(
            var(kind::MulUint),
            "* Uint",
            mixed::<Fp<M>, Uint<M::Width>>,
            #[inline(always)]
            |&(a, k)| a * k,
        )
        .chained(
            var(kind::AddU64),
            "+ Uint<U64>",
            mixed::<Fp<M>, Uint<U64>>,
            #[inline(always)]
            |&(a, k)| a + k,
        )
        .chained(
            var(kind::AddUint),
            "+ Uint",
            mixed::<Fp<M>, Uint<M::Width>>,
            #[inline(always)]
            |&(a, k)| a + k,
        )
        .chained(
            var(kind::SubU64),
            "- Uint<U64>",
            mixed::<Fp<M>, Uint<U64>>,
            #[inline(always)]
            |&(a, k)| a - k,
        )
        .chained(
            var(kind::SubUint),
            "- Uint",
            mixed::<Fp<M>, Uint<M::Width>>,
            #[inline(always)]
            |&(a, k)| a - k,
        )
        .case(
            var(kind::FromU64),
            "From<u64>",
            one::<u64>,
            #[inline(always)]
            |&k| Fp::<M>::from(k),
        )
        .chained(
            ct(kind::MulU64),
            "ConstantTime * ConstantTime<Uint<U64>>",
            mixed::<ConstantTime<Fp<M>>, ConstantTime<Uint<U64>>>,
            #[inline(always)]
            |&(a, k)| a * k,
        )
        .chained(
            ct(kind::MulUint),
            "ConstantTime * ConstantTime<Uint>",
            mixed::<ConstantTime<Fp<M>>, ConstantTime<Uint<M::Width>>>,
            #[inline(always)]
            |&(a, k)| a * k,
        )
        .chained(
            ct(kind::AddU64),
            "ConstantTime + ConstantTime<Uint<U64>>",
            mixed::<ConstantTime<Fp<M>>, ConstantTime<Uint<U64>>>,
            #[inline(always)]
            |&(a, k)| a + k,
        )
        .chained(
            ct(kind::AddUint),
            "ConstantTime + ConstantTime<Uint>",
            mixed::<ConstantTime<Fp<M>>, ConstantTime<Uint<M::Width>>>,
            #[inline(always)]
            |&(a, k)| a + k,
        )
        .chained(
            ct(kind::SubU64),
            "ConstantTime - ConstantTime<Uint<U64>>",
            mixed::<ConstantTime<Fp<M>>, ConstantTime<Uint<U64>>>,
            #[inline(always)]
            |&(a, k)| a - k,
        )
        .chained(
            ct(kind::SubUint),
            "ConstantTime - ConstantTime<Uint>",
            mixed::<ConstantTime<Fp<M>>, ConstantTime<Uint<M::Width>>>,
            #[inline(always)]
            |&(a, k)| a - k,
        )
        .absent(constant(Kind::FromU64), PUBLIC_INTEGER)
}

/// The reason for an absent constant-time conversion of a `u64`, which is a public operand.
const PUBLIC_INTEGER: &str = "a u64 is public: From<u64> takes time that depends on it";

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
    let cells = word_ops(
        cells,
        Timing::Variable,
        &element,
        ("From<u64>", <F as From<u64>>::from),
        read,
        |_| None,
    );
    let invert = |k: F| k.inverse().expect("a nonzero k");
    let cells = division_op(
        cells,
        kind::DivU64,
        Timing::Variable,
        "* inverse of From<u64>",
        &element,
        &|limbs: &[u64]| u64::from_limbs(limbs),
        <F as From<u64>>::from,
        invert,
        read,
    );
    let bigint = |limbs: &[u64]| {
        let mut value = F::BigInt::default();
        value.as_mut()[..limbs.len()].copy_from_slice(limbs);
        value
    };
    let cells = division_op(
        cells,
        kind::DivUint,
        Timing::Variable,
        "* inverse of from_bigint",
        &element,
        &bigint,
        |k| F::from_bigint(k).expect("k < p"),
        invert,
        read,
    );
    let mut cells = absent_kinds(cells, &UINT_KINDS, NO_REDUCING_CONVERSION)
        .chain(
            var(kind::Add),
            "+",
            two,
            #[inline(always)]
            |&(a, b)| a + b,
            read,
        )
        .chain(
            var(kind::Sub),
            "-",
            two,
            #[inline(always)]
            |&(a, b)| a - b,
            read,
        )
        .chain(
            var(kind::Mul),
            "*",
            two,
            #[inline(always)]
            |&(a, b)| a * b,
            read,
        )
        .chain(
            var(kind::Square),
            "square",
            single,
            #[inline(always)]
            |a| a.square(),
            read,
        )
        .api(
            var(kind::Invert),
            "inverse",
            single,
            #[inline(always)]
            |a| a.inverse(),
            read_option,
        )
        .api(
            var(kind::Sqrt),
            "sqrt",
            single,
            #[inline(always)]
            |a| a.sqrt(),
            read_option,
        );
    for kind in KINDS.into_iter().chain(DIVISION_KINDS) {
        cells = cells.absent(constant(kind), NO_CONSTANT_TIME);
    }
    cells
}

/// The divisions by an integer.
const DIVISION_KINDS: [Kind; 2] = [Kind::DivU64, Kind::DivUint];

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

/// The `crypto-bigint` moduli of the fields of `prime_field::standard`.
mod cb {
    use crypto_bigint::Uint;
    use crypto_bigint::modular::{
        ConstMontyParams, ConstPrimeMontyParams, FixedMontyParams, PrimeParams,
    };
    use prime_field::standard;

    use super::words;

    /// Declares, per line, the modulus `$name` of `$limbs` limbs with the value of
    /// `standard::$name` and the multiplicative generator `$generator`.
    macro_rules! params {
        ($($name:ident: $limbs:literal, $generator:literal;)*) => {
            $(
                #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
                pub struct $name;

                impl ConstMontyParams<$limbs> for $name {
                    const LIMBS: usize = $limbs;
                    const PARAMS: FixedMontyParams<$limbs> = FixedMontyParams::new_vartime(
                        Uint::from_words(words::<standard::$name, $limbs>())
                            .to_odd()
                            .expect_copied("an odd modulus"),
                    );
                }

                impl ConstPrimeMontyParams<$limbs> for $name {
                    const PRIME_PARAMS: PrimeParams<$limbs> = PrimeParams::new_vartime(
                        &<$name as ConstMontyParams<$limbs>>::PARAMS,
                        $generator,
                    );
                }
            )*
        };
    }

    params! {
        Mersenne31: 1, 7;
        BabyBear: 1, 31;
        Goldilocks: 1, 7;
        Bn254Scalar: 4, 5;
        Secp256k1Base: 4, 3;
        P256Base: 4, 2;
        P256Scalar: 4, 11;
        Tom256Base: 4, 6;
        Bls12381Base: 6, 2;
        P521Base: 9, 3;
    }
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
        "Tom-256 base" => monty::<cb::Tom256Base, 4, V>(cells),
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
    let element = |limbs: &[u64]| Form::<MOD, L>::from_limbs(limbs);
    let read = |e: &Form<MOD, L>| e.to_limbs();
    let new = |k: crypto_bigint::Uint<L>| Form::<MOD, L>::new(&k);
    let words = |limbs: &[u64]| crypto_bigint::Uint::<L>::from_words(array::<L>(limbs));
    let cells = word_ops(
        cells,
        Timing::Constant,
        &element,
        ("new", move |k| new(crypto_bigint::Uint::from_u64(k))),
        read,
        |_| None,
    );
    let cells = integer_op(
        cells,
        kind::MulUint,
        Timing::Constant,
        "* new",
        &element,
        &words,
        (new, |a, b| a * b),
        read,
    );
    let cells = integer_op(
        cells,
        kind::AddUint,
        Timing::Constant,
        "+ new",
        &element,
        &words,
        (new, |a, b| a + b),
        read,
    );
    let cells = integer_op(
        cells,
        kind::SubUint,
        Timing::Constant,
        "- new",
        &element,
        &words,
        (new, |a, b| a - b),
        read,
    );
    let cells = absent_in(cells, Timing::Variable, &UINT_KINDS, NO_VARIABLE_TIME);
    let invert = |k: Form<MOD, L>| k.invert().into_option().expect("a nonzero k");
    let invert_vartime = |k: Form<MOD, L>| k.invert_vartime().into_option().expect("a nonzero k");
    let word = |limbs: &[u64]| u64::from_limbs(limbs);
    let from_word = move |k| new(crypto_bigint::Uint::from_u64(k));
    let cells = if L > 1 {
        let cells = division_op(
            cells,
            kind::DivU64,
            Timing::Constant,
            "* invert of new",
            &element,
            &word,
            from_word,
            invert,
            read,
        );
        division_op(
            cells,
            kind::DivU64,
            Timing::Variable,
            "* invert_vartime of new",
            &element,
            &word,
            from_word,
            invert_vartime,
            read,
        )
    } else {
        cells.neither(Kind::DivU64, WORD_REACHES_P)
    };
    let cells = division_op(
        cells,
        kind::DivUint,
        Timing::Constant,
        "* invert of new",
        &element,
        &words,
        new,
        invert,
        read,
    );
    let cells = division_op(
        cells,
        kind::DivUint,
        Timing::Variable,
        "* invert_vartime of new",
        &element,
        &words,
        new,
        invert_vartime,
        read,
    );
    ops!(cells, Form<MOD, L>, ct, vartime, NO_VARIABLE_TIME,
        add: #[inline(always)] |(a, b)| a + b,
        sub: #[inline(always)] |(a, b)| a - b,
        mul: #[inline(always)] |(a, b)| a * b,
        square: #[inline(always)] |a| a.square(),
    )
    .case(
        ct(kind::Invert),
        "invert",
        one::<Form<MOD, L>>,
        #[inline(always)]
        |a| a.invert(),
    )
    .case(
        var(kind::Invert),
        "invert_vartime",
        one::<Form<MOD, L>>,
        #[inline(always)]
        |a| a.invert_vartime(),
    )
    .case(
        ct(kind::Sqrt),
        "sqrt",
        one::<Form<MOD, L>>,
        #[inline(always)]
        |a| a.sqrt(),
    )
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

/// k256's own normalization, which every k256 case declares: its results are not normalized.
impl ToLimbs for k256::FieldElement {
    fn to_limbs(&self) -> Option<Limbs> {
        Some(from_be_bytes(&self.normalize().to_bytes()))
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

/// A RustCrypto field's `a * invert(from(k))` in the constant-time rows, each case declaring
/// `deviation`: `word` converts a `k` of 64 bits, `from_bytes` the big-endian bytes of a `k` below
/// `p`. The variable-time rows record that the crate has no such function.
fn rust_crypto_divisions<E, V: Visitor<Op, Field>>(
    cells: Crate<V>,
    deviation: Option<&'static str>,
    element: &impl Fn(&[u64]) -> E,
    word: (&'static str, impl Fn(u64) -> E + Copy),
    from_bytes: impl Fn([u8; 32]) -> E + Copy,
    invert: impl Fn(E) -> E + Copy,
    read: impl Fn(&E) -> Option<Limbs> + Copy,
) -> Crate<V>
where
    E: Copy + core::ops::Mul<Output = E>,
{
    let cells = division_op(
        cells.declare(deviation),
        kind::DivU64,
        Timing::Constant,
        word.0,
        element,
        &|limbs: &[u64]| u64::from_limbs(limbs),
        word.1,
        invert,
        read,
    );
    let cells = division_op(
        cells.declare(deviation),
        kind::DivUint,
        Timing::Constant,
        "* invert of from_bytes",
        element,
        &be_bytes_32,
        from_bytes,
        invert,
        read,
    );
    absent_in(cells, Timing::Variable, &DIVISION_KINDS, NO_VARIABLE_TIME)
}

/// The RustCrypto curve crates: `k256` for secp256k1, `p256` for P-256. Their field arithmetic
/// selects by masks and `subtle`; `k256`'s `+` and `-` add limbs without a carry and leave the
/// reduction to a later `normalize`. `p256`'s scalar adds `invert_vartime`, Stein's algorithm.
fn rust_crypto<V: Visitor<Op, Field>>(cells: Crate<V>) -> Crate<V> {
    match cells.key().field.as_str() {
        "secp256k1 base" => {
            type Element = k256::FieldElement;
            let mut cells = cells
                .deviating(SKIPPED_REDUCTION)
                .chained(
                    ct(kind::Add),
                    "+",
                    pair::<Element>,
                    #[inline(always)]
                    |&(a, b)| a + b,
                )
                .deviating(SKIPPED_REDUCTION)
                .chained(
                    ct(kind::Sub),
                    "-",
                    pair::<Element>,
                    #[inline(always)]
                    |&(a, b)| a - b,
                )
                .deviating(SKIPPED_NORMALIZATION)
                .chained(
                    ct(kind::Mul),
                    "*",
                    pair::<Element>,
                    #[inline(always)]
                    |&(a, b)| a * b,
                )
                .deviating(SKIPPED_NORMALIZATION)
                .chained(
                    ct(kind::Square),
                    "square",
                    one::<Element>,
                    #[inline(always)]
                    |a| a.square(),
                )
                .deviating(SKIPPED_NORMALIZATION)
                .case(
                    ct(kind::Invert),
                    "invert",
                    one::<Element>,
                    #[inline(always)]
                    |a| a.invert(),
                )
                .deviating(SKIPPED_NORMALIZATION)
                .case(
                    ct(kind::Sqrt),
                    "sqrt",
                    one::<Element>,
                    #[inline(always)]
                    |a| a.sqrt(),
                );
            for kind in KINDS {
                cells = cells.absent(vartime(kind), NO_VARIABLE_TIME);
            }
            let cells = word_ops(
                cells,
                Timing::Constant,
                &<Element as FromLimbs>::from_limbs,
                ("from_u64", Element::from_u64),
                <Element as ToLimbs>::to_limbs,
                |kind| {
                    if kind == Kind::MulU64 {
                        Some(SKIPPED_NORMALIZATION)
                    } else if kind == Kind::AddU64 || kind == Kind::SubU64 {
                        Some(SKIPPED_REDUCTION)
                    } else {
                        None
                    }
                },
            );
            let cells = rust_crypto_divisions(
                cells,
                Some(SKIPPED_NORMALIZATION),
                &<Element as FromLimbs>::from_limbs,
                ("* invert of from_u64", Element::from_u64),
                |bytes| {
                    Element::from_bytes(&bytes.into())
                        .into_option()
                        .expect("k < p")
                },
                |k: Element| k.invert().into_option().expect("a nonzero k"),
                <Element as ToLimbs>::to_limbs,
            );
            absent_kinds(cells, &UINT_KINDS, NO_REDUCING_CONVERSION)
        }
        "P-256 base" => {
            let cells = ops!(cells, p256::FieldElement, ct, vartime, NO_VARIABLE_TIME,
                add: #[inline(always)] |&(a, b)| a + b,
                sub: #[inline(always)] |&(a, b)| a - b,
                mul: #[inline(always)] |&(a, b)| a * b,
                square: #[inline(always)] |a| a.square(),
            )
            .case(
                ct(kind::Invert),
                "invert",
                one::<p256::FieldElement>,
                #[inline(always)]
                |a| a.invert(),
            )
            .absent(vartime(Kind::Invert), NO_VARIABLE_TIME)
            .case(
                ct(kind::Sqrt),
                "sqrt",
                one::<p256::FieldElement>,
                #[inline(always)]
                |a| a.sqrt(),
            )
            .absent(vartime(Kind::Sqrt), NO_VARIABLE_TIME);
            let cells = word_ops(
                cells,
                Timing::Constant,
                &<p256::FieldElement as FromLimbs>::from_limbs,
                ("From<u64>", <p256::FieldElement as From<u64>>::from),
                <p256::FieldElement as ToLimbs>::to_limbs,
                |_| None,
            );
            let cells = rust_crypto_divisions(
                cells,
                None,
                &<p256::FieldElement as FromLimbs>::from_limbs,
                (
                    "* invert of From<u64>",
                    <p256::FieldElement as From<u64>>::from,
                ),
                |bytes| {
                    p256::FieldElement::from_bytes(&bytes.into())
                        .into_option()
                        .expect("k < p")
                },
                |k: p256::FieldElement| k.invert().into_option().expect("a nonzero k"),
                <p256::FieldElement as ToLimbs>::to_limbs,
            );
            absent_kinds(cells, &UINT_KINDS, NO_REDUCING_CONVERSION)
        }
        "P-256 scalar" => {
            use p256::elliptic_curve::ff::Field;
            use p256::elliptic_curve::ops::{Invert, Reduce};
            type Scalar = p256::Scalar;
            let element = <Scalar as FromLimbs>::from_limbs;
            let words = |limbs: &[u64]| p256::U256::from_words(array::<4>(limbs));
            let reduce = <Scalar as Reduce<p256::U256>>::reduce;
            let read = <Scalar as ToLimbs>::to_limbs;
            let cells = word_ops(
                cells,
                Timing::Constant,
                &element,
                ("From<u64>", <Scalar as From<u64>>::from),
                read,
                |_| None,
            );
            let cells = integer_op(
                cells,
                kind::MulUint,
                Timing::Constant,
                "* Reduce<U256>",
                &element,
                &words,
                (reduce, |a, b| a * b),
                read,
            );
            let cells = integer_op(
                cells,
                kind::AddUint,
                Timing::Constant,
                "+ Reduce<U256>",
                &element,
                &words,
                (reduce, |a, b| a + b),
                read,
            );
            let cells = integer_op(
                cells,
                kind::SubUint,
                Timing::Constant,
                "- Reduce<U256>",
                &element,
                &words,
                (reduce, |a, b| a - b),
                read,
            );
            let cells = absent_in(cells, Timing::Variable, &UINT_KINDS, NO_VARIABLE_TIME);
            let invert = |k: Scalar| k.invert().into_option().expect("a nonzero k");
            let invert_vartime = |k: Scalar| {
                Invert::invert_vartime(&k)
                    .into_option()
                    .expect("a nonzero k")
            };
            let word = |limbs: &[u64]| u64::from_limbs(limbs);
            let from_word = <Scalar as From<u64>>::from;
            let cells = division_op(
                cells,
                kind::DivU64,
                Timing::Constant,
                "* invert of From<u64>",
                &element,
                &word,
                from_word,
                invert,
                read,
            );
            let cells = division_op(
                cells,
                kind::DivU64,
                Timing::Variable,
                "* invert_vartime of From<u64>",
                &element,
                &word,
                from_word,
                invert_vartime,
                read,
            );
            let cells = division_op(
                cells,
                kind::DivUint,
                Timing::Constant,
                "* invert of Reduce<U256>",
                &element,
                &words,
                reduce,
                invert,
                read,
            );
            let cells = division_op(
                cells,
                kind::DivUint,
                Timing::Variable,
                "* invert_vartime of Reduce<U256>",
                &element,
                &words,
                reduce,
                invert_vartime,
                read,
            );
            ops!(cells, p256::Scalar, ct, vartime, NO_VARIABLE_TIME,
                add: #[inline(always)] |&(a, b)| a + b,
                sub: #[inline(always)] |&(a, b)| a - b,
                mul: #[inline(always)] |&(a, b)| a * b,
                square: #[inline(always)] |a| a.square(),
            )
            .case(
                ct(kind::Invert),
                "invert",
                one::<p256::Scalar>,
                p256::Scalar::invert,
            )
            .case(
                var(kind::Invert),
                "invert_vartime",
                one::<p256::Scalar>,
                Invert::invert_vartime,
            )
            .case(ct(kind::Sqrt), "sqrt", one::<p256::Scalar>, Field::sqrt)
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
/// A halo2curves field read from the limbs of any integer of its width by `from_raw`, which reduces
/// it.
trait Raw {
    /// The limbs that `from_raw` takes.
    type Words: Copy;

    /// The limbs of an integer.
    fn words(limbs: &[u64]) -> Self::Words;

    /// The element of the integer.
    fn raw(words: Self::Words) -> Self;
}

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

        impl Raw for $t {
            type Words = [u64; $limbs];

            fn words(limbs: &[u64]) -> Self::Words {
                array::<$limbs>(limbs)
            }

            fn raw(words: Self::Words) -> Self {
                Self::from_raw(words)
            }
        }
    };
}

halo2_field!(halo2curves::bn256::Fr, 4, from_le_bytes);
halo2_field!(halo2curves::secp256k1::Fp, 4, from_le_bytes);
halo2_field!(halo2curves::secp256r1::Fp, 4, from_le_bytes);
halo2_field!(halo2curves::secp256r1::Fq, 4, from_le_bytes);
halo2_field!(halo2curves::t256::Fp, 4, from_be_bytes);
halo2_field!(halo2curves::bls12381::Fq, 6, from_be_bytes);

/// `halo2curves`.
fn halo2curves<V: Visitor<Op, Field>>(cells: Crate<V>) -> Crate<V> {
    match cells.key().field.as_str() {
        "BN254 scalar" => ff_field::<halo2curves::bn256::Fr, V>(cells),
        "secp256k1 base" => ff_field::<halo2curves::secp256k1::Fp, V>(cells),
        "P-256 base" => ff_field::<halo2curves::secp256r1::Fp, V>(cells),
        "P-256 scalar" => ff_field::<halo2curves::secp256r1::Fq, V>(cells),
        "Tom-256 base" => ff_field::<halo2curves::t256::Fp, V>(cells),
        "BLS12-381 base" => ff_field::<halo2curves::bls12381::Fq, V>(cells),
        _ => cells.none("halo2curves declares no such field"),
    }
}

/// A field of halo2curves through the `ff::Field` trait: masks in `+`, `-` and `*`, and the
/// constant-time Tonelli–Shanks or exponentiation of `ff` in `sqrt`. Its `invert` runs the
/// variable-time division steps of halo2curves' `BYInverter`, which branch on the trailing
/// zeros of `g`.
fn ff_field<F, V: Visitor<Op, Field>>(cells: Crate<V>) -> Crate<V>
where
    F: halo2curves::ff::Field + From<u64> + Raw + FromLimbs + ToLimbs,
{
    let modulus = spec(cells.key()).modulus;
    let inverse_exponent = limbs(&(big(modulus) - 2u32), modulus.len());
    let element = <F as FromLimbs>::from_limbs;
    let read = <F as ToLimbs>::to_limbs;
    let cells = word_ops(
        cells,
        Timing::Constant,
        &element,
        ("From<u64>", <F as From<u64>>::from),
        read,
        |_| None,
    );
    let cells = integer_op(
        cells,
        kind::MulUint,
        Timing::Constant,
        "* from_raw",
        &element,
        &F::words,
        (F::raw, |a, b| a * b),
        read,
    );
    let cells = integer_op(
        cells,
        kind::AddUint,
        Timing::Constant,
        "+ from_raw",
        &element,
        &F::words,
        (F::raw, |a, b| a + b),
        read,
    );
    let cells = integer_op(
        cells,
        kind::SubUint,
        Timing::Constant,
        "- from_raw",
        &element,
        &F::words,
        (F::raw, |a, b| a - b),
        read,
    );
    let cells = absent_in(cells, Timing::Variable, &UINT_KINDS, NO_VARIABLE_TIME);
    let invert = |k: F| k.invert().into_option().expect("a nonzero k");
    let power = |k: F| k.pow(&inverse_exponent);
    let word = |limbs: &[u64]| u64::from_limbs(limbs);
    let cells = division_op(
        cells,
        kind::DivU64,
        Timing::Variable,
        "* invert of From<u64>",
        &element,
        &word,
        <F as From<u64>>::from,
        invert,
        read,
    );
    let cells = division_op(
        cells,
        kind::DivU64,
        Timing::Constant,
        "* pow(p − 2) of From<u64>",
        &element,
        &word,
        <F as From<u64>>::from,
        power,
        read,
    );
    let cells = division_op(
        cells,
        kind::DivUint,
        Timing::Variable,
        "* invert of from_raw",
        &element,
        &F::words,
        F::raw,
        invert,
        read,
    );
    let cells = division_op(
        cells,
        kind::DivUint,
        Timing::Constant,
        "* pow(p − 2) of from_raw",
        &element,
        &F::words,
        F::raw,
        power,
        read,
    );
    ops!(cells, F, ct, vartime, NO_VARIABLE_TIME,
        add: #[inline(always)] |&(a, b)| a + b,
        sub: #[inline(always)] |&(a, b)| a - b,
        mul: #[inline(always)] |&(a, b)| a * b,
        square: #[inline(always)] |a| a.square(),
    )
    .case(
        var(kind::Invert),
        "invert",
        one::<F>,
        #[inline(always)]
        |a| a.invert(),
    )
    .case(
        ct(kind::Invert),
        "pow(p − 2)",
        one::<F>,
        #[inline(always)]
        |a| a.pow(&inverse_exponent),
    )
    .case(
        ct(kind::Sqrt),
        "sqrt",
        one::<F>,
        #[inline(always)]
        |a| a.sqrt(),
    )
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

        /// Plonky3's own normalization, which the cases of a non-canonical [`Representation`]
        /// declare.
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
        "Mersenne-31" => p3::<p3_mersenne_31::Mersenne31, V>(
            cells,
            true,
            #[inline(always)]
            |a| a.try_sqrt(),
            Representation::ZeroAsP,
        ),
        "BabyBear" => p3::<p3_baby_bear::BabyBear, V>(
            cells,
            false,
            #[inline(always)]
            |a| a.try_sqrt(),
            Representation::Canonical,
        ),
        "Goldilocks" => p3::<p3_goldilocks::Goldilocks, V>(
            cells,
            false,
            #[inline(always)]
            |a| a.try_sqrt(),
            Representation::Below2To64,
        ),
        _ => cells.none("Plonky3 declares no such field"),
    }
}

/// The values that a Plonky3 field stores, which `as_canonical_u64` normalizes.
#[derive(Clone, Copy)]
enum Representation {
    /// Every value lies below `p`: BabyBear's Montgomery form.
    Canonical,
    /// Every value lies below 2^31, and `p` is a second encoding of zero: Mersenne-31.
    ZeroAsP,
    /// Every value lies below 2^64: Goldilocks.
    Below2To64,
}

impl Representation {
    /// How the results of `kind` deviate from canonical residues; `None` when they are canonical.
    /// An inverse or a square root of a nonzero element is nonzero, so it never takes the second
    /// encoding of zero.
    const fn deviation(self, kind: Kind) -> Option<&'static str> {
        match (self, kind) {
            (
                Self::ZeroAsP,
                Kind::Add
                | Kind::Sub
                | Kind::Mul
                | Kind::Square
                | Kind::MulU64
                | Kind::MulUint
                | Kind::AddU64
                | Kind::AddUint
                | Kind::SubU64
                | Kind::SubUint
                | Kind::DivU64
                | Kind::DivUint,
            ) => Some("returns p for a zero result, which is not canonical"),
            (Self::Below2To64, _) => Some(SKIPPED_REDUCTION),
            (Self::Canonical, _) | (Self::ZeroAsP, Kind::Invert | Kind::Sqrt | Kind::FromU64) => {
                None
            }
        }
    }
}

/// A Plonky3 field: `+`, `*`, `square`, `try_inverse` and `try_sqrt` decide their corrections and
/// steps with `if` on the value; Mersenne-31's `-` is arithmetic alone, BabyBear's and
/// Goldilocks' `-` branch on the borrow. `subtraction_constant` tells the arithmetic `-`; `sqrt` is the
/// field's own `try_sqrt`, which BabyBear specializes in an inherent method; `representation`
/// tells which results deviate from canonical residues.
fn p3<
    F: p3_field::Field + p3_field::integers::QuotientMap<u64> + FromLimbs + ToLimbs,
    V: Visitor<Op, Field>,
>(
    cells: Crate<V>,
    subtraction_constant: bool,
    sqrt: impl Fn(&F) -> Option<F> + Copy,
    representation: Representation,
) -> Crate<V> {
    let (sub_row, sub_other, sub_reason) = if subtraction_constant {
        (ct(kind::Sub), vartime(Kind::Sub), NO_VARIABLE_TIME)
    } else {
        (var(kind::Sub), constant(Kind::Sub), NO_CONSTANT_TIME)
    };
    let deviation = |kind| representation.deviation(kind);
    let mut cells = cells
        .declare(deviation(Kind::Add))
        .chained(
            var(kind::Add),
            "+",
            pair::<F>,
            #[inline(always)]
            |&(a, b)| a + b,
        )
        .declare(deviation(Kind::Sub))
        .chained(
            sub_row,
            "-",
            pair::<F>,
            #[inline(always)]
            |&(a, b)| a - b,
        )
        .absent(sub_other, sub_reason)
        .declare(deviation(Kind::Mul))
        .chained(
            var(kind::Mul),
            "*",
            pair::<F>,
            #[inline(always)]
            |&(a, b)| a * b,
        )
        .declare(deviation(Kind::Square))
        .chained(
            var(kind::Square),
            "square",
            one::<F>,
            #[inline(always)]
            |a| a.square(),
        )
        .declare(deviation(Kind::Invert))
        .case(
            var(kind::Invert),
            "try_inverse",
            one::<F>,
            #[inline(always)]
            |a| a.try_inverse(),
        )
        .declare(deviation(Kind::Sqrt))
        .case(
            var(kind::Sqrt),
            "try_sqrt",
            one::<F>,
            #[inline(always)]
            move |a| sqrt(a),
        );
    for kind in [Kind::Add, Kind::Mul, Kind::Square, Kind::Invert, Kind::Sqrt] {
        cells = cells.absent(constant(kind), NO_CONSTANT_TIME);
    }
    let element = <F as FromLimbs>::from_limbs;
    let read = <F as ToLimbs>::to_limbs;
    let from_int = <F as p3_field::integers::QuotientMap<u64>>::from_int;
    let cells = word_ops(
        cells,
        Timing::Variable,
        &element,
        ("from_int", from_int),
        read,
        deviation,
    );
    // The field fits 64 bits, so an integer of its width is a `u64`.
    let word = <u64 as FromLimbs>::from_limbs;
    let cells = integer_op(
        cells.declare(deviation(Kind::MulUint)),
        kind::MulUint,
        Timing::Variable,
        "* from_int",
        &element,
        &word,
        (from_int, |a, b| a * b),
        read,
    );
    let cells = integer_op(
        cells.declare(deviation(Kind::AddUint)),
        kind::AddUint,
        Timing::Variable,
        "+ from_int",
        &element,
        &word,
        (from_int, |a, b| a + b),
        read,
    );
    let cells = integer_op(
        cells.declare(deviation(Kind::SubUint)),
        kind::SubUint,
        Timing::Variable,
        "- from_int",
        &element,
        &word,
        (from_int, |a, b| a - b),
        read,
    );
    let cells = division_op(
        cells.declare(deviation(Kind::DivUint)),
        kind::DivUint,
        Timing::Variable,
        "* try_inverse of from_int",
        &element,
        &word,
        from_int,
        |k: F| k.try_inverse().expect("a nonzero k"),
        read,
    )
    .absent(constant(Kind::DivUint), NO_CONSTANT_TIME)
    .neither(Kind::DivU64, WORD_REACHES_P);
    absent_in(cells, Timing::Constant, &UINT_KINDS, NO_CONSTANT_TIME)
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
        "Goldilocks" => {
            let element = <BaseElement as FromLimbs>::from_limbs;
            let read = <BaseElement as ToLimbs>::to_limbs;
            let cells = word_ops(
                cells,
                Timing::Constant,
                &element,
                ("new", BaseElement::new),
                read,
                |_| None,
            );
            // The field fits 64 bits, so an integer of its width is a `u64`.
            let word = <u64 as FromLimbs>::from_limbs;
            let cells = integer_op(
                cells,
                kind::MulUint,
                Timing::Constant,
                "* new",
                &element,
                &word,
                (BaseElement::new, |a, b| a * b),
                read,
            );
            let cells = integer_op(
                cells,
                kind::AddUint,
                Timing::Constant,
                "+ new",
                &element,
                &word,
                (BaseElement::new, |a, b| a + b),
                read,
            );
            let cells = integer_op(
                cells,
                kind::SubUint,
                Timing::Constant,
                "- new",
                &element,
                &word,
                (BaseElement::new, |a, b| a - b),
                read,
            );
            let cells = absent_in(cells, Timing::Variable, &UINT_KINDS, NO_VARIABLE_TIME);
            let cells = division_op(
                cells,
                kind::DivUint,
                Timing::Constant,
                "* inv of new",
                &element,
                &word,
                BaseElement::new,
                |k: BaseElement| k.inv(),
                read,
            )
            .absent(vartime(Kind::DivUint), NO_VARIABLE_TIME)
            .neither(Kind::DivU64, WORD_REACHES_P);
            ops!(cells, BaseElement, ct, vartime, NO_VARIABLE_TIME,
                add: #[inline(always)] |&(a, b)| a + b,
                sub: #[inline(always)] |&(a, b)| a - b,
                mul: #[inline(always)] |&(a, b)| a * b,
                square: #[inline(always)] |a| a.square(),
            )
            .case(
                ct(kind::Invert),
                "inv",
                one::<BaseElement>,
                #[inline(always)]
                |a| a.inv(),
            )
            .absent(vartime(Kind::Invert), NO_VARIABLE_TIME)
            .neither(Kind::Sqrt, NO_SQRT_POWER)
        }
        _ => cells.none("winter-math declares no such field"),
    }
}

// ---------------------------------------------------------------------------------------------
// longfellow-zk runtime-algebra

/// `runtime-algebra`: `P256Field` and `Q256Field` for P-256, `FpGenericField` elsewhere.
fn runtime_algebra<V: Visitor<Op, Field>>(cells: Crate<V>) -> Crate<V> {
    use runtime_algebra::fp_generic::FpGenericField;
    match cells.key().field.as_str() {
        "Mersenne-31" => runtime::<1, _, V>(
            FpGenericField::<1, 1, 3>::new_generic(words::<Mersenne31, 1>()),
            cells,
        ),
        "BabyBear" => runtime::<1, _, V>(
            FpGenericField::<1, 1, 3>::new_generic(words::<BabyBear, 1>()),
            cells,
        ),
        "Goldilocks" => runtime::<1, _, V>(
            FpGenericField::<1, 1, 3>::new_generic(words::<Goldilocks, 1>()),
            cells,
        ),
        "BN254 scalar" => runtime::<4, _, V>(
            FpGenericField::<4, 4, 9>::new_generic(words::<Bn254Scalar, 4>()),
            cells,
        ),
        "secp256k1 base" => runtime::<4, _, V>(
            FpGenericField::<4, 4, 9>::new_generic(words::<Secp256k1Base, 4>()),
            cells,
        ),
        "P-256 base" => runtime::<4, _, V>(runtime_algebra::p256::P256Field::new(), cells),
        "P-256 scalar" => runtime::<4, _, V>(runtime_algebra::Q256Field::new(), cells),
        "Tom-256 base" => runtime::<4, _, V>(
            FpGenericField::<4, 4, 9>::new_generic(words::<Tom256Base, 4>()),
            cells,
        ),
        "BLS12-381 base" => runtime::<6, _, V>(
            FpGenericField::<6, 6, 13>::new_generic(words::<Bls12381Base, 6>()),
            cells,
        ),
        "P-521 base" => runtime::<9, _, V>(
            FpGenericField::<9, 9, 19>::new_generic(words::<P521Base, 9>()),
            cells,
        ),
        field => panic!("no field {field}"),
    }
}

/// The operations of a longfellow-zk runtime field `F` over `W` words: `addf`, `subf` and `mulf`
/// select by `cmov` in inline assembly; `invert` runs the binary extended Euclidean algorithm,
/// which branches on the parity of the value.
fn runtime<const W: usize, F: RuntimeSerializableField<W>, V: Visitor<Op, Field>>(
    field: F,
    cells: Crate<V>,
) -> Crate<V>
where
    F::E: Copy,
{
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
    let (inverse, root) = exponents(cells.key());
    let (inverse, root) = (&inverse, root.as_deref());
    let cells = match root {
        Some(root) => cells.api(
            ct(kind::Sqrt),
            "mulf to (p + 1) / 4",
            single,
            #[inline(always)]
            |a| power(a, root, |x, y| field.mulf(x, y)),
            read,
        ),
        None => cells.absent(constant(Kind::Sqrt), NO_SQRT_POWER),
    };
    let words = [Kind::MulU64, Kind::AddU64, Kind::SubU64, Kind::FromU64];
    // `words64_to_element` refuses a value at or above `p`, which a `u64` reaches on one limb.
    let cells = if W > 1 {
        let word = <u64 as FromLimbs>::from_limbs;
        let from = |k: u64| {
            let mut words = [0; W];
            words[0] = k;
            field
                .words64_to_element(&words)
                .expect("an integer below p")
        };
        let cells = integer_op(
            cells,
            kind::MulU64,
            Timing::Variable,
            "mulf of words64_to_element",
            &element,
            &word,
            (from, |a, b| field.mulf(&a, &b)),
            read,
        );
        let cells = integer_op(
            cells,
            kind::AddU64,
            Timing::Variable,
            "addf of words64_to_element",
            &element,
            &word,
            (from, |a, b| field.addf(&a, &b)),
            read,
        );
        let cells = integer_op(
            cells,
            kind::SubU64,
            Timing::Variable,
            "subf of words64_to_element",
            &element,
            &word,
            (from, |a, b| field.subf(&a, &b)),
            read,
        );
        let cells = cells.api(
            var(kind::FromU64),
            "words64_to_element",
            one::<u64>,
            #[inline(always)]
            move |&k| from(k),
            read,
        );
        let cells = integer_op(
            cells,
            kind::DivU64,
            Timing::Variable,
            "mulf by invert of words64_to_element",
            &element,
            &word,
            (from, |a, b| field.mulf(&a, &field.invert(&b))),
            read,
        );
        let cells = integer_op(
            cells,
            kind::DivU64,
            Timing::Constant,
            "mulf by mulf to p − 2 of words64_to_element",
            &element,
            &word,
            (from, |a, b| {
                field.mulf(&a, &power(&b, inverse, |x, y| field.mulf(x, y)))
            }),
            read,
        );
        absent_in(cells, Timing::Constant, &words, NO_CONSTANT_TIME)
    } else {
        let cells = absent_kinds(
            cells,
            &words,
            "words64_to_element refuses an integer at or above p",
        );
        cells.neither(Kind::DivU64, WORD_REACHES_P)
    };
    let integer = |limbs: &[u64]| array::<W>(limbs);
    let from = |k: [u64; W]| field.words64_to_element(&k).expect("k < p");
    let cells = integer_op(
        cells,
        kind::DivUint,
        Timing::Variable,
        "mulf by invert of words64_to_element",
        &element,
        &integer,
        (from, |a, b| field.mulf(&a, &field.invert(&b))),
        read,
    );
    let cells = integer_op(
        cells,
        kind::DivUint,
        Timing::Constant,
        "mulf by mulf to p − 2 of words64_to_element",
        &element,
        &integer,
        (from, |a, b| {
            field.mulf(&a, &power(&b, inverse, |x, y| field.mulf(x, y)))
        }),
        read,
    );
    absent_kinds(cells, &UINT_KINDS, NO_REDUCING_CONVERSION)
        .chain(
            ct(kind::Add),
            "addf",
            two,
            #[inline(always)]
            |(a, b)| field.addf(a, b),
            read,
        )
        .chain(
            ct(kind::Sub),
            "subf",
            two,
            #[inline(always)]
            |(a, b)| field.subf(a, b),
            read,
        )
        .chain(
            ct(kind::Mul),
            "mulf",
            two,
            #[inline(always)]
            |(a, b)| field.mulf(a, b),
            read,
        )
        .absent(vartime(Kind::Add), NO_VARIABLE_TIME)
        .absent(vartime(Kind::Sub), NO_VARIABLE_TIME)
        .absent(vartime(Kind::Mul), NO_VARIABLE_TIME)
        .chain(
            ct(kind::Square),
            "mulf(a, a)",
            single,
            #[inline(always)]
            |a| field.mulf(a, a),
            read,
        )
        .absent(vartime(Kind::Square), NO_VARIABLE_TIME)
        .api(
            var(kind::Invert),
            "invert",
            single,
            #[inline(always)]
            |a| field.invert(a),
            read,
        )
        .api(
            ct(kind::Invert),
            "mulf to p − 2",
            single,
            #[inline(always)]
            |a| power(a, inverse, |x, y| field.mulf(x, y)),
            read,
        )
        .absent(vartime(Kind::Sqrt), NO_VARIABLE_TIME)
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
    let (_, root) = exponents(cells.key());
    let cells = absent_kinds(
        cells,
        &INTEGER_KINDS,
        "no conversion of an integer: bytes_to_element takes a byte slice",
    );
    let cells = cells
        .api(
            var(kind::Add),
            "addf",
            two,
            #[inline(always)]
            |(a, b)| field.addf(a, b),
            read,
        )
        .api(
            var(kind::Sub),
            "subf",
            two,
            #[inline(always)]
            |(a, b)| field.subf(a, b),
            read,
        )
        .api(
            var(kind::Mul),
            "mulf",
            two,
            #[inline(always)]
            |(a, b)| field.mulf(a, b),
            read,
        )
        .api(
            var(kind::Invert),
            "invert",
            single,
            #[inline(always)]
            |a| field.invert(a),
            read,
        )
        .absent(constant(Kind::Add), NO_CONSTANT_TIME)
        .absent(constant(Kind::Sub), NO_CONSTANT_TIME)
        .absent(constant(Kind::Mul), NO_CONSTANT_TIME)
        .absent(constant(Kind::Invert), NO_CONSTANT_TIME)
        .api(
            var(kind::Square),
            "mulf(a, a)",
            single,
            #[inline(always)]
            |a| field.mulf(a, a),
            read,
        )
        .absent(constant(Kind::Square), NO_CONSTANT_TIME)
        .absent(constant(Kind::Sqrt), NO_CONSTANT_TIME);
    match root.as_deref() {
        Some(root) => cells.api(
            var(kind::Sqrt),
            "mulf to (p + 1) / 4",
            single,
            #[inline(always)]
            |a| power(a, root, |x, y| field.mulf(x, y)),
            read,
        ),
        None => cells.absent(vartime(Kind::Sqrt), NO_SQRT_POWER),
    }
}
