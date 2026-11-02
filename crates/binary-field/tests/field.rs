//! Oracles independent of the crate: a bit-serial multiply-and-reduce over limb vectors, the field
//! axioms, and published test vectors.

use binary_field::generic_array::GenericArray;
use binary_field::typenum::{U8, U16, U17, U41, U64, U127, U128, U163};
use binary_field::{
    Aes, B233, B571, Gcm, Gf2, Gf2p8, Gf2p128, Poly16, Poly64, Poly256, Polynomial, Pow, Secret,
    Square,
};

/// Declares the field of `x^degree + sum(x^tap)`.
macro_rules! field {
    ($name:ident, $degree:ty, [$($tap:literal),*]) => {
        struct $name;
        impl Polynomial for $name {
            type Degree = $degree;
            const TAPS: &'static [usize] = &[$($tap),*];
        }
    };
}

// Irreducible polynomials beyond the standard fields, one per plan shape, most with taps above
// half the degree, whose reductions take more than one fold.
field!(F8High, U8, [7, 6, 1, 0]);
field!(F16High, U16, [12, 10, 3, 0]);
field!(F17, U17, [3, 0]);
field!(F41, U41, [3, 0]);
field!(F64High, U64, [40, 39, 33, 0]);
field!(F127High, U127, [97, 0]);
field!(F128Tap63, U128, [63, 62, 17, 0]);
field!(F163, U163, [7, 6, 3, 0]);

/// A splitmix64 stream, so that every run sees the same operands.
struct Stream(u64);

impl Stream {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    fn element<P: Polynomial>(&mut self) -> Gf2<P> {
        let mut limbs = GenericArray::<u64, binary_field::Limbs<P>>::default();
        for limb in limbs.iter_mut() {
            *limb = self.next();
        }
        let degree = Gf2::<P>::DEGREE;
        if !degree.is_multiple_of(64) {
            limbs[degree / 64] &= (1 << (degree % 64)) - 1;
        }
        Gf2::from_limbs(limbs).unwrap()
    }
}

fn bit(limbs: &[u64], index: usize) -> bool {
    (limbs[index / 64] >> (index % 64)) & 1 == 1
}

fn flip(limbs: &mut [u64], index: usize) {
    limbs[index / 64] ^= 1 << (index % 64);
}

/// `a * b` in `GF(2)[x] / (x^n + sum(x^t for t in taps))`, bit by bit.
fn reference_mul(a: &[u64], b: &[u64], degree: usize, taps: &[usize]) -> Vec<u64> {
    let mut product = vec![0u64; (2 * degree).div_ceil(64) + 1];
    for i in 0..degree {
        if bit(b, i) {
            for j in 0..degree {
                if bit(a, j) {
                    flip(&mut product, i + j);
                }
            }
        }
    }
    for d in (degree..2 * degree).rev() {
        if bit(&product, d) {
            flip(&mut product, d);
            for &tap in taps {
                flip(&mut product, d - degree + tap);
            }
        }
    }
    product.truncate(degree.div_ceil(64));
    product
}

fn check_field<P: Polynomial>(rounds: usize) {
    let mut stream = Stream(Gf2::<P>::DEGREE as u64);
    let degree = Gf2::<P>::DEGREE;
    for _ in 0..rounds {
        let a = stream.element::<P>();
        let b = stream.element::<P>();
        let expected = reference_mul(&a.into_limbs(), &b.into_limbs(), degree, P::TAPS);
        assert_eq!(
            (a * b).into_limbs().as_slice(),
            &expected[..],
            "mul at degree {degree}"
        );
        check_secret(a, b);
        assert_eq!(
            Gf2::from_poly(a.into_poly()),
            a,
            "poly round trip at degree {degree}"
        );
        assert_eq!(a.square(), a * a, "square at degree {degree}");
        assert_eq!(a.sqrt().square(), a, "sqrt at degree {degree}");
        assert_eq!(
            a.sqrt() + b.sqrt(),
            (a + b).sqrt(),
            "sqrt is additive at degree {degree}"
        );
        if a.is_zero() {
            assert_eq!(a.invert(), None);
        } else {
            assert_eq!(
                a * a.invert().unwrap(),
                Gf2::<P>::ONE,
                "inverse at degree {degree}"
            );
        }
        assert_eq!(
            a.trace() ^ b.trace(),
            (a + b).trace(),
            "trace is additive at degree {degree}"
        );
        assert_eq!(
            a.trace(),
            frobenius_sum(a),
            "trace is the Frobenius sum at degree {degree}"
        );
        assert_eq!(a.times_x(), a * Gf2::<P>::X, "times_x at degree {degree}");
        assert_eq!(a.over_x() * Gf2::<P>::X, a, "over_x at degree {degree}");
        assert_eq!(
            Gf2::<P>::from_le_bytes(&a.to_le_bytes()).unwrap(),
            a,
            "little-endian bytes at degree {degree}"
        );
        assert_eq!(
            Gf2::<P>::from_be_bytes(&a.to_be_bytes()).unwrap(),
            a,
            "big-endian bytes at degree {degree}"
        );
    }
    assert_eq!(
        Gf2::<P>::ONE.trace(),
        degree % 2 == 1,
        "trace of 1 at degree {degree}"
    );
    assert_eq!(Gf2::<P>::ZERO.invert(), None);
    assert_eq!(Gf2::<P>::ONE.invert(), Some(Gf2::<P>::ONE));
}

/// The operations of `Secret`, which run other algorithms than the defaults of `Gf2` for the fields
/// with tables, against the defaults, which the caller checks against the references.
fn check_secret<P: Polynomial>(a: Gf2<P>, b: Gf2<P>) {
    let (x, y) = (Secret::new(a), Secret::new(b));
    let degree = Gf2::<P>::DEGREE;
    assert_eq!((x * y).reveal(), a * b, "secret mul at degree {degree}");
    assert_eq!(
        (x * b).reveal(),
        a * b,
        "secret by public at degree {degree}"
    );
    assert_eq!(
        (a * y).reveal(),
        a * b,
        "public by secret at degree {degree}"
    );
    assert_eq!((x + y).reveal(), a + b, "secret add at degree {degree}");
    assert_eq!((x - b).reveal(), a - b, "secret sub at degree {degree}");
    assert_eq!(
        x.square().reveal(),
        a.square(),
        "secret square at degree {degree}"
    );
    assert_eq!(
        x.sqrt().reveal(),
        a.sqrt(),
        "secret sqrt at degree {degree}"
    );
    assert_eq!(
        x.invert().map(Secret::reveal),
        a.invert(),
        "secret inverse at degree {degree}"
    );
}

/// `a + a^2 + ... + a^(2^(n-1))`, read back as a bit of GF(2).
fn frobenius_sum<P: Polynomial>(a: Gf2<P>) -> bool {
    let mut sum = Gf2::<P>::ZERO;
    let mut power = a;
    for _ in 0..Gf2::<P>::DEGREE {
        sum += power;
        power = power.square();
    }
    assert!(sum == Gf2::<P>::ZERO || sum == Gf2::<P>::ONE);
    sum == Gf2::<P>::ONE
}

#[test]
fn fields_match_the_bit_serial_reference() {
    // Unoptimized builds inline every operation into one frame per field, which outgrows the
    // 2 MiB stack of a test thread.
    std::thread::Builder::new()
        .stack_size(16 << 20)
        .spawn(|| {
            check_field::<Aes>(200);
            check_field::<Poly16>(200);
            check_field::<Poly64>(200);
            check_field::<Gcm>(200);
            check_field::<B233>(60);
            check_field::<Poly256>(60);
            check_field::<B571>(20);
            check_field::<F8High>(200);
            check_field::<F16High>(200);
            check_field::<F17>(100);
            check_field::<F41>(100);
            check_field::<F64High>(60);
            check_field::<F127High>(40);
            check_field::<F128Tap63>(40);
            check_field::<F163>(20);
        })
        .unwrap()
        .join()
        .unwrap();
}

#[test]
fn x_to_the_degree_is_r() {
    assert_eq!(Gf2p8::X.pow(8u32), Gf2p8::from_u64(0b1_1011).unwrap());
    assert_eq!(Gf2p128::X.pow(128u64), Gf2p128::from_u64(0x87).unwrap());
    let a = Stream(571).element::<B571>();
    let order = Gf2::<B571>::ONE.into_uint();
    assert_eq!(a.pow(Gf2::<B571>::X.into_uint() * order), a * a);
}

#[test]
fn aes_inverses_match_the_table() {
    // FIPS 197, section 4.2 and the S-box: {53}^-1 = {CA}; {02}^-1 = {8D}.
    let inverse = |value: u64| Gf2p8::from_u64(value).unwrap().invert().unwrap();
    assert_eq!(inverse(0x53), Gf2p8::from_u64(0xCA).unwrap());
    assert_eq!(inverse(0x02), Gf2p8::from_u64(0x8D).unwrap());
    assert_eq!(inverse(0x01), Gf2p8::ONE);
}

#[test]
fn values_at_or_above_the_degree_are_refused() {
    assert!(Gf2p8::from_u64(0xFF).is_ok());
    assert!(Gf2p8::from_u64(0x100).is_err());
    assert!(Gf2::<B233>::from_u128(1 << 100).is_ok());
    let mut top = GenericArray::<u64, binary_field::Limbs<B233>>::default();
    top[3] = 1 << 41;
    assert!(Gf2::<B233>::from_limbs(top).is_err());
    top[3] = 1 << 40;
    assert!(Gf2::<B233>::from_limbs(top).is_ok());
    let mut bytes = GenericArray::<u8, binary_field::Bytes<B233>>::default();
    bytes[0] = 0x02;
    assert!(Gf2::<B233>::from_be_bytes(&bytes).is_err());
    bytes[0] = 0x01;
    assert!(Gf2::<B233>::from_be_bytes(&bytes).is_ok());
}

fn hex(text: &str) -> [u8; 16] {
    let mut block = [0; 16];
    for (byte, pair) in block.iter_mut().zip(text.as_bytes().chunks(2)) {
        *byte = u8::from_str_radix(core::str::from_utf8(pair).unwrap(), 16).unwrap();
    }
    block
}

#[test]
fn gcm_view_matches_the_gcm_test_vectors() {
    // NIST GCM test case 2: H = E_K(0^128), C the single ciphertext block, X1 = C * H.
    let h = Gf2p128::from_gcm_block(hex("66e94bd4ef8a2c3b884cfa59ca342b2e"));
    let c = Gf2p128::from_gcm_block(hex("0388dace60b6a392f328c2b971b2fe78"));
    let x1 = c * h;
    assert_eq!(x1.to_gcm_block(), hex("5e2ec746917062882c85b0685353deb7"));
    // GHASH over C and the length block len(A) || len(C) = 0 || 128.
    let lengths = Gf2p128::from_gcm_block(hex("00000000000000000000000000000080"));
    let ghash = (x1 + lengths) * h;
    assert_eq!(
        ghash.to_gcm_block(),
        hex("f38cbb1ad69223dcc3457ae5b6b0f885")
    );
    assert_eq!(Gf2p128::from_gcm_block(h.to_gcm_block()), h);
}

#[test]
fn gcm_view_matches_the_ghash_crate() {
    use ghash::universal_hash::UniversalHash;
    let mut stream = Stream(0x6854);
    for _ in 0..50 {
        let h = stream.element::<Gcm>().to_gcm_block();
        let blocks = [
            stream.element::<Gcm>().to_gcm_block(),
            stream.element::<Gcm>().to_gcm_block(),
        ];
        let mut oracle = ghash::GHash::new(&h.into());
        for block in &blocks {
            oracle.update(&[(*block).into()]);
        }
        let expected: [u8; 16] = oracle.finalize().into();
        let key = Gf2p128::from_gcm_block(h);
        let mut sum = Gf2p128::ZERO;
        for block in blocks {
            sum = (sum + Gf2p128::from_gcm_block(block)) * key;
        }
        assert_eq!(sum.to_gcm_block(), expected);
    }
}

#[test]
fn polyval_view_matches_rfc_8452() {
    // RFC 8452, appendix A.
    let h = Gf2p128::from_polyval_block(hex("25629347589242761d31f826ba4b757b"));
    let x1 = Gf2p128::from_polyval_block(hex("4f4f95668c83dfb6401762bb2d01a262"));
    let x2 = Gf2p128::from_polyval_block(hex("d1a24ddd2721d006bbe45f20d3c9f362"));
    assert_eq!(
        (x1 * h).to_polyval_block(),
        hex("cedac64537ff50989c16011551086d77")
    );
    assert_eq!(
        ((x1 * h + x2) * h).to_polyval_block(),
        hex("f7a3b47b846119fae5b7866cf5e5b77e")
    );
    assert_eq!(Gf2p128::from_polyval_block(h.to_polyval_block()), h);
}

#[test]
fn polyval_view_matches_the_polyval_crate() {
    use polyval::universal_hash::UniversalHash;
    let mut stream = Stream(0x7056);
    for _ in 0..50 {
        let h = stream.element::<Gcm>().to_polyval_block();
        let blocks = [
            stream.element::<Gcm>().to_polyval_block(),
            stream.element::<Gcm>().to_polyval_block(),
        ];
        let mut oracle = polyval::Polyval::new(&h.into());
        for block in &blocks {
            oracle.update(&[(*block).into()]);
        }
        let expected: [u8; 16] = oracle.finalize().into();
        let key = Gf2p128::from_polyval_block(h);
        let mut sum = Gf2p128::ZERO;
        for block in blocks {
            sum = (sum + Gf2p128::from_polyval_block(block)) * key;
        }
        assert_eq!(sum.to_polyval_block(), expected);
    }
}
