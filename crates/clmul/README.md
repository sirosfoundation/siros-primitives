# clmul

[![CI](https://github.com/sirosfoundation/siros-primitives/actions/workflows/ci.yml/badge.svg)](https://github.com/sirosfoundation/siros-primitives/actions/workflows/ci.yml)
![Apache-2.0/MIT licensed](https://img.shields.io/badge/license-Apache--2.0%2FMIT-blue.svg)
![Rust 1.93+](https://img.shields.io/badge/rustc-1.93+-blue.svg)

Carry-less multiplication of words, limb slices and GF(2^128) elements. `no_std`, no
allocation, no panics.

```rust
use clmul::{clmul64, clmul_into, const_clmul64, gf2p128_mul};

// (x + 1)^2 = x^2 + 1: the cross terms cancel.
assert_eq!(clmul64(0b11, 0b11), 0b101);

// x^64 * x^64 = x^128, across little-endian limbs.
let mut product = [0; 4];
clmul_into(&mut product, &[0, 1], &[0, 1]);
assert_eq!(product, [0, 0, 1, 0]);

// x^127 * x = x^7 + x^2 + x + 1 in GF(2^128).
assert_eq!(gf2p128_mul([0, 1 << 63], [0b10, 0]), [0x87, 0]);

const SQUARE: u128 = const_clmul64(0b11, 0b11);
assert_eq!(SQUARE, 0b101);
```

```sh
# The build's target features pick the backend; there is no runtime detection.
RUSTFLAGS="-C target-feature=+pclmulqdq,+sse4.1"   # x86-64: PCLMULQDQ
RUSTFLAGS="-C target-feature=+aes"                 # aarch64: PMULL
# Otherwise: SSE2/AVX2 vector combs on x86-64, integer combs elsewhere.
```

## Features

- `biguint`: `CarrylessMul` on `biguint::Uint`, and `Poly<N>`, a polynomial held in vector
  registers.

## License

Licensed under either of [Apache License, Version 2.0](https://www.apache.org/licenses/LICENSE-2.0)
or [MIT license](https://opensource.org/licenses/MIT) at your option.
