//! The run record: what `bench` measured and how, read back by `report`.

use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::Error;

/// What a cell of the comparison runs.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum Kind {
    /// The crate's own function for the operation.
    Api,
    /// No case; the detail gives the reason.
    Absent,
}

/// One cell of the comparison, as `compare --cases` lists it.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct Cell {
    /// `op/bits/crate`, criterion's id of the case.
    pub(crate) case: String,
    /// The operation.
    pub(crate) op: String,
    /// The width in bits.
    pub(crate) bits: u32,
    /// The crate.
    #[serde(rename = "crate")]
    pub(crate) library: String,
    /// What the cell runs.
    pub(crate) kind: Kind,
    /// The function, or the reason for the absence.
    pub(crate) detail: String,
    /// The operations in one iteration of the case.
    pub(crate) batch: u32,
}

/// One build of the bench.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct Build {
    /// `default` or `native`.
    pub(crate) name: String,
    /// The `RUSTFLAGS` of the build.
    pub(crate) rustflags: String,
    /// Seconds since the Unix epoch when its timing run started.
    pub(crate) started: u64,
    /// Seconds since the Unix epoch when its timing run finished.
    pub(crate) finished: u64,
}

/// A crate and the version that `Cargo.lock` resolves.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct Version {
    /// The crate.
    pub(crate) name: String,
    /// Its version.
    pub(crate) version: String,
}

/// Everything that `report` needs besides criterion's results.
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
    /// The logical CPUs that the operating system let the run use.
    pub(crate) available_cpus: usize,
    /// The operator's description of the machine.
    pub(crate) machine: Option<String>,
    /// The arguments of the timing runs to criterion.
    pub(crate) criterion_args: Vec<String>,
    /// Why the times are upper bounds, when they are.
    pub(crate) provisional: Option<String>,
    /// The versions of the compared crates and of criterion.
    pub(crate) versions: Vec<Version>,
    /// The builds, in run order.
    pub(crate) builds: Vec<Build>,
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
