#![doc = include_str!("../README.md")]

mod barrier;
#[cfg(not(target_arch = "wasm32"))]
mod browser;
mod count;
mod counters;
#[cfg(not(target_arch = "wasm32"))]
mod document;
mod error;
#[cfg(target_arch = "wasm32")]
mod host;
mod record;
#[cfg(not(target_arch = "wasm32"))]
mod run;
mod suite;
#[cfg(not(target_arch = "wasm32"))]
mod window;
#[cfg(not(target_arch = "wasm32"))]
mod workspace;

#[cfg(all(target_arch = "x86_64", target_feature = "avx2"))]
pub use barrier::Ymm;
pub use barrier::{Barrier, Memory};
#[cfg(target_arch = "x86_64")]
pub use barrier::{Gpr, Xmm};
#[cfg(not(target_arch = "wasm32"))]
pub use browser::{
    Browser, CHROME, CHROME_WIDE, Cpus, FIREFOX, FIREFOX_WIDE, Pref, Threads,
    run as run_in_browser, version as browser_version,
};
pub use count::{CPU, FEWEST_STEP_CYCLES, REPETITIONS, RETRIES, TARGET, main};
#[cfg(not(target_arch = "wasm32"))]
pub use document::{LOAD, Layout, Operation, markdown_table, report, sentence};
pub use error::Error;
pub use record::{
    Cell, Class, Count, Engine, Events, Key, Latency, Line, Load, Measurement, Outcome, Overhead,
    Record, Sibling, Sweep, Version,
};
#[cfg(not(target_arch = "wasm32"))]
pub use run::{
    Build, Measured, NATIVE, Options, Paths, Runner, WASM, bench, cargo, execute, output,
};
pub use suite::{
    BATCH, Call, CaseId, Cells, Chain, Function, Inputs, Limbs, Rng, Row, Sample, Selected, Suite,
    Visitor, run_batch, run_chain, trim, visit,
};
#[cfg(not(target_arch = "wasm32"))]
pub use window::Window;
