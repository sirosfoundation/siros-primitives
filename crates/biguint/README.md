# biguint

[![CI](https://github.com/sirosfoundation/siros-primitives/actions/workflows/ci.yml/badge.svg)](https://github.com/sirosfoundation/siros-primitives/actions/workflows/ci.yml)
![Apache-2.0/MIT licensed](https://img.shields.io/badge/license-Apache--2.0%2FMIT-blue.svg)
![Rust 1.93+](https://img.shields.io/badge/rustc-1.93+-blue.svg)

Fixed-width unsigned integers of any bit width: `no_std`, without `unsafe`, allocation or
panics. Rust 1.93 or newer.

`Uint<N>` holds exactly `N` bits for any `typenum` width `N`, in `ceil(N / 64)` little-endian
`u64` limbs. It provides wrapping, carrying and widening arithmetic, division by a divisor of any
width, modular arithmetic, shifts, rotations, comparisons, byte encodings and exact conversions.
`<<` and `>>` take an amount of any primitive integer type or any `Uint`; an amount that is
negative or at least `N` gives `0`.

## Example

```rust
use biguint::typenum::U521;
use biguint::{U256, U512, Uint, WideningMul};

let top = U256::ONE << 255usize;
assert_eq!(U256::from_u64(3) * top, top);
assert_eq!(top.widening_mul(top), U512::ONE << 510usize);
assert_eq!(
    U256::from_u64(7).checked_mul_mod(U256::from_u64(9), U256::from_u64(10)),
    Some(U256::from_u64(3))
);

let max521: Uint<U521> = !Uint::ZERO;
assert_eq!(max521.bit_len(), 521);
```

## Division

A divisor is a `NonZeroUint` (`NonZero<Uint<N>>`); `NonZero::new` returns `None` for zero.
`div_rem`, `/` and `%` take a `NonZeroUint` of any width and return plain values; the remainder
has the divisor's width. `checked_div_rem` takes a `Uint` and returns `None` for a zero divisor.
`|`, `saturating_add`, `saturating_mul`, `checked_add`, `checked_mul` and `checked_pow` return a
`NonZeroUint` for a `NonZeroUint` operand.

```rust
use biguint::{NonZeroUint, U128, U256};

let divisor = NonZeroUint::new(U128::from_u64(10)).expect("a nonzero divisor");
let a = U256::from_u64(1234);
assert_eq!(a.div_rem(divisor), (U256::from_u64(123), U128::from_u64(4)));
assert_eq!(a / divisor, U256::from_u64(123));
assert_eq!(a % divisor, U128::from_u64(4));
assert_eq!(a.checked_div_rem(U128::ZERO), None);
```

## num-traits

`Uint` implements these `num-traits` traits, each with the inherent method of the same name:
`Zero`, `One`, `ConstZero`, `ConstOne`, `Bounded`, `Pow<u32>`, `MulAdd`,
`Checked{Add,Sub,Mul,Neg,Shl,Shr}`, `Wrapping{Add,Sub,Mul,Neg,Shl,Shr}`,
`Saturating{Add,Sub,Mul}`, `Overflowing{Add,Sub,Mul}`, `ToPrimitive`, `FromPrimitive`, `NumCast`,
and `ToBytes` and `FromBytes` on `ceil(N / 8)` bytes.

- `checked_shl` and `checked_shr` return `None` for a shift of `N` or more; `wrapping_shl` and
  `wrapping_shr` shift by the amount modulo `N`.
- `FromBytes` reads the bytes modulo `2^N`; `FromPrimitive` keeps the integer part of a float;
  `NumCast` returns `None` from `2^128` up; `to_f32` and `to_f64` round to nearest, ties to even.
- `checked_div` and `checked_rem` are inherent methods. `CheckedDiv`, `CheckedRem`, `Num`,
  `Unsigned`, `PrimInt` and `Euclid` are not implemented: they require `Div<Self>` and
  `Rem<Self>`.
- `NonZero<T>` has the crate's `One` wherever `T` has it.

## Modular arithmetic

`add_mod`, `sub_mod` and `neg_mod` take a `NonZeroUint` modulus of the operands' width and
operands of any value. `mul_mod` takes a `NonZeroUint` modulus and operands of any widths and
values; `checked_mul_mod` takes a `Uint` modulus and returns `None` for zero.

```rust
use biguint::{NonZeroUint, U256};

let m = NonZeroUint::new(U256::from_u64(97)).expect("a nonzero modulus");
assert_eq!(U256::from_u64(90).add_mod(U256::from_u64(10), m), U256::from_u64(3));
assert_eq!(U256::from_u64(3).sub_mod(U256::from_u64(10), m), U256::from_u64(90));
assert_eq!(U256::from_u64(3).neg_mod(m), U256::from_u64(94));
assert_eq!(U256::from_u64(50).mul_mod(U256::from_u64(4), m), U256::from_u64(6));
```

A `Modulus` prepares a modulus once for many operations. `Modulus::scope` runs a closure on a
`Ring`: `reduce` makes a residue from a value of any width, `residue` from a value below the
modulus (otherwise `NotReduced`). Residues have `+`, `-`, unary `-`, `*`, `square`, `pow` and
`get`. Residues of different scopes fail to compile together.

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

## Memory

No operation allocates. A 4096-bit `*`, `div_rem` or `mul_mod` uses at most 7.9 KiB of stack in a
release build.

## Features

- `not-precomputed`: removes the 512-byte table that seeds the division reciprocal on targets
  other than x86-64; a 32-bit division computes the seed. x86-64 holds no table in either build.
  Cargo unifies features: one dependent that enables it removes the table from the whole build.

[BENCHMARKS.md](https://github.com/sirosfoundation/siros-primitives/blob/main/crates/biguint/BENCHMARKS.md)
compares `biguint` with other big-integer crates.

## License

Licensed under either of [Apache License, Version 2.0](https://www.apache.org/licenses/LICENSE-2.0)
or [MIT license](https://opensource.org/licenses/MIT) at your option.
