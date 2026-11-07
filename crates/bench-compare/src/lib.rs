#![doc = include_str!("../README.md")]

mod barrier;
mod count;
mod counters;
mod error;
mod fingerprint;
#[cfg(target_arch = "wasm32")]
mod host;
mod record;
mod selection;
mod suite;
#[cfg(not(target_arch = "wasm32"))]
mod window;

#[cfg(all(target_arch = "x86_64", target_feature = "avx2"))]
pub use barrier::Ymm;
pub use barrier::{Barrier, Memory};
#[cfg(target_arch = "x86_64")]
pub use barrier::{Gpr, Xmm};
pub use count::{CPU, FEWEST_STEP_CYCLES, REPETITIONS, RETRIES, TARGET, main};
#[cfg(target_os = "linux")]
pub use counters::pin_thread;
#[cfg(not(target_arch = "wasm32"))]
pub use counters::{Counters, Counts, pin};
pub use error::Error;
pub use fingerprint::Fingerprint;
pub use record::{
    Cell, Class, Count, Events, Key, Latency, Line, Listed, ListedCall, ListedCase, ListedChain,
    Load, Outcome, Overhead, Sibling, Sweep,
};
pub use selection::{Plan, Selection, SelectionError};
pub use suite::{
    BATCH, Call, CaseId, Cells, Chain, Function, Inputs, Limbs, Rng, Row, Sample, Selected, Suite,
    Visitor, run_batch, run_chain, trim, visit,
};
#[cfg(not(target_arch = "wasm32"))]
pub use window::{Window, load};
