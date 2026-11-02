# binary-field

[![CI](https://github.com/sirosfoundation/siros-primitives/actions/workflows/ci.yml/badge.svg)](https://github.com/sirosfoundation/siros-primitives/actions/workflows/ci.yml)
![Apache-2.0/MIT licensed](https://img.shields.io/badge/license-Apache--2.0%2FMIT-blue.svg)
![Rust 1.93+](https://img.shields.io/badge/rustc-1.93+-blue.svg)

Binary Galois fields `GF(2^n) = GF(2)[x] / (f(x))` in pure Rust for any degree `n` and any
irreducible `f`: `no_std`, without `unsafe`, allocation or panics.

A field is declared by its polynomial. `Gf2<P>` holds an element, a polynomial of degree below `n`,
in the registers its plan chooses: a byte or a 16-bit word in a general register up to degree 16,
one vector register up to degree 128, and `ceil(n / 128)` vector registers above. It provides
addition, multiplication, squaring, inversion, powers by exponents of any width, square roots,
traces, and `x` and `x^-1` multiples. Its operations take the fastest path, which may index tables
and take time that depends on the value. `Secret<P>` wraps an element whose operations take one
path for every value, and reveals it on request.

Bit `i` of an element's value is the coefficient of `x^i`. `from_poly` and `into_poly` convert it
from and to `clmul::Poly<n>`, the vector registers it lives in above degree 16. `into_limbs`,
`into_uint`, `to_le_bytes` and `to_be_bytes` give it as little-endian `u64` limbs, a
`biguint::Uint` or bytes, and `from_limbs`, `from_uint`, `from_u64`, `from_u128`, `from_le_bytes`
and `from_be_bytes` take them back, refusing a bit at or above `n`.

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
and squares come from
[`clmul`](https://github.com/sirosfoundation/siros-primitives/tree/main/crates/clmul):
PCLMULQDQ, PMULL, or combs of integer products in builds without them, and its `Poly` of vector
registers. The
plan knows which through `clmul::HARDWARE`. "With tables" means the default build or
`precomputed` ([Tables](#tables)).

| Operation | Case | Algorithm |
|---|---|---|
| Product | `Gf2`, degree up to 8, with `precomputed` | The table of every product |
| Product | `Gf2`, degree up to 8, by default | Logarithm and antilogarithm tables of a generator |
| Product | `Gf2`, degree 9 to 16, with `precomputed` and without hardware word products | Logarithm and antilogarithm tables of a generator |
| Product | degree up to 8, with hardware word products and a byte shuffle (SSSE3 or `aarch64`) | One carry-less product in a vector register, whose part at and above `x^n` folds by two nibble maps |
| Product | degree up to 8, without hardware word products | Integer products of two bit lanes of spacing 2 over the low 6 bits of one factor, four products, and masked copies for its top 2 bits |
| Product | degree 9 to 16, without hardware word products | Integer products of three bit lanes of spacing 3, two lane products per 64-bit product: five products |
| Product | degree up to 64 | One carry-less product of the words |
| Product | degree 128, `d` below 64, with hardware word products | `a0 b + a1 c` for `a = a0 + a1 x^64` and `c = x^64 b mod f = b0 x^64 + b1 r`: one word product gives `c`, four give `u + w x^64`, and the top word of `w` folds by one product with `r`, without overflow |
| Product | above | Karatsuba's three word products per pair of registers (Karatsuba and Ofman, 1962), schoolbook across registers |
| Reduction | degree 33 to 64, `d` at most 4 | `(h r mod x^n) + t r` for the part `h` at and above `x^n` and the overflow `t` of `h r`: one carry-less product with hardware word products, shifts per tap otherwise, and in parallel `t r` as the image of the top nibble of `h` under a nibble map; a square takes the map of the top two bits of the element |
| Reduction | degree 33 to 64, `2d <= n`, with hardware word products | `(h + t) r mod x^n` for the part `h` at and above `x^n` and the overflow `t` of `h r`: shifts give `t`, one carry-less product the fold |
| Reduction | degree above 128, `2d <= n` and `d` below 64, with hardware word products | Two rounds of carry-less products with `r`, the second for the overflow of the first (Gueron and Kounavis, 2010) |
| Reduction | `2d <= n`, with `d` the degree of `r`, such as `x^233 + x^74 + 1` | One shifted copy of the overflow per tap, with the overflow of the fold added first |
| Reduction | otherwise | Rounds of shifted copies per tap, as many as the taps need |
| Square | `Gf2` with tables, degree up to 8, and up to 16 with `precomputed` | The table of every square |
| Square | `Gf2` with the default tables, degree 9 to 16 | The images of the two bytes |
| Square | degree up to 8, with a byte shuffle | The constant-time linear map as two nibble maps in a vector register: one byte shuffle per nibble |
| Square | degree up to 8 | A constant-time linear map: each output byte is the parities of an input byte under eight row masks, by two products (Warren, Hacker's Delight, 7-1 and 7-6) |
| Square | degree 9 to 16 | The carry-less product of the word with itself with hardware word products, its bits spread to twice their position otherwise; then the reduction |
| Square | up to degree 64 with `2d <= n` | The bits spread to twice their position by `clmul`'s square; in a run of squarings, the carry-less product of the word with itself with hardware word products, which takes fewer instructions; then the reduction |
| Square | otherwise | The bits spread to twice their position: a byte shuffle of a nibble table with SSSE3, the carry-less product of each word with itself with PCLMULQDQ or PMULL, interleaving otherwise; then the reduction |
| Inverse | `Gf2` with tables, degree up to 8, and up to 16 with `precomputed` | The table of every inverse |
| Inverse | otherwise | `a^(2^n - 2)` along a short addition chain for `n - 1` (Itoh and Tsujii, 1988; Knuth, TAOCP vol. 2, 4.6.3): one product and one Frobenius power per step, 10 products for GF(2¹²⁸) |
| Frobenius power | `Gf2` with tables, degree 17 to 128 | The sum of the images of the windows of the value: byte windows with `precomputed`, 3-bit or 4-bit windows within 16 KiB by default, composed with squarings |
| Frobenius power | otherwise | Squarings; up to degree 16, the constant-time linear map of the power, as two nibble maps up to degree 8 with a byte shuffle |
| Power | any exponent width | Square-and-multiply over `biguint::Exponent`, which `u32`, `u64` and `Uint<N>` implement |
| Square root | `Gf2` with tables, degree up to 8, and up to 16 with `precomputed` | The table of every root |
| Square root | `Gf2` with the default tables, degree 9 to 16 | The images of the two bytes |
| Square root | degree up to 16 | The constant-time linear map of `a^(2^(n-1))`, as two nibble maps up to degree 8 with a byte shuffle |
| Square root | degree up to 1024 | Even and odd coefficient split: `sqrt(a) = e(x) + sqrt(x) o(x)`, with `sqrt(x) = x^(2^(n-1))` a compile-time constant. `o` has half the coefficients: up to 64 bits one gather splits `a` and `a / x` side by side in a register, and at 128 bits `sqrt(x) o(x)` takes two word products and one fold |
| Square root | larger degree | The Frobenius map, `n - 1` squarings at run time; the const evaluator refuses the longer chain |
| Trace | any | The parity of `a & m`, where bit `k` of the compile-time mask `m` is the trace of `x^k` from Newton's identities on the coefficients of `f` |
| GCM view | `GF(2^128)` with the GCM polynomial | Bit reversal of the big-endian block, the order of NIST SP 800-38D |
| POLYVAL view | the same field | Bit reversal of the little-endian block times `x`: RFC 8452 works in the reciprocal polynomial with a factor `x^-128`, and this map is the field isomorphism that carries its sum and product |

Timing: the operations of `Secret` take one path for every value: its inverse for every nonzero
value, which `invert` discloses. The tables of the operations of `Gf2` take time that depends on
the value, and `pow` on the exponent. A selection by a secret bit is a mask that passes through
`core::hint::black_box`, which keeps the compiler from turning it into a branch. CI runs every
operation of `Secret` on secret operands under Valgrind's memcheck with
[`ct-check`](https://github.com/sirosfoundation/siros-primitives/tree/main/crates/ct-check),
which fails on a branch or a memory address that depends on them.

## Tables

Three configurations of features decide the lookup tables of a build:

- the default: at most 16 KiB of tables per field;
- `precomputed`: the fastest tables, of any size;
- `not-precomputed`: no lookup table; every operation computes as `Secret`'s does. It wins over
  `precomputed`, so either feature, set by any dependent, decides the whole build.

The bytes of each field's constants: the `.rodata` of a release binary that adds, multiplies,
squares, inverts and takes square roots in the field, less an empty program's (`x86_64`, default
target features). Without tables, the bytes left are the masks and constants that instructions
read.

| Field | Default | `precomputed` | `not-precomputed` |
|---|---:|---:|---:|
| GF(2⁸) | 1,536 | 66,304 | 0 |
| GF(2¹⁶) | 2,048 | 786,432 | 0 |
| GF(2⁶⁴) | 8,608 | 65,952 | 416 |
| GF(2¹²⁸) | 11,872 | 262,752 | 608 |
| GF(2²³³) | 976 | 976 | 976 |
| GF(2²⁵⁶) | 600 | 600 | 600 |
| GF(2⁵⁷¹) | 2,216 | 2,216 | 2,216 |

The cycles of the operations that differ, batch / chain, from one sweep per configuration on an
AMD Ryzen 9 5950X (`-C target-cpu=native`, load 18):

| Operation | Default | `precomputed` | `not-precomputed` |
|---|---|---|---|
| GF(2⁸) mul | 3.6 / 12.0 | 3.1 / 9.5 | 6.8 / 18.9 |
| GF(2⁸) square | 2.1 / 5.0 | 2.1 / 5.0 | 3.5 / 12.4 |
| GF(2⁸) sqrt | 2.4 / 5.0 | 2.1 / 5.0 | 3.1 / 12.3 |
| GF(2⁸) invert | 2.3 / 5.1 | 2.1 / 5.0 | 55 / 123 |
| GF(2¹⁶) square | 4.2 / 9.0 | 3.8 / 6.0 | 7.0 / 20.0 |
| GF(2¹⁶) sqrt | 3.9 / 9.0 | 2.9 / 6.0 | 14.5 / 37.1 |
| GF(2¹⁶) invert | 170 / 264 | 3.5 / 6.0 | 171 / 264 |
| GF(2⁶⁴) invert | 235 / 284 | 198 / 241 | 670 / 721 |
| GF(2¹²⁸) invert | 714 / 740 | 506 / 421 | 1962 / 1985 |

The default and `not-precomputed` builds run the same GF(2¹⁶) inversion, 335 instructions.

## Benchmarks

[BENCHMARKS.md](https://github.com/sirosfoundation/siros-primitives/blob/main/crates/binary-field/BENCHMARKS.md)
compares `binary-field` with gf256, galois_field_2pm, g2p, reed-solomon-erasure, hekate-math,
Plonky3, longfellow-zk's `runtime-algebra`, `compile-algebra` and `core-algebra`, and polyval on
add, mul, square, invert and sqrt over GF(2⁸), GF(2¹⁶), GF(2⁶⁴), GF(2¹²⁸), GF(2²³³), GF(2²⁵⁶) and
GF(2⁵⁷¹), each operation in a constant-time and a variable-time row, with `precomputed`. The
figures are cycles per operation that the CPU's performance counters count, in builds with and
without carry-less multiplication. The unpublished
[`binary-field-bench-report`](https://github.com/sirosfoundation/siros-primitives/tree/main/crates/binary-field/bench-report)
regenerates it:

```sh
cargo run -p binary-field-bench-report -- bench    # the CPU's counts of both builds
cargo run -p binary-field-bench-report -- report   # BENCHMARKS.md and benchmarks/*.svg
```

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
