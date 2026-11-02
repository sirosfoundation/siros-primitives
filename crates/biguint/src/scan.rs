//! Scans over limbs: equality, the zero test, and the positions of the top and bottom nonzero
//! limbs. Up to [`REDUCED_LIMBS`] limbs, a test reduces every limb without a branch. Above, the
//! limb at the scan's start decides most values, and the other limbs reduce without branches,
//! which the compiler vectorizes. The position scans start at their end limb at every width.

/// The most limbs that a test reduces in full.
const REDUCED_LIMBS: usize = 8;

/// Whether every limb is zero.
#[inline(always)]
pub(crate) fn is_zero(limbs: &[u64]) -> bool {
    match limbs.split_first() {
        Some((&low, rest)) if limbs.len() > REDUCED_LIMBS => low == 0 && or_all(rest) == 0,
        _ => or_all(limbs) == 0,
    }
}

/// Whether the equally long `a` and `b` are equal.
#[inline(always)]
pub(crate) fn equal(a: &[u64], b: &[u64]) -> bool {
    match (a.split_first(), b.split_first()) {
        (Some((&a_low, a_rest)), Some((&b_low, b_rest))) if a.len() > REDUCED_LIMBS => {
            a_low == b_low && differences(a_rest, b_rest) == 0
        }
        _ => differences(a, b) == 0,
    }
}

/// The index and value of the top nonzero limb; `None` when every limb is zero.
#[inline(always)]
pub(crate) fn top_nonzero(limbs: &[u64]) -> Option<(usize, u64)> {
    let (&top, rest) = limbs.split_last()?;
    if top != 0 {
        return Some((rest.len(), top));
    }
    rest.chunks(64)
        .enumerate()
        .rev()
        .find_map(|(group, chunk)| {
            // `None` for a chunk of zeros, whose mask has 64 leading zeros.
            let index = 63usize.checked_sub(nonzero_mask(chunk).leading_zeros() as usize)?;
            chunk.get(index).map(|&limb| (group * 64 + index, limb))
        })
}

/// The index and value of the bottom nonzero limb; `None` when every limb is zero.
#[inline(always)]
pub(crate) fn bottom_nonzero(limbs: &[u64]) -> Option<(usize, u64)> {
    let (&bottom, rest) = limbs.split_first()?;
    if bottom != 0 {
        return Some((0, bottom));
    }
    rest.chunks(64).enumerate().find_map(|(group, chunk)| {
        // A chunk of zeros has a mask of 64 trailing zeros, past its end.
        let index = nonzero_mask(chunk).trailing_zeros() as usize;
        chunk.get(index).map(|&limb| (1 + group * 64 + index, limb))
    })
}

/// The OR of the limbs.
#[inline(always)]
fn or_all(limbs: &[u64]) -> u64 {
    limbs.iter().fold(0, |acc, &limb| acc | limb)
}

/// The OR of the XORs of the limbs of the equally long `a` and `b`: zero exactly when they are
/// equal.
#[inline(always)]
fn differences(a: &[u64], b: &[u64]) -> u64 {
    a.iter().zip(b).fold(0, |acc, (&x, &y)| acc | (x ^ y))
}

/// One bit per nonzero limb of at most 64 `limbs`, from the bottom.
#[inline(always)]
fn nonzero_mask(limbs: &[u64]) -> u64 {
    limbs.iter().enumerate().fold(0, |mask, (index, &limb)| {
        mask | (u64::from(limb != 0) << (index % 64))
    })
}
