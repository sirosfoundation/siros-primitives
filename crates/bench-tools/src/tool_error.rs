//! The errors of the report tools and of `bench-browser`.

use std::io;
use std::num::ParseIntError;
use std::path::{Path, PathBuf};
use std::process::ExitStatus;

use bench_compare::{Error, SelectionError};

use crate::module::ModuleError;

/// Why a report tool or `bench-browser` stopped.
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
    Harness(#[from] Error),
    /// A module that the reader does not take.
    #[error(transparent)]
    Module(#[from] ModuleError),
    /// A selection that names no cell.
    #[error(transparent)]
    Selection(#[from] SelectionError),
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
    /// A JSON file could not be read or written.
    #[error("{}: {source}", path.display())]
    Json {
        /// The file.
        path: PathBuf,
        /// The cause.
        source: serde_json::Error,
    },
    /// A command could not start.
    #[error("{command}: {source}")]
    Spawn {
        /// The command line.
        command: String,
        /// The cause.
        source: io::Error,
    },
    /// A command failed.
    #[error("{command}: {status}")]
    Failed {
        /// The command line.
        command: String,
        /// Its exit status.
        status: ExitStatus,
    },
    /// The system clock is before the Unix epoch.
    #[error("the system clock: {0}")]
    Clock(std::time::SystemTimeError),
    /// The output of `cargo metadata` is no workspace description.
    #[error("the output of cargo metadata: {0}")]
    Metadata(serde_json::Error),
    /// A line of a cargo build with `--message-format=json` is no message.
    #[error("the messages of cargo: {0}")]
    Messages(serde_json::Error),
    /// The browser of a WebAssembly bench, or its page, failed.
    #[error("the browser: {reason}")]
    Browser {
        /// What failed.
        reason: String,
    },
    /// A WebAssembly bench stopped.
    #[error("the WebAssembly bench: {reason}")]
    Bench {
        /// The reason that it sent, or the error that its page caught.
        reason: String,
    },
    /// A build whose target features lack AVX2.
    #[error("a build without AVX2: RUSTFLAGS=\"{rustflags}\"")]
    NoAvx2 {
        /// The flags of the build.
        rustflags: String,
    },
    /// The workspace's `[profile.bench]` lacks `lto = "fat"`.
    #[error("{}: [profile.bench] has no lto = \"fat\"", manifest.display())]
    NoFatLto {
        /// The workspace's `Cargo.toml`.
        manifest: PathBuf,
    },
    /// The bench of a build lacks a feature that the build enables.
    #[error("the bench of {package} was built without its feature {feature}")]
    Unbuilt {
        /// The measured package.
        package: String,
        /// The feature.
        feature: String,
    },
    /// A unit of a build has a feature that the comparison forbids.
    #[error("{unit} was built with the forbidden feature {feature}")]
    Forbidden {
        /// The package of the unit, as cargo names it.
        unit: String,
        /// The feature.
        feature: String,
    },
}

impl ToolError {
    /// A file or directory at `path` could not be read or written.
    pub(crate) fn io(path: &Path, source: io::Error) -> Self {
        Error::Io {
            path: path.to_path_buf(),
            source,
        }
        .into()
    }

    /// The JSON file at `path` could not be read or written.
    pub(crate) fn json(path: &Path, source: serde_json::Error) -> Self {
        Self::Json {
            path: path.to_path_buf(),
            source,
        }
    }

    /// `origin` lacks `what`.
    pub(crate) fn missing(what: &str, origin: &str) -> Self {
        Error::Missing {
            what: what.to_owned(),
            origin: origin.to_owned(),
        }
        .into()
    }

    /// The text `text` of `what` is no whole number.
    pub(crate) fn number(what: &str, text: &str, source: ParseIntError) -> Self {
        Error::Number {
            what: what.to_owned(),
            text: text.to_owned(),
            source,
        }
        .into()
    }
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
