# prime-field

[![CI](https://github.com/sirosfoundation/siros-primitives/actions/workflows/ci.yml/badge.svg)](https://github.com/sirosfoundation/siros-primitives/actions/workflows/ci.yml)
![Apache-2.0/MIT licensed](https://img.shields.io/badge/license-Apache--2.0%2FMIT-blue.svg)
![Rust 1.93+](https://img.shields.io/badge/rustc-1.93+-blue.svg)

Prime fields declared by their modulus, in pure Rust: `no_std`, no `unsafe`, no allocation, no
panics.

## Features

- `Fp<M>`: an element of the prime field of `M`. One `impl Modulus`, giving the prime, declares a
  field of 31 to thousands of bits.
- `standard`: Mersenne-31, BabyBear, Goldilocks, BN254, secp256k1, P-256, Tom-256, BLS12-381 and
  P-521, with element aliases by curve such as `standard::p256::{Base, Scalar}`.
- Arithmetic: `+ - *` and their assigning forms, unary `-`, `double`, `square`, `invert` on a
  `NonZero` element, `pow` by a `Uint` exponent, `sqrt`, and `Sum` and `Product` over iterators.
- Integer operands: `Fp + Uint<K>`, `Fp - Uint<K>`, `Fp * Uint<K>` and their assigning forms take an
  integer `k` of any width and value, reduced modulo `p`. The product takes one limb product and
  one quotient step per limb of `k`, from its top nonzero limb; a small `k` costs a fraction of
  a product of two elements. `From<u64>` takes the same path.
- Division: `Fp / NonZeroUint<K>` and `/=` take a nonzero `k` of fewer bits than the width, over a
  modulus that fills its width; a wider `k`, or a modulus short of its width, fails to compile. A `k`
  below `2^64` takes its inverse by the extended Euclidean algorithm on machine words; a wider `k`
  takes the inverse of its element.
- `const_add`, `const_sub`, `const_neg`, `const_double`, `const_mul` and `const_square` compute in
  const contexts.
- Conversions: `from_u64`, `from_uint` and `to_uint`, and little- and big-endian bytes. A value at
  or above the modulus is refused with `NotCanonical`.
- Constant time: `ConstantTime<Fp>` runs every operation with no branch and no memory address that
  depends on the wrapped values.
- [Benchmarks](https://github.com/sirosfoundation/siros-primitives/blob/main/crates/prime-field/BENCHMARKS.md)
  against other prime-field crates.

## Usage

```rust
use prime_field::biguint::{typenum::U64, Uint};
use prime_field::standard::secp256k1::Base as Fq;
use prime_field::NonZero;

let x = Fq::from_u64(7);
assert_eq!(x * Uint::<U64>::from_u64(3), x + x + x);
assert_eq!(x - Uint::<U64>::from_u64(7), Fq::ZERO);
assert_eq!(x / NonZero::new(Uint::<U64>::from_u64(7)).ok_or("zero")?, Fq::ONE);
let y = NonZero::new(x).ok_or("zero")?.invert().get();
assert_eq!(x * y, Fq::ONE);
assert_eq!(Fq::from_u64(4).sqrt().map(|r| r.square()), Some(Fq::from_u64(4)));
assert_eq!([x, y].into_iter().product::<Fq>(), Fq::ONE);

const FORTY_NINE: Fq = Fq::from_u64(7).const_mul(Fq::from_u64(7));
assert_eq!(FORTY_NINE, Fq::from_u64(49));
# Ok::<(), &str>(())
```

## Declaring a field

`MODULUS` holds an odd prime; `Width` is a `typenum` width that fits it.

```rust
use prime_field::biguint::{typenum::U255, Uint};
use prime_field::{Fp, Modulus, NonZero};

/// 2^255 - 19.
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

let two = NonZero::new(Fp::<Curve25519Base>::from_u64(2)).ok_or("zero")?;
assert_eq!(two.invert().get().double(), Fp::ONE);
# Ok::<(), &str>(())
```

## Constant time

`nonzero` discloses whether the value is zero, and `sqrt` whether a root exists. The time of
`pow` depends on its public exponent. A wrapped integer operand takes time that depends on its width
alone. `from_u64` takes the same time for every word, so `ConstantTime::new(Fp::from_u64(k))` wraps a
secret `k`.

```rust
use prime_field::standard::secp256k1::Base as Fq;
use prime_field::ConstantTime;

let k = ConstantTime::new(Fq::from_u64(7));
assert_eq!((k * Fq::from_u64(2) + k.square()).reveal(), Fq::from_u64(63));

let inverse = k.nonzero().ok_or("zero")?.invert();
assert_eq!(inverse.reveal().get() * Fq::from_u64(7), Fq::ONE);
# Ok::<(), &str>(())
```

## License

Licensed under either of [Apache License, Version 2.0](https://www.apache.org/licenses/LICENSE-2.0)
or [MIT license](https://opensource.org/licenses/MIT) at your option.
