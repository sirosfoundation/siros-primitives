use std::io;
use std::num::{ParseFloatError, ParseIntError};
#[cfg(not(target_arch = "wasm32"))]
use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;

/// Why a bench stopped.
#[derive(Debug, thiserror::Error)]
pub enum Error {
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
    /// The operand sets of a case have no layout at a 4 KiB boundary.
    #[cfg(target_arch = "wasm32")]
    #[error("operand sets: {0}")]
    OperandSets(std::alloc::LayoutError),
    /// A load average that does not parse.
    #[error("load average {text:?}: {source}")]
    Load {
        /// The text.
        text: String,
        /// The cause.
        source: ParseFloatError,
    },
    /// A line of the bench is no cell.
    #[error("the bench: {0}")]
    Cells(serde_json::Error),
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
    /// The plan that the driver sent a WebAssembly bench is no UTF-8.
    #[cfg(target_arch = "wasm32")]
    #[error("the plan of the run: {0}")]
    Plan(std::string::FromUtf8Error),
    /// The selection that the driver sent a WebAssembly bench.
    #[cfg(target_arch = "wasm32")]
    #[error(transparent)]
    Selection(crate::selection::SelectionError),
    /// The target has no perf_event.
    #[error("measuring needs the perf_event counters of Linux")]
    NoPerfEvent,
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
    /// The fewest cycles of a case's long runs are no more than the fewest of its short runs.
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
    /// The fewest count of a WebAssembly case's long runs less the fewest of its short runs
    /// counts fewer cycles than 50 times the noise of the fixed cost of a count.
    #[error(
        "{case}: the long runs count {cycles:.0} cycles more than the short runs, within 50 times \
         the noise of the cost of a count, {noise:.0}: other work ran on the counted threads. Short \
         runs: {short}; long runs: {long}"
    )]
    Noisy {
        /// The case and its form.
        case: String,
        /// The difference.
        cycles: f64,
        /// The noise of the fixed cost of a count.
        noise: f64,
        /// The spread of the cycles of the short runs.
        short: String,
        /// The spread of the cycles of the long runs.
        long: String,
    },
    /// A chain step counted fewer cycles than
    /// [`FEWEST_STEP_CYCLES`](crate::FEWEST_STEP_CYCLES) over every repetition of its case: the
    /// compiler regrouped the chain, or a disturbance hit every short run and spared a long one.
    #[error(
        "{case}: {cycles:.2} cycles per chain step over {repetitions} repetitions, under {}{}: the \
         compiler regrouped the chain, or a disturbance hit every short run and spared a long one",
        crate::count::FEWEST_STEP_CYCLES,
        load.map_or_else(String::new, |load| format!(", at a 1-minute load of {load}"))
    )]
    Reassociated {
        /// The case and its form.
        case: String,
        /// The cycles per step.
        cycles: f64,
        /// The repetitions of the case.
        repetitions: usize,
        /// The 1-minute load at the stop, where the bench reads it.
        load: Option<f64>,
    },
}

impl Error {
    /// The error with the 1-minute load `load` at the stop, for the errors that name it.
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn at_load(self, load: f64) -> Self {
        if let Self::Reassociated {
            case,
            cycles,
            repetitions,
            ..
        } = self
        {
            Self::Reassociated {
                case,
                cycles,
                repetitions,
                load: Some(load),
            }
        } else {
            self
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn io(path: &Path, source: io::Error) -> Self {
        Self::Io {
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

    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn number(what: &str, text: &str, source: ParseIntError) -> Self {
        Self::Number {
            what: what.to_owned(),
            text: text.to_owned(),
            source,
        }
    }
}
