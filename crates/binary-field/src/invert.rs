//! Inversion: `a^(2^n - 2)` through the chain of Itoh and Tsujii (1988), or the extended
//! Euclidean algorithm over GF(2)[x] (Hankerson, Menezes and Vanstone, algorithm 2.48). The plan
//! picks the cheaper one for the degree and the target.

use biguint::Square;
use const_default::ConstDefault;
use generic_array::GenericArray;

use crate::element::{Gf2, Limbs, WideLimbs};
use crate::kernel::{bit_len, copy, head, is_one, is_zero, set_bit, xor_shifted};
use crate::plan::{Plan, Polynomial};

/// The inverse of `a`, or `None` for `0`.
#[inline]
pub(crate) fn invert<P: Polynomial>(a: Gf2<P>) -> Option<Gf2<P>> {
    if a.is_zero() {
        None
    } else if Plan::<P>::INVERT_BY_ITOH_TSUJII {
        Some(itoh_tsujii(a))
    } else {
        euclid(a)
    }
}

/// `a^(2^n - 2) = (a^(2^(n-1) - 1))^2`, with `a^(2^k - 1)` built along the bits of `n - 1`: a
/// doubling of `k` costs `k` squarings and one product, an increment one squaring and one product.
/// The sequence depends on the degree alone.
#[inline]
fn itoh_tsujii<P: Polynomial>(a: Gf2<P>) -> Gf2<P> {
    let exponent = Plan::<P>::DEGREE - 1;
    let mut power = a;
    let mut k = 1;
    for index in (0..Plan::<P>::CHAIN_BITS - 1).rev() {
        let doubled = (0..k).fold(power, |square, _| square.square());
        power = doubled * power;
        k *= 2;
        if (exponent >> index) & 1 == 1 {
            power = power.square() * a;
            k += 1;
        }
    }
    power.square()
}

/// The extended Euclidean algorithm on `u = a` and `v = f`, with `g1 a = u` and `g2 a = v` modulo
/// `f` throughout: each step shifts the shorter pair under the longer and adds, until `u = 1`.
/// `None` when `u` reaches `0`, which happens only when `f` is reducible.
#[inline]
fn euclid<P: Polynomial>(a: Gf2<P>) -> Option<Gf2<P>> {
    let mut u = GenericArray::<u64, WideLimbs<P>>::DEFAULT;
    let mut v = GenericArray::<u64, WideLimbs<P>>::DEFAULT;
    let mut g1 = GenericArray::<u64, WideLimbs<P>>::DEFAULT;
    let mut g2 = GenericArray::<u64, WideLimbs<P>>::DEFAULT;
    copy(&mut u, a.as_limbs());
    set_bit(&mut v, Plan::<P>::DEGREE);
    let mut taps = Plan::<P>::TAPS;
    while let [tap, rest @ ..] = taps {
        set_bit(&mut v, *tap);
        taps = rest;
    }
    set_bit(&mut g1, 0);
    while !is_one(&u) {
        if is_zero(&u) {
            return None;
        }
        let (u_len, v_len) = (bit_len(&u), bit_len(&v));
        if u_len < v_len {
            core::mem::swap(&mut u, &mut v);
            core::mem::swap(&mut g1, &mut g2);
        }
        let shift = u_len.abs_diff(v_len);
        let addend = v;
        xor_shifted(&mut u, &addend, shift);
        let addend = g2;
        xor_shifted(&mut g1, &addend, shift);
    }
    let mut limbs = GenericArray::<u64, Limbs<P>>::DEFAULT;
    copy(&mut limbs, head(&g1, Plan::<P>::LIMBS));
    Some(Gf2::from_reduced(limbs))
}
