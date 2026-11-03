//! The copies that builds without debug assertions run, which the tests' builds do not reach
//! through `each`.

extern crate std;

use std::vec::Vec;

use typenum::{U0, U1, U2, U5, U6, U16, U17, Unsigned};

use super::sealed::Copies;

fn copies<N: Copies + Unsigned>() -> Vec<usize> {
    let mut seen = Vec::new();
    N::copies(3, &mut |index| seen.push(index));
    seen
}

#[test]
fn copies_visit_every_index_once_in_order() {
    for (len, seen) in [
        (U0::USIZE, copies::<U0>()),
        (U1::USIZE, copies::<U1>()),
        (U2::USIZE, copies::<U2>()),
        (U5::USIZE, copies::<U5>()),
        (U6::USIZE, copies::<U6>()),
        (U16::USIZE, copies::<U16>()),
        (U17::USIZE, copies::<U17>()),
    ] {
        assert_eq!(seen, (3..3 + len).collect::<Vec<_>>());
    }
}
