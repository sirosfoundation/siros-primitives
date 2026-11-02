//! Short addition chains for inverses. With `b(c) = a^(2^c - 1)`, `b(x + y) = b(x)^(2^y) b(y)`, so
//! an addition chain for `n - 1` builds `a^(2^(n-1) - 1)` with one product and one Frobenius power
//! per step, and its square is `a^(2^n - 2)`, the inverse (Itoh and Tsujii, 1988). The chains come
//! from a dynamic program over two rules (Knuth, TAOCP vol. 2, 4.6.3): `m = (m - 1) + 1`, and
//! `m = p q`, a chain for `p` followed by a chain for `q` scaled by `p`. It finds the shortest
//! chains for `2^k - 1` up to `k = 8`: 4 steps for 7, 5 for 15, 8 for 63, 10 for 127 and 255.

use core::marker::PhantomData;

use crate::plan::{Plan, Polynomial};

/// The steps an inverse unrolls; longer chains take the binary chain in a loop.
pub(crate) const MAX_STEPS: usize = 24;

/// The largest `n - 1` the dynamic program covers.
const MAX_EXPONENT: usize = 4096;

/// One step of a chain: `b(values[raised] + values[kept]) = b(values[raised])^(2^power)
/// b(values[kept])`, with `power = values[kept]`, the smaller of the two.
#[derive(Clone, Copy)]
pub(crate) struct Step {
    /// The index of the value whose `b` the step raises to `2^power`.
    pub(crate) raised: usize,
    /// The index of the value whose `b` the step multiplies by.
    pub(crate) kept: usize,
    /// The exponent of the Frobenius power: `values[kept]`.
    pub(crate) power: usize,
}

/// An addition chain from `values[0] = 1`: step `s` computes `values[s + 1]`.
#[derive(Clone, Copy)]
pub(crate) struct Chain {
    /// The steps, `len` of them.
    pub(crate) steps: [Step; MAX_STEPS],
    /// The values, `len + 1` of them.
    pub(crate) values: [usize; MAX_STEPS + 1],
    /// The steps of the chain, or `None` past `MAX_STEPS` or `MAX_EXPONENT`.
    pub(crate) len: Option<usize>,
}

/// The chain of the inverse of the field with the polynomial `P`.
pub(crate) struct Chains<P>(PhantomData<P>);

impl<P: Polynomial> Chains<P> {
    /// An addition chain for `n - 1`.
    pub(crate) const CHAIN: Chain = chain(Plan::<P>::DEGREE - 1);
}

/// How the dynamic program reaches each `m`: `0` for `(m - 1) + 1`, `p` for `p (m / p)`.
type Rules = [u16; MAX_EXPONENT + 1];

/// The rules of the shortest chains the two rules give, for every `m` up to `target`.
#[expect(
    clippy::indexing_slicing,
    reason = "const evaluation, with every index at most `target`, within the arrays"
)]
const fn rules(target: usize) -> Rules {
    let mut lengths = [0u16; MAX_EXPONENT + 1];
    let mut rules = [0u16; MAX_EXPONENT + 1];
    let mut m = 2;
    while m <= target {
        lengths[m] = lengths[m - 1] + 1;
        rules[m] = 0;
        let mut p = 2;
        while p * p <= m {
            if m.is_multiple_of(p) {
                let length = lengths[p] + lengths[m / p];
                if length < lengths[m] {
                    lengths[m] = length;
                    rules[m] = p as u16;
                }
            }
            p += 1;
        }
        m += 1;
    }
    rules
}

/// The chain for `target`, which must be at least 1.
const fn chain(target: usize) -> Chain {
    let empty = Step {
        raised: 0,
        kept: 0,
        power: 0,
    };
    let mut chain = Chain {
        steps: [empty; MAX_STEPS],
        values: [0; MAX_STEPS + 1],
        len: Some(0),
    };
    chain.values[0] = 1;
    if target > MAX_EXPONENT {
        chain.len = None;
        return chain;
    }
    let rules = rules(target);
    if !emit(&mut chain, &rules, target, 0) {
        chain.len = None;
    }
    chain
}

/// Appends to `chain` the chain for `m` scaled by the value at `base`, the chain's last; `false`
/// when it outgrows `MAX_STEPS`.
#[expect(
    clippy::indexing_slicing,
    reason = "const evaluation: `m` is at most `MAX_EXPONENT` and every step index is checked"
)]
const fn emit(chain: &mut Chain, rules: &Rules, m: usize, base: usize) -> bool {
    if m == 1 {
        return true;
    }
    let rule = rules[m] as usize;
    if rule == 0 {
        // m = (m - 1) + 1: the chain for m - 1, then one step that adds `scale`.
        if !emit(chain, rules, m - 1, base) {
            return false;
        }
        let Some(len) = chain.len else {
            return false;
        };
        push(chain, len, base)
    } else {
        // m = p q: the chain for p, then the chain for q scaled by p.
        if !emit(chain, rules, rule, base) {
            return false;
        }
        let Some(len) = chain.len else {
            return false;
        };
        emit(chain, rules, m / rule, len)
    }
}

/// Appends the step `values[a] + values[b]`, raising the larger and keeping the smaller.
#[expect(
    clippy::indexing_slicing,
    reason = "const evaluation, with `len` below `MAX_STEPS` checked first"
)]
const fn push(chain: &mut Chain, a: usize, b: usize) -> bool {
    let Some(len) = chain.len else {
        return false;
    };
    if len >= MAX_STEPS {
        return false;
    }
    let (raised, kept) = if chain.values[a] >= chain.values[b] {
        (a, b)
    } else {
        (b, a)
    };
    chain.steps[len] = Step {
        raised,
        kept,
        power: chain.values[kept],
    };
    chain.values[len + 1] = chain.values[a] + chain.values[b];
    chain.len = Some(len + 1);
    true
}

#[cfg(test)]
mod tests {
    use super::chain;

    /// The chains are addition chains of the shortest lengths, OEIS A003313: l(7) = 4, l(15) = 5,
    /// l(63) = 8, l(127) = 10, l(255) = 10.
    #[test]
    fn chains_are_shortest() {
        for (target, steps) in [(7, 4), (15, 5), (63, 8), (127, 10), (255, 10)] {
            let chain = chain(target);
            assert_eq!(chain.len, Some(steps), "steps for {target}");
            for (index, step) in chain.steps[..steps].iter().enumerate() {
                assert!(step.raised <= index && step.kept <= index);
                assert_eq!(step.power, chain.values[step.kept]);
                assert_eq!(
                    chain.values[index + 1],
                    chain.values[step.raised] + chain.values[step.kept]
                );
            }
            assert_eq!(chain.values[steps], target);
        }
    }
}
