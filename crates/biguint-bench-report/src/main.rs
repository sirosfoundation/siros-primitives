//! Runs the benchmarks that compare `biguint` with other big-integer crates and writes
//! `crates/biguint/BENCHMARKS.md` with one SVG chart per operation.
//!
//! ```text
//! cargo run -p biguint-bench-report -- bench [--machine <description>]
//! cargo run -p biguint-bench-report -- report [--provisional <reason>]
//! ```
//!
//! `bench` runs the `compare` bench of `biguint` twice: for the default target, and with
//! `-C target-cpu=native`. Each build has its own target directory under `target/benchmarks`.
//! It records the commit, the compiler, the machine and the cells of the comparison in
//! `target/benchmarks/run.json`. `--machine` adds a description of the machine.
//!
//! `report` reads that record and criterion's `estimates.json` files and writes the document and
//! the charts. `--provisional` marks the times as upper bounds and gives the reason, for example
//! the load of the machine; the record keeps it for later reports.

mod chart;
mod document;
mod record;
mod results;
mod run;

use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, ExitStatus};

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
        [command] if command == "report" => report(&paths, None),
        [command, flag, reason] if command == "report" && flag == "--provisional" => {
            report(&paths, Some(reason.clone()))
        }
        _ => Err(Error::Usage),
    }
}

/// Writes the document and the charts from the recorded run. A `provisional` reason replaces
/// the recorded one.
fn report(paths: &Paths, provisional: Option<String>) -> Result<(), Error> {
    let mut record = Record::read(&paths.record())?;
    if provisional.is_some() {
        record.provisional = provisional;
        record.write(&paths.record())?;
    }
    let measured = record
        .builds
        .iter()
        .map(|build| {
            let times = results::times(&paths.criterion(&build.name), build, &record.cells)?;
            Ok(results::Measured { build, times })
        })
        .collect::<Result<Vec<_>, Error>>()?;
    document::write(paths, &record, &measured)
}

/// The locations that the tool reads and writes.
pub(crate) struct Paths {
    /// The workspace root.
    root: PathBuf,
}

impl Paths {
    fn new() -> Self {
        let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
        Self {
            root: manifest.join("..").join(".."),
        }
    }

    /// The directory of all results.
    fn results(&self) -> PathBuf {
        self.root.join("target").join("benchmarks")
    }

    /// The run record.
    fn record(&self) -> PathBuf {
        self.results().join("run.json")
    }

    /// The target directory of the build `name`.
    fn build(&self, name: &str) -> PathBuf {
        self.results().join(name)
    }

    /// Criterion's output directory for the build `name`.
    fn criterion(&self, name: &str) -> PathBuf {
        self.build(name).join("criterion")
    }

    /// The crate whose benchmarks the document describes.
    fn library(&self) -> PathBuf {
        self.root.join("crates").join("biguint")
    }
}

/// Why the tool stopped.
#[derive(Debug, thiserror::Error)]
pub(crate) enum Error {
    /// The arguments name no command.
    #[error(
        "usage: biguint-bench-report bench [--machine <description>] \
         | report [--provisional <reason>]"
    )]
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
    /// The machine did not report its logical CPUs.
    #[error("the number of logical CPUs: {0}")]
    Parallelism(io::Error),
    /// The system clock is before the Unix epoch.
    #[error("the system clock: {0}")]
    Clock(std::time::SystemTimeError),
    /// A line of `compare --cases` is no cell.
    #[error("compare --cases: {0}")]
    Cases(serde_json::Error),
    /// Criterion has no result for a case of the run.
    #[error("{} has no result for {case}", directory.display())]
    NoResult {
        /// The criterion directory.
        directory: PathBuf,
        /// The case.
        case: String,
    },
    /// Criterion's result for a case is older than the run.
    #[error("the result for {case} in {} predates the run", directory.display())]
    Stale {
        /// The criterion directory.
        directory: PathBuf,
        /// The case.
        case: String,
    },
    /// The chart palette has fewer colors than the comparison has crates.
    #[error("{crates} crates for a palette of {slots} colors")]
    Palette {
        /// The crates of the comparison.
        crates: usize,
        /// The colors of the palette.
        slots: usize,
    },
    /// Two criterion results have the same id.
    #[error("{} has two results for {case}", directory.display())]
    Duplicate {
        /// The criterion directory.
        directory: PathBuf,
        /// The case.
        case: String,
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

/// The standard output of `command`, which must succeed.
fn output(command: &mut Command) -> Result<String, Error> {
    let line = format!("{command:?}");
    let output = command.output().map_err(|source| Error::Spawn {
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

/// Runs `command` with the terminal as its output; it must succeed.
fn status(command: &mut Command) -> Result<(), Error> {
    let line = format!("{command:?}");
    let status = command.status().map_err(|source| Error::Spawn {
        command: line.clone(),
        source,
    })?;
    if status.success() {
        Ok(())
    } else {
        Err(Error::Failed {
            command: line,
            status,
        })
    }
}
