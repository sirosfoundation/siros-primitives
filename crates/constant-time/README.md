# constant-time

[![CI](https://github.com/sirosfoundation/siros-primitives/actions/workflows/ci.yml/badge.svg)](https://github.com/sirosfoundation/siros-primitives/actions/workflows/ci.yml)
![Apache-2.0/MIT licensed](https://img.shields.io/badge/license-Apache--2.0%2FMIT-blue.svg)
![Rust 1.93+](https://img.shields.io/badge/rustc-1.93+-blue.svg)

`ConstantTime<T>`: operations on the wrapped value whose branches and memory addresses never
depend on it. `no_std`, no dependencies, no `unsafe`, no allocation.

`biguint`, `prime-field` and `binary-field` implement it for their types and re-export it.
Implement the `Ct*` trait of each operation to wrap your own type:

```rust
use constant_time::{ConstantTime, CtAdd, CtSquare, Public};

#[derive(Clone, Copy)]
struct Word(u64);

// A public value may meet a wrapped one in an operator.
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

Public values may steer the time: widths, moduli, divisors, shift amounts and public operands.
`invert` and an `Option` square root disclose whether the result exists.

## License

Licensed under either of [Apache License, Version 2.0](https://www.apache.org/licenses/LICENSE-2.0)
or [MIT license](https://opensource.org/licenses/MIT) at your option.
