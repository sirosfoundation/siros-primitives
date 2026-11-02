use std::io;
use std::path::{Path, PathBuf};
use std::process::ExitStatus;

/// Why a tool stopped.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The arguments name no command.
    #[error("usage: {tool} bench [--machine <description>] | report")]
    Usage {
        /// The tool.
        tool: String,
    },
    /// A file or directory could not be read or written.
    #[error("{}: {source}", path.display())]
    Io {
        /// The file or directory.
        path: PathBuf,
        /// The cause.
        source: io::Error,
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
    /// The output of a command or the content of a file lacks a value.
    #[error("{origin} has no {what}")]
    Missing {
        /// The value.
        what: String,
        /// Where it should be.
        origin: String,
    },
    /// The system clock is before the Unix epoch.
    #[error("the system clock: {0}")]
    Clock(std::time::SystemTimeError),
    /// A line of the bench is no cell.
    #[error("the bench: {0}")]
    Cells(serde_json::Error),
    /// The chart palette has fewer colors than the comparison has crates.
    #[error("{crates} crates for a palette of {slots} colors")]
    Palette {
        /// The crates of the comparison.
        crates: usize,
        /// The colors of the palette.
        slots: usize,
    },
}

impl Error {
    pub(crate) fn io(path: &Path, source: io::Error) -> Self {
        Self::Io {
            path: path.to_path_buf(),
            source,
        }
    }

    pub(crate) fn json(path: &Path, source: serde_json::Error) -> Self {
        Self::Json {
            path: path.to_path_buf(),
            source,
        }
    }

    pub(crate) fn missing(what: &str, origin: &str) -> Self {
        Self::Missing {
            what: what.to_owned(),
            origin: origin.to_owned(),
        }
    }
}
