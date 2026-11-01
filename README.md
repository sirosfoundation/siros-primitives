# siros-primitives

Big-integer primitives in Rust: `no_std`, no `unsafe`, no panics.

| Crate | Contents |
|---|---|
| [`biguint`](crates/biguint) | `Uint<N>`: an unsigned integer of exactly `N` bits for any `typenum` width. Wrapping arithmetic, carrying add and subtract, widening multiplication, division by a divisor of any width, modular multiplication, shifts, rotations, comparisons, byte encodings and exact conversions. |

## Usage

```rust
use biguint::typenum::U521;
use biguint::{U256, U512, Uint, WideningMul};

let top = U256::ONE << 255usize;
assert_eq!(U256::from_u64(3) * top, top);
assert_eq!(top.widening_mul(top), U512::ONE << 510usize);
assert_eq!(U256::from_u64(7).mul_mod(U256::from_u64(9), U256::from_u64(10)), Some(U256::from_u64(3)));

let max521: Uint<U521> = !Uint::ZERO;
assert_eq!(max521.bit_len(), 521);
```

Arithmetic wraps modulo `2^N`. Builds with the `adx` target feature (for example
`-C target-cpu=native` on a recent x86-64) run the carry chain on the `adc`/`sbb` intrinsics.

The width selects the algorithm at compile time. Products of 16 limbs (1024 bits) and more use
Karatsuba; divisions and modular products by a divisor of 24 limbs (1536 bits) or more that fills
its width use divide-and-conquer division.

## Features

- `precomputed` (default): a 512-byte table seeds the reciprocal that every division and
  modular multiplication starts from. Without it, a 32-bit division computes the seed.

## Build

```sh
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

`rust-toolchain.toml` pins the compiler. The minimum supported Rust version is 1.88.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in
the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without
any additional terms or conditions.
