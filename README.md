# siros-primitives

Big-integer primitives in Rust: `no_std`, no `unsafe`, no panics.

| Crate | Contents |
|---|---|
| [`siros-primitives-core`](crates/siros-primitives-core) | `Uint<N>`: an unsigned integer of exactly `N` bits for any `typenum` width. Wrapping arithmetic, carrying add and subtract, widening multiplication, division, shifts, rotations, comparisons, byte encodings and exact conversions. |

## Usage

```rust
use siros_primitives_core::typenum::U521;
use siros_primitives_core::{U256, U512, Uint, WideningMul};

let top = U256::ONE << 255usize;
assert_eq!(U256::from_u64(3) * top, top);
assert_eq!(top.widening_mul(top), U512::ONE << 510usize);

let max521: Uint<U521> = !Uint::ZERO;
assert_eq!(max521.bit_len(), 521);
```

Arithmetic wraps modulo `2^N`. Builds with the `adx` target feature (for example
`-C target-cpu=native` on a recent x86-64) run the carry chain on the `adc`/`sbb` intrinsics.

## Build

```sh
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

`rust-toolchain.toml` pins the compiler. The minimum supported Rust version is 1.85.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in
the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without
any additional terms or conditions.
