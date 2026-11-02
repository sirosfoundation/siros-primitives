# binary-field

[![CI](https://github.com/sirosfoundation/siros-primitives/actions/workflows/ci.yml/badge.svg)](https://github.com/sirosfoundation/siros-primitives/actions/workflows/ci.yml)
![Apache-2.0/MIT licensed](https://img.shields.io/badge/license-Apache--2.0%2FMIT-blue.svg)
![Rust 1.93+](https://img.shields.io/badge/rustc-1.93+-blue.svg)

Binary Galois fields `GF(2^n) = GF(2)[x] / (f(x))` for any degree `n` and any irreducible `f`:
`no_std`, without `unsafe`, allocation or panics. Rust 1.93 or newer.

`Gf2<P>` is an element of the field declared by the polynomial `P`. It provides addition,
multiplication, squaring, inversion, powers by exponents of any width, square roots, traces, and
`x` and `x^-1` multiples. Its operations take the fastest path, which may index tables and take
time that depends on the value. `Secret<P>` wraps an element whose operations take one path for
every value, and reveals it on request.

Bit `i` of an element's value is the coefficient of `x^i`.

- `from_poly` and `into_poly` convert from and to `clmul::Poly<n>`.
- `into_limbs`, `into_uint`, `to_le_bytes` and `to_be_bytes` give little-endian `u64` limbs, a
  `biguint::Uint` or bytes.
- `from_limbs`, `from_uint`, `from_u64`, `from_u128`, `from_le_bytes` and `from_be_bytes` take
  them back and refuse a bit at or above `n`.

`Gf2p8`, `Gf2p16`, `Gf2p64`, `Gf2p128`, `Gf2p233`, `Gf2p256` and `Gf2p571` are the fields of
AES, GCM, the NIST curves B-233 and B-571 and three other low-weight polynomials. `Gf2p128` adds
`from_gcm_block`, `to_gcm_block`, `from_polyval_block` and `to_polyval_block`.

## Example

```rust
use binary_field::typenum::U163;
use binary_field::{Gf2, Gf2p8, Gf2p128, Polynomial, Pow};

// FIPS 197: {53}^-1 = {CA} in the field of AES.
let a = Gf2p8::from_u64(0x53).unwrap();
assert_eq!(a.invert(), Gf2p8::from_u64(0xCA).ok());
assert_eq!(a.sqrt() * a.sqrt(), a);

// GHASH: the product of two GCM blocks.
let h = Gf2p128::from_gcm_block(*b"\x66\xe9\x4b\xd4\xef\x8a\x2c\x3b\x88\x4c\xfa\x59\xca\x34\x2b\x2e");
let c = Gf2p128::from_gcm_block(*b"\x03\x88\xda\xce\x60\xb6\xa3\x92\xf3\x28\xc2\xb9\x71\xb2\xfe\x78");
assert_eq!(
    (c * h).to_gcm_block(),
    *b"\x5e\x2e\xc7\x46\x91\x70\x62\x88\x2c\x85\xb0\x68\x53\x53\xde\xb7"
);

// A field of your own: x^163 + x^7 + x^6 + x^3 + 1.
struct B163;
impl Polynomial for B163 {
    type Degree = U163;
    const TAPS: &'static [usize] = &[7, 6, 3, 0];
}
let x = Gf2::<B163>::X;
assert_eq!(x.pow(163u32), Gf2::<B163>::from_u64(0b1100_1001).unwrap());
```

## Declaring a field

`Polynomial` names `f(x) = x^n + r(x)`: `Degree` is `n` as a `typenum` unsigned integer, and
`TAPS` lists the exponents of the terms of `r`, which must be distinct, below `n`, and include
`0`. A use of the field checks the taps at compile time. Irreducibility is the declarer's
responsibility.

## Timing

`Secret` takes one path for every value; `invert` discloses whether the input is zero. `Gf2` may
index tables with the value, and `pow` takes a time that depends on the exponent. CI runs every
`Secret` operation under Valgrind's memcheck with [`ct-check`](https://github.com/sirosfoundation/siros-primitives/tree/main/crates/ct-check), which fails on a
branch or a memory address that depends on a secret.

## Backends

The word products come from [`clmul`](https://github.com/sirosfoundation/siros-primitives/tree/main/crates/clmul): PCLMULQDQ, PMULL, or integer combs in builds
without them. Enable them with `RUSTFLAGS="-C target-feature=+pclmulqdq,+sse4.1"` on x86-64 or
`-C target-feature=+aes` on aarch64. A byte shuffle (SSSE3, or `aarch64`) speeds up degrees up to
8.

## Features and tables

| Configuration | Lookup tables |
|---|---|
| default | at most 16 KiB per field |
| `precomputed` | the fastest tables, of any size |
| `not-precomputed` | none; every operation computes as `Secret` does |

`not-precomputed` wins over `precomputed`: either feature, set by any dependent, decides the whole
build.

The `.rodata` bytes of a field's constants in a release binary that uses all its operations
(`x86_64`, default target features), less an empty program's:

| Field | Default | `precomputed` | `not-precomputed` |
|---|---:|---:|---:|
| GF(2⁸) | 1,536 | 66,304 | 0 |
| GF(2¹⁶) | 2,048 | 786,432 | 0 |
| GF(2⁶⁴) | 8,608 | 65,952 | 416 |
| GF(2¹²⁸) | 11,872 | 262,752 | 608 |
| GF(2²³³) | 976 | 976 | 976 |
| GF(2²⁵⁶) | 600 | 600 | 600 |
| GF(2⁵⁷¹) | 2,216 | 2,216 | 2,216 |

[BENCHMARKS.md](https://github.com/sirosfoundation/siros-primitives/blob/main/crates/binary-field/BENCHMARKS.md)
compares `binary-field` with other binary-field crates.

## License

Licensed under either of [Apache License, Version 2.0](https://www.apache.org/licenses/LICENSE-2.0)
or [MIT license](https://opensource.org/licenses/MIT) at your option.
