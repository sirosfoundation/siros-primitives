# binary-field

Binary Galois fields `GF(2^n)` in pure Rust: `no_std`, without `unsafe`, allocation or panics.

## Example

```rust
use binary_field::{Gf2p8, Gf2p128, Pow};

let a = Gf2p8::from_u64(0x53).unwrap();
assert_eq!(a.invert().unwrap(), Gf2p8::from_u64(0xCA).unwrap());

let x = Gf2p128::X;
assert_eq!(x.pow(128u32), Gf2p128::from_u64(0x87).unwrap());
```
