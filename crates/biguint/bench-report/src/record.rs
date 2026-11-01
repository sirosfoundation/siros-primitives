//! The run record: what `bench` counted and how, read back by `report`.

use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::Error;

/// One cell of the comparison: an operation at a width in a crate, as the `compare` bench prints
/// it.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct Cell {
    /// The operation.
    pub(crate) op: String,
    /// The width in bits.
    pub(crate) bits: u32,
    /// The crate.
    #[serde(rename = "crate")]
    pub(crate) library: String,
    /// The count, or the reason for its absence.
    #[serde(flatten)]
    pub(crate) outcome: Outcome,
}

/// What a cell holds.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum Outcome {
    /// The crate's function, counted over a batch.
    Counted(Count),
    /// The crate has no function for the operation at the width; the reason.
    Absent(String),
}

/// The instructions of one crate's function over a batch of operand sets.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct Count {
    /// The function.
    pub(crate) function: String,
    /// The operand sets of the batch.
    pub(crate) operations: u32,
    /// The instructions that Callgrind counted over the batch.
    pub(crate) instructions: u64,
}

impl Count {
    /// The mean instructions per operation.
    pub(crate) fn per_operation(&self) -> f64 {
        self.instructions as f64 / f64::from(self.operations)
    }
}

/// A crate and the version that `Cargo.lock` resolves.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct Version {
    /// The crate.
    pub(crate) name: String,
    /// Its version.
    pub(crate) version: String,
}

/// Everything that `report` needs.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct Record {
    /// The UTC date of the run, `YYYY-MM-DD`.
    pub(crate) date: String,
    /// The commit that ran the bench.
    pub(crate) head: String,
    /// The last commit that changed `crates/biguint/src`.
    pub(crate) library_commit: String,
    /// Whether the measured files differed from `head`.
    pub(crate) dirty: bool,
    /// The first line of `rustc -vV`.
    pub(crate) rustc: String,
    /// The LLVM version of the compiler.
    pub(crate) llvm: String,
    /// The target triple of the host.
    pub(crate) host: String,
    /// The CPU that `-C target-cpu=native` selects.
    pub(crate) native_cpu: String,
    /// The operator's description of the machine.
    pub(crate) machine: Option<String>,
    /// The `RUSTFLAGS` of the build.
    pub(crate) rustflags: String,
    /// The output of `valgrind --version`.
    pub(crate) valgrind: String,
    /// The versions of the compared crates.
    pub(crate) versions: Vec<Version>,
    /// Every cell of the comparison.
    pub(crate) cells: Vec<Cell>,
}

impl Record {
    /// Reads the record at `path`.
    pub(crate) fn read(path: &Path) -> Result<Self, Error> {
        let text = fs::read_to_string(path).map_err(|source| Error::io(path, source))?;
        serde_json::from_str(&text).map_err(|source| Error::json(path, source))
    }

    /// Writes the record to `path`.
    pub(crate) fn write(&self, path: &Path) -> Result<(), Error> {
        let text =
            serde_json::to_string_pretty(self).map_err(|source| Error::json(path, source))?;
        fs::write(path, text + "\n").map_err(|source| Error::io(path, source))
    }
}
