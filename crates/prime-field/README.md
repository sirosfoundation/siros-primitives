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
use prime_field::biguint::{U256, Uint};
use prime_field::{Fp, Modulus};

#[derive(Clone, Copy)]
struct Secp256k1Base;

impl Modulus for Secp256k1Base {
    type Width = prime_field::biguint::typenum::U256;
    const MODULUS: U256 = U256::from_limbs([
        0xffff_fffe_ffff_fc2f,
        0xffff_ffff_ffff_ffff,
        0xffff_ffff_ffff_ffff,
        0xffff_ffff_ffff_ffff,
    ]);
}

type Fq = Fp<Secp256k1Base>;

let x = Fq::from_u64(7);
let y = x.invert().ok_or("zero")?;
assert_eq!(x * y, Fq::ONE);
let four = Fq::from_u64(4);
assert_eq!(four.sqrt().map(|root| root.square()), Some(four));
assert_eq!(x.sqrt(), None);
assert_eq!(Fq::from_uint(Fq::MODULUS), Err(prime_field::NotCanonical));
assert_eq!((x - x).to_uint(), Uint::ZERO);
# Ok::<(), &str>(())
```

## Declaring a field

`Width` is the `typenum` bit width of the container; the modulus fits it. `MODULUS` is the
little-endian limbs of an odd prime. `Fp<M>` holds `ceil(Width / 64)` limbs and offers
`add`, `sub`, `neg`, `double`, `mul`, `square`, `invert`, `invert_vartime`, `pow`, `sqrt`, the
conversions `from_u64`, `from_uint`, `to_uint`, `from_le_bytes`, `from_be_bytes`, `to_le_bytes`
and `to_be_bytes`, the constants `ZERO`, `ONE`, `MODULUS` and `BITS`, and the operator traits.
`from_uint` and the byte decoders refuse a value at or above the modulus with `NotCanonical`.

## Algorithms

The modulus selects the algorithm at compile time; the limb loops unroll to the container width.

| Operation | Moduli | Algorithm |
|---|---|---|
| mul | `2^k - c` with `c < 2^64` and at most three folds to land below `2p` (Mersenne-31, Goldilocks, secp256k1, P-521) | Schoolbook product, then pseudo-Mersenne folds `low + high · c` and one conditional subtraction; elements are plain residues |
| mul | every other modulus | Montgomery multiplication by coarsely integrated operand scanning (Koç, Acar and Kaliski 1996); the accumulator drops its second carry limb when the modulus leaves the top bit clear, as in gnark-crypto's and ark-ff's no-carry variant; a Montgomery-friendly modulus (`-p^-1 = 1 mod 2^64`, P-256) loses its per-row multiply to constant folding |
| square | all | The product with each cross term once and doubled, then the reduction of the modulus |
| add, sub, neg | all | `a + b - p` or `a - b`, then `p` added back under the mask of the borrow; `-a` is `0 - a` |
| invert | up to 64 bits | Fermat: `x^(p - 2)` by an addition chain |
| invert | wider moduli | Bernstein–Yang safegcd (*Fast constant-time gcd computation and modular inversion*, 2019) in the half-delta formulation of libsecp256k1: 62 division steps per batch on the low limbs, a 2×2 matrix of 62-bit factors applied to the values and to the Bézout coefficients modulo `p`, the proven `(45907 k + 26313) / 19929` steps for a `k`-bit modulus, in constant time |
| invert_vartime | wider moduli | The same batches; the even steps of a batch skip at once by the trailing zeros of `g`, and the loop stops when `g` reaches zero |
| sqrt | `p = 3 mod 4` | `x^((p + 1) / 4)` by an addition chain, checked by squaring |
| sqrt | `p = 1 mod 4` | Tonelli–Shanks from `x^((t - 1) / 2)` by an addition chain; the two-adic root `z^t` of the least non-residue `z`, found by the Jacobi symbol at compile time, and its first 64 repeated squares form a table, so each round costs the squarings that find the order of the residue and two multiplications |
| pow | all | Sliding windows over a `biguint::Uint` exponent, with a table of up to 32 odd powers whose width follows the exponent's bit length |

An addition chain is computed at compile time from its exponent, after the hybrid of
McLoughlin's `addchain`: the runs of ones come from an addition sequence over their lengths, the
other bits from sliding windows of odd powers, and the shortest chain over every window width is
kept. secp256k1's `(p + 1) / 4` takes libsecp256k1's 253 squarings and 13 multiplications.

Timing: `add`, `sub`, `neg`, `mul` and `square` take one path for every operand value, and
`invert` for every nonzero value; an addition chain's steps depend on the exponent alone. Each
selection by a secret condition is a mask that passes through `core::hint::black_box`, which
keeps the compiler from turning it into a branch. CI runs these operations on secret operands
under Valgrind's memcheck with
[`ct-check`](https://github.com/sirosfoundation/siros-primitives/tree/main/crates/ct-check),
which fails on a branch or a memory address that depends on them.
`invert_vartime`, `pow` and `sqrt` take time that depends on the values.

## Benchmarks

[BENCHMARKS.md](https://github.com/sirosfoundation/siros-primitives/blob/main/crates/prime-field/BENCHMARKS.md)
compares `prime-field` with ark-ff, crypto-bigint, k256/p256, halo2curves, Plonky3, winter-math
and longfellow-zk's `runtime-algebra` and `compile-algebra` on add, sub, mul, square, invert,
invert_vartime and sqrt over Mersenne-31, BabyBear, Goldilocks, the BN254 scalar field, the
secp256k1, P-256, BLS12-381 and P-521 base fields and the P-256 scalar field, in instructions per
operation that Callgrind counts. The unpublished
[`prime-field-bench-report`](https://github.com/sirosfoundation/siros-primitives/tree/main/crates/prime-field/bench-report)
regenerates it:

```sh
cargo run -p prime-field-bench-report -- bench    # Callgrind counts of the native build
cargo run -p prime-field-bench-report -- report   # BENCHMARKS.md and benchmarks/*.svg
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
