//! Both carry backends and the split product against `u128` arithmetic. The portable backend and
//! the split product compile here on every target, so this build also checks the code that other
//! builds ship.

use super::{arch, mul_add_halves, portable};

const EDGES: [u64; 8] = [
    0,
    1,
    2,
    u32::MAX as u64,
    1 << 63,
    u64::MAX - 1,
    u64::MAX,
    0xa5a5_a5a5_5a5a_5a5a,
];

type CarryFn = fn(u8, u64, u64) -> (u64, u8);

fn add_reference(carry: u8, lhs: u64, rhs: u64) -> (u64, u8) {
    let wide = u128::from(lhs) + u128::from(rhs) + u128::from(carry);
    (wide as u64, (wide >> 64) as u8)
}

fn sub_reference(borrow: u8, lhs: u64, rhs: u64) -> (u64, u8) {
    let wide = i128::from(lhs) - i128::from(rhs) - i128::from(borrow);
    (wide as u64, u8::from(wide < 0))
}

#[test]
fn carry_backends_match_wide_arithmetic() {
    let cases: [(&str, CarryFn, CarryFn); 4] = [
        ("arch add", arch::add_carry, add_reference),
        ("portable add", portable::add_carry, add_reference),
        ("arch sub", arch::sub_borrow, sub_reference),
        ("portable sub", portable::sub_borrow, sub_reference),
    ];
    for (name, backend, reference) in cases {
        for lhs in EDGES {
            for rhs in EDGES {
                for carry in [0, 1] {
                    assert_eq!(
                        backend(carry, lhs, rhs),
                        reference(carry, lhs, rhs),
                        "{name}: {carry} {lhs:#x} {rhs:#x}"
                    );
                }
            }
        }
    }
}

#[test]
fn split_product_matches_wide_arithmetic() {
    for a in EDGES {
        for b in EDGES {
            for c in EDGES {
                for d in EDGES {
                    let wide = u128::from(a) * u128::from(b) + u128::from(c) + u128::from(d);
                    assert_eq!(
                        mul_add_halves(a, b, c, d),
                        (wide as u64, (wide >> 64) as u64),
                        "{a:#x} {b:#x} {c:#x} {d:#x}"
                    );
                }
            }
        }
    }
}
