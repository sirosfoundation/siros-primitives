# biguint

[![CI](https://github.com/sirosfoundation/siros-primitives/actions/workflows/ci.yml/badge.svg)](https://github.com/sirosfoundation/siros-primitives/actions/workflows/ci.yml)
![Apache-2.0/MIT licensed](https://img.shields.io/badge/license-Apache--2.0%2FMIT-blue.svg)
![Rust 1.93+](https://img.shields.io/badge/rustc-1.93+-blue.svg)

Fixed-width unsigned integers of any bit width, in pure Rust.

## Features

- `Uint<N>` for any `typenum` width `N`: `U521` is 521 bits, in `ceil(N / 64)` little-endian `u64`
  limbs.
- Every operation folds to its compile-time width: unrolled carry chains for small widths,
  Karatsuba and divide-and-conquer division for large ones.
- Wrapping, carrying and widening arithmetic, shifts, rotations, comparisons, byte encodings and
  exact conversions between widths.
- Division by a `NonZeroUint` of any width: no failure path, no panic.
- Modular arithmetic on operands of any value, and a prepared `Modulus` with branded residues,
  Montgomery for odd moduli.
- Constant time on request: `ConstantTime<Uint>` and `ConstantTimeResidue`. CI checks every such
  operation under Valgrind with [`ct-check`](https://github.com/sirosfoundation/siros-primitives/tree/main/crates/ct-check).
- `num-traits`: every integer trait that does not need `Div<Self>` or `Rem<Self>`, which could
  panic.
- `no_std`, no `unsafe`, no allocation, no panics. A 4096-bit `*`, `div_rem` or `mul_mod` uses at
  most 7.9 KiB of stack.
- [Benchmarks](https://github.com/sirosfoundation/siros-primitives/blob/main/crates/biguint/BENCHMARKS.md)
  against other big-integer crates.

## Arithmetic

```rust
use biguint::typenum::U521;
use biguint::{U256, U512, Uint, WideningMul};

let top = U256::ONE << 255;
assert_eq!(top.widening_mul(top), U512::ONE << 510);

let max521: Uint<U521> = !Uint::ZERO;
assert_eq!(max521.bit_len(), 521);
```

## Division and modular arithmetic

The divisor's width sets the remainder's width. Modular operations accept operands of any value.

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

A `Modulus` prepares a modulus once. Residues of different scopes fail to compile together.

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

Operations default to the fastest path. `ConstantTime` runs them with no branch and no memory
address that depends on the wrapped values. Widths, moduli, divisors, shift amounts and public
operands may steer the time.

```rust
use biguint::{ConstantTime, Modulus, NonZeroUint, U256};

let key = ConstantTime::new(U256::from_u64(7));
let m = NonZeroUint::new(U256::from_u64(97)).ok_or("zero")?;
assert_eq!(key.mul_mod(ConstantTime::new(U256::from_u64(20)), m).reveal(), U256::from_u64(43));

let power = Modulus::from(m).scope(|ring| {
    let base = ring.reduce_constant_time(key);
    base.pow(ConstantTime::new(U256::from_u64(3))).get().reveal()
});
assert_eq!(power, U256::from_u64(343 % 97));
# Ok::<(), &str>(())
```

## Cargo features

- `not-precomputed`: drops the 512-byte table that seeds the division reciprocal on targets other
  than x86-64. One dependent enabling it decides the whole build.

## License

Licensed under either of [Apache License, Version 2.0](https://www.apache.org/licenses/LICENSE-2.0)
or [MIT license](https://opensource.org/licenses/MIT) at your option.
