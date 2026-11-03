# constant-time

[![CI](https://github.com/sirosfoundation/siros-primitives/actions/workflows/ci.yml/badge.svg)](https://github.com/sirosfoundation/siros-primitives/actions/workflows/ci.yml)
![Apache-2.0/MIT licensed](https://img.shields.io/badge/license-Apache--2.0%2FMIT-blue.svg)
![Rust 1.93+](https://img.shields.io/badge/rustc-1.93+-blue.svg)

A zero-cost wrapper for constant-time arithmetic, shared by the SIROS primitives.

## Features

- `ConstantTime<T>` holds a `T` in the same memory; `new` and `reveal` cost nothing.
- Its operations run with no branch and no memory address that depends on the wrapped values.
- Operators combine two wrapped values, or a wrapped value and a `Public` one on either side.
- Modular operations, division by a public divisor, `square`, `pow`, `invert` and `sqrt` are
  inherent methods: nothing to import.
- `biguint`, `prime-field` and `binary-field` implement it for their types and re-export it.
- No comparison and no formatting: nothing reveals a value by accident.
- `no_std`, no dependencies, no `unsafe`, no allocation.

## Usage

Use it through `biguint`, `prime-field` or `binary-field`. To wrap your own type, implement
the `Ct*` trait of each operation, in time that depends on no wrapped value. Widths,
moduli, divisors, shift amounts and public operands may steer the time.

```rust
use constant_time::{ConstantTime, CtAdd, CtSquare, Public};

#[derive(Clone, Copy)]
struct Word(u64);

impl Public for Word {}

impl CtAdd for Word {
    type Output = Self;
    fn ct_add(self, rhs: Self) -> Self {
        Self(self.0.wrapping_add(rhs.0))
    }
}

impl CtSquare for Word {
    fn ct_square(self) -> Self {
        Self(self.0.wrapping_mul(self.0))
    }
}

let key = ConstantTime::new(Word(7));
let sum = (key + Word(2)).square() + key;
assert_eq!(sum.reveal().0, 88);
```

| `ConstantTime` | Trait on `T` |
|---|---|
| `+ - * & \| ^` and their assigning forms | `CtAdd`, `CtSub`, `CtMul`, `CtBitAnd`, `CtBitOr`, `CtBitXor` |
| unary `-`, `!` | `CtNeg`, `CtNot` |
| `<<`, `>>` by a public amount | `CtShl`, `CtShr` |
| `/`, `%`, `div_rem` by a public divisor | `CtDivRem` |
| `square`, `pow`, `invert`, `sqrt` | `CtSquare`, `CtPow`, `CtInvert`, `CtSqrt` |
| `double` | `CtAdd` |
| `add_mod`, `sub_mod`, `neg_mod`, `mul_mod` | `CtAddMod`, `CtSubMod`, `CtNegMod`, `CtMulMod` |
| `get` | `CtGet` |

`invert`, and a `sqrt` whose root is an `Option`, disclose whether the result exists.

## License

Licensed under either of [Apache License, Version 2.0](https://www.apache.org/licenses/LICENSE-2.0)
or [MIT license](https://opensource.org/licenses/MIT) at your option.
