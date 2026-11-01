use core::fmt::{self, Write};

use crate::bits::Bits;
use crate::uint::Uint;

/// Lowercase hexadecimal with a `0x` prefix, from the most significant nonzero digit.
impl<N: Bits> fmt::Debug for Uint<N> {
    #[inline]
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("0x")?;
        let mut significant = self.limbs.iter().rev().skip_while(|limb| **limb == 0);
        let Some(top) = significant.next() else {
            return formatter.write_char('0');
        };
        write!(formatter, "{top:x}")?;
        significant.try_for_each(|limb| write!(formatter, "{limb:016x}"))
    }
}
