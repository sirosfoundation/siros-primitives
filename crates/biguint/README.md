# biguint

[![CI](https://github.com/sirosfoundation/siros-primitives/actions/workflows/ci.yml/badge.svg)](https://github.com/sirosfoundation/siros-primitives/actions/workflows/ci.yml)
![Apache-2.0/MIT licensed](https://img.shields.io/badge/license-Apache--2.0%2FMIT-blue.svg)
![Rust 1.93+](https://img.shields.io/badge/rustc-1.93+-blue.svg)

Fixed-width unsigned integers of any bit width in pure Rust: `no_std`, without `unsafe` or
panics.

`Uint<N>` holds an integer of exactly `N` bits for any `typenum` width `N`, in `ceil(N / 64)`
little-endian `u64` limbs. It provides wrapping arithmetic, carrying addition and subtraction,
widening multiplication, division by a divisor of any width, modular arithmetic, shifts,
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

## Division

A divisor is a `NonZeroUint`. `NonZeroUint::new` returns `None` for zero, so the check runs once
per divisor, and `get` returns the value. `div_rem`, `/` and `%` take a `NonZeroUint` and return
plain values.
`checked_div_rem` takes a `Uint` and returns `None` for a zero divisor. The divisor may have any
width, and the remainder has the divisor's.

```rust
use biguint::{NonZeroUint, U128, U256};

let divisor = NonZeroUint::new(U128::from_u64(10)).expect("a nonzero divisor");
let a = U256::from_u64(1234);
assert_eq!(a.div_rem(divisor), (U256::from_u64(123), U128::from_u64(4)));
assert_eq!(a / divisor, U256::from_u64(123));
assert_eq!(a % divisor, U128::from_u64(4));
assert_eq!(a.checked_div_rem(U128::ZERO), None);
```

## Modular arithmetic

`add_mod`, `sub_mod` and `neg_mod` take a `NonZeroUint` modulus of the operands' width and
operands of any value. `mul_mod` takes operands and a modulus of any widths, and returns `None` for
a zero modulus.

```rust
use biguint::{NonZeroUint, U256};

let m = NonZeroUint::new(U256::from_u64(97)).expect("a nonzero modulus");
assert_eq!(U256::from_u64(90).add_mod(U256::from_u64(10), m), U256::from_u64(3));
assert_eq!(U256::from_u64(3).sub_mod(U256::from_u64(10), m), U256::from_u64(90));
assert_eq!(U256::from_u64(3).neg_mod(m), U256::from_u64(94));
```

A `Modulus` prepares a modulus once for many operations. `Modulus::scope` runs a closure on a
`Ring`, which makes residues. `reduce` takes a value of any width. `residue` takes a value below
the modulus and returns `NotReduced` for any other. Residues have `+`, `-`, unary `-`, `*`,
`square` and `pow`, and `get` returns the value below the modulus. Each scope gives its residues a
lifetime of their own: an expression on residues of two scopes fails to compile, and the residue
operations run without checks. An odd modulus keeps residues in Montgomery form. An even modulus
divides each product by the modulus, normalized and with its reciprocal computed once.

```rust
use biguint::{Modulus, U256};

let p = Modulus::new(U256::from_u64(97)).expect("a nonzero modulus");
let value = p.scope(|ring| {
    let a = ring.reduce(U256::from_u64(200));
    let b = ring.residue(U256::from_u64(90)).expect("a value below the modulus");
    (a * b + a.pow(U256::from_u64(3))).get()
});
assert_eq!(value, U256::from_u64((6 * 90 + 6 * 6 * 6) % 97));
```

## Algorithms

The width selects the algorithm at compile time, and every operand length folds to a constant.

| Operation | Width | Algorithm |
|---|---|---|
| Product | below 16 limbs | Schoolbook rows of two carry chains, unrolled up to 16 limbs per operand |
| Product | 16 limbs (1024 bits) and more | Karatsuba with GMP's `toom22` interpolation, up to three halvings |
| Product modulo `2^N` | 17 limbs and more | Full product of the low parts and truncated cross products, split again, after GMP's `dc_mullo` |
| Division | dividend and divisor within 128 bits | `u128` division, which compiles to the hardware divide |
| Division | divisor of one or two significant limbs | Möller–Granlund 2-by-1 and 3-by-2 reciprocal steps, with the remainder in registers |
| Division | other divisors | Knuth's algorithm D with Möller–Granlund 3-by-2 reciprocals |
| Division | divisor and quotient of 24 limbs (1536 bits) and more, the divisor filling its width | Divide-and-conquer, after GMP's `dcpi1_div_qr` |
| `add_mod`, `sub_mod`, `neg_mod` | up to 8 limbs | One carry chain, then the correction as a second chain and a conditional move per limb |
| `add_mod`, `sub_mod`, `neg_mod` | 9 limbs and more | One carry chain; the top limbs decide the correction, and a full comparison runs only when they are equal |
| `Modulus` product | odd modulus | Montgomery multiplication |
| `Modulus` product | even modulus of 4 to 8 limbs (256 to 512 bits) whose top limb is nonzero | Product, then Barrett's reduction with a precomputed reciprocal |
| `Modulus` product | other even moduli | Product, then division by the prepared modulus from one limb below the top |
| `Modulus` square | below 16 limbs | Products of distinct limbs once, doubled, plus the squares of the limbs |
| `Residue::pow` | any | Sliding windows of up to 6 bits over odd powers |
| Shift | up to 2 limbs | `u128` shifts |
| Shift | 3 limbs and more | Funnel shifts of the limbs, written at the limb offset in one vectorized pass |

## Benchmarks

[BENCHMARKS.md](https://github.com/sirosfoundation/siros-primitives/blob/main/crates/biguint/BENCHMARKS.md)
compares `biguint` with `u64`/`u128`, ruint, crypto-bigint, bnum, uint and num-bigint on add, mul,
div, mod, mulmod, add_mod and sub_mod at 64 to 4096 bits, in cycles per operation
that the CPU's performance counters count. The unpublished
[`biguint-bench-report`](https://github.com/sirosfoundation/siros-primitives/tree/main/crates/biguint/bench-report)
regenerates it:

```sh
cargo run -p biguint-bench-report -- bench    # the CPU's counts of the native build
cargo run -p biguint-bench-report -- report   # BENCHMARKS.md and benchmarks/*.svg
```

## Memory

No operation allocates. In release builds, 4096-bit `mul_mod` and `div_rem` run on an 8 KiB
thread stack. Unoptimized builds need about 1.9 MiB for `mul_mod` and 1.2 MiB for `div_rem`.

## Features

Every division and modular multiplication starts from the reciprocal of the divisor's top limb.
By default, targets other than x86-64 seed it from a 512-byte table. `not-precomputed` removes the
table, and a 32-bit division computes the seed. Cargo unifies features, so one crate in a build
that enables it removes the table from the whole build. x86-64 computes the reciprocal with one
128-by-64 division and holds no table in either build.

| Build | `.rodata` of a 256-bit division, thumbv7em-none-eabi | Cycles of a 256-bit division by a 128-bit divisor, Zen 3 |
|---|---|---|
| default | 512 bytes | 105 with the table; 91 on x86-64 |
| `not-precomputed` | 0 bytes | 111 with the 32-bit seed; 91 on x86-64 |

The `.rodata` counts subtract an empty program's. The cycles of the table and the 32-bit seed come
from x86-64 builds that run the seeded reciprocal.

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
