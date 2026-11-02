//! The checked operations on secret operands, over the fields of the `compare` benches of
//! `prime-field` and `binary-field`. Each runs once through [`run_batch`], which keeps a symbol of
//! its own for every operation and passes the operands and the result through `black_box`.

use std::process::ExitCode;

use bench_compare::run_batch;
use binary_field::{Aes, B233, B571, Gcm, Gf2, Poly16, Poly64, Poly256, Polynomial, Pow, Square};
use prime_field::biguint::Uint;
use prime_field::biguint::typenum::{U31, U64, U254, U256, U381, U521};
use prime_field::{Fp, Modulus};

use crate::request::{Unhandled, classify};

/// Runs every checked operation and prints how many ran.
pub(crate) fn main() -> ExitCode {
    let operations = PRIME_FIELDS
        .iter()
        .chain(&BINARY_FIELDS)
        .try_fold(0, |operations, field| {
            Ok::<_, Unhandled>(operations + field()?)
        });
    match operations {
        Ok(operations) => {
            println!("{operations} operations on secret operands");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("ct-check: {error}");
            ExitCode::FAILURE
        }
    }
}

// The operands, from the hexadecimal digits of π. Memcheck tracks which bits are secret, whatever
// their values.
const A: u64 = 0x243f_6a88_85a3_08d3;
const B: u64 = 0x1319_8a2e_0370_7344;

/// Declares one `Modulus` marker per field.
macro_rules! moduli {
    ($($name:ident: $width:ty = [$($limb:expr),+ $(,)?];)*) => {
        $(
            #[derive(Clone, Copy)]
            struct $name;

            impl Modulus for $name {
                type Width = $width;
                const MODULUS: Uint<$width> = Uint::from_limbs([$($limb),+]);
            }
        )*
    };
}

moduli! {
    Mersenne31: U31 = [0x7fff_ffff];
    BabyBear: U31 = [0x7800_0001];
    Goldilocks: U64 = [0xffff_ffff_0000_0001];
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
        u64::MAX,
        u64::MAX,
        u64::MAX,
        u64::MAX,
        u64::MAX,
        u64::MAX,
        u64::MAX,
        u64::MAX,
        0x1ff,
    ];
}

/// The checks of one field: the number of operations that ran.
type Field = fn() -> Result<usize, Unhandled>;

/// The fields of the `compare` bench of `prime-field`.
const PRIME_FIELDS: [Field; 9] = [
    prime::<Mersenne31>,
    prime::<BabyBear>,
    prime::<Goldilocks>,
    prime::<Bn254Scalar>,
    prime::<Secp256k1Base>,
    prime::<P256Base>,
    prime::<P256Scalar>,
    prime::<Bls12381Base>,
    prime::<P521Base>,
];

/// The fields of the `compare` bench of `binary-field`.
const BINARY_FIELDS: [Field; 7] = [
    binary::<Aes>,
    binary::<Poly16>,
    binary::<Poly64>,
    binary::<Gcm>,
    binary::<B233>,
    binary::<Poly256>,
    binary::<B571>,
];

/// Runs `add`, `sub`, `neg`, `double`, `mul`, `square` and `invert` over the field of `M`.
fn prime<M: Modulus>() -> Result<usize, Unhandled> {
    let (a, b) = (Fp::<M>::from_u64(A), Fp::<M>::from_u64(B));
    let mut pair = [(a, b)];
    let mut single = [a];
    classify(&mut pair)?;
    classify(&mut single)?;
    Ok(run_batch(&pair, |&(a, b)| a + b)
        + run_batch(&pair, |&(a, b)| a - b)
        + run_batch(&single, |&a| -a)
        + run_batch(&single, |a| a.double())
        + run_batch(&pair, |&(a, b)| a * b)
        + run_batch(&single, |a| a.square())
        + run_batch(&single, |a| a.invert()))
}

/// Runs the sum, the product, `square`, `sqrt`, `trace`, `times_x`, `over_x` and, with hardware
/// carry-less products, `invert` over the field of `P`. Without them the plan of every field
/// inverts by the Euclidean algorithm, whose time depends on the value.
fn binary<P: Polynomial>() -> Result<usize, Unhandled> {
    let (a, b) = (Gf2::<P>::X.pow(A), Gf2::<P>::X.pow(B));
    let mut pair = [(a, b)];
    let mut single = [a];
    classify(&mut pair)?;
    classify(&mut single)?;
    let operations = run_batch(&pair, |&(a, b)| a + b)
        + run_batch(&pair, |&(a, b)| a * b)
        + run_batch(&single, |&a| a.square())
        + run_batch(&single, |a| a.sqrt())
        + run_batch(&single, Gf2::trace)
        + run_batch(&single, |a| a.times_x())
        + run_batch(&single, |a| a.over_x());
    Ok(if clmul::HARDWARE {
        operations + run_batch(&single, |a| a.invert())
    } else {
        operations
    })
}
