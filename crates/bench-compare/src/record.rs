//! The lines that a bench sends: its cells, how they ran, and what they run.

use std::fmt;
use std::hash::Hash;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

/// A row key: the width or the field of a row. It serializes as the row's own fields of a cell
/// (`"bits": 64`, `"field": "P-256 base"`) and displays as the row's label.
pub trait Key:
    Clone + Eq + Hash + fmt::Display + Serialize + DeserializeOwned + Send + Sync + 'static
{
}

impl<K> Key for K where
    K: Clone + Eq + Hash + fmt::Display + Serialize + DeserializeOwned + Send + Sync + 'static
{
}

/// One cell of the comparison: an operation on a row in a crate, as the bench prints it.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Cell<K> {
    /// The operation.
    pub op: String,
    /// The row.
    #[serde(flatten)]
    pub key: K,
    /// The crate.
    #[serde(rename = "crate")]
    pub library: String,
    /// The counts, or the reason for their absence.
    #[serde(flatten)]
    pub outcome: Outcome,
}

/// What a cell holds.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Outcome {
    /// The crate's function, measured.
    Counted(Count),
    /// The crate has no function for the operation on the row; the reason.
    Absent(String),
}

/// One crate's function, measured in a loop of independent operations and, for a case with a
/// chain form, in the chain `x = op(x, y)`.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Count {
    /// The function.
    pub function: String,
    /// How its results deviate from the row's definition, in one phrase; `None` for a function
    /// that meets it.
    pub deviation: Option<String>,
    /// The loop of independent operations.
    pub batch: Events,
    /// The chain; absent for a case without a chain form.
    pub chain: Option<Latency>,
}

/// The chain of a case: of its chains, one per class that can hold its value, the one with the
/// fewest cycles per step.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct Latency {
    /// Where the chain holds its value between steps.
    pub class: Class,
    /// The events per step.
    pub events: Events,
}

/// Where a chain holds its value between steps.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Class {
    /// The general-purpose registers, for a value of up to 64 bytes.
    Gpr,
    /// The `xmm` registers, for a value of up to 128 bytes.
    Xmm,
    /// The `ymm` registers of an AVX2 build, for a value of 17 to 256 bytes.
    Ymm,
    /// One place on the stack, for a value of more than 64 bytes.
    Memory,
    /// The locals of a WebAssembly build, with a copy in memory after each step.
    Locals,
}

impl fmt::Display for Class {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Gpr => "general-purpose registers",
            Self::Xmm => "xmm registers",
            Self::Ymm => "ymm registers",
            Self::Memory => "memory",
            Self::Locals => "WebAssembly locals",
        })
    }
}

/// The user-mode events per operation that the CPU's counters count in one form of a case.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct Events {
    /// The operations of a short run; a long run has twice as many.
    pub operations: usize,
    /// Core cycles.
    pub cycles: f64,
    /// Retired instructions.
    pub instructions: f64,
    /// Mispredicted branches.
    pub branch_misses: f64,
}

/// The 1-minute load average of the machine before and after a sweep.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct Load {
    /// Before the first repetition.
    pub before: f64,
    /// After the last repetition.
    pub after: f64,
}

/// An SMT sibling of the measuring CPU, which shares its core.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct Sibling {
    /// The CPU.
    pub cpu: usize,
    /// The share of the sweep, from 0 to 1, that the sibling spent busy.
    pub busy: f64,
}

/// Where and when the bench measured.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Sweep {
    /// The CPU that the measuring thread ran on.
    pub cpu: usize,
    /// The load around the sweep.
    pub load: Load,
    /// The SMT siblings of the CPU.
    pub siblings: Vec<Sibling>,
}

/// A line that the bench prints: every cell, then the sweep. A WebAssembly bench sends the cost
/// of a count first, and its driver adds the sweep.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Line<K> {
    /// The cost of a count of a WebAssembly bench.
    Overhead(Overhead),
    /// A cell.
    Cell(Cell<K>),
    /// The sweep.
    Sweep(Sweep),
}

/// A line of a WebAssembly bench that lists its cells without running any: what the tool needs to
/// tell whether the code of a cell changed.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Listed<K> {
    /// The operation.
    pub op: String,
    /// The row.
    #[serde(flatten)]
    pub key: K,
    /// The row, as a selection names it.
    pub row: String,
    /// The crate.
    #[serde(rename = "crate")]
    pub library: String,
    /// The case, or the reason for its absence.
    #[serde(flatten)]
    pub call: ListedCall,
}

/// What a listed cell runs.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ListedCall {
    /// The crate's function.
    Case(ListedCase),
    /// The crate has no function for the operation on the row; the reason.
    Absent(String),
}

/// What a case runs. The indices are those of the module's function table.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ListedCase {
    /// The function.
    pub function: String,
    /// How its results deviate from the row's definition, in one phrase; `None` for a function
    /// that meets it.
    pub deviation: Option<String>,
    /// The loop of independent operations.
    pub batch: usize,
    /// The chains, one per class that can hold the value of a step.
    pub chains: Vec<ListedChain>,
    /// The function that draws one input.
    pub draw: usize,
    /// The function that builds what the inputs share.
    pub setup: usize,
    /// The function that offers the crate's cells, which builds what the operation can capture.
    pub adapter: usize,
    /// A hash of the operands that the row draws from a fixed seed.
    pub operands: String,
    /// The bytes that the operation captures, in hexadecimal digits.
    pub captures: String,
}

/// A chain of a listed case.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct ListedChain {
    /// Where the chain holds its value between steps.
    pub class: Class,
    /// The loop of the chain.
    pub entry: usize,
}

/// The cost of a count of a WebAssembly bench, from empty counts before its sweep: the requests
/// to the driver and the wake-up of the measuring thread.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct Overhead {
    /// The median cycles of an empty count.
    pub cycles: f64,
    /// The median instructions of an empty count.
    pub instructions: f64,
    /// The noise of the fixed cost of a count in a figure: the largest difference of the fewest
    /// counts of two sets of [`REPETITIONS`](crate::REPETITIONS) empty counts, among four pairs of
    /// sets.
    pub noise: f64,
}
