#![doc = include_str!("../README.md")]

mod chart;
mod count;
mod document;
mod error;
mod record;
mod run;
mod suite;

pub use count::main;
pub use document::{Layout, markdown_table, report, sentence};
pub use error::Error;
pub use record::{Cell, Count, Key, Outcome, Record, Version};
pub use run::{Measured, Paths, bench, execute};
pub use suite::{
    BATCH, Call, CaseId, Cells, Limbs, Rng, Row, Sample, Suite, Visitor, run_batch, trim, visit,
};
