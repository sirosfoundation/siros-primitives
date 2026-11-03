# biguint

[![CI](https://github.com/sirosfoundation/siros-primitives/actions/workflows/ci.yml/badge.svg)](https://github.com/sirosfoundation/siros-primitives/actions/workflows/ci.yml)
![Apache-2.0/MIT licensed](https://img.shields.io/badge/license-Apache--2.0%2FMIT-blue.svg)
![Rust 1.93+](https://img.shields.io/badge/rustc-1.93+-blue.svg)

Fixed-width unsigned integers of any bit width. `no_std`, no `unsafe`, no allocation, no panics.

```rust
use biguint::typenum::U521;
use biguint::{NonZeroUint, U128, U256, U512, Uint, WideningMul};

let top = U256::ONE << 255;
assert_eq!(top.widening_mul(top), U512::ONE << 510);

let max521: Uint<U521> = !Uint::ZERO;
assert_eq!(max521.bit_len(), 521);

// Division takes a nonzero divisor of any width; the remainder has its width.
let ten = NonZeroUint::new(U128::from_u64(10)).ok_or("zero")?;
assert_eq!(U256::from_u64(1234).div_rem(ten), (U256::from_u64(123), U128::from_u64(4)));
assert_eq!(U256::from_u64(1234).checked_div_rem(U128::ZERO), None);

// Modular arithmetic on operands of any value.
let m = NonZeroUint::new(U256::from_u64(97)).ok_or("zero")?;
assert_eq!(U256::from_u64(90).add_mod(U256::from_u64(10), m), U256::from_u64(3));
assert_eq!(U256::from_u64(50).mul_mod(U256::from_u64(4), m), U256::from_u64(6));
# Ok::<(), &str>(())
```

```rust
use biguint::{ConstantTime, Modulus, U256};

// A prepared modulus: Montgomery for odd moduli. Residues of two scopes do not compile together.
let p = Modulus::new(U256::from_u64(97)).ok_or("zero")?;
let value = p.scope(|ring| {
    let a = ring.reduce(U256::from_u64(200));
    let b = ring.residue(U256::from_u64(90)).expect("below the modulus");
    (a * b + a.pow(U256::from_u64(3))).get()
});
assert_eq!(value, U256::from_u64((6 * 90 + 6 * 6 * 6) % 97));

// Constant time: no branch or memory address depends on the wrapped values.
let key = ConstantTime::new(U256::from_u64(7));
let power = p.scope(|ring| {
    let base = ring.reduce_constant_time(key);
    base.pow(ConstantTime::new(U256::from_u64(3))).get().reveal()
});
assert_eq!(power, U256::from_u64(343 % 97));
# Ok::<(), &str>(())
```

`Uint` implements the `num-traits` integer traits except those that need `Div<Self>` or
`Rem<Self>`, which could panic.

## Features

- `not-precomputed`: drops the 512-byte reciprocal table on targets other than x86-64.

[Benchmarks](https://github.com/sirosfoundation/siros-primitives/blob/main/crates/biguint/BENCHMARKS.md) against other big-integer crates.

## License

Licensed under either of [Apache License, Version 2.0](https://www.apache.org/licenses/LICENSE-2.0)
or [MIT license](https://opensource.org/licenses/MIT) at your option.
