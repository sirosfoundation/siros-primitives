# prime-field

Prime fields declared by their modulus. One `impl Modulus` per field; every other constant and
every strategy choice derives from the modulus at compile time.

## Example

```rust
use prime_field::biguint::{U256, Uint};
use prime_field::{Fp, Modulus};

#[derive(Clone, Copy)]
struct Secp256k1Base;

impl Modulus for Secp256k1Base {
    type Width = prime_field::biguint::typenum::U256;
    const MODULUS: U256 = U256::from_limbs([
        0xffff_fffe_ffff_fc2f,
        0xffff_ffff_ffff_ffff,
        0xffff_ffff_ffff_ffff,
        0xffff_ffff_ffff_ffff,
    ]);
}

type Fq = Fp<Secp256k1Base>;

let x = Fq::from_u64(7);
let y = x.invert().ok_or("zero")?;
assert_eq!(x * y, Fq::ONE);
let four = Fq::from_u64(4);
assert_eq!(four.sqrt().map(|root| root.square()), Some(four));
assert_eq!(x.sqrt(), None);
assert_eq!(Fq::from_uint(Fq::MODULUS), Err(prime_field::NotCanonical));
assert_eq!((x - x).to_uint(), Uint::ZERO);
# Ok::<(), &str>(())
```
