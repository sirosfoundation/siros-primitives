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

// Fat LTO lays out functions in the order in which it first reaches them from the items of this
// crate, so a module that no native bench runs stays after those that one runs. A thread that a tool
// spawns comes before every item: the tools spawn processes.
#[cfg(not(target_arch = "wasm32"))]
mod changed;
#[cfg(not(target_arch = "wasm32"))]
mod decide;
#[cfg(not(target_arch = "wasm32"))]
mod fewest;
mod fingerprint;
#[cfg(not(target_arch = "wasm32"))]
mod identity;
#[cfg(not(target_arch = "wasm32"))]
mod module;
mod selection;
#[cfg(not(target_arch = "wasm32"))]
mod tool_error;
#[cfg(not(target_arch = "wasm32"))]
mod wasm;

#[cfg(all(target_arch = "x86_64", target_feature = "avx2"))]
pub use barrier::Ymm;
pub use barrier::{Barrier, Memory};
#[cfg(target_arch = "x86_64")]
pub use barrier::{Gpr, Xmm};
#[cfg(not(target_arch = "wasm32"))]
pub use browser::{
    Browser, CHROME, CHROME_WIDE, Cpus, FIREFOX, FIREFOX_WIDE, LISTING, Pref, Threads,
    list as list_in_browser, run as run_in_browser, version as browser_version,
};
#[cfg(not(target_arch = "wasm32"))]
pub use changed::{CellId, code, compact, harness, merge, overtaken, select};
pub use count::{CPU, FEWEST_STEP_CYCLES, REPETITIONS, RETRIES, TARGET, main};
#[cfg(not(target_arch = "wasm32"))]
pub use decide::{Decision, decide};
#[cfg(not(target_arch = "wasm32"))]
pub use document::{LOAD, Layout, Operation, check_records, markdown_table, report, sentence};
pub use error::Error;
#[cfg(not(target_arch = "wasm32"))]
pub use fewest::fewest;
pub use fingerprint::Fingerprint;
#[cfg(not(target_arch = "wasm32"))]
pub use identity::{CodeHash, Identity};
#[cfg(not(target_arch = "wasm32"))]
pub use module::ModuleError;
pub use record::{
    Cell, Class, Count, Engine, Events, Key, Latency, Line, Listed, ListedCall, ListedCase,
    ListedChain, Load, Measurement, Outcome, Overhead, Record, Recorded, Run, Sibling, Sweep,
    Version,
};
#[cfg(not(target_arch = "wasm32"))]
pub use run::{Build, Measured, NATIVE, Options, Paths, bench, cargo, execute, output};
pub use selection::{Plan, Selection, SelectionError};
pub use suite::{
    BATCH, Call, CaseId, Cells, Chain, Function, Inputs, Limbs, Rng, Row, Sample, Selected, Suite,
    Visitor, run_batch, run_chain, trim, visit,
};
#[cfg(not(target_arch = "wasm32"))]
pub use tool_error::ToolError;
#[cfg(not(target_arch = "wasm32"))]
pub use wasm::{FLAVOURS, Flavour, KEEP_RELOCATIONS, SELECTED_RUNS, WASM, WasmBuild};
#[cfg(not(target_arch = "wasm32"))]
pub use window::Window;
