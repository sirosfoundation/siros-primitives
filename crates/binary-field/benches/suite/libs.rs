//! One adapter per crate: the function that the crate runs for each operation over each field.
//!
//! Each crate uses its own public function for an operation, in the row of its timing class: the
//! constant-time row when no branch and no memory address of the function depends on a value, by
//! its source, and the variable-time row otherwise. The other row of the operation gets an absent
//! cell, as does a crate without the function or the field, with the reason. Elements enter and
//! leave each crate in its own representation through a [`Carrier`]; the conversions run before
//! and after the counted batch.

use core::ops::{Add, Mul};

use bench_compare::{Cells, Limbs, Row, Visitor};
use binary_field::generic_array::GenericArray;
use binary_field::{Gf2, Secret, Square};
use core_algebra::{AlgebraicField, SerializableField, SupportsU128Conversions};
use hekate_math::{
    BinaryFieldExtras, Block8, Block16, Block32, Block64, Block128, Block256, TowerField,
};
use p3_binary_field::{BinaryField16, Ghash128, Poly64, Rijndael8b, TowerLevel};
use polyval::hazmat::FieldElement;

use super::basis::{Basis, Level, Tower};
use super::reference::Modulus;
use super::{AES, Field, GF8, GF16, GF64, GF128, GF233, GF256, GF571, Kind, Op, Timing, spec};

/// The timing classes, by name at the adapters.
const CONSTANT: Timing = Timing::Constant;
const VARIABLE: Timing = Timing::Variable;

/// The reason for the absent cell of a function in the row of the other timing class.
const fn only(timing: Timing) -> &'static str {
    match timing {
        Timing::Constant => "constant time, in the constant-time row",
        Timing::Variable => "variable time, in the variable-time row",
    }
}

/// The cells of one crate in one row.
type Crate<V> = Cells<Op, Field, V>;

/// Offers the cells of every crate in `row`.
pub fn offer<V: Visitor<Op, Field>>(row: Row<Op, Field, V>) -> Row<Op, Field, V> {
    row.lib("binary-field", this_crate)
        .lib("gf256", gf256)
        .lib("galois_field_2pm", galois_field_2pm)
        .lib("g2p", g2p)
        .lib("reed-solomon-erasure", reed_solomon_erasure)
        .lib("hekate-math", hekate_math)
        .lib("plonky3", plonky3)
        .lib("runtime-algebra", runtime_algebra)
        .lib("compile-algebra", compile_algebra)
        .lib("core-algebra", core_algebra)
        .lib("polyval", polyval)
        .lib("galois_field_2pm lut", galois_field_2pm_lut)
}

/// The reasons for absent cells that several crates share.
const NO_SQUARE: &str = "no squaring function";
const NO_SQRT: &str = "no square root";
const NO_INVERSE: &str = "no inverse function";
const GF128_ONLY: &str = "GF(2¹²⁸) only";

/// Carries a row's elements, as limbs, to a crate's elements of type `T` and back.
trait Carrier<T> {
    /// The crate's element of the row element `limbs`.
    fn to_crate(&self, limbs: &[u64]) -> T;

    /// The row element of the crate's `element`.
    fn to_row(&self, element: &T) -> Limbs;
}

/// A crate's element type, by the bits that it stores.
trait Raw {
    fn from_raw(bits: &[u64]) -> Self;
    fn raw(&self) -> Limbs;
}

/// The crate stores an element in the row's polynomial basis: bit `i` is the coefficient of
/// `x^i`.
struct Same;

/// The carrier of a crate that stores the row's elements as the row does.
const fn same() -> Same {
    Same
}

impl<T: Raw> Carrier<T> for Same {
    fn to_crate(&self, limbs: &[u64]) -> T {
        T::from_raw(limbs)
    }

    fn to_row(&self, element: &T) -> Limbs {
        element.raw()
    }
}

/// The crate stores an element in a tower basis.
impl<T: Raw> Carrier<T> for Basis {
    fn to_crate(&self, limbs: &[u64]) -> T {
        T::from_raw(&self.crate_bits(limbs))
    }

    fn to_row(&self, element: &T) -> Limbs {
        self.row_element(&element.raw())
    }
}

/// The result is an element.
fn element<C: Carrier<T>, T>(carrier: &C, result: &T) -> Option<Limbs> {
    Some(carrier.to_row(result))
}

/// The result is an element, or `None` for the zero operand.
fn maybe<C: Carrier<T>, T>(carrier: &C, result: &Option<T>) -> Option<Limbs> {
    result.as_ref().map(|value| carrier.to_row(value))
}

/// The cells of one crate in one row, whose operands and results pass through the carrier that
/// `carrier` builds when a case is checked.
struct Carried<V, F> {
    cells: Crate<V>,
    carrier: F,
}

impl<V: Visitor<Op, Field>, C, F: Fn() -> C + Copy> Carried<V, F> {
    /// The case of `op` on two operands: `function`, as `run`; `read` reads its result.
    fn binary_in<T, R>(
        self,
        op: Op,
        function: &'static str,
        run: impl Fn(&(T, T)) -> R + Copy,
        read: fn(&C, &R) -> Option<Limbs>,
    ) -> Self
    where
        C: Carrier<T>,
    {
        let prepare = |carrier: &C, set: &[Limbs]| match set {
            [a, b] => (carrier.to_crate(a), carrier.to_crate(b)),
            _ => panic!("{} operands where two were drawn", set.len()),
        };
        Self {
            cells: self
                .cells
                .api_with(op, function, self.carrier, prepare, run, read),
            carrier: self.carrier,
        }
    }

    /// The case of `op` on one operand.
    fn unary_in<T, R>(
        self,
        op: Op,
        function: &'static str,
        run: impl Fn(&T) -> R + Copy,
        read: fn(&C, &R) -> Option<Limbs>,
    ) -> Self
    where
        C: Carrier<T>,
    {
        let prepare = |carrier: &C, set: &[Limbs]| match set {
            [a] => carrier.to_crate(a),
            _ => panic!("{} operands where one was drawn", set.len()),
        };
        Self {
            cells: self
                .cells
                .api_with(op, function, self.carrier, prepare, run, read),
            carrier: self.carrier,
        }
    }

    /// The case of `kind` on two operands in the row of `timing`, and the absent cell of the other
    /// row.
    fn binary<T, R>(
        self,
        kind: Kind,
        timing: Timing,
        function: &'static str,
        run: impl Fn(&(T, T)) -> R + Copy,
        read: fn(&C, &R) -> Option<Limbs>,
    ) -> Self
    where
        C: Carrier<T>,
    {
        let cells = self.binary_in(kind.in_row(timing), function, run, read);
        Self {
            cells: cells
                .cells
                .absent(kind.in_row(timing.other()), only(timing)),
            carrier: cells.carrier,
        }
    }

    /// The case of `kind` on one operand in the row of `timing`, and the absent cell of the other
    /// row.
    fn unary<T, R>(
        self,
        kind: Kind,
        timing: Timing,
        function: &'static str,
        run: impl Fn(&T) -> R + Copy,
        read: fn(&C, &R) -> Option<Limbs>,
    ) -> Self
    where
        C: Carrier<T>,
    {
        let cells = self.unary_in(kind.in_row(timing), function, run, read);
        Self {
            cells: cells
                .cells
                .absent(kind.in_row(timing.other()), only(timing)),
            carrier: cells.carrier,
        }
    }

    /// Records that the crate has no case for `kind` in either row, for `reason`.
    fn absent(self, kind: Kind, reason: &'static str) -> Self {
        Self {
            cells: self
                .cells
                .absent(kind.in_row(CONSTANT), reason)
                .absent(kind.in_row(VARIABLE), reason),
            ..self
        }
    }

    /// The cells.
    fn cells(self) -> Crate<V> {
        self.cells
    }
}

/// The cells of a crate in a row.
trait Crates<V>: Sized {
    /// The cells, with operands and results that pass through the carrier that `carrier` builds.
    fn carry<C, F: Fn() -> C + Copy>(self, carrier: F) -> Carried<V, F>;

    /// Records the same reason for every operation.
    fn none(self, reason: &'static str) -> Self;
}

impl<V: Visitor<Op, Field>> Crates<V> for Crate<V> {
    fn carry<C, F: Fn() -> C + Copy>(self, carrier: F) -> Carried<V, F> {
        Carried {
            cells: self,
            carrier,
        }
    }

    fn none(self, reason: &'static str) -> Self {
        Op::ALL
            .into_iter()
            .fold(self, |cells, op| cells.absent(op, reason))
    }
}

/// The unsigned word of at most two limbs, which must fit it.
fn word<W: TryFrom<u128>>(bits: &[u64]) -> W {
    let value = match bits {
        [] => 0,
        [low] => u128::from(*low),
        [low, high] => u128::from(*low) | (u128::from(*high) << 64),
        _ => panic!("{} limbs for one word", bits.len()),
    };
    W::try_from(value).unwrap_or_else(|_| panic!("{value:#x} does not fit the word"))
}

/// The limbs of an unsigned word.
fn limbs(value: impl Into<u128>) -> Limbs {
    let value = value.into();
    vec![value as u64, (value >> 64) as u64]
}

/// Implements [`Raw`] for types stored as one unsigned word: `$from` builds the element from the
/// word `$word` of type `$w`, `$to` reads the word of `$element`.
macro_rules! raw_word {
    ($($t:ty => |$word:ident: $w:ty| $from:expr, |$element:ident| $to:expr;)*) => {
        $(
            impl Raw for $t {
                fn from_raw(bits: &[u64]) -> Self {
                    let $word: $w = word(bits);
                    $from
                }

                fn raw(&self) -> Limbs {
                    let $element = self;
                    limbs($to)
                }
            }
        )*
    };
}

/// Add, an exclusive or, and mul of the class `mul` by the operators of a type that holds the
/// row's elements.
fn operators<T, V>(cells: Crate<V>, mul: Timing) -> Carried<V, impl Fn() -> Same + Copy>
where
    T: Raw + Copy + Add<Output = T> + Mul<Output = T>,
    V: Visitor<Op, Field>,
{
    cells
        .carry(same)
        .binary(Kind::Add, CONSTANT, "+", |&(a, b): &(T, T)| a + b, element)
        .binary(Kind::Mul, mul, "*", |&(a, b): &(T, T)| a * b, element)
}

// ---------------------------------------------------------------------------------------------
// binary-field

impl<P: binary_field::Polynomial> Raw for Gf2<P> {
    fn from_raw(bits: &[u64]) -> Self {
        let array = GenericArray::<u64, binary_field::Limbs<P>>::try_from_slice(bits)
            .unwrap_or_else(|_| panic!("{} limbs for the field", bits.len()));
        Self::from_limbs(*array).expect("an element of the field")
    }

    fn raw(&self) -> Limbs {
        self.into_limbs().to_vec()
    }
}

/// `binary-field`, the crate under test.
fn this_crate<V: Visitor<Op, Field>>(cells: Crate<V>) -> Crate<V> {
    match cells.key().field.as_str() {
        GF8 => ours::<binary_field::Aes, V>(cells),
        GF16 => ours::<binary_field::Poly16, V>(cells),
        GF64 => ours::<binary_field::Poly64, V>(cells),
        GF128 => ours::<binary_field::Gcm, V>(cells),
        GF233 => ours::<binary_field::B233, V>(cells),
        GF256 => ours::<binary_field::Poly256, V>(cells),
        GF571 => ours::<binary_field::B571, V>(cells),
        field => panic!("no field {field}"),
    }
}

impl<P: binary_field::Polynomial> Raw for Secret<P> {
    fn from_raw(bits: &[u64]) -> Self {
        Self::new(Gf2::from_raw(bits))
    }

    fn raw(&self) -> Limbs {
        self.reveal().raw()
    }
}

/// `Gf2`'s defaults in the variable-time rows, `Secret`'s operations in the constant-time rows.
fn ours<P: binary_field::Polynomial, V: Visitor<Op, Field>>(cells: Crate<V>) -> Crate<V> {
    type Public<P> = Gf2<P>;
    cells
        .carry(same)
        .binary_in(
            Kind::Add.in_row(VARIABLE),
            "+",
            |&(a, b): &(Public<P>, _)| a + b,
            element,
        )
        .binary_in(
            Kind::Mul.in_row(VARIABLE),
            "*",
            |&(a, b): &(Public<P>, _)| a * b,
            element,
        )
        .unary_in(
            Kind::Square.in_row(VARIABLE),
            "square",
            |&a: &Public<P>| a.square(),
            element,
        )
        .unary_in(
            Kind::Invert.in_row(VARIABLE),
            "invert",
            |&a: &Public<P>| a.invert(),
            maybe,
        )
        .unary_in(
            Kind::Sqrt.in_row(VARIABLE),
            "sqrt",
            |&a: &Public<P>| a.sqrt(),
            element,
        )
        .binary_in(
            Kind::Add.in_row(CONSTANT),
            "Secret +",
            |&(a, b): &(Secret<P>, _)| a + b,
            element,
        )
        .binary_in(
            Kind::Mul.in_row(CONSTANT),
            "Secret *",
            |&(a, b): &(Secret<P>, _)| a * b,
            element,
        )
        .unary_in(
            Kind::Square.in_row(CONSTANT),
            "Secret::square",
            |&a: &Secret<P>| a.square(),
            element,
        )
        .unary_in(
            Kind::Invert.in_row(CONSTANT),
            "Secret::invert",
            |&a: &Secret<P>| a.invert(),
            maybe,
        )
        .unary_in(
            Kind::Sqrt.in_row(CONSTANT),
            "Secret::sqrt",
            |&a: &Secret<P>| a.sqrt(),
            element,
        )
        .cells()
}

// ---------------------------------------------------------------------------------------------
// gf256

/// The AES field of `gf256`'s `gf` macro.
mod gf256_aes {
    #![expect(
        clippy::manual_strip,
        reason = "the expansion of gf256's `gf` macro strips a prefix by hand"
    )]

    /// GF(2⁸) with the polynomial of AES, in the default mode of the macro for 8 bits: log and
    /// antilog tables.
    #[gf256::gf::gf(polynomial = 0x11b, generator = 0x3)]
    pub type Gf256Aes;
}

use gf256::gf::{gf2p16, gf2p64};
use gf256_aes::Gf256Aes;

raw_word! {
    Gf256Aes => |word: u8| Self::new(word), |element| element.get();
    gf2p16 => |word: u16| Self::new(word), |element| element.get();
    gf2p64 => |word: u64| Self::new(word), |element| element.get();
}

/// Offers the operations of the `gf256` type `$t`, whose product and reciprocal have the timing
/// `$timing`.
macro_rules! gf256_ops {
    ($cells:expr, $t:ty, $timing:expr) => {
        operators::<$t, V>($cells, $timing)
            .absent(Kind::Square, NO_SQUARE)
            .unary(
                Kind::Invert,
                $timing,
                "checked_recip",
                |&a: &$t| a.checked_recip(),
                maybe,
            )
            .absent(Kind::Sqrt, NO_SQRT)
            .cells()
    };
}

/// `gf256`: its `gf2p16` and `gf2p64`, and the AES field of its `gf` macro. The macro's table mode
/// at 8 bits branches on zero and indexes its tables by the operands; its Barrett mode above takes
/// carry-less products, and the reciprocal's exponent is fixed.
fn gf256<V: Visitor<Op, Field>>(cells: Crate<V>) -> Crate<V> {
    match cells.key().field.as_str() {
        GF8 => gf256_ops!(cells, Gf256Aes, VARIABLE),
        GF16 => gf256_ops!(cells, gf2p16, CONSTANT),
        GF64 => gf256_ops!(cells, gf2p64, CONSTANT),
        _ => cells.none("fields of at most 64 bits"),
    }
}

// ---------------------------------------------------------------------------------------------
// galois_field_2pm

/// The moduli of the rows as `galois_field_2pm` writes them: one bit per coefficient.
const AES_BITS: u128 = 0x11b;
const GF16_BITS: u128 = 0x1_002d;
const GF64_BITS: u128 = 0x1_0000_0000_0000_001b;

raw_word! {
    galois_field_2pm::gf2::GFu8<AES_BITS> => |word: u8| Self { value: word }, |element| element.value;
    galois_field_2pm::gf2::GFu16<GF16_BITS> => |word: u16| Self { value: word }, |element| element.value;
    galois_field_2pm::gf2::GFu64<GF64_BITS> => |word: u64| Self { value: word }, |element| element.value;
    galois_field_2pm::gf2_lut::GFu16<GF16_BITS> => |word: u16| Self { value: word }, |element| element.value;
}

/// A field of `galois_field_2pm`, either module: the products branch on the bits of an operand or
/// index tables by it, and the inverse is the extended Euclidean algorithm.
fn galois<T, V>(cells: Crate<V>) -> Crate<V>
where
    T: galois_field_2pm::GaloisField + Raw + Add<Output = T> + Mul<Output = T>,
    V: Visitor<Op, Field>,
{
    operators::<T, V>(cells, VARIABLE)
        .absent(Kind::Square, NO_SQUARE)
        .unary(
            Kind::Invert,
            VARIABLE,
            "inverse",
            |a: &T| a.inverse(),
            element,
        )
        .absent(Kind::Sqrt, NO_SQRT)
        .cells()
}

/// `galois_field_2pm`'s `gf2` module, which computes products by carry-less multiplication.
fn galois_field_2pm<V: Visitor<Op, Field>>(cells: Crate<V>) -> Crate<V> {
    use galois_field_2pm::gf2::{GFu8, GFu16, GFu64};
    match cells.key().field.as_str() {
        GF8 => galois::<GFu8<AES_BITS>, V>(cells),
        GF16 => galois::<GFu16<GF16_BITS>, V>(cells),
        GF64 => galois::<GFu64<GF64_BITS>, V>(cells),
        _ => cells.none("fields of at most 127 bits"),
    }
}

/// `galois_field_2pm`'s `gf2_lut` module: log and antilog tables, for a primitive polynomial of
/// at most 16 bits.
fn galois_field_2pm_lut<V: Visitor<Op, Field>>(cells: Crate<V>) -> Crate<V> {
    match cells.key().field.as_str() {
        GF8 => cells.none("the AES polynomial is not primitive"),
        GF16 => galois::<galois_field_2pm::gf2_lut::GFu16<GF16_BITS>, V>(cells),
        _ => cells.none("tables of at most 16 bits"),
    }
}

// ---------------------------------------------------------------------------------------------
// g2p

g2p::g2p!(G2pAes, 8, modulus: 0b1_0001_1011);
g2p::g2p!(G2p16, 16, modulus: 0x1_002d);

raw_word! {
    G2pAes => |word: u8| Self(word), |element| element.0;
    G2p16 => |word: u16| Self(word), |element| element.0;
}

/// A field of `g2p`, whose product indexes its tables by the operands.
fn g2p_field<T, V>(cells: Crate<V>) -> Crate<V>
where
    T: Raw + Copy + Add<Output = T> + Mul<Output = T>,
    V: Visitor<Op, Field>,
{
    operators::<T, V>(cells, VARIABLE)
        .absent(Kind::Square, NO_SQUARE)
        .absent(Kind::Invert, NO_INVERSE)
        .absent(Kind::Sqrt, NO_SQRT)
        .cells()
}

/// `g2p`.
fn g2p<V: Visitor<Op, Field>>(cells: Crate<V>) -> Crate<V> {
    match cells.key().field.as_str() {
        GF8 => g2p_field::<G2pAes, V>(cells),
        GF16 => g2p_field::<G2p16, V>(cells),
        _ => cells.none("fields of at most 32 bits"),
    }
}

// ---------------------------------------------------------------------------------------------
// reed-solomon-erasure

/// `x⁸ + x⁴ + x³ + x² + 1`, the modulus of `reed-solomon-erasure`'s GF(2⁸).
const RS_ERASURE_8: Modulus = Modulus {
    degree: 8,
    taps: &[4, 3, 2, 0],
};

/// An element of `reed-solomon-erasure`'s GF(2⁸): one byte, bit `i` the coefficient of `x^i`.
impl Raw for u8 {
    fn from_raw(bits: &[u64]) -> Self {
        word(bits)
    }

    fn raw(&self) -> Limbs {
        limbs(*self)
    }
}

/// An element `hi·y + lo` of `reed-solomon-erasure`'s GF(2¹⁶): the bytes `[hi, lo]`.
impl Raw for [u8; 2] {
    fn from_raw(bits: &[u64]) -> Self {
        word::<u16>(bits).to_be_bytes()
    }

    fn raw(&self) -> Limbs {
        limbs(u16::from_be_bytes(*self))
    }
}

/// `reed-solomon-erasure`: its `galois_8` module, and its `galois_16` module, the extension of
/// that field by `y² + 2y + 128`.
fn reed_solomon_erasure<V: Visitor<Op, Field>>(cells: Crate<V>) -> Crate<V> {
    use reed_solomon_erasure::galois_8;
    use reed_solomon_erasure::galois_16::Field as Galois16;
    let modulus = spec(cells.key()).1.modulus;
    let tower = move |levels: Vec<Level>| {
        Basis::new(
            &modulus,
            &Tower {
                base: RS_ERASURE_8,
                levels,
            },
        )
    };
    match cells.key().field.as_str() {
        GF8 => rs_erasure(
            cells.carry(move || tower(Vec::new())),
            galois_8::add,
            galois_8::mul,
        ),
        GF16 => rs_erasure(
            cells.carry(move || {
                tower(vec![Level {
                    a: vec![2],
                    b: vec![128],
                }])
            }),
            <Galois16 as reed_solomon_erasure::Field>::add,
            <Galois16 as reed_solomon_erasure::Field>::mul,
        ),
        _ => cells.none("GF(2⁸) and GF(2¹⁶) only"),
    }
}

/// A field of `reed-solomon-erasure`, with its `add` and its `mul`, which indexes tables by the
/// operands.
fn rs_erasure<T: Raw + Copy, V: Visitor<Op, Field>>(
    cells: Carried<V, impl Fn() -> Basis + Copy>,
    add: impl Fn(T, T) -> T + Copy,
    mul: impl Fn(T, T) -> T + Copy,
) -> Crate<V> {
    cells
        .binary(
            Kind::Add,
            CONSTANT,
            "add",
            move |&(a, b): &(T, T)| add(a, b),
            element,
        )
        .binary(
            Kind::Mul,
            VARIABLE,
            "mul",
            move |&(a, b): &(T, T)| mul(a, b),
            element,
        )
        .absent(Kind::Square, NO_SQUARE)
        .absent(Kind::Invert, NO_INVERSE)
        .absent(Kind::Sqrt, NO_SQRT)
        .cells()
}

// ---------------------------------------------------------------------------------------------
// hekate-math

raw_word! {
    Block8 => |word: u8| Self(word), |element| element.0;
    Block16 => |word: u16| Self(word), |element| element.0;
    Block64 => |word: u64| Self(word), |element| element.0;
    Block128 => |word: u128| Self(word), |element| element.0;
}

impl Raw for Block256 {
    fn from_raw(bits: &[u64]) -> Self {
        match bits {
            [low @ .., _, _] if low.len() == 2 => Self([word(low), word(&bits[2..])]),
            _ => panic!("{} limbs for GF(2²⁵⁶)", bits.len()),
        }
    }

    fn raw(&self) -> Limbs {
        let [low, high] = self.0;
        let mut bits = limbs(low);
        bits.extend(limbs(high));
        bits
    }
}

/// `hekate-math`'s tower up to `bits` bits: the AES field, then each level extended by
/// `X² + X + τ`, with the `τ` that the level declares.
fn hekate_tower(bits: usize) -> Tower {
    let taus = [
        limbs(<Block8 as TowerField>::EXTENSION_TAU.0),
        limbs(<Block16 as TowerField>::EXTENSION_TAU.0),
        limbs(<Block32 as TowerField>::EXTENSION_TAU.0),
        limbs(<Block64 as TowerField>::EXTENSION_TAU.0),
        limbs(<Block128 as TowerField>::EXTENSION_TAU.0),
    ];
    let levels = (bits / AES.degree).trailing_zeros() as usize;
    Tower {
        base: AES,
        levels: taus
            .into_iter()
            .take(levels)
            .map(|tau| Level { a: vec![1], b: tau })
            .collect(),
    }
}

/// `hekate-math`: its tower fields, from the AES field at 8 bits. Without its `table-math`
/// feature every operation masks and shifts, without a table or a branch on a value.
fn hekate_math<V: Visitor<Op, Field>>(cells: Crate<V>) -> Crate<V> {
    let modulus = spec(cells.key()).1.modulus;
    let tower = move || Basis::new(&modulus, &hekate_tower(modulus.degree));
    match cells.key().field.as_str() {
        GF8 => operators::<Block8, V>(cells, CONSTANT)
            .unary(
                Kind::Square,
                CONSTANT,
                "square",
                |&a: &Block8| a.square(),
                element,
            )
            .unary(
                Kind::Invert,
                CONSTANT,
                "invert",
                |a: &Block8| a.invert(),
                element,
            )
            .absent(Kind::Sqrt, NO_SQRT)
            .cells(),
        GF16 => hekate::<Block16, V>(cells.carry(tower)),
        GF64 => hekate::<Block64, V>(cells.carry(tower)),
        GF128 => hekate::<Block128, V>(cells.carry(tower)),
        GF256 => hekate::<Block256, V>(cells.carry(tower)),
        _ => cells.none("no tower field of that degree"),
    }
}

fn hekate<T: BinaryFieldExtras + Raw, V: Visitor<Op, Field>>(
    cells: Carried<V, impl Fn() -> Basis + Copy>,
) -> Crate<V> {
    cells
        .binary(Kind::Add, CONSTANT, "+", |&(a, b): &(T, T)| a + b, element)
        .binary(Kind::Mul, CONSTANT, "*", |&(a, b): &(T, T)| a * b, element)
        .unary(
            Kind::Square,
            CONSTANT,
            "square",
            |a: &T| a.square(),
            element,
        )
        .unary(
            Kind::Invert,
            CONSTANT,
            "invert",
            |a: &T| a.invert(),
            element,
        )
        .absent(Kind::Sqrt, NO_SQRT)
        .cells()
}

// ---------------------------------------------------------------------------------------------
// plonky3

raw_word! {
    Rijndael8b => |word: u8| Self::from_byte(word), |element| element.to_byte();
    BinaryField16 => |word: u16| Self::from_repr(word), |element| element.to_repr();
    Poly64 => |word: u64| Self::new(word), |element| Poly64::to_bits(*element);
    Ghash128 => |word: u128| Self::from_le_bytes(word.to_le_bytes()), |element| element.to_repr();
}

/// Plonky3's Wiedemann tower up to 16 bits: GF(2), then each level `T_k` extended by
/// `X_k² + X_(k-1)·X_k + 1`, with `X_(-1) = 1`.
fn wiedemann_tower() -> Tower {
    Tower {
        base: Modulus {
            degree: 1,
            taps: &[0],
        },
        levels: [0b1, 0b10, 0b100, 0b1_0000]
            .into_iter()
            .map(|generator| Level {
                a: vec![generator],
                b: vec![1],
            })
            .collect(),
    }
}

/// The timing classes of a Plonky3 field's product, square, inverse and square root.
#[derive(Clone, Copy)]
struct P3Timings {
    mul: Timing,
    square: Timing,
    invert: Timing,
    sqrt: Timing,
}

/// Plonky3's `p3-binary-field`: `Rijndael8b`, `BinaryField16` in the Wiedemann tower, `Poly64`
/// and `Ghash128`.
///
/// - `Rijndael8b` multiplies bit by bit under masks, and squares and roots by bit-matrix products.
/// - `BinaryField16` multiplies through the logarithm tables of its byte field, and squares, roots
///   and inverts through byte tables.
/// - `Poly64` takes carry-less products, folds by them or by a table in a register, and inverts by
///   a fixed chain of squarings and products.
/// - `Ghash128` takes carry-less products, and inverts through tables of Frobenius maps.
fn plonky3<V: Visitor<Op, Field>>(cells: Crate<V>) -> Crate<V> {
    let modulus = spec(cells.key()).1.modulus;
    let constant = P3Timings {
        mul: CONSTANT,
        square: CONSTANT,
        invert: CONSTANT,
        sqrt: CONSTANT,
    };
    match cells.key().field.as_str() {
        GF8 => p3::<Rijndael8b, _, V>(cells.carry(same), constant),
        GF16 => p3::<BinaryField16, _, V>(
            cells.carry(move || Basis::new(&modulus, &wiedemann_tower())),
            P3Timings {
                mul: VARIABLE,
                square: VARIABLE,
                invert: VARIABLE,
                sqrt: VARIABLE,
            },
        ),
        GF64 => p3::<Poly64, _, V>(cells.carry(same), constant),
        GF128 => p3::<Ghash128, _, V>(
            cells.carry(same),
            P3Timings {
                invert: VARIABLE,
                ..constant
            },
        ),
        _ => cells.none("Plonky3 declares no such field"),
    }
}

fn p3<T: p3_field::Field, C: Carrier<T>, V: Visitor<Op, Field>>(
    cells: Carried<V, impl Fn() -> C + Copy>,
    timings: P3Timings,
) -> Crate<V> {
    cells
        .binary(Kind::Add, CONSTANT, "+", |&(a, b): &(T, T)| a + b, element)
        .binary(
            Kind::Mul,
            timings.mul,
            "*",
            |&(a, b): &(T, T)| a * b,
            element,
        )
        .unary(
            Kind::Square,
            timings.square,
            "square",
            |a: &T| a.square(),
            element,
        )
        .unary(
            Kind::Invert,
            timings.invert,
            "try_inverse",
            |a: &T| a.try_inverse(),
            maybe,
        )
        .unary(
            Kind::Sqrt,
            timings.sqrt,
            "try_sqrt",
            |a: &T| a.try_sqrt(),
            maybe,
        )
        .cells()
}

// ---------------------------------------------------------------------------------------------
// longfellow-zk

/// A longfellow-zk field object, which converts its own elements.
#[derive(Clone, Copy)]
struct Longfellow<'a, F>(&'a F);

impl<F: SupportsU128Conversions + SerializableField> Carrier<F::E> for Longfellow<'_, F> {
    fn to_crate(&self, limbs: &[u64]) -> F::E {
        self.0.u128_to_element(word(limbs))
    }

    fn to_row(&self, element: &F::E) -> Limbs {
        let mut bytes = [0; 16];
        self.0.to_bytes_into(element, &mut bytes);
        limbs(u128::from_le_bytes(bytes))
    }
}

/// The operations of a longfellow-zk `Gf2_128Field`, whose product and inverse have the timings
/// `mul` and `invert`.
fn longfellow<F, V>(field: &F, cells: Crate<V>, mul: Timing, invert: Timing) -> Crate<V>
where
    F: AlgebraicField + SupportsU128Conversions + SerializableField,
    V: Visitor<Op, Field>,
{
    cells
        .carry(|| Longfellow(field))
        .binary(
            Kind::Add,
            CONSTANT,
            "addf",
            |(a, b)| field.addf(a, b),
            element,
        )
        .binary(Kind::Mul, mul, "mulf", |(a, b)| field.mulf(a, b), element)
        .absent(Kind::Square, NO_SQUARE)
        .unary(Kind::Invert, invert, "invert", |a| field.invert(a), element)
        .absent(Kind::Sqrt, NO_SQRT)
        .cells()
}

/// longfellow-zk's prover-side `Gf2_128Field`: carry-less products, and an inverse that branches
/// on the bits of the value.
fn runtime_algebra<V: Visitor<Op, Field>>(cells: Crate<V>) -> Crate<V> {
    match cells.key().field.as_str() {
        GF128 => longfellow(
            &runtime_algebra::gf2_128::Gf2_128Field::new(),
            cells,
            CONSTANT,
            VARIABLE,
        ),
        _ => cells.none(GF128_ONLY),
    }
}

/// longfellow-zk's compile-side `Gf2_128Field`: carry-less products, and the inverse
/// `a^(2^128 - 2)` along a fixed sequence of them.
fn compile_algebra<V: Visitor<Op, Field>>(cells: Crate<V>) -> Crate<V> {
    match cells.key().field.as_str() {
        GF128 => longfellow(
            &compile_algebra::gf2_128::Gf2_128Field::new(),
            cells,
            CONSTANT,
            CONSTANT,
        ),
        _ => cells.none(GF128_ONLY),
    }
}

/// longfellow-zk's reference `Gf2_128Field`: a product that branches on each bit of an operand,
/// and an inverse along it.
fn core_algebra<V: Visitor<Op, Field>>(cells: Crate<V>) -> Crate<V> {
    match cells.key().field.as_str() {
        GF128 => longfellow(
            &core_algebra::gf2_128::Gf2_128Field::new(),
            cells,
            VARIABLE,
            VARIABLE,
        ),
        _ => cells.none(GF128_ONLY),
    }
}

// ---------------------------------------------------------------------------------------------
// polyval

/// POLYVAL's representation of GF(2¹²⁸) (RFC 8452): the element `a` is the block of the bit
/// reversal of `a·x⁻¹`, the map under which POLYVAL's product is the GCM field's.
struct Polyval {
    modulus: Modulus,
    x: Limbs,
    x_inverse: Limbs,
}

impl Polyval {
    fn new(modulus: Modulus) -> Self {
        let x = modulus.power_of_x(1);
        Self {
            x_inverse: modulus.inverse(&x),
            modulus,
            x,
        }
    }
}

impl Carrier<FieldElement> for Polyval {
    fn to_crate(&self, limbs: &[u64]) -> FieldElement {
        let shifted = self.modulus.mul(limbs, &self.x_inverse);
        FieldElement::from(word::<u128>(&shifted).reverse_bits())
    }

    fn to_row(&self, element: &FieldElement) -> Limbs {
        let reversed = limbs(u128::from(*element).reverse_bits());
        self.modulus.mul(&reversed, &self.x)
    }
}

/// `polyval`'s `hazmat::FieldElement`.
fn polyval<V: Visitor<Op, Field>>(cells: Crate<V>) -> Crate<V> {
    let modulus = spec(cells.key()).1.modulus;
    match cells.key().field.as_str() {
        GF128 => cells
            .carry(move || Polyval::new(modulus))
            .binary(
                Kind::Add,
                CONSTANT,
                "+",
                |&(a, b): &(FieldElement, _)| a + b,
                element,
            )
            .binary(
                Kind::Mul,
                CONSTANT,
                "*",
                |&(a, b): &(FieldElement, _)| a * b,
                element,
            )
            .absent(Kind::Square, NO_SQUARE)
            .absent(Kind::Invert, NO_INVERSE)
            .absent(Kind::Sqrt, NO_SQRT)
            .cells(),
        _ => cells.none(GF128_ONLY),
    }
}
