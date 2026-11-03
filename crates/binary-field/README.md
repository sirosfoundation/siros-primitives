# binary-field

[![CI](https://github.com/sirosfoundation/siros-primitives/actions/workflows/ci.yml/badge.svg)](https://github.com/sirosfoundation/siros-primitives/actions/workflows/ci.yml)
![Apache-2.0/MIT licensed](https://img.shields.io/badge/license-Apache--2.0%2FMIT-blue.svg)
![Rust 1.93+](https://img.shields.io/badge/rustc-1.93+-blue.svg)

Binary fields GF(2^n) of any degree, declared by their polynomial, in pure Rust.

## Features

- One `impl Polynomial` declares a field of any degree. The taps are checked at compile time.
- Standard fields: AES GF(2⁸), GF(2¹⁶), GF(2⁶⁴), GCM/POLYVAL GF(2¹²⁸), NIST B-233 and B-571, and
  GF(2²⁵⁶).
- Add, mul, square, invert, pow, sqrt, trace, `x` and `x⁻¹` multiples, GCM and POLYVAL block
  conversions.
- Hardware carry-less multiplication when the build enables it (PCLMULQDQ, PMULL), integer and
  vector combs otherwise, through [`clmul`](https://github.com/sirosfoundation/siros-primitives/tree/main/crates/clmul).
- Lookup tables bounded by default, and selectable by feature.
- Constant time on request: `ConstantTime<Gf2>`. CI checks every such operation under Valgrind
  with [`ct-check`](https://github.com/sirosfoundation/siros-primitives/tree/main/crates/ct-check).
- `no_std`, no `unsafe`, no allocation, no panics.
- [Benchmarks](https://github.com/sirosfoundation/siros-primitives/blob/main/crates/binary-field/BENCHMARKS.md)
  against other binary-field crates, with and without carry-less multiplication.

## Usage

```rust
use binary_field::{Gf2p8, Gf2p128};

// FIPS 197: {53}^-1 = {CA} in the field of AES.
let a = Gf2p8::from_u64(0x53).unwrap();
assert_eq!(a.invert(), Gf2p8::from_u64(0xCA).ok());

// GHASH: the product of two GCM blocks.
let h = Gf2p128::from_gcm_block(*b"\x66\xe9\x4b\xd4\xef\x8a\x2c\x3b\x88\x4c\xfa\x59\xca\x34\x2b\x2e");
let c = Gf2p128::from_gcm_block(*b"\x03\x88\xda\xce\x60\xb6\xa3\x92\xf3\x28\xc2\xb9\x71\xb2\xfe\x78");
assert_eq!(
    (c * h).to_gcm_block(),
    *b"\x5e\x2e\xc7\x46\x91\x70\x62\x88\x2c\x85\xb0\x68\x53\x53\xde\xb7"
);
```

## Constant time

`Gf2` takes the fastest path and may index tables with the value. `ConstantTime<Gf2>` runs with no
branch and no memory address that depends on the wrapped values. `invert` discloses whether the
input is zero.

```rust
use binary_field::{ConstantTime, Gf2p128};

let h = Gf2p128::from_u64(0x87).unwrap();
let secret = ConstantTime::new(h);
assert_eq!((secret * h).reveal(), h * h);
```

## Declaring a field

`Degree` is `n`; `TAPS` lists the exponents of the other terms, below `n` and including `0`.
Irreducibility is yours to check.

```rust
use binary_field::typenum::U163;
use binary_field::{Gf2, Polynomial, Pow};

/// x^163 + x^7 + x^6 + x^3 + 1.
struct B163;
impl Polynomial for B163 {
    type Degree = U163;
    const TAPS: &'static [usize] = &[7, 6, 3, 0];
}

let x = Gf2::<B163>::X;
assert_eq!(x.pow(163u32), Gf2::<B163>::from_u64(0b1100_1001).unwrap());
```

## Hardware

The build's target features select the backend; there is no runtime detection.

```sh
RUSTFLAGS="-C target-feature=+pclmulqdq,+sse4.1" cargo build   # x86-64: PCLMULQDQ
RUSTFLAGS="-C target-feature=+aes" cargo build                 # aarch64: PMULL
RUSTFLAGS="-C target-cpu=native" cargo build                   # whatever the build machine has
```

## Cargo features

`.rodata` bytes of each field's tables in an x86-64 release build:

| Field | default | `precomputed` | `not-precomputed` |
|---|---:|---:|---:|
| GF(2⁸) | 1,536 | 66,304 | 0 |
| GF(2¹⁶) | 2,048 | 786,432 | 0 |
| GF(2⁶⁴) | 8,608 | 65,952 | 416 |
| GF(2¹²⁸) | 11,872 | 262,752 | 608 |
| GF(2²³³) | 976 | 976 | 976 |
| GF(2²⁵⁶) | 600 | 600 | 600 |
| GF(2⁵⁷¹) | 2,216 | 2,216 | 2,216 |

- default: at most 16 KiB per field.
- `precomputed`: the fastest tables, of any size.
- `not-precomputed`: no tables; wins over `precomputed`. One dependent enabling either decides
  the whole build.

## License

Licensed under either of [Apache License, Version 2.0](https://www.apache.org/licenses/LICENSE-2.0)
or [MIT license](https://opensource.org/licenses/MIT) at your option.
