# constant-time

[![CI](https://github.com/sirosfoundation/siros-primitives/actions/workflows/ci.yml/badge.svg)](https://github.com/sirosfoundation/siros-primitives/actions/workflows/ci.yml)
![Apache-2.0/MIT licensed](https://img.shields.io/badge/license-Apache--2.0%2FMIT-blue.svg)
![Rust 1.93+](https://img.shields.io/badge/rustc-1.93+-blue.svg)

`ConstantTime<T>`: a wrapper whose operations run in constant time over the wrapped values. No
branch and no memory address depends on them. `no_std`, without dependencies, `unsafe` or
allocation. Rust 1.93 or newer.

`ConstantTime<T>` holds a `T` in the same memory; `ConstantTime::new` and `ConstantTime::reveal`
cost nothing. Its operators and methods exist for each `T` that implements the matching trait of
this crate:

| `ConstantTime` | trait on `T` |
|---|---|
| `+ - * & \| ^` and their assigning forms | `CtAdd`, `CtSub`, `CtMul`, `CtBitAnd`, `CtBitOr`, `CtBitXor` |
| unary `-`, `!` | `CtNeg`, `CtNot` |
| `<<`, `>>` by a public amount | `CtShl`, `CtShr` |
| `/`, `%`, `div_rem` by a public divisor | `CtDivRem` |
| `square`, `pow`, `invert`, `sqrt` | `CtSquare`, `CtPow`, `CtInvert`, `CtSqrt` |
| `double` | `CtAdd` |
| `add_mod`, `sub_mod`, `neg_mod`, `mul_mod` by a public modulus | `CtAddMod`, `CtSubMod`, `CtNegMod`, `CtMulMod` |
| `get` | `CtGet` |

An operator combines two `ConstantTime` values, or a `ConstantTime` and a value whose type
implements `Public`. An operator with a `Public` value on the left is implemented by the crate of
that value's type. `From` wraps every `Public` value that converts into `T`.

A `Ct*` implementation runs in time that depends on no value of `self` and of its other wrapped
operands. Widths, moduli, divisors, shift amounts, public operands and the exponents of a `CtPow`
over a public type may steer it. `CtInvert`, and a `CtSqrt` whose root is an `Option`, disclose
whether the result exists. `ConstantTime` implements no comparison and no formatting.

`biguint` implements the traits for `Uint` and `Residue`, `prime-field` for `Fp` and
`binary-field` for `Gf2`. Each re-exports `ConstantTime`.

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
