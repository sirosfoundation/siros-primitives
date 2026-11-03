# constant-time

[![CI](https://github.com/sirosfoundation/siros-primitives/actions/workflows/ci.yml/badge.svg)](https://github.com/sirosfoundation/siros-primitives/actions/workflows/ci.yml)
![Apache-2.0/MIT licensed](https://img.shields.io/badge/license-Apache--2.0%2FMIT-blue.svg)
![Rust 1.93+](https://img.shields.io/badge/rustc-1.93+-blue.svg)

A zero-cost wrapper whose operations take the same path for every wrapped value, in Rust: `no_std`,
no dependencies, no `unsafe`, no allocation.

## Features

- `ConstantTime<T>`: holds a `T` in the same memory. `new` and `reveal` cost nothing.
- Its operations run with no branch and no memory address that depends on the wrapped values.
  Widths, moduli, divisors, shift amounts and public operands may steer the time.
- Operators: `+ - * & | ^` and their assigning forms, unary `-`, `!`, and `<<` and `>>` by a
  public amount, between two wrapped values or with a `Public` value on either side.
- Methods, with nothing to import: `square`, `double`, `pow`, `invert`, `sqrt`, `add_mod`,
  `sub_mod`, `neg_mod`, `mul_mod`, `div_rem` and `get`.
- `From` wraps every `Public` value that converts into `T`.
- No comparison and no formatting. A value leaves only through `reveal`, and through whether
  `invert`, or a `sqrt` that returns an `Option`, found a result.
- `biguint`, `prime-field` and `binary-field` implement it for their types and re-export it.

## Wrapping your own type

Implement the `Ct*` trait of each operation, in time that depends on no wrapped value.

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
| `<<`, `>>` | `CtShl`, `CtShr` |
| `/`, `%`, `div_rem` | `CtDivRem` |
| `square`, `pow`, `invert`, `sqrt` | `CtSquare`, `CtPow`, `CtInvert`, `CtSqrt` |
| `double` | `CtAdd` |
| `add_mod`, `sub_mod`, `neg_mod`, `mul_mod` | `CtAddMod`, `CtSubMod`, `CtNegMod`, `CtMulMod` |
| `get` | `CtGet` |

## License

Licensed under either of [Apache License, Version 2.0](https://www.apache.org/licenses/LICENSE-2.0)
or [MIT license](https://opensource.org/licenses/MIT) at your option.
