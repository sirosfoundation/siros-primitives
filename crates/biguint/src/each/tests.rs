//! The calls that builds without debug assertions make, which the tests' builds do not reach
//! through `each`.

extern crate std;

use std::vec::Vec;

use typenum::{U0, U1, U2, U5, U15, U16, U17, U33, Unsigned};

use super::copies;

fn seen<N: Unsigned>() -> Vec<usize> {
    let mut seen = Vec::new();
    copies::<N>(&mut |index| seen.push(index));
    seen
}

#[test]
fn copies_visit_every_index_once_in_order() {
    for (len, seen) in [
        (U0::USIZE, seen::<U0>()),
        (U1::USIZE, seen::<U1>()),
        (U2::USIZE, seen::<U2>()),
        (U5::USIZE, seen::<U5>()),
        (U15::USIZE, seen::<U15>()),
        (U16::USIZE, seen::<U16>()),
        (U17::USIZE, seen::<U17>()),
        (U33::USIZE, seen::<U33>()),
    ] {
        assert_eq!(seen, (0..len).collect::<Vec<_>>());
    }
}
