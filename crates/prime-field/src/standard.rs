//! The moduli of standard fields, and aliases of their elements by curve.

use biguint::Uint;
use biguint::typenum::{U31, U64, U254, U256, U381, U521};

use crate::Modulus;

/// Declares one modulus marker per line: its documentation, name, width and little-endian limbs.
macro_rules! moduli {
    ($($(#[doc = $doc:literal])+ $name:ident: $width:ty = [$($limb:expr),+ $(,)?];)+) => {
        $(
            $(#[doc = $doc])+
            #[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
            pub struct $name;

            impl Modulus for $name {
                type Width = $width;
                const MODULUS: Uint<$width> = Uint::from_limbs([$($limb),+]);
            }
        )+
    };
}

moduli! {
    /// `2^31 - 1`, Mersenne-31.
    Mersenne31: U31 = [0x7fff_ffff];
    /// `15 · 2^27 + 1`, BabyBear.
    BabyBear: U31 = [0x7800_0001];
    /// `2^64 - 2^32 + 1`, Goldilocks.
    Goldilocks: U64 = [0xffff_ffff_0000_0001];
    /// The scalar field of BN254 (alt_bn128), the order of its groups.
    Bn254Scalar: U254 = [
        0x43e1_f593_f000_0001,
        0x2833_e848_79b9_7091,
        0xb850_45b6_8181_585d,
        0x3064_4e72_e131_a029,
    ];
    /// The base field of secp256k1 (SEC 2): `2^256 - 2^32 - 977`.
    Secp256k1Base: U256 = [
        0xffff_fffe_ffff_fc2f,
        0xffff_ffff_ffff_ffff,
        0xffff_ffff_ffff_ffff,
        0xffff_ffff_ffff_ffff,
    ];
    /// The base field of P-256 (FIPS 186): `2^256 - 2^224 + 2^192 + 2^96 - 1`.
    P256Base: U256 = [
        0xffff_ffff_ffff_ffff,
        0x0000_0000_ffff_ffff,
        0x0000_0000_0000_0000,
        0xffff_ffff_0000_0001,
    ];
    /// The scalar field of P-256 (FIPS 186), the order of its group.
    P256Scalar: U256 = [
        0xf3b9_cac2_fc63_2551,
        0xbce6_faad_a717_9e84,
        0xffff_ffff_ffff_ffff,
        0xffff_ffff_0000_0000,
    ];
    /// The base field of BLS12-381.
    Bls12381Base: U381 = [
        0xb9fe_ffff_ffff_aaab,
        0x1eab_fffe_b153_ffff,
        0x6730_d2a0_f6b0_f624,
        0x6477_4b84_f385_12bf,
        0x4b1b_a7b6_434b_acd7,
        0x1a01_11ea_397f_e69a,
    ];
    /// The base field of P-521 (FIPS 186): `2^521 - 1`.
    P521Base: U521 = [
        u64::MAX,
        u64::MAX,
        u64::MAX,
        u64::MAX,
        u64::MAX,
        u64::MAX,
        u64::MAX,
        u64::MAX,
        0x1ff,
    ];
    /// The base field of Tom-256, the prime-order curve whose group order is the P-256 base
    /// prime.
    Tom256Base: U256 = [
        0x9313_5661_b1c4_b117,
        0x7e72_b42b_30e7_3177,
        0x0000_0000_0000_0001,
        0xffff_ffff_0000_0001,
    ];
}

/// The scalar field of Tom-256: its group order is the P-256 base prime, so the two fields are
/// one type.
pub type Tom256Scalar = P256Base;

/// Tom-256.
pub mod tom256 {
    use crate::Fp;

    /// An element of the base field.
    pub type Base = Fp<super::Tom256Base>;
    /// An element of the scalar field, the base field of P-256.
    pub type Scalar = Fp<super::Tom256Scalar>;
}

/// P-256.
pub mod p256 {
    use crate::Fp;

    /// An element of the base field.
    pub type Base = Fp<super::P256Base>;
    /// An element of the scalar field.
    pub type Scalar = Fp<super::P256Scalar>;
}

/// secp256k1.
pub mod secp256k1 {
    use crate::Fp;

    /// An element of the base field.
    pub type Base = Fp<super::Secp256k1Base>;
}

/// BN254.
pub mod bn254 {
    use crate::Fp;

    /// An element of the scalar field.
    pub type Scalar = Fp<super::Bn254Scalar>;
}

/// BLS12-381.
pub mod bls12_381 {
    use crate::Fp;

    /// An element of the base field.
    pub type Base = Fp<super::Bls12381Base>;
}

/// P-521.
pub mod p521 {
    use crate::Fp;

    /// An element of the base field.
    pub type Base = Fp<super::P521Base>;
}
