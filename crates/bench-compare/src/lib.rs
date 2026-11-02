#![doc = include_str!("../README.md")]

mod barrier;
mod count;
mod counters;
mod document;
mod error;
mod record;
mod run;
mod suite;

#[cfg(all(target_arch = "x86_64", target_feature = "avx2"))]
pub use barrier::Ymm;
pub use barrier::{Barrier, Memory};
#[cfg(target_arch = "x86_64")]
pub use barrier::{Gpr, Xmm};
pub use count::{FEWEST_STEP_CYCLES, REPETITIONS, TARGET, main};
pub use document::{LOAD, Layout, Operation, markdown_table, report, sentence};
pub use error::Error;
pub use record::{
    Cell, Class, Count, Events, Key, Latency, Line, Load, Measurement, Outcome, Record, Sibling,
    Sweep, Version,
};
pub use run::{Build, Measured, NATIVE, Options, Paths, bench, cargo, execute, output};
pub use suite::{
    BATCH, Call, CaseId, Cells, Chain, Function, Inputs, Limbs, Rng, Row, Sample, Suite, Visitor,
    run_batch, run_chain, trim, visit,
};
