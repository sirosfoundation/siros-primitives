//! Barrett's reduction (Barrett, 1986) of a product of residues by an even modulus `m` of `k`
//! limbs, normalized to `d = m * 2^s` with its top bit set, through the reciprocal
//! `β^k + ν = floor((β^(2k) - 1) / d)`, `β = 2^64`. For `T = t * 2^s` below `d * β^k`, the estimate
//! `floor(floor(T / β^(k-1)) * (β^k + ν) / β^(k+1))` is the quotient `floor(T / d)` or one or two
//! below it (Menezes, van Oorschot and Vanstone, *Handbook of Applied Cryptography*, 14.42): the
//! remainder takes two products and at most two subtractions of `d`, where a division takes one
//! quotient limb after the other.
//!
//! The products run unrolled, so the reduction pays from 4 to 8 limbs. In one sweep on Zen 3, an
//! even context product took 114 cycles this way against 132 by division at 256 bits, 223 against
//! 304 at 384 bits and 383 against 478 at 512 bits; it took 42 against 30 at 128 bits and 1,815
//! against 1,459 at 1024 bits.

use generic_array::GenericArray;

use crate::bits::Bits;
use crate::div::{Invariant, Top};
use crate::kernel::{
    add_assign, fits, reduced, shl_bits_into, shr_bits_into, split, split_mut, sub_into,
    subtract_if_above,
};
use crate::mul::basecase;
use crate::uint::Uint;

/// Whether a context of `limbs` limbs reduces the products of its even modulus by Barrett's
/// reduction.
#[inline(always)]
pub(crate) const fn applies(limbs: usize) -> bool {
    4 <= limbs && limbs <= 8
}

/// The reduction by an even modulus that fills its `k` limbs.
pub(crate) struct Barrett<N: Bits> {
    /// `d = m * 2^shift`, its top bit set, with a zero limb above it.
    normalized: GenericArray<u64, N::Window>,
    /// `ν = floor((β^(2k) - 1) / d) - β^k`.
    reciprocal: GenericArray<u64, N::Limbs>,
    /// The leading zeros of `m`'s top limb.
    shift: u32,
}

impl<N: Bits> Barrett<N> {
    /// The reduction by `value`, whose top limb is nonzero, prepared from its division `divisor`:
    /// `floor((β^(2k) - 1) / d) = floor(floor((β^(2k) - 1) / 2^shift) / m)`.
    #[inline(always)]
    pub(crate) fn new(value: &Uint<N>, divisor: &Invariant<N::Limbs>) -> Self {
        let shift = value.limbs.last().copied().unwrap_or(0).leading_zeros();
        let mut normalized = GenericArray::<u64, N::Window>::default();
        shl_bits_into(&mut normalized, &value.limbs, shift);
        let mut ones = GenericArray::<u64, N::ProductWindow>::default();
        for slot in ones.iter_mut().take(2 * N::LIMBS) {
            *slot = u64::MAX;
        }
        let mut dividend = GenericArray::<u64, N::ProductWindow>::default();
        shr_bits_into(&mut dividend, &ones, shift);
        let mut quotient = GenericArray::<u64, N::ProductWindow>::default();
        let (dividend, _) = split_mut(&mut dividend, 2 * N::LIMBS);
        discard_remainder(divisor.remainder::<N::ProductWindow, N>(
            dividend,
            Top::Any,
            &mut quotient,
        ));
        // The quotient lies in `[β^k, 2β^k)`: its limb `k` is one, and `ν` is the limbs below.
        let mut reciprocal = GenericArray::<u64, N::Limbs>::default();
        for (slot, &limb) in reciprocal.iter_mut().zip(quotient.iter()) {
            *slot = limb;
        }
        Self {
            normalized,
            reciprocal,
            shift,
        }
    }

    /// `t mod m` for a product `t` of two residues, in `2k` limbs.
    #[inline(always)]
    pub(crate) fn reduce(&self, t: &[u64]) -> Uint<N> {
        let k = N::LIMBS;
        // `T = t * 2^shift` fits in `2k` limbs: `t < m^2`.
        let mut shifted = GenericArray::<u64, N::ProductWindow>::default();
        shl_bits_into(&mut shifted, t, self.shift);
        // The estimate: the limbs from `k + 1` up of `H * β^k + H * ν` for `H = floor(T / β^(k-1))`.
        let mut high = GenericArray::<u64, N::Window>::default();
        {
            let (_, top) = split(&shifted, k - 1);
            for (slot, &limb) in high.iter_mut().zip(top) {
                *slot = limb;
            }
        }
        let mut product = GenericArray::<u64, N::ProductWindow>::default();
        basecase::product(&mut product, &high, &self.reciprocal);
        {
            // The estimate stays at most the quotient, below `β^k`: the sum fits in `2k + 1` limbs.
            let (_, upper) = split_mut(&mut product, k);
            fits(add_assign(upper, &high));
        }
        let mut estimate = GenericArray::<u64, N::Window>::default();
        {
            let (_, quotient) = split(&product, k + 1);
            for (slot, &limb) in estimate.iter_mut().zip(quotient) {
                *slot = limb;
            }
        }
        // `R = T - estimate * d`, below `3d`: the low `k + 1` limbs hold it.
        let mut scaled = GenericArray::<u64, N::Window>::default();
        basecase::product(&mut scaled, &estimate, &self.normalized);
        let mut remainder = GenericArray::<u64, N::Window>::default();
        let (low, _) = split(&shifted, k + 1);
        fits(sub_into(&mut remainder, low, &scaled));
        reduced(subtract_if_above(&mut remainder, &self.normalized));
        reduced(subtract_if_above(&mut remainder, &self.normalized));
        let mut out = Uint::<N>::ZERO;
        shr_bits_into(&mut out.limbs, &remainder, self.shift);
        out
    }
}

/// Drops the remainder of `β^(2k) - 1` by the modulus: the reciprocal is the quotient.
#[inline(always)]
fn discard_remainder<N: Bits>(_remainder: Uint<N>) {}
