# biguint

[![CI](https://github.com/sirosfoundation/siros-primitives/actions/workflows/ci.yml/badge.svg)](https://github.com/sirosfoundation/siros-primitives/actions/workflows/ci.yml)
![Apache-2.0/MIT licensed](https://img.shields.io/badge/license-Apache--2.0%2FMIT-blue.svg)
![Rust 1.93+](https://img.shields.io/badge/rustc-1.93+-blue.svg)

Fixed-width unsigned integers of any bit width, in pure Rust: `no_std`, no `unsafe`, no
allocation, no panics.

## Features

- `Uint<N>`: exactly `N` bits for any `typenum` width `N`, in `ceil(N / 64)` limbs. `U8` to `U512`
  name the common widths.
- Arithmetic: the operators, `carrying_add`, `borrowing_sub`, `mul_add`, `wrapping_pow`, and the
  checked, wrapping, saturating and overflowing forms. `widening_mul` returns the whole product.
- Division: `div_rem`, `/` and `%` by a `NonZeroUint` of any width, with no failure path.
  `checked_div_rem`, `checked_div` and `checked_rem` take a divisor that may be zero.
- Modular arithmetic: `add_mod`, `sub_mod`, `neg_mod` and `mul_mod` on operands of any value.
- Prepared moduli: `Modulus::scope` gives a `Ring` whose `Residue`s add, subtract, multiply,
  square and raise to powers. Residues of two scopes fail to compile together.
- Bits: shifts by any integer or `Uint` amount, rotations, `& | ^ !`, `bit_at`, `bit_len`,
  `count_ones`, `leading_zeros`, `trailing_zeros` and `swap_bytes`.
- Conversions: `resize` and `try_resize` between widths, limbs, little- and big-endian bytes,
  `u64`, `u128`, floats, and `From` and `TryFrom` for the primitive integers.
- `NonZero<T>`: a value checked nonzero once, for divisors and moduli. A `NonZeroUint` of up to
  128 bits is a `core::num::NonZero`, so a division by it has no zero test. `Divisor` names the
  types it holds.
- Constant time: `ConstantTime<Uint>` and `ConstantTimeResidue` run every operation with no
  branch and no memory address that depends on the wrapped values.
- `num-traits`: every integer trait that needs no `Div<Self>` or `Rem<Self>`.
- [Benchmarks](https://github.com/sirosfoundation/siros-primitives/blob/main/crates/biguint/BENCHMARKS.md)
  against other big-integer crates.

## Usage

```rust
use biguint::typenum::U521;
use biguint::{U256, U512, Uint, WideningMul};

let top = U256::ONE << 255;
assert_eq!(top.widening_mul(top), U512::ONE << 510);
assert_eq!(top.checked_add(top), None);

let max: Uint<U521> = !Uint::ZERO;
assert_eq!(max.bit_len(), 521);
```

## Division and modular arithmetic

The remainder takes the divisor's width.

```rust
use biguint::{NonZeroUint, U128, U256};

let ten = NonZeroUint::new(U128::from_u64(10)).ok_or("zero")?;
assert_eq!(U256::from_u64(1234).div_rem(ten), (U256::from_u64(123), U128::from_u64(4)));
assert_eq!(U256::from_u64(1234).checked_div_rem(U128::ZERO), None);

let m = NonZeroUint::new(U256::from_u64(97)).ok_or("zero")?;
assert_eq!(U256::from_u64(90).add_mod(U256::from_u64(10), m), U256::from_u64(3));
assert_eq!(U256::from_u64(50).mul_mod(U256::from_u64(4), m), U256::from_u64(6));
# Ok::<(), &str>(())
```

## Prepared moduli

```rust
use biguint::{Modulus, U256};

let p = Modulus::new(U256::from_u64(97)).ok_or("zero")?;
let value = p.scope(|ring| {
    let a = ring.reduce(U256::from_u64(200));
    let b = ring.residue(U256::from_u64(90)).expect("below the modulus");
    (a * b + a.pow(U256::from_u64(3))).get()
});
assert_eq!(value, U256::from_u64((6 * 90 + 6 * 6 * 6) % 97));
# Ok::<(), &str>(())
```

## Constant time

Widths, moduli, divisors, shift amounts and public operands may steer the time.

```rust
use biguint::{ConstantTime, Modulus, NonZeroUint, U256};

let key = ConstantTime::new(U256::from_u64(7));
let m = NonZeroUint::new(U256::from_u64(97)).ok_or("zero")?;
assert_eq!(key.mul_mod(ConstantTime::new(U256::from_u64(20)), m).reveal(), U256::from_u64(43));
assert_eq!((key * U256::from_u64(3) + key).reveal(), U256::from_u64(28));

let power = Modulus::from(m).scope(|ring| {
    let base = ring.reduce_constant_time(key);
    base.pow(ConstantTime::new(U256::from_u64(3))).get().reveal()
});
assert_eq!(power, U256::from_u64(343 % 97));
# Ok::<(), &str>(())
```

## Cargo features

- `not-precomputed`: drops the 512-byte table that seeds the division reciprocal on targets other
  than x86-64.

## License

Licensed under either of [Apache License, Version 2.0](https://www.apache.org/licenses/LICENSE-2.0)
or [MIT license](https://opensource.org/licenses/MIT) at your option.
