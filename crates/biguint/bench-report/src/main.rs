#![doc = include_str!("../README.md")]

mod chart;
mod document;
mod record;
mod run;

use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, ExitStatus, Stdio};

use record::Record;

fn main() -> ExitCode {
    match execute(std::env::args().skip(1).collect()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("biguint-bench-report: {error}");
            ExitCode::FAILURE
        }
    }
}

/// Runs the command that `args` name.
fn execute(args: Vec<String>) -> Result<(), Error> {
    let paths = Paths::new();
    match args.as_slice() {
        [command] if command == "bench" => run::bench(&paths, None),
        [command, flag, machine] if command == "bench" && flag == "--machine" => {
            run::bench(&paths, Some(machine.clone()))
        }
        [command] if command == "report" => {
            document::write(&paths, &Record::read(&paths.record())?)
        }
        _ => Err(Error::Usage),
    }
}

/// The locations that the tool reads and writes.
pub(crate) struct Paths {
    /// The workspace root.
    root: PathBuf,
    /// The crate whose benchmarks the document describes: the directory that holds this tool.
    library: PathBuf,
}

impl Paths {
    fn new() -> Self {
        let library = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
        Self {
            root: library.join("..").join(".."),
            library,
        }
    }

    /// The target directory of the bench build, which holds the run record.
    fn results(&self) -> PathBuf {
        self.root.join("target").join("benchmarks")
    }

    /// The run record.
    fn record(&self) -> PathBuf {
        self.results().join("run.json")
    }

    /// The crate whose benchmarks the document describes.
    fn library(&self) -> PathBuf {
        self.library.clone()
    }
}

/// Why the tool stopped.
#[derive(Debug, thiserror::Error)]
pub(crate) enum Error {
    /// The arguments name no command.
    #[error("usage: biguint-bench-report bench [--machine <description>] | report")]
    Usage,
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
    /// A line of the `compare` bench is no cell.
    #[error("the compare bench: {0}")]
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
    fn io(path: &Path, source: io::Error) -> Self {
        Self::Io {
            path: path.to_path_buf(),
            source,
        }
    }

    fn json(path: &Path, source: serde_json::Error) -> Self {
        Self::Json {
            path: path.to_path_buf(),
            source,
        }
    }

    fn missing(what: &str, origin: &str) -> Self {
        Self::Missing {
            what: what.to_owned(),
            origin: origin.to_owned(),
        }
    }
}

/// The standard output of `command`, which must succeed. Its standard error goes to the terminal.
fn output(command: &mut Command) -> Result<String, Error> {
    let line = format!("{command:?}");
    let output = command
        .stderr(Stdio::inherit())
        .output()
        .map_err(|source| Error::Spawn {
            command: line.clone(),
            source,
        })?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    } else {
        Err(Error::Failed {
            command: line,
            status: output.status,
        })
    }
}
