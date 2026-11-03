# clmul

[![CI](https://github.com/sirosfoundation/siros-primitives/actions/workflows/ci.yml/badge.svg)](https://github.com/sirosfoundation/siros-primitives/actions/workflows/ci.yml)
![Apache-2.0/MIT licensed](https://img.shields.io/badge/license-Apache--2.0%2FMIT-blue.svg)
![Rust 1.93+](https://img.shields.io/badge/rustc-1.93+-blue.svg)

Carry-less multiplication of words, limb slices and GF(2^128) elements, in Rust.

## Features

- `clmul64`: two words into a `u128`. `clmul_into`: limb slices of any length.
- `gf2p128_mul` and `Gf2p128Accumulator`: products and sums in GF(2¹²⁸) with the GCM polynomial.
- `const_clmul64` and `const_clmul_into` compute the same products in const contexts.
- With the `biguint` feature: `CarrylessMul` on `biguint::Uint`, and `Poly<N>`, a polynomial held
  in `ceil(N / 128)` vector registers, with products, squares, shifts and nibble maps.
- Backends picked at compile time from the build's target features, with no runtime detection.
- `no_std`, no allocation, no panics. The hardware backends hold the crate's only `unsafe`.

## Usage

Bit `i` of a word is the coefficient of `x^i`; limbs are little-endian.

```rust
use clmul::{clmul64, clmul_into, const_clmul64, gf2p128_mul};

// (x + 1)^2 = x^2 + 1: the cross terms cancel.
assert_eq!(clmul64(0b11, 0b11), 0b101);

// x^64 * x^64 = x^128, across limbs.
let mut product = [0; 4];
clmul_into(&mut product, &[0, 1], &[0, 1]);
assert_eq!(product, [0, 0, 1, 0]);

// x^127 * x = x^7 + x^2 + x + 1 in GF(2^128).
assert_eq!(gf2p128_mul([0, 1 << 63], [0b10, 0]), [0x87, 0]);

const SQUARE: u128 = const_clmul64(0b11, 0b11);
assert_eq!(SQUARE, 0b101);
```

## Backends

| Target | Target features | Word products | Squares |
|---|---|---|---|
| `x86_64` | | Vector comb on SSE2 | Bit interleaving |
| `x86_64` | `ssse3` | Vector comb on SSE2 | Byte shuffle of a nibble table |
| `x86_64` | `avx2` | Vector comb; AVX2 for two-register products | Byte shuffle of a nibble table |
| `x86_64` | `pclmulqdq` | PCLMULQDQ | PCLMULQDQ |
| `x86_64` | `pclmulqdq`, `ssse3` | PCLMULQDQ | Byte shuffle of a nibble table |
| little-endian `aarch64` | `aes` | PMULL | PMULL |
| any other | | Integer comb on pairs of `u64` | Bit interleaving |

```sh
RUSTFLAGS="-C target-feature=+pclmulqdq,+sse4.1" cargo build   # x86-64
RUSTFLAGS="-C target-feature=+aes" cargo build                 # aarch64
```

A default `x86_64` build has neither PCLMULQDQ nor SSSE3. `HARDWARE` is `true` in builds with
PCLMULQDQ or PMULL. The backend is fixed when the crate compiles: to select at run time, build once
per feature set and load the build the CPU supports.

## License

Licensed under either of [Apache License, Version 2.0](https://www.apache.org/licenses/LICENSE-2.0)
or [MIT license](https://opensource.org/licenses/MIT) at your option.
