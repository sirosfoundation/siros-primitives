//! The functions that a WebAssembly bench imports from the page that runs it, under the module
//! name `bench`. The page passes each call to the driver over HTTP: the driver owns the counters of
//! the thread that runs the module.
#![expect(
    unsafe_code,
    reason = "a call of an imported function is unsafe; each wrapper passes the import a valid \
              address and length"
)]

use crate::counters::Counts;

#[link(wasm_import_module = "bench")]
unsafe extern "C" {
    /// Resets and enables the counters.
    #[link_name = "begin"]
    fn import_begin();
    /// Disables the counters and writes their cycles, instructions and branch misses to the three
    /// `u64` at `counts`.
    #[link_name = "end"]
    fn import_end(counts: *mut u64);
    /// Sends the UTF-8 line of `len` bytes at `line` to the driver.
    #[link_name = "emit"]
    fn import_emit(line: *const u8, len: usize);
    /// Sends the UTF-8 reason of `len` bytes at `reason` for which the bench stops.
    #[link_name = "fail"]
    fn import_fail(reason: *const u8, len: usize);
}

/// Resets and enables the counters of the measuring thread.
pub(crate) fn begin() {
    // SAFETY: the import takes no argument.
    unsafe { import_begin() }
}

/// Disables the counters of the measuring thread and returns what they counted since [`begin`].
pub(crate) fn end() -> Counts {
    let mut counts = [0_u64; 3];
    // SAFETY: `counts` holds the three `u64` that the import writes.
    unsafe { import_end(counts.as_mut_ptr()) };
    let [cycles, instructions, branch_misses] = counts;
    Counts {
        cycles,
        instructions,
        branch_misses,
    }
}

/// Sends `line` to the driver.
pub(crate) fn emit(line: &str) {
    // SAFETY: the import reads `line.len()` bytes at `line.as_ptr()`.
    unsafe { import_emit(line.as_ptr(), line.len()) }
}

/// Sends the driver the reason for which the bench stops.
pub(crate) fn fail(reason: &str) {
    // SAFETY: the import reads `reason.len()` bytes at `reason.as_ptr()`.
    unsafe { import_fail(reason.as_ptr(), reason.len()) }
}
