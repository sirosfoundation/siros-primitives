# prime-field

[![CI](https://github.com/sirosfoundation/siros-primitives/actions/workflows/ci.yml/badge.svg)](https://github.com/sirosfoundation/siros-primitives/actions/workflows/ci.yml)
![Apache-2.0/MIT licensed](https://img.shields.io/badge/license-Apache--2.0%2FMIT-blue.svg)
![Rust 1.93+](https://img.shields.io/badge/rustc-1.93+-blue.svg)

Prime fields declared by their modulus, in pure Rust.

## Features

- One `impl Modulus` declares a field, from 31 bits to thousands. Every constant derives from the
  modulus at compile time.
- The modulus picks the representation: Mersenne residues for one-limb `2^k - 1`, plain residues
  with folding for `2^k - c` (secp256k1, P-521), Montgomery for every other.
- Standard fields: Mersenne-31, BabyBear, Goldilocks, BN254 scalar, secp256k1, P-256 base and
  scalar, Tom-256, BLS12-381 base and P-521, each with curve-named aliases.
- Add, sub, neg, double, mul, square, invert, pow, sqrt, operators, byte and integer conversions.
  The `const_*` twins compute in const contexts.
- Constant time on request: `ConstantTime<Fp>`. CI checks every such operation under Valgrind
  with [`ct-check`](https://github.com/sirosfoundation/siros-primitives/tree/main/crates/ct-check).
- `no_std`, no `unsafe`, no allocation, no panics.
- [Benchmarks](https://github.com/sirosfoundation/siros-primitives/blob/main/crates/prime-field/BENCHMARKS.md)
  against other prime-field crates.

## Usage

```rust
use prime_field::standard::secp256k1::Base as Fq;

let x = Fq::from_u64(7);
let y = x.invert().ok_or("zero")?;
assert_eq!(x * y, Fq::ONE);
assert_eq!(Fq::from_u64(4).sqrt().map(|r| r.square()), Some(Fq::from_u64(4)));

const FORTY_NINE: Fq = Fq::from_u64(7).const_mul(Fq::from_u64(7));
assert_eq!(FORTY_NINE, Fq::from_u64(49));
# Ok::<(), &str>(())
```

## Constant time

`Fp` takes the fastest path. `ConstantTime<Fp>` runs with no branch and no memory address that
depends on the wrapped values. `invert` and `sqrt` disclose whether the result exists, and `pow`'s
time depends on its public exponent.

```rust
use prime_field::standard::secp256k1::Base as Fq;
use prime_field::ConstantTime;

let k = ConstantTime::new(Fq::from_u64(7));
assert_eq!((k * Fq::from_u64(2) + k.square()).reveal(), Fq::from_u64(63));
```

## Declaring a field

`MODULUS` holds the little-endian limbs of an odd prime; `Width` is a `typenum` width that fits
it.

```rust
use prime_field::biguint::{typenum::U255, Uint};
use prime_field::{Fp, Modulus};

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

let two = Fp::<Curve25519Base>::from_u64(2);
assert_eq!(two.invert().map(|half| half.double()), Some(Fp::ONE));
```

## License

Licensed under either of [Apache License, Version 2.0](https://www.apache.org/licenses/LICENSE-2.0)
or [MIT license](https://opensource.org/licenses/MIT) at your option.
