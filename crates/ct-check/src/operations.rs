//! The checked operations on secret operands, over the fields of the `compare` benches of
//! `prime-field` and `binary-field`. Each runs once through [`run_batch`], which keeps a symbol of
//! its own for every operation and passes the operands and the result through `black_box`.

use std::process::ExitCode;

use bench_compare::run_batch;
use binary_field::{Aes, B233, B571, Gcm, Gf2, Poly16, Poly64, Poly256, Polynomial, Pow, Square};
use prime_field::biguint::Uint;
use prime_field::standard::{
    BabyBear, Bls12381Base, Bn254Scalar, Goldilocks, Mersenne31, P256Base, P256Scalar, P521Base,
    Secp256k1Base, Tom256Base,
};
use prime_field::{Fp, Modulus, Secret};

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

/// The checks of one field: the number of operations that ran.
type Field = fn() -> Result<usize, Unhandled>;

/// The fields of the `compare` bench of `prime-field`.
const PRIME_FIELDS: [Field; 10] = [
    prime::<Mersenne31>,
    prime::<BabyBear>,
    prime::<Goldilocks>,
    prime::<Bn254Scalar>,
    prime::<Secp256k1Base>,
    prime::<P256Base>,
    prime::<P256Scalar>,
    prime::<Tom256Base>,
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

/// Runs every operation of [`Secret`] over the field of `M`: the operators between two secrets
/// and between a secret and a public element on either side, with their assigning forms, `-`,
/// `square`, `double`, `pow` by a public exponent, `invert` and `sqrt`. Only the secrets are
/// marked undefined.
fn prime<M: Modulus>() -> Result<usize, Unhandled> {
    let (a, b) = (Fp::<M>::from_u64(A), Fp::<M>::from_u64(B));
    let mut pair = [(Secret::new(a), Secret::new(b))];
    let mut single = [Secret::new(a)];
    classify(&mut pair)?;
    classify(&mut single)?;
    let public = b;
    let exponent = Uint::<M::Width>::from_u64(B);
    let assigned = |apply: fn(&mut Secret<M>, Secret<M>, Fp<M>)| {
        move |&(a, b): &(Secret<M>, Secret<M>)| {
            let mut target = a;
            apply(&mut target, b, public);
            target
        }
    };
    Ok(run_batch(&pair, |&(a, b)| a + b)
        + run_batch(&pair, |&(a, b)| a - b)
        + run_batch(&pair, |&(a, b)| a * b)
        + run_batch(&single, |&a| a + public)
        + run_batch(&single, |&a| a - public)
        + run_batch(&single, |&a| a * public)
        + run_batch(&single, |&a| public + a)
        + run_batch(&single, |&a| public - a)
        + run_batch(&single, |&a| public * a)
        + run_batch(&pair, assigned(|target, b, _| *target += b))
        + run_batch(&pair, assigned(|target, b, _| *target -= b))
        + run_batch(&pair, assigned(|target, b, _| *target *= b))
        + run_batch(&pair, assigned(|target, _, public| *target += public))
        + run_batch(&pair, assigned(|target, _, public| *target -= public))
        + run_batch(&pair, assigned(|target, _, public| *target *= public))
        + run_batch(&single, |&a| -a)
        + run_batch(&single, |a| a.square())
        + run_batch(&single, |a| a.double())
        + run_batch(&single, |a| a.pow(exponent))
        + run_batch(&single, |a| a.invert())
        + run_batch(&single, |a| a.sqrt()))
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
