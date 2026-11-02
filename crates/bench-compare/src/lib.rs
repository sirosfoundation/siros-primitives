#![doc = include_str!("../README.md")]

mod barrier;
mod chart;
mod count;
mod counters;
mod document;
mod error;
mod record;
mod run;
mod suite;

pub use barrier::{Barrier, Gpr, Xmm};
pub use count::{REPETITIONS, TARGET, main};
pub use document::{LOAD, Layout, markdown_table, report, sentence};
pub use error::Error;
pub use record::{
    Cell, Count, Events, Key, Line, Load, Measurement, Outcome, Record, Sibling, Sweep, Version,
};
pub use run::{Build, Measured, NATIVE, Options, Paths, bench, cargo, execute, output};
pub use suite::{
    BATCH, Call, CaseId, Cells, Chain, Inputs, Limbs, Rng, Row, Sample, Suite, Visitor, run_batch,
    run_chain, trim, visit,
};
