//! The barriers of the chain `x = barrier(op(x, y))`. A barrier hides each intermediate `x` from
//! the optimizer, which then computes every step in order: an associative operation stays a chain
//! and turns into no tree. Each barrier keeps `x` in one register class; a case names the class
//! where its type lives, so that each crate pays for its own register moves. A value of more than
//! [`REGISTERS`] bytes goes through memory, and so does every value off x86-64.
#![cfg_attr(
    target_arch = "x86_64",
    expect(unsafe_code, reason = "a register barrier is inline assembly")
)]

/// The most bytes that a barrier holds in registers.
pub const REGISTERS: usize = 64;

/// Where the value of a chain stays between its steps.
pub trait Barrier {
    /// `x`, opaque to the optimizer.
    fn hide<T>(x: T) -> T;
}

/// The general-purpose registers, one per 8 bytes. Integer words take them: `u64` arrays, `u128`,
/// multi-limb integers and field elements, and the pointer and lengths of a heap value.
#[derive(Clone, Copy)]
pub struct Gpr;

/// The vector registers, one per 16 bytes. Elements built on `__m128i` take them.
#[derive(Clone, Copy)]
pub struct Xmm;

#[cfg(target_arch = "x86_64")]
mod registers {
    use core::arch::asm;
    use core::arch::x86_64::__m128i;
    use core::hint::black_box;
    use core::mem::{MaybeUninit, size_of};
    use core::ptr;

    use super::{Barrier, Gpr, REGISTERS, Xmm};

    /// `x` through `N` registers that hold a `W` each: the bytes of `x` go into `N` words, each
    /// word that holds them passes through `pass`, and the words come back as a `T`. A larger `x`
    /// passes through [`black_box`].
    #[inline(always)]
    fn through<T, W: Copy, const N: usize>(x: T, pass: impl Fn(&mut MaybeUninit<W>)) -> T {
        let size = size_of::<T>();
        if size > size_of::<[W; N]>() {
            return black_box(x);
        }
        let x = MaybeUninit::new(x);
        let mut words = [MaybeUninit::<W>::uninit(); N];
        // SAFETY: `size` bytes fit in `words`, and both are valid for them; a `MaybeUninit` holds
        // the uninitialized bytes of padding.
        unsafe {
            ptr::copy_nonoverlapping(x.as_ptr().cast::<u8>(), words.as_mut_ptr().cast(), size);
        }
        for word in &mut words[..size.div_ceil(size_of::<W>())] {
            pass(word);
        }
        let mut hidden = MaybeUninit::<T>::uninit();
        // SAFETY: as above, the other way.
        unsafe {
            ptr::copy_nonoverlapping(
                words.as_ptr().cast::<u8>(),
                hidden.as_mut_ptr().cast(),
                size,
            );
        }
        // SAFETY: `hidden` holds the bytes of `x`, which `pass` left as they were; `x` itself, a
        // `MaybeUninit`, drops nothing.
        unsafe { hidden.assume_init() }
    }

    impl Barrier for Gpr {
        #[inline(always)]
        fn hide<T>(x: T) -> T {
            through::<T, u64, { REGISTERS / 8 }>(x, |word| {
                // SAFETY: the empty block leaves its register as it found it and touches no memory.
                unsafe {
                    asm!(
                        "/* {0} */",
                        inout(reg) * word,
                        options(pure, nomem, nostack, preserves_flags)
                    );
                }
            })
        }
    }

    impl Barrier for Xmm {
        #[inline(always)]
        fn hide<T>(x: T) -> T {
            through::<T, __m128i, { REGISTERS / 16 }>(x, |chunk| {
                // SAFETY: the empty block leaves its register as it found it and touches no memory.
                unsafe {
                    asm!(
                        "/* {0} */",
                        inout(xmm_reg) * chunk,
                        options(pure, nomem, nostack, preserves_flags)
                    );
                }
            })
        }
    }
}

#[cfg(not(target_arch = "x86_64"))]
impl Barrier for Gpr {
    #[inline(always)]
    fn hide<T>(x: T) -> T {
        core::hint::black_box(x)
    }
}

#[cfg(not(target_arch = "x86_64"))]
impl Barrier for Xmm {
    #[inline(always)]
    fn hide<T>(x: T) -> T {
        core::hint::black_box(x)
    }
}
