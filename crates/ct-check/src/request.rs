//! Memcheck's client requests, by the instruction sequence that `valgrind.h` defines for amd64.
#![expect(unsafe_code, reason = "a client request is inline assembly")]

use core::arch::asm;

/// `VG_USERREQ__MAKE_MEM_UNDEFINED` of `memcheck.h`: `VG_USERREQ_TOOL_BASE('M', 'C') + 1`.
const MAKE_MEM_UNDEFINED: usize = 0x4d43_0001;

/// Memcheck handled no client request: the operations ran outside `valgrind --tool=memcheck`.
#[derive(Debug, thiserror::Error)]
#[error("memcheck handled no client request; `cargo run -p ct-check` runs the operations under it")]
pub(crate) struct Unhandled;

/// Marks the bytes of `value` undefined. From here on memcheck reports a conditional jump on them,
/// or on a value computed from them, and a memory address computed from them.
pub(crate) fn classify<T: Copy>(value: &mut T) -> Result<(), Unhandled> {
    let address = core::ptr::from_mut(value).expose_provenance();
    // Memcheck answers this request with -1; outside memcheck the answer stays 0.
    match request([MAKE_MEM_UNDEFINED, address, size_of::<T>(), 0, 0, 0]) {
        0 => Err(Unhandled),
        _ => Ok(()),
    }
}

/// Issues the client request whose code and five arguments are `block`. Returns the tool's answer,
/// or 0 when no tool handles the request.
fn request(block: [usize; 6]) -> usize {
    let mut answer = 0;
    // SAFETY: the special instruction preamble of `valgrind.h` for amd64 and the `xchg rbx, rbx`
    // of a client request. Natively the four rotations of `rdi` add up to 128 bits, and the
    // exchange swaps `rbx` with itself, so both registers keep their values and `rdx` keeps 0.
    // Under Valgrind the sequence reads the six words at `rax` and writes the answer to `rdx`.
    // `block` holds the address of the marked value, so the compiler stores that value before the
    // request and reloads it after.
    unsafe {
        asm!(
            "rol rdi, 3",
            "rol rdi, 13",
            "rol rdi, 61",
            "rol rdi, 51",
            "xchg rbx, rbx",
            in("rax") block.as_ptr(),
            inout("rdx") answer,
            options(nostack),
        );
    }
    answer
}
