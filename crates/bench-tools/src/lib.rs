#![doc = include_str!("../README.md")]

mod browser;
mod changed;
mod decide;
mod document;
mod fewest;
mod identity;
mod module;
mod record;
mod run;
mod tool_error;
mod wasm;
mod workspace;

pub use browser::{
    Browser, CHROME, CHROME_WIDE, Cpus, FIREFOX, FIREFOX_WIDE, LISTING, Pref, Threads,
    list as list_in_browser, run as run_in_browser, version as browser_version,
};
pub use changed::{CellId, code, compact, harness, merge, overtaken, select};
pub use decide::{Decision, decide};
pub use document::{LOAD, Layout, Operation, check_records, markdown_table, report, sentence};
pub use fewest::fewest;
pub use identity::{CodeHash, Identity};
pub use module::ModuleError;
pub use record::{Engine, Measurement, Record, Recorded, Run, Unfinished, Version};
pub use run::{Build, Measured, NATIVE, Options, Paths, SELECTED_RUNS, execute};
pub use tool_error::{ToolError, UnmatchedVerdict};
pub use wasm::{FLAVOURS, Flavour, KEEP_RELOCATIONS, WASM, WasmBuild, streams};
