//! The checked operations of `ConstantTime`: over `Uint` and its residues at 64, 256, 521 and 4096
//! bits under an odd and an even modulus, and over the fields of the `compare` benches of
//! `prime-field` and `binary-field`. Each runs once through [`run_batch`], which keeps a symbol of
//! its own for every operation and hides the address of the operands and of the result from the
//! compiler. Only the wrapped operands are marked undefined.

use core::ops::Add;
use std::process::ExitCode;

use bench_compare::run_batch;
use biguint::generic_array::GenericArray;
use biguint::generic_array::sequence::GenericSequence;
use biguint::typenum::{Sum, U64, U256, U521, U4096};
use biguint::{Bits, ConstantTime, NonZeroUint, Uint};
use binary_field::{Aes, B233, B571, Gcm, Gf2, Poly16, Poly64, Poly256, Polynomial, Pow};
use prime_field::standard::{
    BabyBear, Bls12381Base, Bn254Scalar, Goldilocks, Mersenne31, P256Base, P256Scalar, P521Base,
    Secp256k1Base, Tom256Base,
};
use prime_field::{Fp, Modulus};

use crate::request::{Unhandled, classify};

/// Runs every checked operation and prints how many ran.
pub(crate) fn main() -> ExitCode {
    let operations = INTEGERS
        .iter()
        .chain(&PRIME_FIELDS)
        .chain(&BINARY_FIELDS)
        .try_fold(0, |operations, check| {
            Ok::<_, Unhandled>(operations + check()?)
        });
    match operations {
        Ok(operations) => {
            println!("{operations} operations on undefined operands");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("ct-check: {error}");
            ExitCode::FAILURE
        }
    }
}

// The operands, from the hexadecimal digits of π. Memcheck tracks which bits are undefined,
// whatever their values.
const A: u64 = 0x243f_6a88_85a3_08d3;
const B: u64 = 0x1319_8a2e_0370_7344;

/// The checks of one type: the number of operations that ran.
type Check = fn() -> Result<usize, Unhandled>;

/// The widths of `Uint`.
const INTEGERS: [Check; 4] = [
    integers::<U64>,
    integers::<U256>,
    integers::<U521>,
    integers::<U4096>,
];

/// The fields of the `compare` bench of `prime-field`.
const PRIME_FIELDS: [Check; 10] = [
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
const BINARY_FIELDS: [Check; 7] = [
    binary::<Aes>,
    binary::<Poly16>,
    binary::<Poly64>,
    binary::<Gcm>,
    binary::<B233>,
    binary::<Poly256>,
    binary::<B571>,
];

/// An `N`-bit value whose limbs rotate `seed`.
fn spread<N: Bits>(seed: u64) -> Uint<N> {
    Uint::from_generic_limbs(GenericArray::generate(|index| {
        seed.rotate_left(index as u32 * 7)
    }))
}

/// Two operands of a binary operation.
type Pair<T> = (ConstantTime<T>, ConstantTime<T>);

/// The exponent width of `pow`.
type Exponent = U256;

/// Runs, for each operator of `ConstantTime<Uint<N>>`, the five forms: between two wrapped values,
/// with a public `Uint` on either side, and the assigning forms of both.
macro_rules! operators {
    ($pair:expr, $public:expr; $($op:tt $assign:tt),*) => {
        0 $(
            + run_batch($pair, |&(a, b)| a $op b)
            + run_batch($pair, |&(a, _)| a $op $public)
            + run_batch($pair, |&(a, _)| $public $op a)
            + run_batch($pair, |&(a, b)| {
                let mut target = a;
                target $assign b;
                target
            })
            + run_batch($pair, |&(a, _)| {
                let mut target = a;
                target $assign $public;
                target
            })
        )*
    };
}

/// Runs every operation of `ConstantTime<Uint<N>>`, and of `ConstantTimeResidue` under an odd and
/// an even modulus.
fn integers<N>() -> Result<usize, Unhandled>
where
    N: Bits + Add<N>,
    Sum<N, N>: Bits,
    Uint<N>: Copy,
{
    let (a, b) = (spread::<N>(A), spread::<N>(B));
    let mut pair = [(ConstantTime::new(a), ConstantTime::new(b))];
    let mut exponent = [ConstantTime::new(spread::<Exponent>(B))];
    classify(&mut pair)?;
    classify(&mut exponent)?;
    let [exponent] = exponent;
    let public = b;
    // `b` has bits set above its lowest two.
    let nonzero = |value| NonZeroUint::new(value).expect("a value with bits set");
    let moduli = [nonzero(b | Uint::<N>::ONE), nonzero(b & !Uint::<N>::ONE)];
    let limb = NonZeroUint::new(Uint::<U64>::from_u64(B)).expect("a nonzero constant");
    let mut count = operators!(&pair, public; + +=, - -=, * *=, & &=, | |=, ^ ^=)
        + run_batch(&pair, |&(a, _)| !a)
        + run_batch(&pair, |&(a, _)| a << 67u32)
        + run_batch(&pair, |&(a, _)| a >> 67u32)
        + run_batch(&pair, |&(a, _)| {
            let mut target = a;
            target <<= 3u32;
            target >>= 130usize;
            target
        })
        + run_batch(&pair, |&(a, _)| a.square())
        + run_batch(&pair, |&(a, _)| a.double())
        + run_batch(&pair, |&(a, _)| a.pow(exponent))
        + run_batch(&pair, |&(a, _)| a % limb)
        + run_batch(&pair, |&(a, _)| a.div_rem(limb));
    for modulus in moduli {
        count += run_batch(&pair, |&(a, b)| a.add_mod(b, modulus))
            + run_batch(&pair, |&(a, b)| a.sub_mod(b, modulus))
            + run_batch(&pair, |&(a, _)| a.neg_mod(modulus))
            + run_batch(&pair, |&(a, b)| a.mul_mod(b, modulus))
            + run_batch(&pair, |&(a, _)| a.div_rem(modulus))
            + run_batch(&pair, |&(a, _)| a / modulus)
            + run_batch(&pair, |&(a, _)| a % modulus)
            + residues(&pair, modulus, exponent);
    }
    Ok(count)
}

/// Runs every operation of `ConstantTimeResidue` modulo `modulus`: the reduction, `get`,
/// `square`, `pow` by a wrapped exponent, `-`, and `+ - *` between two residues and with a public
/// residue on either side, with the assigning forms.
fn residues<N: Bits>(
    pair: &[Pair<Uint<N>>],
    modulus: NonZeroUint<N>,
    exponent: ConstantTime<Uint<Exponent>>,
) -> usize
where
    Uint<N>: Copy,
{
    biguint::Modulus::from(modulus).scope(|ring| {
        let reduced: Vec<_> = pair
            .iter()
            .map(|&(a, b)| (ring.reduce_constant_time(a), ring.reduce_constant_time(b)))
            .collect();
        let public = ring.reduce(Uint::<N>::from_u64(B));
        run_batch(pair, |&(a, _)| ring.reduce_constant_time(a))
            + run_batch(&reduced, |&(x, _)| x.get())
            + run_batch(&reduced, |&(x, _)| x.square())
            + run_batch(&reduced, |&(x, _)| x.pow(exponent))
            + run_batch(&reduced, |&(x, _)| -x)
            + operators!(&reduced, public; + +=, - -=, * *=)
    })
}

/// Runs every operation of `ConstantTime<Fp<M>>`: the operators between two wrapped values and
/// with a public element on either side, with their assigning forms, `-`, `square`, `double`,
/// `pow` by a public exponent, `invert` and `sqrt`.
fn prime<M: Modulus>() -> Result<usize, Unhandled> {
    let (a, b) = (Fp::<M>::from_u64(A), Fp::<M>::from_u64(B));
    let mut pair = [(ConstantTime::new(a), ConstantTime::new(b))];
    classify(&mut pair)?;
    let public = b;
    let exponent = Uint::<M::Width>::from_u64(B);
    Ok(operators!(&pair, public; + +=, - -=, * *=)
        + run_batch(&pair, |&(a, _)| -a)
        + run_batch(&pair, |&(a, _)| a.square())
        + run_batch(&pair, |&(a, _)| a.double())
        + run_batch(&pair, |&(a, _)| a.pow(exponent))
        + run_batch(&pair, |&(a, _)| a.invert())
        + run_batch(&pair, |&(a, _)| a.sqrt()))
}

/// Runs every operation of `ConstantTime<Gf2<P>>`: the operators between two wrapped values and
/// with a public element on either side, with their assigning forms, `-`, `square`, `sqrt` and
/// `invert`.
fn binary<P: Polynomial>() -> Result<usize, Unhandled> {
    let (a, b) = (Gf2::<P>::X.pow(A), Gf2::<P>::X.pow(B));
    let mut pair = [(ConstantTime::new(a), ConstantTime::new(b))];
    classify(&mut pair)?;
    let public = b;
    Ok(operators!(&pair, public; + +=, - -=, * *=)
        + run_batch(&pair, |&(a, _)| -a)
        + run_batch(&pair, |&(a, _)| a.square())
        + run_batch(&pair, |&(a, _)| a.sqrt())
        + run_batch(&pair, |&(a, _)| a.invert()))
}
