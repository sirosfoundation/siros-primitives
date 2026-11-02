# prime-field

[![CI](https://github.com/sirosfoundation/siros-primitives/actions/workflows/ci.yml/badge.svg)](https://github.com/sirosfoundation/siros-primitives/actions/workflows/ci.yml)
![Apache-2.0/MIT licensed](https://img.shields.io/badge/license-Apache--2.0%2FMIT-blue.svg)
![Rust 1.93+](https://img.shields.io/badge/rustc-1.93+-blue.svg)

Prime fields declared by their modulus, in pure Rust: `no_std`, without `unsafe`, allocation or
panics. One `impl Modulus` per field; every other constant and every strategy choice derives from
the modulus at compile time, so each operation compiles to straight-line code with the modulus
limbs as immediates, for moduli from 31 to thousands of bits.

## Example

```rust
use prime_field::biguint::Uint;
use prime_field::standard::secp256k1::Base as Fq;
use prime_field::Secret;

// The fastest path; the time may depend on the values.
let x = Fq::from_u64(7);
let y = x.invert().ok_or("zero")?;
assert_eq!(x * y, Fq::ONE);
let four = Fq::from_u64(4);
assert_eq!(four.sqrt().map(|root| root.square()), Some(four));
assert_eq!(x.sqrt(), None);

// Constant time.
let k = Secret::new(x);
let d = k * Secret::new(y) + k.square();
assert_eq!(d.reveal(), Fq::from_u64(50));
assert_eq!(k.invert().map(Secret::reveal), Some(y));

// Constant contexts.
const SEVEN: Fq = Fq::from_u64(7);
const FORTY_NINE: Fq = SEVEN.const_mul(SEVEN);
assert_eq!(FORTY_NINE, Fq::from_u64(49));

assert_eq!(Fq::from_uint(Fq::MODULUS), Err(prime_field::NotCanonical));
assert_eq!((x - x).to_uint(), Uint::ZERO);
# Ok::<(), &str>(())
```

## Standard fields

`prime_field::standard` declares the moduli `Mersenne31`, `BabyBear`, `Goldilocks`,
`Bn254Scalar`, `Secp256k1Base`, `P256Base`, `P256Scalar`, `Tom256Base`, `Bls12381Base` and
`P521Base`. `Tom256Scalar` is `P256Base`: the group order of Tom-256 is the P-256 base prime.
Its submodules name the elements by curve: `tom256::{Base, Scalar}`, `p256::{Base, Scalar}`,
`secp256k1::Base`, `bn254::Scalar`, `bls12_381::Base` and `p521::Base`. The one-limb fields
are `Fp<Mersenne31>`, `Fp<BabyBear>` and `Fp<Goldilocks>`.

## Declaring a field

```rust
use prime_field::biguint::typenum::U255;
use prime_field::biguint::Uint;
use prime_field::{Fp, Modulus};

/// `2^255 - 19`, the base field of Curve25519.
#[derive(Clone, Copy)]
struct Curve25519Base;

impl Modulus for Curve25519Base {
    type Width = U255;
    const MODULUS: Uint<U255> = Uint::from_limbs([
        0xffff_ffff_ffff_ffed,
        0xffff_ffff_ffff_ffff,
        0xffff_ffff_ffff_ffff,
        0x7fff_ffff_ffff_ffff,
    ]);
}

let two = Fp::<Curve25519Base>::from_u64(2);
assert_eq!(two.invert().map(|half| half.double()), Some(Fp::ONE));
```

`Width` is the `typenum` bit width of the container; the modulus fits it. `MODULUS` is the
little-endian limbs of an odd prime. `Fp<M>` holds `ceil(Width / 64)` limbs and offers `add`,
`sub`, `neg`, `double`, `mul`, `square`, `invert`, `pow`, `sqrt` and the operator traits, the
constant-context twins `const_add`, `const_sub`, `const_neg`, `const_double`, `const_mul` and
`const_square`, the conversions `from_u64`, `from_uint`, `to_uint`, `from_le_bytes`,
`from_be_bytes`, `to_le_bytes` and `to_be_bytes`, and the constants `ZERO`, `ONE`, `MODULUS` and
`BITS`. `from_uint` and the byte decoders refuse a value at or above the modulus with
`NotCanonical`.

`Secret<M>` holds an `Fp<M>` in the same memory; `Secret::new` and `Secret::reveal` cost nothing.
Its operators combine two secrets, or a secret and a public `Fp` on either side, into a secret;
it also offers `square`, `double`, `pow` by a public exponent, `invert` and `sqrt`.

## Timing

`Fp`'s operations take the path with the fewest cycles, and their time may depend on the values:
`invert` and `sqrt` branch throughout, and a Montgomery product's final subtraction branches on
the top limb when the modulus is below `R / 4`, where the subtraction is rare.

`Secret`'s operations take one path for every operand value: no branch and no memory address
depends on a secret. `invert` and `sqrt` disclose whether their result exists, through the
`Option`; an addition chain's steps and `pow`'s windows depend on the exponent alone. Each
selection by a secret condition is a conditional move by `core::hint::select_unpredictable`, or
a mask that passes through `core::hint::black_box`. CI runs every `Secret` operation on secret
operands of x86-64 builds under Valgrind's memcheck with
[`ct-check`](https://github.com/sirosfoundation/siros-primitives/tree/main/crates/ct-check),
which fails on a branch or a memory address that depends on them.

## Algorithms

The modulus selects the representation and the algorithms at compile time; the limb loops unroll
in the source to the container width, and carry chains run on the `adc` and `sbb` intrinsics on
x86-64. The `const_*` twins run the loops of the constant-evaluable kernels.

| Operation | Moduli | Algorithm |
|---|---|---|
| representation | one limb, `2^k - 1` (Mersenne-31) | Residues in `[1, p]`, with `0` held as `p`: a sum or a folded product above `p` is one at or above `2^k`, which the signed overflow of a 32-bit or 64-bit addition flags at `k = 31` or `63` |
| representation | `2^k - c` of several limbs, `c < 2^64`, at most three folds to land below `2p` (secp256k1, P-521) | Plain residues |
| representation | every other modulus | Montgomery residues `x R mod p`, `R = 2^(64 L)` |
| add, double | several limbs | `a + b - p`, then `a + b` back by conditional moves on the borrow; for a modulus whose limbs are mostly all ones (P-521), `p` back under the mask of the borrow |
| sub, neg | several limbs | `a - b`, then `p` added under the mask of the borrow; for a modulus of at most four arbitrary limbs with a spare top bit (BN254), `a - b + p` chosen by conditional moves on the carry |
| add, sub, neg, double | one limb | `a + b - p` or `a - b + p`, chosen by a conditional move on the borrow |
| mul | one limb, `2^k - 1` | The product folded once, `low + high`, and one conditional subtraction |
| mul | one limb, Montgomery | One Montgomery step `(x + m p) / 2^64`; when the product fits the limb and the modulus is prime the step lands below `p`; for `p = 2^64 - 2^32 + 1`, whose inverse modulo `2^64` is `1 + 2^32`, the multiplier is a shift and an addition, as in winter-math; BabyBear's step chains three multiplications, the product, `m` and `m p`, where Goldilocks' chains one, so the smaller field has the longer latency |
| mul | plain residues of several limbs | Schoolbook product, then pseudo-Mersenne folds `low + high · c` and one conditional subtraction; secp256k1's folds take 5 multiplications by `c`, P-521's none, as `c = 1` |
| mul | Montgomery, `-p^-1 = 1 mod 2^64` (P-256) | Separated operand scanning (Koç, Acar and Kaliski 1996): the product, then reduction rows whose multipliers are limbs of the product; P-256's limbs `2^64 - 1`, `2^32 - 1` and `0` fold into additions, which leaves its reduction 4 multiplications |
| mul | other Montgomery moduli | Coarsely integrated operand scanning; the accumulator drops its second carry limb when the modulus leaves the top bit clear; the reduction takes `L^2 + L` multiplications, so a BN254 product takes 36 where secp256k1's takes 21 and P-256's 20 |
| square | several limbs | The product with each cross term once and doubled, then the reduction; a BLS12-381 square takes 63 multiplications, 42 of them in the reduction, where P-521's takes 45 and folds without any |
| mul, square | more than four limbs | Each row of products adds the low halves of its products in one carry chain and the high halves, one limb up, in a second || invert, `Fp` | one limb | The extended Euclidean algorithm with one hardware division per step, then for a Montgomery residue a product by `R^3` |
| invert, `Fp` | wider moduli | Bernstein–Yang safegcd (*Fast constant-time gcd computation and modular inversion*, 2019) in batches of 62 division steps on the low limbs, as libsecp256k1's `divsteps_62_var`: a run of trailing zeros of `g` halves it at once, and the steps that add `f` to `g` combine into one multiple of `f` that clears up to 6 low bits; the loop stops when `g` reaches zero |
| invert, `Secret` | one limb | Fermat: `x^(p - 2)` by an addition chain |
| invert, `Secret` | wider moduli | The half-delta division steps of libsecp256k1's `divsteps_59`, with masked selections, for the proven `(45907 k + 26313) / 19929` steps of a `k`-bit modulus, in batches of two halves of 30 steps; a row `(a, b)` of a half's matrix is held as the one word `a + 2^32 b` |
| sqrt | `p = 3 mod 4` | `x^((p + 1) / 4)` by an addition chain, checked by squaring |
| sqrt, `Fp` | `p = 1 mod 4` | Tonelli–Shanks from `x^((t - 1) / 2)` by an addition chain; the two-adic root `z^t` of the least non-residue `z`, found by the Jacobi symbol at compile time, and its first 64 repeated squares form a table, so each round costs the squarings that find the order of the residue and two multiplications; BN254, with `2^28` dividing `p - 1`, adds up to 28 rounds to an exponentiation as long as the one of a `p = 3 mod 4` field |
| sqrt, `Secret` | `p = 1 mod 4` | The constant-time Tonelli–Shanks of the `ff` crate: every round runs every squaring a round may need and keeps the results by conditional moves; on BN254 that is 28 rounds of 406 squarings in all |
| pow | all | Sliding windows over a `biguint::Uint` exponent, with a table of up to 32 odd powers whose width follows the exponent's bit length |

An addition chain is computed at compile time from its exponent, after the hybrid of
McLoughlin's `addchain`: the runs of ones come from an addition sequence over their lengths, the
other bits from sliding windows of odd powers, and the shortest chain over every window width is
kept. secp256k1's `(p + 1) / 4` takes libsecp256k1's 253 squarings and 13 multiplications.

## Benchmarks

[BENCHMARKS.md](https://github.com/sirosfoundation/siros-primitives/blob/main/crates/prime-field/BENCHMARKS.md)
compares `prime-field` with ark-ff, crypto-bigint, k256/p256, halo2curves, Plonky3, winter-math
and longfellow-zk's `runtime-algebra` and `compile-algebra` on add, sub, mul, square, invert and
sqrt over Mersenne-31, BabyBear, Goldilocks, the BN254 scalar field, the secp256k1, P-256,
Tom-256, BLS12-381 and P-521 base fields and the P-256 scalar field. Each operation has a
variable-time row,
`Fp` against the functions whose time may depend on the values, and a constant-time row, `Secret`
against the functions whose time depends on no value. The figures are cycles per
operation that the CPU's performance counters count. The unpublished
[`prime-field-bench-report`](https://github.com/sirosfoundation/siros-primitives/tree/main/crates/prime-field/bench-report)
regenerates it:

```sh
cargo run -p prime-field-bench-report -- bench    # the CPU's counts of the native build
cargo run -p prime-field-bench-report -- report   # BENCHMARKS.md
```

## Features

None. `biguint`'s `not-precomputed` feature has no effect on this crate.

## Minimum supported Rust version

Rust 1.93 or newer.

## License

The crate is licensed under either of

- [Apache License, Version 2.0](https://www.apache.org/licenses/LICENSE-2.0)
- [MIT license](https://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in
the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
