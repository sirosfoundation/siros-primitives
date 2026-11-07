//! What hides a measured operation from the optimizer: the address of each operand set, the sink
//! of each result, and the barriers of the chain `x = op(x, y)`.
//!
//! An empty inline-assembly block does the hiding and emits no instruction. A barrier hides each
//! intermediate `x`, so the optimizer computes every step in order: an associative operation
//! stays a chain and turns into no tree. A register barrier holds `x` in one class of registers,
//! each register filled by one word of `x`, and can only add moves between classes: a chain runs
//! once per class that holds its value, and the faster run counts. [`Memory`] keeps `x` in one
//! place on the stack, whose address a block takes after each step: one store and one load per
//! word and step. Off x86-64, `core::hint::black_box` and volatile loads do the hiding, and every
//! chain runs through [`Memory`]; a WebAssembly chain also runs through `Locals`, which keeps `x`
//! in locals and stores a copy of it after each step.
#![expect(
    unsafe_code,
    reason = "a barrier is inline assembly or a volatile load, and `touch` reads padding bytes"
)]

#[cfg(all(target_arch = "x86_64", target_feature = "avx2"))]
pub use registers::Ymm;
#[cfg(target_arch = "x86_64")]
pub use registers::{Gpr, Xmm};

use crate::record::Class;
use crate::suite::Chain;

/// The most bytes that the general-purpose registers hold: eight words of 8 bytes.
const GPR_BYTES: usize = 64;

/// The bytes of one `xmm` register. A value of more bytes is the smallest that the `ymm`
/// registers hold.
#[cfg(target_arch = "x86_64")]
const XMM_WORD: usize = 16;

/// The most bytes that the `xmm` registers hold: eight words of 16 bytes.
#[cfg(target_arch = "x86_64")]
const XMM_BYTES: usize = 128;

/// The most bytes that the `ymm` registers hold: eight words of 32 bytes.
#[cfg(all(target_arch = "x86_64", target_feature = "avx2"))]
const YMM_BYTES: usize = 256;

/// Where the value of a chain stays between its steps.
pub trait Barrier {
    /// The class of the barrier in the record.
    const CLASS: Class;

    /// The fewest bytes of a value that the barrier holds.
    const LEAST: usize;

    /// The most bytes of a value that the barrier holds.
    const MOST: usize;

    /// Runs `run` on the inputs of `batch` as the chain `x = op(x, y)`, with `x` held by the
    /// barrier, and returns the number of runs: from the second input on, the result of the run
    /// before replaces the first operand. An empty block takes the address of the last result.
    fn chain<I: Chain<R>, R>(batch: &[I], run: impl Fn(&I) -> R) -> usize;
}

/// Memory: `x` stays in one place on the stack, and after each step an empty block takes its
/// address and may read and rewrite it. The operation stores its result there and the next one
/// loads it: one store and one load per word and step. The optimizer can neither forward the
/// stored value nor skip the store, and it can merge the operation's own result with that place,
/// so a result that the operation builds in memory takes no copy.
pub struct Memory;

impl Barrier for Memory {
    const CLASS: Class = Class::Memory;

    /// A value larger than the general-purpose registers hold, and every value off x86-64.
    const LEAST: usize = if cfg!(target_arch = "x86_64") {
        GPR_BYTES + 1
    } else {
        0
    };
    const MOST: usize = usize::MAX;

    #[inline(always)]
    fn chain<I: Chain<R>, R>(batch: &[I], run: impl Fn(&I) -> R) -> usize {
        let Some((first, rest)) = batch.split_first() else {
            return 0;
        };
        let mut x = run(opaque(first));
        let mut runs = 1;
        for input in rest {
            clobber(&mut x);
            x = run(&opaque(input).chain(x));
            runs += 1;
        }
        sink(&x);
        runs
    }
}

/// The locals of WebAssembly, which the engine keeps in registers: after each step, a copy of `x`
/// goes to one place on the stack, whose address `core::hint::black_box` takes. The optimizer
/// computes every intermediate `x` and keeps `x` itself in locals, so the chain runs one store
/// per word and step beside it and no load. This is the `DoNotOptimize` of Google Benchmark on
/// a compiler without inline assembly: a store of the value that the optimizer cannot skip.
#[cfg(target_arch = "wasm32")]
pub struct Locals;

#[cfg(target_arch = "wasm32")]
impl Barrier for Locals {
    const CLASS: Class = Class::Locals;

    const LEAST: usize = 0;
    const MOST: usize = usize::MAX;

    #[inline(always)]
    fn chain<I: Chain<R>, R>(batch: &[I], run: impl Fn(&I) -> R) -> usize {
        let Some((first, rest)) = batch.split_first() else {
            return 0;
        };
        let mut x = run(opaque(first));
        if const { size_of::<I>() <= PLAIN_WALK_BYTES } {
            for input in rest {
                show(&x);
                x = run(&input.chain(x));
            }
        } else if let Some(mut walk) = HiddenWalk::of(rest) {
            for _ in rest {
                show(&x);
                x = run(&walk.input().chain(x));
                walk.step();
            }
        }
        // A sink of `x` itself would keep `x` in memory, with one store and one load per step.
        show(&x);
        batch.len()
    }
}

/// The most bytes of an operand set whose chain through [`Locals`] walks plain addresses. The load
/// of a hidden step raises a one-instruction step from 1.1 cycles to 1.7, and LLVM's induction
/// variable costs a set this small no address. Larger sets hide their addresses as every other
/// chain does: LLVM otherwise bases their loads on a limb past the first, with an `i32.add` per
/// limb, and the 384-bit sum ran 60 instructions per step against 50.
#[cfg(target_arch = "wasm32")]
const PLAIN_WALK_BYTES: usize = 2 * size_of::<u128>();

/// Stores a copy of the bytes of `value` where an empty block may read it. `value` itself keeps
/// its place: the block can neither read nor rewrite it.
#[cfg(target_arch = "wasm32")]
#[inline(always)]
fn show<T>(value: &T) {
    // SAFETY: a `MaybeUninit<T>` has the layout of a `T`, and `value` is a valid `T`; the copy, a
    // `MaybeUninit`, drops nothing, so it duplicates no ownership.
    let copy =
        unsafe { core::ptr::read(core::ptr::from_ref(value).cast::<core::mem::MaybeUninit<T>>()) };
    // `black_box` stores its argument and passes the block its address; nothing reads the value
    // that it returns, so the optimizer drops its load.
    #[expect(
        unused_results,
        reason = "the value that `black_box` returns is the copy"
    )]
    core::hint::black_box(copy);
}

/// An empty block that may read and rewrite the memory of `value`: every store to it happens
/// before the block, and every load from it after.
#[inline(always)]
fn clobber<T>(value: &mut T) {
    #[cfg(target_arch = "x86_64")]
    registers::clobber(value);
    #[cfg(not(target_arch = "x86_64"))]
    #[expect(
        unused_results,
        reason = "the reference that `black_box` returns is `value`"
    )]
    core::hint::black_box(value);
}

/// The bytes of a cache line.
const LINE: usize = 64;

/// Reads one byte of every cache line of `batch`, so every run starts with its operand sets in
/// the caches whatever their drawing and conversion evicted. A crate whose conversion allocates
/// per element otherwise counted its runs from farther memory: its single-`xor` add took 19 to 27
/// cycles against 2 for the same instructions in other crates.
pub(crate) fn touch<T>(batch: &[T]) {
    // SAFETY: the bytes of `batch` are initialized memory of its length; a `MaybeUninit<u8>` holds
    // any byte, padding included.
    let bytes = unsafe {
        core::slice::from_raw_parts(
            batch.as_ptr().cast::<core::mem::MaybeUninit<u8>>(),
            core::mem::size_of_val(batch),
        )
    };
    touch_bytes(bytes);
}

/// Reads one byte of every cache line of `bytes`.
fn touch_bytes(bytes: &[core::mem::MaybeUninit<u8>]) {
    for line in bytes.iter().step_by(LINE) {
        // SAFETY: `line` is a reference, valid for reads.
        let byte = unsafe { core::ptr::read_volatile(line) };
        sink(&byte);
    }
}

/// `value` behind an address that the optimizer cannot see through, so it can neither precompute
/// what the operation reads there nor merge the reads of two operand sets.
#[inline(always)]
pub(crate) fn opaque<T>(value: &T) -> &T {
    #[cfg(target_arch = "x86_64")]
    return registers::opaque(value);
    #[cfg(not(target_arch = "x86_64"))]
    return through(value);
}

/// `value` from a volatile store and a volatile load, which hide it as `core::hint::black_box`
/// does. The empty block after them bounds the search of LLVM's DAG combiner to the operation's
/// own loads: it gives up after 18 steps, and a 512-bit sum by value takes 17 to pass its first
/// store over its 16 loads.
#[cfg(not(target_arch = "x86_64"))]
#[inline(always)]
fn through<T>(value: &T) -> &T {
    let mut slot = value;
    // SAFETY: `slot` is a local, valid for writes.
    unsafe { core::ptr::write_volatile(&mut slot, value) };
    // SAFETY: `slot` is a local, valid for reads.
    let hidden = unsafe { core::ptr::read_volatile(&slot) };
    core::hint::black_box(());
    hidden
}

/// A walk over the inputs of a nonempty batch that holds one address in a local and reads its step
/// and its end by volatile loads. LLVM can neither see the address of an input nor count the loop,
/// so it folds no input into a strength-reduced address and unrolls for no operation, where it
/// would unroll a WebAssembly loop of at most 16 IR instructions and halve the exit tests of only
/// the operations that small. Nothing that the walk adds waits on a store, and the operation keeps
/// every other local.
#[cfg(target_arch = "wasm32")]
pub(crate) struct HiddenWalk<'a, I> {
    /// The address of an input of the batch.
    input: *const I,
    /// The bytes from one input to the next.
    step: usize,
    /// The address after the last input.
    end: *const I,
    batch: core::marker::PhantomData<&'a [I]>,
}

#[cfg(target_arch = "wasm32")]
impl<'a, I> HiddenWalk<'a, I> {
    /// The walk over `batch`, unless it is empty.
    #[inline(always)]
    pub(crate) fn of(batch: &'a [I]) -> Option<Self> {
        const {
            assert!(
                size_of::<I>() > 0,
                "the end of a batch needs inputs with a size"
            )
        };
        let range = batch.as_ptr_range();
        (range.start != range.end).then_some(Self {
            input: range.start,
            step: size_of::<I>(),
            end: range.end,
            batch: core::marker::PhantomData,
        })
    }

    /// The current input.
    #[inline(always)]
    pub(crate) fn input(&self) -> &'a I {
        // SAFETY: `input` addresses an input of the batch, which lives for `'a`.
        unsafe { &*self.input }
    }

    /// Moves past the current input.
    #[inline(always)]
    fn step(&mut self) {
        // SAFETY: `step` is a field, valid for reads.
        let step = unsafe { core::ptr::read_volatile(&self.step) };
        self.input = self.input.wrapping_byte_add(step);
    }

    /// Moves past the current input. Returns whether an input remains.
    #[inline(always)]
    pub(crate) fn advance(&mut self) -> bool {
        self.step();
        // SAFETY: `end` is a field, valid for reads.
        self.input != unsafe { core::ptr::read_volatile(&self.end) }
    }
}

/// The alignment of [`AtPage`]: the addresses that hardware compares between a load and the stores
/// before it are the low 12 bits.
#[cfg(target_arch = "wasm32")]
const PAGE: usize = 4096;

/// A copy of the operand sets of a batch that starts at a 4 KiB boundary. The allocator otherwise
/// puts each case's operand sets at an address of its own, and the same WebAssembly function counts
/// differently per case: the 256-bit product took Chrome from 16.4 to 18.2 cycles, and at this
/// alignment from 17.06 to 17.11.
#[cfg(target_arch = "wasm32")]
pub(crate) struct AtPage<I> {
    /// The operand sets, which own what the copy's bits address.
    batch: Vec<I>,
    /// The start of the copy.
    copy: core::ptr::NonNull<I>,
    /// The layout of the copy.
    layout: std::alloc::Layout,
}

#[cfg(target_arch = "wasm32")]
impl<I> AtPage<I> {
    /// The copy of `batch`, unless the size of `batch` has no layout.
    pub(crate) fn of(batch: Vec<I>) -> Result<Self, std::alloc::LayoutError> {
        let layout = std::alloc::Layout::from_size_align(
            size_of_val(batch.as_slice()).max(1),
            PAGE.max(align_of::<I>()),
        )?;
        // SAFETY: `layout` has a nonzero size.
        let block = unsafe { std::alloc::alloc(layout) }.cast::<I>();
        let Some(copy) = core::ptr::NonNull::new(block) else {
            std::alloc::handle_alloc_error(layout);
        };
        // SAFETY: `copy` is a new block of the size of `batch`, aligned for `I`.
        unsafe { core::ptr::copy_nonoverlapping(batch.as_ptr(), copy.as_ptr(), batch.len()) };
        Ok(Self {
            batch,
            copy,
            layout,
        })
    }
}

#[cfg(target_arch = "wasm32")]
impl<I> core::ops::Deref for AtPage<I> {
    type Target = [I];

    fn deref(&self) -> &[I] {
        // SAFETY: the copy holds the bits of the `batch.len()` values of `batch`, which stay alive
        // and unchanged as long as `self`, and nothing writes the copy.
        unsafe { core::slice::from_raw_parts(self.copy.as_ptr(), self.batch.len()) }
    }
}

#[cfg(target_arch = "wasm32")]
impl<I> Drop for AtPage<I> {
    fn drop(&mut self) {
        // SAFETY: `copy` came from `alloc` with `layout`. Its bits are `batch`'s to drop.
        unsafe { std::alloc::dealloc(self.copy.as_ptr().cast::<u8>(), self.layout) };
    }
}

/// Uses `value` where the optimizer cannot see, so it cannot skip the operation that computed it.
/// Its bytes must be in memory at its address: a value in registers costs one store per register,
/// a value already in memory nothing.
#[inline(always)]
pub(crate) fn sink<T>(value: &T) {
    #[cfg(target_arch = "x86_64")]
    registers::sink(value);
    // A value of up to two words stays in locals: a copy of it takes one store per word, where
    // its address takes as many and one more for the address.
    #[cfg(target_arch = "wasm32")]
    if size_of::<T>() <= WASM_WORDS {
        show(value);
        return;
    }
    #[cfg(not(target_arch = "x86_64"))]
    #[expect(
        unused_results,
        reason = "the reference that `black_box` returns is `value`"
    )]
    core::hint::black_box(value);
}

/// The bytes of the two `i64` locals that hold a value of WebAssembly.
#[cfg(target_arch = "wasm32")]
const WASM_WORDS: usize = 16;

#[cfg(target_arch = "x86_64")]
mod registers {
    use core::arch::asm;
    use core::arch::x86_64::__m128i;
    #[cfg(target_feature = "avx2")]
    use core::arch::x86_64::__m256i;
    use core::mem::{MaybeUninit, size_of};
    use core::{ptr, slice};

    #[cfg(target_feature = "avx2")]
    use super::YMM_BYTES;
    use super::{Barrier, GPR_BYTES, XMM_BYTES, XMM_WORD};
    use crate::record::Class;
    use crate::suite::Chain;

    /// The general-purpose registers, one per word of 8, 4, 2 or 1 bytes: a value takes as many
    /// 8-byte words as it fills, then one word of each smaller width that its last bytes need.
    pub struct Gpr;

    /// The `xmm` registers, one per 16 bytes.
    pub struct Xmm;

    /// The `ymm` registers of AVX2 builds, one per 32 bytes.
    #[cfg(target_feature = "avx2")]
    pub struct Ymm;

    /// A class of registers that holds a value one word at a time. The length of the value fixes
    /// every word and its offset at compile time.
    trait Words {
        /// Passes the bytes of `source` through registers of the class into `target`, of the same
        /// length.
        fn pass(source: &[MaybeUninit<u8>], target: &mut [MaybeUninit<u8>]);
    }

    impl Words for Gpr {
        #[inline(always)]
        fn pass(source: &[MaybeUninit<u8>], target: &mut [MaybeUninit<u8>]) {
            let (words, mut source) = source.as_chunks::<8>();
            let (targets, mut target) = target.as_chunks_mut::<8>();
            for (word, target) in words.iter().zip(targets) {
                pass::<u64>(word, target);
            }
            // The last bytes, at most 7, take one word of each width that their length has.
            for width in [4, 2, 1] {
                if source.len() & width != 0 {
                    let (word, rest) = source.split_at(width);
                    let (into, others) = target.split_at_mut(width);
                    match width {
                        4 => pass::<u32>(word, into),
                        2 => pass::<u16>(word, into),
                        _ => pass::<u8>(word, into),
                    }
                    (source, target) = (rest, others);
                }
            }
        }
    }

    impl Words for Xmm {
        #[inline(always)]
        fn pass(source: &[MaybeUninit<u8>], target: &mut [MaybeUninit<u8>]) {
            vector::<__m128i, XMM_WORD>(source, target);
        }
    }

    #[cfg(target_feature = "avx2")]
    impl Words for Ymm {
        #[inline(always)]
        fn pass(source: &[MaybeUninit<u8>], target: &mut [MaybeUninit<u8>]) {
            vector::<__m256i, 32>(source, target);
        }
    }

    /// Passes `source` into `target` through vector registers that hold an `N`-byte `W` each: the
    /// last bytes, fewer than `N`, take one more register.
    #[inline(always)]
    fn vector<W: Word, const N: usize>(source: &[MaybeUninit<u8>], target: &mut [MaybeUninit<u8>]) {
        let (words, source) = source.as_chunks::<N>();
        let (targets, target) = target.as_chunks_mut::<N>();
        for (word, target) in words.iter().zip(targets) {
            pass::<W>(word, target);
        }
        if !source.is_empty() {
            pass::<W>(source, target);
        }
    }

    impl Barrier for Gpr {
        const CLASS: Class = Class::Gpr;

        const LEAST: usize = 0;
        const MOST: usize = GPR_BYTES;

        #[inline(always)]
        fn chain<I: Chain<R>, R>(batch: &[I], run: impl Fn(&I) -> R) -> usize {
            chain::<Self, I, R>(batch, run)
        }
    }

    impl Barrier for Xmm {
        const CLASS: Class = Class::Xmm;

        const LEAST: usize = 0;
        const MOST: usize = XMM_BYTES;

        #[inline(always)]
        fn chain<I: Chain<R>, R>(batch: &[I], run: impl Fn(&I) -> R) -> usize {
            chain::<Self, I, R>(batch, run)
        }
    }

    #[cfg(target_feature = "avx2")]
    impl Barrier for Ymm {
        const CLASS: Class = Class::Ymm;

        /// A value that one `xmm` register cannot hold, up to [`YMM_BYTES`].
        const LEAST: usize = XMM_WORD + 1;
        const MOST: usize = YMM_BYTES;

        #[inline(always)]
        fn chain<I: Chain<R>, R>(batch: &[I], run: impl Fn(&I) -> R) -> usize {
            chain::<Self, I, R>(batch, run)
        }
    }

    /// The chain of [`Barrier::chain`] with `x` held in the registers of `C`: each result passes
    /// through them on its way into the next operand set, which the next run builds in registers
    /// from the operands beside the first.
    #[inline(always)]
    fn chain<C: Words, I: Chain<R>, R>(batch: &[I], run: impl Fn(&I) -> R) -> usize {
        let Some((first, rest)) = batch.split_first() else {
            return 0;
        };
        let mut result = run(opaque(first));
        let mut runs = 1;
        for input in rest {
            result = run(&opaque(input).chain(through::<C, R>(result)));
            runs += 1;
        }
        // The sink takes the address of a value of its own: an address of `result` would keep
        // `result` in memory, with one store per step.
        sink(&through::<C, R>(result));
        runs
    }

    /// `x` through the registers of `C`, one word at a time.
    #[inline(always)]
    fn through<C: Words, T>(x: T) -> T {
        let mut source = MaybeUninit::new(x);
        let mut target = MaybeUninit::<T>::uninit();
        C::pass(bytes(&mut source), bytes(&mut target));
        // SAFETY: every byte of `target` is the byte of `x` at its offset, which the blocks left
        // as they found it; `source`, a `MaybeUninit`, drops nothing.
        unsafe { target.assume_init() }
    }

    /// The bytes of `value`.
    #[inline(always)]
    fn bytes<T>(value: &mut MaybeUninit<T>) -> &mut [MaybeUninit<u8>] {
        // SAFETY: a `MaybeUninit<T>` spans `size_of::<T>()` bytes, each a valid
        // `MaybeUninit<u8>`, borrowed as long as `value`.
        unsafe {
            slice::from_raw_parts_mut(value.as_mut_ptr().cast::<MaybeUninit<u8>>(), size_of::<T>())
        }
    }

    /// Passes `source` through one register that holds a `W` into `target`, of the same length.
    /// Bytes past `source` are zero in the register: a word of fewer bytes than its register
    /// keeps no dependency on the register's earlier value.
    #[inline(always)]
    fn pass<W: Word>(source: &[MaybeUninit<u8>], target: &mut [MaybeUninit<u8>]) {
        let mut word = MaybeUninit::<W>::zeroed();
        bytes(&mut word)[..source.len()].copy_from_slice(source);
        W::hide(&mut word);
        target.copy_from_slice(&bytes(&mut word)[..target.len()]);
    }

    /// A word that fills one register of its class.
    trait Word: Sized {
        /// `word`, opaque to the optimizer, in one register.
        fn hide(word: &mut MaybeUninit<Self>);
    }

    impl Word for u8 {
        #[inline(always)]
        fn hide(word: &mut MaybeUninit<Self>) {
            // SAFETY: the empty block leaves its register as it found it and touches no memory.
            unsafe {
                asm!(
                    "/* {0} */",
                    inout(reg_byte) * word,
                    options(pure, nomem, nostack, preserves_flags)
                );
            }
        }
    }

    impl Word for u16 {
        #[inline(always)]
        fn hide(word: &mut MaybeUninit<Self>) {
            // SAFETY: as for `u8`.
            unsafe {
                asm!(
                    "/* {0:x} */",
                    inout(reg) * word,
                    options(pure, nomem, nostack, preserves_flags)
                );
            }
        }
    }

    impl Word for u32 {
        #[inline(always)]
        fn hide(word: &mut MaybeUninit<Self>) {
            // SAFETY: as for `u8`.
            unsafe {
                asm!(
                    "/* {0:e} */",
                    inout(reg) * word,
                    options(pure, nomem, nostack, preserves_flags)
                );
            }
        }
    }

    impl Word for u64 {
        #[inline(always)]
        fn hide(word: &mut MaybeUninit<Self>) {
            // SAFETY: as for `u8`.
            unsafe {
                asm!(
                    "/* {0:r} */",
                    inout(reg) * word,
                    options(pure, nomem, nostack, preserves_flags)
                );
            }
        }
    }

    impl Word for __m128i {
        #[inline(always)]
        fn hide(word: &mut MaybeUninit<Self>) {
            // SAFETY: as for `u8`.
            unsafe {
                asm!(
                    "/* {0} */",
                    inout(xmm_reg) * word,
                    options(pure, nomem, nostack, preserves_flags)
                );
            }
        }
    }

    #[cfg(target_feature = "avx2")]
    impl Word for __m256i {
        #[inline(always)]
        fn hide(word: &mut MaybeUninit<Self>) {
            // SAFETY: as for `u8`.
            unsafe {
                asm!(
                    "/* {0} */",
                    inout(ymm_reg) * word,
                    options(pure, nomem, nostack, preserves_flags)
                );
            }
        }
    }

    /// The address of `value` through a register.
    #[inline(always)]
    #[expect(
        clippy::pointers_in_nomem_asm_block,
        reason = "the empty block reads nothing behind the address it holds"
    )]
    pub(super) fn opaque<T>(value: &T) -> &T {
        let mut address = ptr::from_ref(value);
        // SAFETY: the empty block leaves its register as it found it and touches no memory.
        unsafe {
            asm!(
                "/* {0} */",
                inout(reg) address,
                options(pure, nomem, nostack, preserves_flags)
            );
        }
        // SAFETY: `address` is the address of `value`, which the block left as it was.
        unsafe { &*address }
    }

    /// An empty block that may read the memory of `value`.
    #[inline(always)]
    pub(super) fn sink<T>(value: &T) {
        // SAFETY: the empty block reads and writes nothing.
        unsafe {
            asm!(
                "/* {0} */",
                in(reg) ptr::from_ref(value),
                options(readonly, nostack, preserves_flags)
            );
        }
    }

    /// An empty block that may read and write the memory of `value`.
    #[inline(always)]
    pub(super) fn clobber<T>(value: &mut T) {
        // SAFETY: the empty block reads and writes nothing.
        unsafe {
            asm!(
                "/* {0} */",
                in(reg) ptr::from_mut(value),
                options(nostack, preserves_flags)
            );
        }
    }
}
