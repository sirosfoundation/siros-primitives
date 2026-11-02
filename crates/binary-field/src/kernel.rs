//! Kernels over little-endian `u64` limb slices, each a `const fn`: the compile-time constants of a
//! field and its runtime operations share them. Every length and shift is a constant after
//! inlining, so each loop folds to straight-line code.

/// `slice.split_at(mid)` with `mid` clamped to the length.
#[inline(always)]
pub(crate) const fn split(slice: &[u64], mid: usize) -> (&[u64], &[u64]) {
    slice.split_at(if mid < slice.len() { mid } else { slice.len() })
}

/// `slice.split_at_mut(mid)` with `mid` clamped to the length.
#[inline(always)]
pub(crate) const fn split_mut(slice: &mut [u64], mid: usize) -> (&mut [u64], &mut [u64]) {
    let mid = if mid < slice.len() { mid } else { slice.len() };
    slice.split_at_mut(mid)
}

/// The first `len` limbs of `slice`, or all of them when it is shorter.
#[inline(always)]
pub(crate) const fn head(slice: &[u64], len: usize) -> &[u64] {
    split(slice, len).0
}

/// The first `len` limbs of `slice`, or all of them when it is shorter.
#[inline(always)]
pub(crate) const fn head_mut(slice: &mut [u64], len: usize) -> &mut [u64] {
    split_mut(slice, len).0
}

/// Sets every limb to zero.
#[inline(always)]
pub(crate) const fn clear(slice: &mut [u64]) {
    let mut rest = slice;
    while let [limb, tail @ ..] = rest {
        *limb = 0;
        rest = tail;
    }
}

/// Copies `src` into the start of `dst` and zeroes the limbs of `dst` above it.
#[inline(always)]
pub(crate) const fn copy(dst: &mut [u64], src: &[u64]) {
    let (low, high) = split_mut(dst, src.len());
    xor_into(low, src, true);
    clear(high);
}

/// Adds `src` to the start of `dst`, limb by limb; `overwrite` replaces instead of adding.
#[inline(always)]
pub(crate) const fn xor_into(dst: &mut [u64], src: &[u64], overwrite: bool) {
    let mut dst = dst;
    let mut src = src;
    while let ([slot, dst_tail @ ..], [limb, src_tail @ ..]) = (dst, src) {
        *slot = if overwrite { *limb } else { *slot ^ *limb };
        dst = dst_tail;
        src = src_tail;
    }
}

/// Adds `src << shift` to `dst`; the bits that fall above `dst` are dropped.
#[inline(always)]
pub(crate) const fn xor_shifted(dst: &mut [u64], src: &[u64], shift: usize) {
    let (offset, bit) = (shift / 64, shift % 64);
    let (_, aligned) = split_mut(dst, offset);
    if bit == 0 {
        xor_into(aligned, src, false);
    } else {
        let mut dst = aligned;
        let mut src = src;
        let mut carried = 0;
        while let [slot, dst_tail @ ..] = dst {
            match src {
                [limb, src_tail @ ..] => {
                    *slot ^= (*limb << bit) | carried;
                    carried = *limb >> (64 - bit);
                    src = src_tail;
                }
                [] => {
                    *slot ^= carried;
                    break;
                }
            }
            dst = dst_tail;
        }
    }
}

/// Writes the bits `lo..hi` of `src` to `dst`, bit `lo` at bit 0, and zeroes the rest of `dst`.
#[inline(always)]
pub(crate) const fn extract(dst: &mut [u64], src: &[u64], lo: usize, hi: usize) {
    let (offset, bit) = (lo / 64, lo % 64);
    let (_, src) = split(src, offset);
    let len = (hi - lo).div_ceil(64);
    let (window, above) = split_mut(dst, len);
    clear(above);
    let mut window = window;
    let mut src = src;
    while let [slot, window_tail @ ..] = window {
        let (low, src_tail) = match src {
            [limb, src_tail @ ..] => (*limb >> bit, src_tail),
            [] => (0, src),
        };
        let high = match (bit, src_tail) {
            (0, _) | (_, []) => 0,
            (_, [next, ..]) => *next << (64 - bit),
        };
        *slot = low | high;
        src = src_tail;
        window = window_tail;
    }
    if (hi - lo) % 64 != 0 {
        if let Some(top) = head_mut(dst, len).last_mut() {
            *top &= (1 << ((hi - lo) % 64)) - 1;
        }
    }
}

/// Sets bit `index`.
#[inline(always)]
pub(crate) const fn set_bit(acc: &mut [u64], index: usize) {
    if let [limb, ..] = split_mut(acc, index / 64).1 {
        *limb |= 1 << (index % 64);
    }
}

/// Bit `index`, or `false` above the slice.
#[inline(always)]
pub(crate) const fn bit(acc: &[u64], index: usize) -> bool {
    match split(acc, index / 64).1 {
        [limb, ..] => (*limb >> (index % 64)) & 1 == 1,
        [] => false,
    }
}

/// Writes the carry-less squares of the limbs of `a` to `out`, two limbs each, using the word
/// product `square`.
#[inline(always)]
pub(crate) fn square_limbs(out: &mut [u64], a: &[u64], square: impl Fn(u64) -> u128) {
    let mut out = out;
    let mut a = a;
    while let [limb, a_tail @ ..] = a {
        let product = square(*limb);
        match out {
            [low, high, out_tail @ ..] => {
                *low = product as u64;
                *high = (product >> 64) as u64;
                out = out_tail;
            }
            [low] => {
                *low = product as u64;
                out = &mut [];
            }
            [] => break,
        }
        a = a_tail;
    }
    clear(out);
}

/// Zeroes the bits `lo..hi` of `acc`.
#[inline(always)]
pub(crate) const fn clear_bits(acc: &mut [u64], lo: usize, hi: usize) {
    let (_, mut rest) = split_mut(acc, lo / 64);
    let mut base = (lo / 64) * 64;
    while let [limb, tail @ ..] = rest {
        if base >= hi {
            break;
        }
        let from = lo.saturating_sub(base);
        let to = if hi - base < 64 { hi - base } else { 64 };
        let below_to = if to == 64 { u64::MAX } else { (1 << to) - 1 };
        let below_from = (1u64 << from).wrapping_sub(1);
        *limb &= !(below_to & !below_from);
        base += 64;
        rest = tail;
    }
}

/// Zeroes the bits of `acc` at and above `degree`.
#[inline(always)]
pub(crate) const fn truncate(acc: &mut [u64], degree: usize) {
    let (_, above) = split_mut(acc, degree.div_ceil(64));
    clear(above);
    if degree % 64 != 0 {
        if let Some(top) = head_mut(acc, degree.div_ceil(64)).last_mut() {
            *top &= (1 << (degree % 64)) - 1;
        }
    }
}

/// The 32 low bits of `half`, each moved to twice its position.
#[inline(always)]
const fn spread_half(half: u64) -> u64 {
    let mut x = half & 0xFFFF_FFFF;
    x = (x | (x << 16)) & 0x0000_FFFF_0000_FFFF;
    x = (x | (x << 8)) & 0x00FF_00FF_00FF_00FF;
    x = (x | (x << 4)) & 0x0F0F_0F0F_0F0F_0F0F;
    x = (x | (x << 2)) & 0x3333_3333_3333_3333;
    (x | (x << 1)) & 0x5555_5555_5555_5555
}

/// The even bits of `limb`, each moved to half its position, in the low 32 bits.
#[inline(always)]
const fn gather_even(limb: u64) -> u64 {
    let mut x = limb & 0x5555_5555_5555_5555;
    x = (x | (x >> 1)) & 0x3333_3333_3333_3333;
    x = (x | (x >> 2)) & 0x0F0F_0F0F_0F0F_0F0F;
    x = (x | (x >> 4)) & 0x00FF_00FF_00FF_00FF;
    x = (x | (x >> 8)) & 0x0000_FFFF_0000_FFFF;
    (x | (x >> 16)) & 0x0000_0000_FFFF_FFFF
}

/// Writes the carry-less square of `a` to `out`: bit `i` of `a` becomes bit `2i`. The square takes
/// `2 * a.len()` limbs; `out` may be shorter when the top limbs of the square are zero.
#[inline(always)]
pub(crate) const fn spread(out: &mut [u64], a: &[u64]) {
    let mut out = out;
    let mut a = a;
    while let [limb, a_tail @ ..] = a {
        match out {
            [low, high, out_tail @ ..] => {
                *low = spread_half(*limb);
                *high = spread_half(*limb >> 32);
                out = out_tail;
            }
            [low] => {
                *low = spread_half(*limb);
                out = &mut [];
            }
            [] => break,
        }
        a = a_tail;
    }
    clear(out);
}

/// Writes the bits of `a` at the positions congruent to `parity` modulo 2 to `out`, bit `2i +
/// parity` at bit `i`, and zeroes the rest of `out`.
#[inline(always)]
pub(crate) const fn gather(out: &mut [u64], a: &[u64], parity: usize) {
    let mut out = out;
    let mut a = a;
    while let [slot, out_tail @ ..] = out {
        let (low, a_tail) = match a {
            [low, a_tail @ ..] => (*low, a_tail),
            [] => (0, a),
        };
        let (high, a_tail) = match a_tail {
            [high, a_tail @ ..] => (*high, a_tail),
            [] => (0, a_tail),
        };
        *slot = gather_even(low >> parity) | (gather_even(high >> parity) << 32);
        a = a_tail;
        out = out_tail;
    }
}

/// The parity of the bits that `a` and `mask` share.
#[inline(always)]
pub(crate) const fn masked_parity(a: &[u64], mask: &[u64]) -> bool {
    let mut acc = 0;
    let mut a = a;
    let mut mask = mask;
    while let ([limb, a_tail @ ..], [bits, mask_tail @ ..]) = (a, mask) {
        acc ^= *limb & *bits;
        a = a_tail;
        mask = mask_tail;
    }
    acc.count_ones() % 2 == 1
}

/// Whether every limb is zero.
#[inline(always)]
pub(crate) const fn is_zero(a: &[u64]) -> bool {
    let mut acc = 0;
    let mut a = a;
    while let [limb, tail @ ..] = a {
        acc |= *limb;
        a = tail;
    }
    acc == 0
}

/// The number of bits up to and including the highest set bit, or 0 for zero.
#[inline(always)]
pub(crate) const fn bit_len(a: &[u64]) -> usize {
    let mut len = 0;
    let mut base = 0;
    let mut a = a;
    while let [limb, tail @ ..] = a {
        if *limb != 0 {
            len = base + 64 - limb.leading_zeros() as usize;
        }
        base += 64;
        a = tail;
    }
    len
}

/// Whether `a` is exactly `1`.
#[inline(always)]
pub(crate) const fn is_one(a: &[u64]) -> bool {
    match a {
        [low, rest @ ..] => *low == 1 && is_zero(rest),
        [] => false,
    }
}
