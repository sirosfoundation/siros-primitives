# SIROS primitives

[![CI](https://github.com/sirosfoundation/siros-primitives/actions/workflows/ci.yml/badge.svg)](https://github.com/sirosfoundation/siros-primitives/actions/workflows/ci.yml)
![Apache-2.0/MIT licensed](https://img.shields.io/badge/license-Apache--2.0%2FMIT-blue.svg)
![Rust 1.93+](https://img.shields.io/badge/rustc-1.93+-blue.svg)

Big-integer, prime-field, carry-less and binary-field arithmetic in pure Rust. Every crate is
`no_std` and free of allocation and panics. Rust 1.93 or newer.

| Crate | Description |
|---|---|
| [`biguint`](crates/biguint) | Fixed-width unsigned integers of any bit width |
| [`prime-field`](crates/prime-field) | Prime fields declared by their modulus; `Fp` is variable-time, `Secret` constant-time |
| [`clmul`](crates/clmul) | Carry-less multiplication of words, limb slices and GF(2^128) elements |
| [`binary-field`](crates/binary-field) | Binary fields GF(2^n) of any degree, declared by their polynomial |

`bench-compare`, the `*-bench-report` tools and `ct-check` are unpublished tools.

## Development

```sh
cargo test --workspace --all-features
RUSTFLAGS="-C target-feature=+pclmulqdq,+sse4.1" cargo test -p clmul -p binary-field --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo run -p ct-check   # constant time under Valgrind's memcheck
```

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or
[MIT license](LICENSE-MIT) at your option. Unless you state otherwise, a contribution you submit
is dual licensed as above.
