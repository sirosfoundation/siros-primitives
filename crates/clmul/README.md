# clmul

[![CI](https://github.com/sirosfoundation/siros-primitives/actions/workflows/ci.yml/badge.svg)](https://github.com/sirosfoundation/siros-primitives/actions/workflows/ci.yml)
![Apache-2.0/MIT licensed](https://img.shields.io/badge/license-Apache--2.0%2FMIT-blue.svg)
![Rust 1.93+](https://img.shields.io/badge/rustc-1.93+-blue.svg)

Carry-less multiplication: `no_std`, without allocation or panics. Rust 1.93 or newer.

A `u64` holds a polynomial over GF(2) of degree below 64, bit `i` the coefficient of `x^i`, and a
slice of little-endian `u64` limbs holds a longer one. `clmul64` multiplies two words into a
`u128`, `clmul_into` multiplies limb slices, and `gf2p128_mul` and `Gf2p128Accumulator` multiply
and sum in `GF(2^128) = GF(2)[x] / (x^128 + x^7 + x^2 + x + 1)`. `const_clmul64` and
`const_clmul_into` compute the same products in const contexts.

## Example

```rust
use clmul::{clmul64, clmul_into, const_clmul64, gf2p128_mul};

// (x + 1)^2 = x^2 + 1: the cross terms cancel.
assert_eq!(clmul64(0b11, 0b11), 0b101);

// x^64 * x^64 = x^128, across limbs.
let mut product = [0; 4];
clmul_into(&mut product, &[0, 1], &[0, 1]);
assert_eq!(product, [0, 0, 1, 0]);

// x^127 * x = x^128 = x^7 + x^2 + x + 1 in GF(2^128).
assert_eq!(gf2p128_mul([0, 1 << 63], [0b10, 0]), [0x87, 0]);

const SQUARE: u128 = const_clmul64(0b11, 0b11);
assert_eq!(SQUARE, 0b101);
```

## Backends

The target features of the build select the backend at compile time:

| Target | Target features | Word products | Squares |
|---|---|---|---|
| `x86_64` | | Vector comb on SSE2 | Bit interleaving |
| `x86_64` | `ssse3` | Vector comb on SSE2 | Byte shuffle of a nibble table |
| `x86_64` | `avx2` | Vector comb; AVX2 for products of two registers | Byte shuffle of a nibble table |
| `x86_64` | `pclmulqdq` | PCLMULQDQ | PCLMULQDQ |
| `x86_64` | `pclmulqdq`, `ssse3` | PCLMULQDQ | Byte shuffle of a nibble table |
| little-endian `aarch64` | `aes` | PMULL | PMULL |
| any other | | Integer comb on pairs of `u64` | Bit interleaving |

`clmul64` and the const functions use the integer comb on every target without PCLMULQDQ or PMULL.

`RUSTFLAGS="-C target-feature=+pclmulqdq,+sse4.1"` enables both `x86_64` features,
`-C target-feature=+aes` enables PMULL, and `-C target-cpu=native` enables what the build machine
has. A default `x86_64` build has neither PCLMULQDQ nor SSSE3.

The backend is fixed when the crate compiles; a `#[target_feature]` function or a `multiversion`
clone that calls it still runs the backend of the build. To select at run time, build once per
feature set and load the build that the CPU supports.

`HARDWARE` is `true` in builds with PCLMULQDQ or PMULL.

The `x86_64` and PMULL backends hold the crate's only `unsafe`.

## Features

- `biguint`: the `CarrylessMul` trait multiplies `biguint::Uint` values. `carryless_mul`
  truncates the product to the width of `self`, and `widening_carryless_mul` returns it in
  `N + M` bits. `const_carryless_mul` and `const_widening_carryless_mul` compute them in const
  contexts.

  `Poly<N>` holds a polynomial of degree below `N` in `ceil(N / 128)` backend registers, a
  `Lane<U128>` each. It provides XOR, AND and OR, shifts by any count and within words, the
  carry-less product of two words, `widening_mul`, `widening_square`, truncation, and conversions
  from and to `Uint<N>` and bytes (`from_uint` in const contexts). `Lane<N>` holds a polynomial of
  up to 128 bits in one register, passes between functions in the register, and converts from and
  to `Poly<N>` without an instruction.

  `NibbleMap` is a GF(2)-linear map of nibbles to bytes, given by the images of the four bits.
  `map_low_nibbles` and `map_high_nibbles` replace every byte of a `Poly` by the image of one of
  its nibbles, in constant time on every backend.

## License

Licensed under either of [Apache License, Version 2.0](https://www.apache.org/licenses/LICENSE-2.0)
or [MIT license](https://opensource.org/licenses/MIT) at your option.
