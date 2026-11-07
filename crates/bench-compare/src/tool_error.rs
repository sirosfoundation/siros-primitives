//! The errors of the report tools and of `bench-browser`.

use crate::error::Error;

/// Why a report tool or `bench-browser` stopped: an [`Error`], or a reason that only these tools
/// meet. A native bench holds [`Error`] alone, whose code stays out of these reasons.
#[derive(Debug, thiserror::Error)]
pub enum ToolError {
    /// The arguments name no command.
    #[error(
        "usage: {tool} bench [--cpu <cpu>] [--machine <description>] | bench-wasm --chrome \
         <executable> --firefox <executable> [--changed [--dry-run]] [--runs <runs>] [--target-dir \
         <directory>] [--cpu <cpu>] [--browser-cpu <cpu>] [--machine <description>] | stamp-wasm \
         --chrome <executable> [--target-dir <directory>] | modules --out <directory> [--target-dir \
         <directory>] | report"
    )]
    Usage {
        /// The tool.
        tool: String,
    },
    /// A reason that a bench shares.
    #[error(transparent)]
    Bench(#[from] Error),
    /// A module that the reader does not take.
    #[error(transparent)]
    Module(#[from] crate::module::ModuleError),
    /// A selection that names no cell.
    #[error(transparent)]
    Selection(#[from] crate::selection::SelectionError),
    /// A verdict of a record compares figures of different runs.
    #[error(transparent)]
    Unmatched(Box<UnmatchedVerdict>),
    /// `--changed` went through its rounds, and a re-measured figure still trails a kept one.
    #[error(
        "{build}: {rows} rows compare figures of different runs after {rounds} rounds of \
         re-measuring; the first: {first}"
    )]
    Unsettled {
        /// The build.
        build: String,
        /// The rows.
        rows: usize,
        /// The rounds.
        rounds: usize,
        /// The first of the rows.
        first: String,
    },
    /// `stamp-wasm` keeps a record from taking keys.
    #[error("stamping the record: {reason}")]
    Unstamped {
        /// Why the figures of the record need not hold for the tree.
        reason: String,
    },
}

/// A verdict of a record that compares figures of different runs.
#[derive(Debug, thiserror::Error)]
#[error(
    "{build}: {op} on {row}, {part}: `{subject}` comes from the {subject_run}, and the figure it is \
     compared with, of `{rival}`, from the {rival_run}"
)]
pub struct UnmatchedVerdict {
    /// The build.
    pub build: String,
    /// The operation.
    pub op: String,
    /// The row.
    pub row: String,
    /// Throughput or latency.
    pub part: &'static str,
    /// The crate under test.
    pub subject: String,
    /// The run of its figure.
    pub subject_run: String,
    /// The crate of the figure it is compared with.
    pub rival: String,
    /// The run of that figure.
    pub rival_run: String,
}
