//! The oracle: arithmetic in `GF(2)[x] / (f)` over little-endian limbs, serial over the bits of
//! one operand, written without `binary-field`.

use bench_compare::{Limbs, Rng};

/// `f(x) = x^degree + Σ x^tap`, the modulus of a field `GF(2^degree)` in the polynomial basis:
/// bit `i` of an element is the coefficient of `x^i`.
#[derive(Clone, Copy, Debug)]
pub struct Modulus {
    /// The degree.
    pub degree: usize,
    /// The exponents of the terms below `x^degree`, `0` among them.
    pub taps: &'static [usize],
}

impl Modulus {
    /// The limbs of an element.
    pub const fn limbs(&self) -> usize {
        self.degree.div_ceil(64)
    }

    /// `x^index`, for an `index` below the degree.
    pub fn power_of_x(&self, index: usize) -> Limbs {
        let mut limbs = vec![0; self.limbs()];
        flip(&mut limbs, index);
        limbs
    }

    /// A uniform element.
    pub fn uniform(&self, rng: &mut Rng) -> Limbs {
        let mut limbs = rng.uniform(self.limbs());
        if let Some(top) = limbs.last_mut()
            && !self.degree.is_multiple_of(64)
        {
            *top &= (1 << (self.degree % 64)) - 1;
        }
        limbs
    }

    /// A uniform nonzero element.
    pub fn nonzero(&self, rng: &mut Rng) -> Limbs {
        loop {
            let value = self.uniform(rng);
            if !is_zero(&value) {
                return value;
            }
        }
    }

    /// A uniform element other than 0 and 1: square, inverse and square root fix 1, so a chain of
    /// one of them from 1 repeats the step of 1. A GF(2⁸) square chain from 1 took 19
    /// instructions per step against 218.
    pub fn above_one(&self, rng: &mut Rng) -> Limbs {
        loop {
            let value = self.nonzero(rng);
            if value.iter().skip(1).any(|&limb| limb != 0) || value.first() != Some(&1) {
                return value;
            }
        }
    }

    /// `a * b`: `a << i` added for every set bit `i` of `b`, then reduced.
    pub fn mul(&self, a: &[u64], b: &[u64]) -> Limbs {
        let mut product = vec![0; a.len() + b.len()];
        for (index, limb) in b.iter().enumerate() {
            let mut rest = *limb;
            while rest != 0 {
                xor_shifted(&mut product, a, index * 64 + rest.trailing_zeros() as usize);
                rest &= rest - 1;
            }
        }
        self.reduce(product)
    }

    /// `a²`.
    pub fn square(&self, a: &[u64]) -> Limbs {
        self.mul(a, a)
    }

    /// `wide` modulo `f`: from the top bit down, each set bit at `d >= degree` is replaced by the
    /// taps at `d - degree`, since `x^degree = Σ x^tap` in the field.
    pub fn reduce(&self, mut wide: Limbs) -> Limbs {
        for top in (self.degree..wide.len() * 64).rev() {
            if bit(&wide, top) {
                flip(&mut wide, top);
                for tap in self.taps {
                    flip(&mut wide, top - self.degree + tap);
                }
            }
        }
        wide.resize(self.limbs(), 0);
        wide
    }

    /// `a⁻¹` for a nonzero `a`, by the extended Euclidean algorithm over `GF(2)[x]` (Hankerson,
    /// Menezes and Vanstone, algorithm 2.48): `g1 a = u` and `g2 a = v` modulo `f` throughout, and
    /// each step adds the shorter of `u` and `v`, shifted, to the longer, until `u = 1`. Panics
    /// unless the product of `a` and the result is `1`.
    pub fn inverse(&self, a: &[u64]) -> Limbs {
        // Room for `f` itself and for the shifted `g`, which stay below `x^(2 degree)`.
        let width = 2 * self.degree / 64 + 2;
        let mut u = a.to_vec();
        u.resize(width, 0);
        let mut v = vec![0; width];
        flip(&mut v, self.degree);
        for tap in self.taps {
            flip(&mut v, *tap);
        }
        let mut g1 = vec![0; width];
        flip(&mut g1, 0);
        let mut g2 = vec![0; width];
        while bit_len(&u) != 1 {
            assert!(!is_zero(&u), "{a:x?} has no inverse modulo {self:?}");
            if bit_len(&u) < bit_len(&v) {
                core::mem::swap(&mut u, &mut v);
                core::mem::swap(&mut g1, &mut g2);
            }
            let shift = bit_len(&u) - bit_len(&v);
            xor_shifted(&mut u, &v, shift);
            xor_shifted(&mut g1, &g2, shift);
        }
        let inverse = self.reduce(g1);
        assert_eq!(
            bench_compare::trim(&self.mul(a, &inverse)),
            [1],
            "the inverse of {a:x?} modulo {self:?}"
        );
        inverse
    }
}

/// Bit `index` of `limbs`; the bits past the end are zero.
pub fn bit(limbs: &[u64], index: usize) -> bool {
    limbs
        .get(index / 64)
        .is_some_and(|limb| (limb >> (index % 64)) & 1 == 1)
}

/// Flips bit `index`, which lies in `limbs`.
pub fn flip(limbs: &mut [u64], index: usize) {
    limbs[index / 64] ^= 1 << (index % 64);
}

/// Adds `limbs << shift` to `into`. Panics if a set bit lands past the end of `into`.
pub fn xor_shifted(into: &mut [u64], limbs: &[u64], shift: usize) {
    let (offset, bits) = (shift / 64, shift % 64);
    for (index, limb) in limbs.iter().enumerate() {
        let low = limb << bits;
        let high = if bits == 0 { 0 } else { limb >> (64 - bits) };
        if low != 0 {
            into[offset + index] ^= low;
        }
        if high != 0 {
            into[offset + index + 1] ^= high;
        }
    }
}

/// Adds `limbs` to `into`, limb by limb.
pub fn xor(into: &mut [u64], limbs: &[u64]) {
    for (slot, limb) in into.iter_mut().zip(limbs) {
        *slot ^= limb;
    }
}

/// The number of bits up to the top set bit: the degree plus one, `0` for the zero polynomial.
pub fn bit_len(limbs: &[u64]) -> usize {
    limbs
        .iter()
        .rposition(|limb| *limb != 0)
        .map_or(0, |top| top * 64 + 64 - limbs[top].leading_zeros() as usize)
}

/// Whether every limb is zero.
pub fn is_zero(limbs: &[u64]) -> bool {
    limbs.iter().all(|limb| *limb == 0)
}
