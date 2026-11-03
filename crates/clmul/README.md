# clmul

[![CI](https://github.com/sirosfoundation/siros-primitives/actions/workflows/ci.yml/badge.svg)](https://github.com/sirosfoundation/siros-primitives/actions/workflows/ci.yml)
![Apache-2.0/MIT licensed](https://img.shields.io/badge/license-Apache--2.0%2FMIT-blue.svg)
![Rust 1.93+](https://img.shields.io/badge/rustc-1.93+-blue.svg)

Carry-less multiplication of words, limb slices and GF(2^128) elements, in Rust: `no_std`, no
allocation, no panics.

## Features

- `clmul64`: the product of two words, in a `u128`.
- `clmul_into`: the product of two little-endian limb slices of any lengths.
- `gf2p128_mul` and `Gf2p128Accumulator`: products and sums of products in GF(2¹²⁸) with the GCM
  polynomial.
- `const_clmul64` and `const_clmul_into`: the same products in const contexts.
- `HARDWARE`: `true` when the build multiplies with PCLMULQDQ or PMULL.
- With the `biguint` feature:
  - `CarrylessMul` on `biguint::Uint`: `carryless_mul` keeps the width of `self`,
    `widening_carryless_mul` keeps the whole product. `const_carryless_mul` and
    `const_widening_carryless_mul` compute them in const contexts.
  - `Poly<N>`: a polynomial of degree below `N` in vector registers, with `widening_mul`,
    `widening_square`, word products, shifts, `& | ^`, and `Uint` and byte conversions.
  - `Lane<N>`: a polynomial of up to 128 bits in one register.
  - `NibbleMap`: a GF(2)-linear map of nibbles to bytes, applied to every byte of a `Poly` by
    `map_low_nibbles` and `map_high_nibbles`.

## Usage

Bit `i` of a word is the coefficient of `x^i`.

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

## Integers and polynomials

With the `biguint` feature:

```rust
# #[cfg(feature = "biguint")] {
use biguint::{typenum::U128, Uint};
use clmul::{CarrylessMul, Poly};

let a = Uint::<U128>::from_u64(0b11);
assert_eq!(a.widening_carryless_mul(a), Uint::from_u64(0b101));

let p = Poly::<U128>::from_uint(a);
assert_eq!(p.widening_square().to_uint(), Uint::from_u64(0b101));
# }
```

## Backends

The build's target features select the backend when the crate compiles.

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

To choose at run time, build once per feature set and load the build the CPU supports.

## License

Licensed under either of [Apache License, Version 2.0](https://www.apache.org/licenses/LICENSE-2.0)
or [MIT license](https://opensource.org/licenses/MIT) at your option.
