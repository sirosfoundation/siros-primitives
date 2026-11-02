# binary-field

[![CI](https://github.com/sirosfoundation/siros-primitives/actions/workflows/ci.yml/badge.svg)](https://github.com/sirosfoundation/siros-primitives/actions/workflows/ci.yml)
![Apache-2.0/MIT licensed](https://img.shields.io/badge/license-Apache--2.0%2FMIT-blue.svg)
![Rust 1.88+](https://img.shields.io/badge/rustc-1.88+-blue.svg)

Binary Galois fields `GF(2^n) = GF(2)[x] / (f(x))` in pure Rust for any degree `n` and any
irreducible `f`: `no_std`, without `unsafe`, allocation or panics.

A field is declared by its polynomial. `Gf2<P>` holds an element as a polynomial of degree below
`n` in `ceil(n / 64)` little-endian `u64` limbs, bit `i` the coefficient of `x^i`. It provides
addition, multiplication, squaring, inversion, powers by exponents of any width, square roots,
traces, `x` and `x^-1` multiples, and conversions from and to bytes, limbs and `biguint::Uint`.
`Gf2p8`, `Gf2p16`, `Gf2p64`, `Gf2p128`, `Gf2p233`, `Gf2p256` and `Gf2p571` are the fields of
AES, GCM, the NIST curves B-233 and B-571 and three other low-weight polynomials; `Gf2p128` adds
the GCM and POLYVAL block views.

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
`0`. Any use of the field checks the taps at compile time; irreducibility is the declarer's
responsibility. Every constant the field needs follows from these two items by `const fn`s: the
limb counts, the fold steps of the reduction, the word `r` of the carry-less folds, the trace mask,
`x^-1` and `sqrt(x)`. The degree is a type because the limb count of an element has to be one.

## Algorithms

The degree and the taps select each algorithm at compile time, and every limb count, shift and
loop bound folds to a constant, so each field compiles to straight-line code. The word products
come from
[`clmul`](https://github.com/sirosfoundation/siros-primitives/tree/main/crates/clmul):
PCLMULQDQ, PMULL, or a const comb in builds without them; the plan knows which through
`clmul::HARDWARE`.

| Operation | Case | Algorithm |
|---|---|---|
| Product | up to 6 limbs with hardware products, 1 limb without | Schoolbook over 64-bit carry-less products |
| Product | 7 limbs and more with hardware products, 2 and more without | Karatsuba (Karatsuba and Ofman, 1962), in characteristic 2 the middle term costs only XORs |
| Reduction | `r` of degree below 64 and the carry-less fold cheaper by the cost model | Carry-less folding (Gueron and Kounavis, 2010): each 64-bit chunk of the overflow, aligned to `n`, is one product with `r`, from the top down; a final step folds what the products put back above `n` |
| Reduction | otherwise, such as `x^233 + x^74 + 1` | Shift-and-XOR per tap of the whole overflow, in the two rounds that a low-weight `r` needs |
| Square | hardware products | One carry-less product of each limb with itself: the cross terms cancel |
| Square | otherwise | Bit spreading: bit `i` moves to `2i` by five mask-and-shift steps per half limb |
| Inverse | the cost model prefers it, which is every hardware build | Itoh and Tsujii (1988): `a^(2^n - 2)` along the bits of `n - 1`, `n - 1` squarings and about `log2(n) + popcount(n - 1)` products, without a data-dependent branch |
| Inverse | otherwise | Extended Euclidean algorithm over `GF(2)[x]` (Hankerson, Menezes and Vanstone, algorithm 2.48) |
| Power | any exponent width | Square-and-multiply over `biguint::Exponent`, which `u32`, `u64` and `Uint<N>` implement |
| Square root | degree up to 1024 | Even and odd coefficient split: `sqrt(a) = e(x) + sqrt(x) o(x)`, with `sqrt(x) = x^(2^(n-1))` a compile-time constant |
| Square root | larger degree | The Frobenius map, `n - 1` squarings at run time; the const evaluator refuses the longer chain |
| Trace | any | The parity of `a & m`, where bit `k` of the compile-time mask `m` is the trace of `x^k` from Newton's identities on the coefficients of `f` |
| GCM view | `GF(2^128)` with the GCM polynomial | Bit reversal of the big-endian block, the order of NIST SP 800-38D |
| POLYVAL view | the same field | Bit reversal of the little-endian block times `x`: RFC 8452 works in the reciprocal polynomial with a factor `x^-128`, and this map is the field isomorphism that carries its sum and product |

### Instruction counts

Callgrind counts of one operation, `-C target-cpu=native` on a Zen 3, compared with the crates
that implement the same degree. The table-based crates win at 8 and 16 bits with lookups that
depend on the cache; every other case is on par or ahead.

| Degree | Operation | binary-field | Others |
|---|---|---|---|
| 8 | mul | 20 | gf256 Barrett 18, gf256 table 15, g2p table 7 |
| 8 | invert | 161 | gf256 Barrett 124, gf256 table 9 |
| 16 | mul | 20 | g2p table 18, gf256 table 32 |
| 16 | invert | 301 | gf256 Barrett 388, g2p table 22 |
| 64 | mul | 15 | gf256 Barrett 13, hekate-math tower 3658 |
| 64 | invert | 982 | gf256 Barrett 1270, hekate-math tower 7889 |
| 128 | mul | 22 | longfellow-zk runtime 24, compile 77, core 2123; polyval 62, ghash 65; hekate-math tower 14670 |
| 128 | square | 17 | |
| 128 | invert | 2172 | longfellow-zk runtime 6567, compile 13663, core 549785; hekate-math tower 29880 |
| 571 | mul / square / invert | 430 / 101 / 60748 | |

## Minimum supported Rust version

Rust 1.88 or newer.

## License

The crate is licensed under either of

- [Apache License, Version 2.0](https://www.apache.org/licenses/LICENSE-2.0)
- [MIT license](https://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in
the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
