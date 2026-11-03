# SIROS primitives

*The fastest primitives in the west!*

[![CI](https://github.com/sirosfoundation/siros-primitives/actions/workflows/ci.yml/badge.svg)](https://github.com/sirosfoundation/siros-primitives/actions/workflows/ci.yml)
![Apache-2.0/MIT licensed](https://img.shields.io/badge/license-Apache--2.0%2FMIT-blue.svg)
![Rust 1.93+](https://img.shields.io/badge/rustc-1.93+-blue.svg)

Big-integer, prime-field, carry-less and binary-field arithmetic in pure Rust. `no_std`, no
allocation, no panics. Rust 1.93 or newer.

| Crate | Description |
|---|---|
| [`constant-time`](crates/constant-time) | `ConstantTime<T>`, the constant-time wrapper of every crate |
| [`biguint`](crates/biguint) | Fixed-width unsigned integers of any bit width |
| [`prime-field`](crates/prime-field) | Prime fields declared by their modulus |
| [`clmul`](crates/clmul) | Carry-less multiplication of words, limb slices and GF(2^128) elements |
| [`binary-field`](crates/binary-field) | Binary fields GF(2^n) of any degree, declared by their polynomial |

## Development

```sh
cargo test --workspace --all-features
RUSTFLAGS="-C target-feature=+pclmulqdq,+sse4.1" cargo test -p clmul -p binary-field --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo run -p ct-check                        # constant time under Valgrind's memcheck
cargo run -p biguint-bench-report -- bench    # measure; likewise prime-field- and binary-field-bench-report
cargo run -p biguint-bench-report -- report   # write BENCHMARKS.md from the record
```

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or
[MIT license](LICENSE-MIT) at your option. Unless you state otherwise, a contribution you submit
is dual licensed as above.
