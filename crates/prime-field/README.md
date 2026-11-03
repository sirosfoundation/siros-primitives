# prime-field

[![CI](https://github.com/sirosfoundation/siros-primitives/actions/workflows/ci.yml/badge.svg)](https://github.com/sirosfoundation/siros-primitives/actions/workflows/ci.yml)
![Apache-2.0/MIT licensed](https://img.shields.io/badge/license-Apache--2.0%2FMIT-blue.svg)
![Rust 1.93+](https://img.shields.io/badge/rustc-1.93+-blue.svg)

Prime fields declared by their modulus, from 31 to thousands of bits. `no_std`, no `unsafe`, no
allocation, no panics.

```rust
use prime_field::standard::secp256k1::Base as Fq;
use prime_field::ConstantTime;

// Variable time: the fastest path.
let x = Fq::from_u64(7);
let y = x.invert().ok_or("zero")?;
assert_eq!(x * y, Fq::ONE);
assert_eq!(Fq::from_u64(4).sqrt().map(|r| r.square()), Some(Fq::from_u64(4)));

// Constant time: no branch or memory address depends on the wrapped values.
let k = ConstantTime::new(x);
assert_eq!((k * y + k.square()).reveal(), Fq::from_u64(50));

// Const contexts.
const FORTY_NINE: Fq = Fq::from_u64(7).const_mul(Fq::from_u64(7));
assert_eq!(FORTY_NINE, Fq::from_u64(49));
# Ok::<(), &str>(())
```

```rust
use prime_field::biguint::{typenum::U255, Uint};
use prime_field::{Fp, Modulus};

// A field of your own: 2^255 - 19. The representation follows from the modulus.
#[derive(Clone, Copy)]
struct Curve25519Base;

impl Modulus for Curve25519Base {
    type Width = U255;
    const MODULUS: Uint<U255> = Uint::from_limbs([
        0xffff_ffff_ffff_ffed,
        0xffff_ffff_ffff_ffff,
        0xffff_ffff_ffff_ffff,
        0x7fff_ffff_ffff_ffff,
    ]);
}

let two = Fp::<Curve25519Base>::from_u64(2);
assert_eq!(two.invert().map(|half| half.double()), Some(Fp::ONE));
```

`prime_field::standard` holds Mersenne-31, BabyBear, Goldilocks, BN254, secp256k1, P-256,
Tom-256, BLS12-381 and P-521.

[Benchmarks](https://github.com/sirosfoundation/siros-primitives/blob/main/crates/prime-field/BENCHMARKS.md) against other prime-field crates.

## License

Licensed under either of [Apache License, Version 2.0](https://www.apache.org/licenses/LICENSE-2.0)
or [MIT license](https://opensource.org/licenses/MIT) at your option.
