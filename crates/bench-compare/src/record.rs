//! The run record: what `bench` counted and how, read back by `report`.

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
    /// The count, or the reason for its absence.
    #[serde(flatten)]
    pub outcome: Outcome,
}

/// What a cell holds.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Outcome {
    /// The crate's function, counted over a batch.
    Counted(Count),
    /// The crate has no function for the operation on the row; the reason.
    Absent(String),
}

/// The instructions of one crate's function over a batch of operand sets.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Count {
    /// The function.
    pub function: String,
    /// The operand sets of the batch.
    pub operations: u32,
    /// The instructions that Callgrind counted over the batch.
    pub instructions: u64,
}

impl Count {
    /// The mean instructions per operation.
    #[must_use]
    pub fn per_operation(&self) -> f64 {
        self.instructions as f64 / f64::from(self.operations)
    }
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
    /// The LLVM version of the compiler.
    pub llvm: String,
    /// The target triple of the host.
    pub host: String,
    /// The CPU that `-C target-cpu=native` selects.
    pub native_cpu: String,
    /// The operator's description of the machine.
    pub machine: Option<String>,
    /// The `RUSTFLAGS` of the build.
    pub rustflags: String,
    /// The output of `valgrind --version`.
    pub valgrind: String,
    /// The versions of the compared crates.
    pub versions: Vec<Version>,
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
