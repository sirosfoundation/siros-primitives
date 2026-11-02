# clmul

[![CI](https://github.com/sirosfoundation/siros-primitives/actions/workflows/ci.yml/badge.svg)](https://github.com/sirosfoundation/siros-primitives/actions/workflows/ci.yml)
![Apache-2.0/MIT licensed](https://img.shields.io/badge/license-Apache--2.0%2FMIT-blue.svg)
![Rust 1.93+](https://img.shields.io/badge/rustc-1.93+-blue.svg)

Carry-less multiplication in pure Rust: `no_std`, without allocation or panics.

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

| Target | Target features | Registers | Word products | Squares |
|---|---|---|---|---|
| `x86_64` | | SSE2 | The comb on the words of a register | Bit interleaving, on SSE2 |
| `x86_64` | `ssse3` | SSE2 | The comb on the words of a register | A byte shuffle of a nibble table |
| `x86_64` | `pclmulqdq` | SSE2 | PCLMULQDQ | PCLMULQDQ |
| `x86_64` | `pclmulqdq`, `ssse3` | SSE2 | PCLMULQDQ | A byte shuffle of a nibble table |
| little-endian `aarch64` | `aes` | Advanced SIMD | PMULL | PMULL |
| any other | | Pairs of `u64` | The comb | Bit interleaving |

The comb multiplies five interleaved bit lanes of each word by integer multiplication. With
`sse4.1`, `pextrq` moves the high half of a register out.

`RUSTFLAGS="-C target-feature=+pclmulqdq,+sse4.1"` enables both `x86_64` features,
`-C target-feature=+aes` enables PMULL, and `-C target-cpu=native` enables what the build machine
has. The const functions run the comb on every target.

A default `x86_64` build has neither PCLMULQDQ nor SSSE3, so it multiplies with the comb. Runtime
selection belongs to the application: it compiles its hot loop once per feature set, with
`#[target_feature]` or the `multiversion` crate, and dispatches around it.

`HARDWARE` is `true` in builds with one of the two instructions.

The `x86_64` and PMULL backends hold the crate's only `unsafe`: Rust requires it for each call
into a `#[target_feature]` function from a function without the attribute, even when the build
enables the feature.

## Features

- `biguint`: the `CarrylessMul` trait multiplies `biguint::Uint` values. `carryless_mul`
  truncates the product to the width of `self`, like `*`, and `widening_carryless_mul` returns it
  in `N + M` bits, like `widening_mul`. `const_carryless_mul` and `const_widening_carryless_mul`
  compute them in const contexts.

  `Poly<N>` holds a polynomial of degree below `N` in `ceil(N / 128)` registers of the backend, a
  `Lane` each: XOR, AND and OR, shifts by any count and within words, the carry-less product of
  two words, `widening_mul` (Karatsuba's three word products per pair of registers),
  `widening_square`, truncation and conversions from and to `Uint<N>` and bytes, `from_uint` in
  const contexts. `from_lane` and `into_lane` convert a polynomial of one register, which passes
  between functions in its register.

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
