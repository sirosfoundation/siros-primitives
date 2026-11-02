//! The bench executable: without arguments it counts every case of a [`Suite`] under Valgrind's
//! Callgrind, on all the threads that the run may use, and prints every cell as one JSON line.
//! Each count runs the executable again with `--count <case>`, which checks the results of the
//! case and then runs its batch once through [`run_batch`]. Callgrind counts the instructions from
//! the entry of `run_batch` to its exit.

use std::fs;
use std::num::NonZero;
use std::path::Path;
use std::process::{Command, ExitCode};
use std::thread;

use crate::record::{Cell, Count as Counted, Outcome};
use crate::run::valgrind;
use crate::suite::{BATCH, Call, CaseId, Suite, Visitor, run_batch, visit};

/// Runs the bench executable of `S`, named `name` in its messages, with the process arguments.
#[must_use]
pub fn main<S: Suite>(name: &str) -> ExitCode {
    // `cargo bench` passes `--bench`.
    let args: Vec<String> = std::env::args()
        .skip(1)
        .filter(|arg| arg != "--bench")
        .collect();
    match args.as_slice() {
        [] => match count_all::<S>(name) {
            Ok(lines) => {
                for line in lines {
                    println!("{line}");
                }
                ExitCode::SUCCESS
            }
            Err(error) => {
                eprintln!("{name}: {error}");
                ExitCode::FAILURE
            }
        },
        [flag, case] if flag == "--count" => count::<S>(name, case),
        _ => {
            eprintln!("usage: {name} [--count <op/row/crate>]");
            ExitCode::FAILURE
        }
    }
}

/// Every cell of the comparison.
struct AllCells<O, K> {
    cells: Vec<(CaseId<O, K>, Call)>,
}

impl<O: Clone, K: Clone> Visitor<O, K> for AllCells<O, K> {
    fn note(&mut self, id: &CaseId<O, K>, call: &Call) {
        self.cells.push((id.clone(), *call));
    }

    fn wants(&self, _id: &CaseId<O, K>) -> bool {
        false
    }

    fn measure<I, R>(
        self,
        _id: &CaseId<O, K>,
        _batch: &[I],
        _run: impl Fn(&I) -> R + Copy,
    ) -> Self {
        self
    }
}

/// Counts every case and returns the line of every cell, in the order of [`visit`].
fn count_all<S: Suite>(name: &str) -> Result<Vec<String>, String> {
    let cells = visit::<S, _>(AllCells { cells: Vec::new() }).cells;
    let valgrind = valgrind().map_err(|error| error.to_string())?;
    let program =
        std::env::current_exe().map_err(|error| format!("the bench executable: {error}"))?;
    let directory = std::env::temp_dir().join(format!("{name}-{}", std::process::id()));
    fs::create_dir_all(&directory).map_err(|error| format!("{}: {error}", directory.display()))?;
    let workers = thread::available_parallelism().map_or(1, NonZero::get);
    let counter = Counter {
        valgrind: &valgrind,
        program: &program,
        directory: &directory,
    };
    // Worker `w` takes the cells `w`, `w + workers`, `w + 2 × workers`, and so on.
    let lines = thread::scope(|scope| {
        let handles: Vec<_> = (0..workers)
            .map(|worker| {
                let (cells, counter) = (&cells, &counter);
                scope.spawn(move || {
                    cells
                        .iter()
                        .enumerate()
                        .skip(worker)
                        .step_by(workers)
                        .map(|(index, (id, call))| Ok((index, counter.line(index, id, call)?)))
                        .collect::<Result<Vec<_>, String>>()
                })
            })
            .collect();
        handles
            .into_iter()
            .map(|handle| {
                handle
                    .join()
                    .map_err(|panic| format!("a counting thread panicked: {panic:?}"))?
            })
            .collect::<Result<Vec<_>, String>>()
    })?;
    fs::remove_dir_all(&directory).map_err(|error| format!("{}: {error}", directory.display()))?;
    let mut lines: Vec<(usize, String)> = lines.into_iter().flatten().collect();
    lines.sort_unstable_by_key(|(index, _)| *index);
    Ok(lines.into_iter().map(|(_, line)| line).collect())
}

/// Runs the bench under Callgrind.
struct Counter<'a> {
    valgrind: &'a Path,
    program: &'a Path,
    /// Where Callgrind writes its profiles.
    directory: &'a Path,
}

impl Counter<'_> {
    /// The JSON line of the cell `index`: its operation, row and crate, then either the function
    /// with the instructions of its batch or the reason for its absence.
    fn line<O: std::fmt::Display, K: crate::record::Key>(
        &self,
        index: usize,
        id: &CaseId<O, K>,
        call: &Call,
    ) -> Result<String, String> {
        let outcome = match call {
            Call::Api(function) => Outcome::Counted(Counted {
                function: (*function).to_owned(),
                operations: BATCH as u32,
                instructions: self.instructions(index, id)?,
            }),
            Call::Absent(reason) => Outcome::Absent((*reason).to_owned()),
        };
        let cell = Cell {
            op: id.op.to_string(),
            key: id.key.clone(),
            library: id.lib.to_owned(),
            outcome,
        };
        serde_json::to_string(&cell).map_err(|error| format!("{id}: {error}"))
    }

    /// The instructions of the batch of the case `id`, from the entry of [`run_batch`] to its
    /// exit, counted by Callgrind in a run of `--count <id>`.
    fn instructions<O: std::fmt::Display, K: std::fmt::Display>(
        &self,
        index: usize,
        id: &CaseId<O, K>,
    ) -> Result<u64, String> {
        let profile = self.directory.join(format!("{index}.callgrind"));
        // The counted run starts with an empty environment: the caller's variables would shift
        // its stack, and the alignment of a copy can change the instructions it takes.
        let output = Command::new(self.valgrind)
            .env_clear()
            .arg("--tool=callgrind")
            .arg("--toggle-collect=*run_batch*")
            .arg(format!("--callgrind-out-file={}", profile.display()))
            .arg(self.program)
            .args(["--count", &id.to_string()])
            .output()
            .map_err(|error| format!("{}: {error}", self.valgrind.display()))?;
        if !output.status.success() {
            return Err(format!(
                "{id}: valgrind: {}\n{}",
                output.status,
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        let text = fs::read_to_string(&profile)
            .map_err(|error| format!("{}: {error}", profile.display()))?;
        fs::remove_file(&profile).map_err(|error| format!("{}: {error}", profile.display()))?;
        if !text.lines().any(|line| line == "events: Ir") {
            return Err(format!("{id}: the profile counts other events than Ir"));
        }
        let total = text
            .lines()
            .find_map(|line| line.strip_prefix("totals: "))
            .ok_or_else(|| format!("{id}: the profile has no totals"))?;
        match total.trim().parse::<u64>() {
            Ok(0) => Err(format!(
                "{id}: Callgrind counted no instruction in run_batch"
            )),
            Ok(instructions) => Ok(instructions),
            Err(error) => Err(format!("{id}: totals {total:?}: {error}")),
        }
    }
}

/// Runs the case `wanted` once through [`run_batch`], after its check.
struct CountOne<'a> {
    wanted: &'a str,
    /// The operations that `run_batch` ran.
    operations: Option<usize>,
}

impl<O: std::fmt::Display, K: std::fmt::Display> Visitor<O, K> for CountOne<'_> {
    fn wants(&self, id: &CaseId<O, K>) -> bool {
        id.to_string() == self.wanted
    }

    fn measure<I, R>(self, _id: &CaseId<O, K>, batch: &[I], run: impl Fn(&I) -> R + Copy) -> Self {
        Self {
            operations: Some(run_batch(batch, run)),
            ..self
        }
    }
}

/// Runs the case `wanted` for Callgrind; fails unless it ran the whole batch.
fn count<S: Suite>(name: &str, wanted: &str) -> ExitCode {
    let count = visit::<S, _>(CountOne {
        wanted,
        operations: None,
    });
    match count.operations {
        Some(BATCH) => ExitCode::SUCCESS,
        Some(operations) => {
            eprintln!("{name}: {wanted} ran {operations} operations of {BATCH}");
            ExitCode::FAILURE
        }
        None => {
            eprintln!("{name}: no case {wanted}");
            ExitCode::FAILURE
        }
    }
}
