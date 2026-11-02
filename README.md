# SIROS primitives

[![CI](https://github.com/sirosfoundation/siros-primitives/actions/workflows/ci.yml/badge.svg)](https://github.com/sirosfoundation/siros-primitives/actions/workflows/ci.yml)
![Apache-2.0/MIT licensed](https://img.shields.io/badge/license-Apache--2.0%2FMIT-blue.svg)
![Rust 1.88+](https://img.shields.io/badge/rustc-1.88+-blue.svg)

Big-integer and carry-less arithmetic in pure Rust. Every crate is `no_std` and free of allocation
and panics. `unsafe` appears only in `clmul`, at the calls into its PCLMULQDQ and PMULL backends.

## Crates

| Crate | Description |
|---|---|
| [`biguint`](crates/biguint) | Fixed-width unsigned integers of any bit width, with Karatsuba multiplication and divide-and-conquer division |
| [`clmul`](crates/clmul) | Carry-less multiplication of 64-bit words, limb slices and GF(2^128) elements, on PCLMULQDQ, PMULL or a const portable comb |

## Minimum supported Rust version

Every crate builds with Rust 1.88 or newer.

## Development

```sh
cargo test --workspace --all-features
RUSTFLAGS="-C target-feature=+pclmulqdq,+sse4.1" cargo test -p clmul --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

## License

All crates in this repository are licensed under either of

- [Apache License, Version 2.0](LICENSE-APACHE)
- [MIT license](LICENSE-MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in
the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
