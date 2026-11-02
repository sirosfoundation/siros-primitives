# prime-field

[![CI](https://github.com/sirosfoundation/siros-primitives/actions/workflows/ci.yml/badge.svg)](https://github.com/sirosfoundation/siros-primitives/actions/workflows/ci.yml)
![Apache-2.0/MIT licensed](https://img.shields.io/badge/license-Apache--2.0%2FMIT-blue.svg)
![Rust 1.93+](https://img.shields.io/badge/rustc-1.93+-blue.svg)

Prime fields declared by their modulus: `no_std`, without `unsafe`, allocation or panics, for
moduli from 31 to thousands of bits. Rust 1.93 or newer.

One `impl Modulus` declares a field. Every other constant and the choice of representation derive
from the modulus at compile time.

## Example

```rust
use prime_field::biguint::Uint;
use prime_field::standard::secp256k1::Base as Fq;
use prime_field::ConstantTime;

// The fastest path; the time may depend on the values.
let x = Fq::from_u64(7);
let y = x.invert().ok_or("zero")?;
assert_eq!(x * y, Fq::ONE);
let four = Fq::from_u64(4);
assert_eq!(four.sqrt().map(|root| root.square()), Some(four));
assert_eq!(x.sqrt(), None);

// Constant time.
let k = ConstantTime::new(x);
let d = k * ConstantTime::new(y) + k.square();
assert_eq!(d.reveal(), Fq::from_u64(50));
assert_eq!(k.invert().map(ConstantTime::reveal), Some(y));

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
`P521Base`. `Tom256Scalar` is `P256Base`. Its submodules name the elements by curve:
`tom256::{Base, Scalar}`, `p256::{Base, Scalar}`, `secp256k1::Base`, `bn254::Scalar`,
`bls12_381::Base` and `p521::Base`. The one-limb fields are `Fp<Mersenne31>`, `Fp<BabyBear>` and
`Fp<Goldilocks>`.

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
little-endian limbs of an odd prime.

`Fp<M>` offers `add`, `sub`, `neg`, `double`, `mul`, `square`, `invert`, `pow`, `sqrt` and the
operator traits; the const twins `const_add`, `const_sub`, `const_neg`, `const_double`,
`const_mul` and `const_square`; the conversions `from_u64`, `from_uint`, `to_uint`,
`from_le_bytes`, `from_be_bytes`, `to_le_bytes` and `to_be_bytes`; and the constants `ZERO`, `ONE`,
`MODULUS` and `BITS`. `from_uint` and the byte decoders return `NotCanonical` for a value at or
above the modulus.

`ConstantTime<Fp<M>>`, from the [`constant-time`](https://github.com/sirosfoundation/siros-primitives/tree/main/crates/constant-time)
crate and re-exported here, holds an `Fp<M>` in the same memory; `ConstantTime::new` and
`ConstantTime::reveal` cost nothing. Its operators combine two wrapped elements, or a wrapped and
a public `Fp` on either side, into a wrapped element. It offers `square`, `double`, `pow` by a
public exponent, `invert` and `sqrt`.

## Timing

`Fp` takes the fastest path; its time may depend on the values.

`ConstantTime` takes one path for every wrapped value: no branch and no memory address
depends on it. `invert` and `sqrt` disclose through the `Option` whether the result exists. `pow`
takes a time that depends on the exponent. CI runs every `ConstantTime` operation under
Valgrind's memcheck with
[`ct-check`](https://github.com/sirosfoundation/siros-primitives/tree/main/crates/ct-check), which
fails on a branch or a memory address that depends on a wrapped value.

## Representation

The modulus selects the representation at compile time:

| Modulus | Residues |
|---|---|
| `2^k - 1`, one limb (Mersenne-31) | In `[1, p]`, with `0` held as `p` |
| `2^k - c`, several limbs, `c < 2^64`, at most three folds to land below `2p` (secp256k1, P-521) | Plain |
| Every other | Montgomery, `x R mod p` with `R = 2^(64 L)` |

## Features

None. `biguint`'s `not-precomputed` feature has no effect on this crate.

[BENCHMARKS.md](https://github.com/sirosfoundation/siros-primitives/blob/main/crates/prime-field/BENCHMARKS.md)
compares `prime-field` with other prime-field crates.

## License

Licensed under either of [Apache License, Version 2.0](https://www.apache.org/licenses/LICENSE-2.0)
or [MIT license](https://opensource.org/licenses/MIT) at your option.
