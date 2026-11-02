//! The run record: what `bench` measured and how, read back by `report`.

use std::fmt;
use std::fs;
use std::hash::Hash;
use std::path::Path;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::error::Error;

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
}

impl fmt::Display for Class {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Gpr => "general-purpose registers",
            Self::Xmm => "xmm registers",
            Self::Ymm => "ymm registers",
            Self::Memory => "memory",
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

/// A line that the bench prints: every cell, then the sweep.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Line<K> {
    /// A cell.
    Cell(Cell<K>),
    /// The sweep.
    Sweep(Sweep),
}

/// A crate and the version that `Cargo.lock` resolves.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Version {
    /// The crate.
    pub name: String,
    /// Its version.
    pub version: String,
}

/// Everything that `report` needs.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Record<K> {
    /// The UTC date of the run, `YYYY-MM-DD`.
    pub date: String,
    /// The commit that ran the bench.
    pub head: String,
    /// The last commit that changed the measured library's sources.
    pub library_commit: String,
    /// Whether the measured files differed from `head`.
    pub dirty: bool,
    /// The first line of `rustc -vV`.
    pub rustc: String,
    /// The settings of the workspace's `[profile.bench]`, as `key = value` lines.
    pub profile: Vec<String>,
    /// The LLVM version of the compiler.
    pub llvm: String,
    /// The target triple of the host.
    pub host: String,
    /// The CPU that `-C target-cpu=native` selects.
    pub native_cpu: String,
    /// The CPU's model name in `/proc/cpuinfo`.
    pub cpu_model: String,
    /// The operator's description of the machine.
    pub machine: Option<String>,
    /// The measured package.
    pub package: String,
    /// The features of the measured package that every build enables beside its default ones.
    pub features: Vec<String>,
    /// The versions of the compared crates.
    pub versions: Vec<Version>,
    /// The measurement of every build of the bench.
    pub builds: Vec<Measurement<K>>,
}

/// One build of the bench, measured.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Measurement<K> {
    /// The name of the build; empty for the only build of a bench.
    pub name: String,
    /// The `RUSTFLAGS` of the build.
    pub rustflags: String,
    /// The target features that `rustc --print cfg` reports under these flags.
    pub target_features: Vec<String>,
    /// The sweep that measured the build.
    pub sweep: Sweep,
    /// Every cell of the comparison.
    pub cells: Vec<Cell<K>>,
}

impl<K: Key> Record<K> {
    /// Reads the record at `path`.
    pub fn read(path: &Path) -> Result<Self, Error> {
        let text = fs::read_to_string(path).map_err(|source| Error::io(path, source))?;
        serde_json::from_str(&text).map_err(|source| Error::json(path, source))
    }

    /// Writes the record to `path`.
    pub fn write(&self, path: &Path) -> Result<(), Error> {
        let text =
            serde_json::to_string_pretty(self).map_err(|source| Error::json(path, source))?;
        fs::write(path, text + "\n").map_err(|source| Error::io(path, source))
    }
}
