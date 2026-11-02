use std::io;
use std::num::{ParseFloatError, ParseIntError};
use std::path::{Path, PathBuf};
use std::process::ExitStatus;
use std::time::Duration;

/// Why a tool stopped.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The arguments name no command.
    #[error("usage: {tool} bench [--cpu <cpu>] [--machine <description>] | report")]
    Usage {
        /// The tool.
        tool: String,
    },
    /// The arguments of the bench executable name no CPU.
    #[error("usage: {program} [--cpu <cpu>]")]
    BenchUsage {
        /// The bench executable.
        program: String,
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
    /// A whole number that does not parse.
    #[error("{what} {text:?}: {source}")]
    Number {
        /// What the number is.
        what: String,
        /// The text.
        text: String,
        /// The cause.
        source: ParseIntError,
    },
    /// A load average that does not parse.
    #[error("load average {text:?}: {source}")]
    Load {
        /// The text.
        text: String,
        /// The cause.
        source: ParseFloatError,
    },
    /// The system clock is before the Unix epoch.
    #[error("the system clock: {0}")]
    Clock(std::time::SystemTimeError),
    /// A line of the bench is no cell.
    #[error("the bench: {0}")]
    Cells(serde_json::Error),
    /// A line of a cargo build with `--message-format=json` is no message.
    #[error("the messages of cargo: {0}")]
    Messages(serde_json::Error),
    /// The measuring thread could not pin itself to its CPU.
    #[error("pinning to CPU {cpu}: {source}")]
    Pin {
        /// The CPU.
        cpu: usize,
        /// The cause.
        source: io::Error,
    },
    /// The perf_event group of the measuring thread failed.
    #[error("the performance counters: {source}")]
    Counters {
        /// The cause.
        source: io::Error,
    },
    /// The kernel shared the counters with other groups during a counted run.
    #[error("the counters counted {running:?} of a counted run of {enabled:?}")]
    Multiplexed {
        /// The time enabled.
        enabled: Duration,
        /// The time counted.
        running: Duration,
    },
    /// The target has no perf_event.
    #[error("measuring needs the perf_event counters of Linux")]
    NoPerfEvent,
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
    /// A run ran another number of operations than it drew.
    #[error("{case}: a run of {count} operations ran {runs}")]
    Runs {
        /// The case.
        case: String,
        /// The operations it ran.
        runs: usize,
        /// The operations it drew.
        count: usize,
    },
    /// The long runs of a case counted no more cycles than its short runs, by their medians or by
    /// the median of their differences.
    #[error(
        "{case}: the runs of {} operations count no more cycles than the runs of {operations}",
        2 * operations
    )]
    Flat {
        /// The case and its form.
        case: String,
        /// The operations of a short run.
        operations: usize,
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
    /// A chain step counted fewer cycles than
    /// [`FEWEST_STEP_CYCLES`](crate::FEWEST_STEP_CYCLES): the compiler regrouped the chain.
    #[error(
        "{case}: {cycles:.2} cycles per chain step, under {}: the compiler regrouped the chain",
        crate::count::FEWEST_STEP_CYCLES
    )]
    Reassociated {
        /// The case and its form.
        case: String,
        /// The cycles per step.
        cycles: f64,
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

    pub(crate) fn number(what: &str, text: &str, source: ParseIntError) -> Self {
        Self::Number {
            what: what.to_owned(),
            text: text.to_owned(),
            source,
        }
    }
}
