# biguint

[![CI](https://github.com/sirosfoundation/siros-primitives/actions/workflows/ci.yml/badge.svg)](https://github.com/sirosfoundation/siros-primitives/actions/workflows/ci.yml)
![Apache-2.0/MIT licensed](https://img.shields.io/badge/license-Apache--2.0%2FMIT-blue.svg)
![Rust 1.88+](https://img.shields.io/badge/rustc-1.88+-blue.svg)

Fixed-width unsigned integers of any bit width in pure Rust: `no_std`, without `unsafe` or
panics.

`Uint<N>` holds an integer of exactly `N` bits for any `typenum` width `N`, in `ceil(N / 64)`
little-endian `u64` limbs. It provides wrapping arithmetic, carrying addition and subtraction,
widening multiplication, division by a divisor of any width, modular multiplication, shifts,
rotations, comparisons, byte encodings and exact conversions.

## Example

```rust
use biguint::typenum::U521;
use biguint::{U256, U512, Uint, WideningMul};

let top = U256::ONE << 255usize;
assert_eq!(U256::from_u64(3) * top, top);
assert_eq!(top.widening_mul(top), U512::ONE << 510usize);
assert_eq!(
    U256::from_u64(7).mul_mod(U256::from_u64(9), U256::from_u64(10)),
    Some(U256::from_u64(3))
);

let max521: Uint<U521> = !Uint::ZERO;
assert_eq!(max521.bit_len(), 521);
```

## Algorithms

The width selects the algorithm at compile time, and every operand length folds to a constant.

| Operation | Width | Algorithm |
|---|---|---|
| Product | up to 9 by 9 limbs, or 12 by 12 modulo `2^N` | Schoolbook, unrolled into straight-line code |
| Product | below 16 limbs | Schoolbook rows with one carry limb |
| Product | 16 limbs (1024 bits) and more | Karatsuba with GMP's `toom22` interpolation, up to three halvings |
| Product modulo `2^N` | 13 limbs and more | Full product of the low parts and truncated cross products, split again, after GMP's `dc_mullo` |
| Division | dividend and divisor within 128 bits | `u128` division, which compiles to the hardware divide |
| Division | other divisors | Knuth's algorithm D with Möller–Granlund reciprocals |
| Division | divisor and quotient of 24 limbs (1536 bits) and more, the divisor filling its width | Divide-and-conquer, after GMP's `dcpi1_div_qr` |
| Shift | up to 8 limbs | Branch-free barrel shifter |

## Benchmarks

[BENCHMARKS.md](https://github.com/sirosfoundation/siros-primitives/blob/main/crates/biguint/BENCHMARKS.md)
compares `biguint` with `u64`/`u128`, ruint, crypto-bigint, bnum, uint and num-bigint on add, mul,
div, mod and mulmod at 64 to 4096 bits, in instructions per operation that Callgrind counts. The
unpublished
[`biguint-bench-report`](https://github.com/sirosfoundation/siros-primitives/tree/main/crates/biguint/bench-report)
regenerates it:

```sh
cargo run -p biguint-bench-report -- bench    # Callgrind counts of the native build
cargo run -p biguint-bench-report -- report   # BENCHMARKS.md and benchmarks/*.svg
```

## Memory

No operation allocates. In release builds, 4096-bit `mul_mod` and `div_rem` run on an 8 KiB
thread stack. Unoptimized builds need about 1.9 MiB for `mul_mod` and 1.2 MiB for `div_rem`.

## Features

- `precomputed` (default): a 512-byte table seeds the reciprocal that every division and modular
  multiplication starts from. Without it, a 32-bit division computes the seed.

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
