# binary-field

[![CI](https://github.com/sirosfoundation/siros-primitives/actions/workflows/ci.yml/badge.svg)](https://github.com/sirosfoundation/siros-primitives/actions/workflows/ci.yml)
![Apache-2.0/MIT licensed](https://img.shields.io/badge/license-Apache--2.0%2FMIT-blue.svg)
![Rust 1.93+](https://img.shields.io/badge/rustc-1.93+-blue.svg)

Binary fields GF(2^n) of any degree, declared by their polynomial. `no_std`, no `unsafe`, no
allocation, no panics.

```rust
use binary_field::typenum::U163;
use binary_field::{ConstantTime, Gf2, Gf2p8, Gf2p128, Polynomial, Pow};

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

// Constant time: no branch or memory address depends on the wrapped values.
assert_eq!((ConstantTime::new(c) * h).reveal(), c * h);

// A field of your own: x^163 + x^7 + x^6 + x^3 + 1. Irreducibility is yours to check.
struct B163;
impl Polynomial for B163 {
    type Degree = U163;
    const TAPS: &'static [usize] = &[7, 6, 3, 0];
}
let x = Gf2::<B163>::X;
assert_eq!(x.pow(163u32), Gf2::<B163>::from_u64(0b1100_1001).unwrap());
```

```sh
# Hardware carry-less multiplication, otherwise integer combs.
RUSTFLAGS="-C target-feature=+pclmulqdq,+sse4.1"   # x86-64
RUSTFLAGS="-C target-feature=+aes"                 # aarch64
```

## Features

- default: lookup tables of at most 16 KiB per field.
- `precomputed`: the fastest tables, up to 768 KiB for GF(2¹⁶).
- `not-precomputed`: no tables. Wins over `precomputed` across the whole build.

[Benchmarks](https://github.com/sirosfoundation/siros-primitives/blob/main/crates/binary-field/BENCHMARKS.md) against other binary-field crates.

## License

Licensed under either of [Apache License, Version 2.0](https://www.apache.org/licenses/LICENSE-2.0)
or [MIT license](https://opensource.org/licenses/MIT) at your option.
