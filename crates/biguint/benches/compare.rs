//! Compares `biguint` with other Rust big-integer crates: add, mul, div, mod and mulmod at 64 to
//! 4096 bits, in instructions per operation.
//!
//! Without arguments, as `cargo bench` runs it, the bench counts every case under Valgrind's
//! Callgrind, on all the threads that the run may use, and prints every cell of the comparison as
//! one JSON object per line. Each count runs the bench again with `--count <case>`, which
//! checks the results of the case and then runs its batch once through [`run_batch`]. Callgrind
//! counts the instructions from the entry of `run_batch` to its exit. `biguint-bench-report`
//! writes `BENCHMARKS.md` from the lines.

mod suite;

use std::fs;
use std::num::NonZero;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};
use std::thread;

use suite::{BATCH, Call, CaseId, Visitor, run_batch};

fn main() -> ExitCode {
    // `cargo bench` passes `--bench`.
    let args: Vec<String> = std::env::args()
        .skip(1)
        .filter(|arg| arg != "--bench")
        .collect();
    match args.as_slice() {
        [] => match count_all() {
            Ok(lines) => {
                for line in lines {
                    println!("{line}");
                }
                ExitCode::SUCCESS
            }
            Err(error) => {
                eprintln!("compare: {error}");
                ExitCode::FAILURE
            }
        },
        [flag, case] if flag == "--count" => count(case),
        _ => {
            eprintln!("usage: compare [--count <op/bits/crate>]");
            ExitCode::FAILURE
        }
    }
}

/// Every cell of the comparison.
#[derive(Default)]
struct Cells {
    cells: Vec<(CaseId, Call)>,
}

impl Visitor for Cells {
    fn note(&mut self, id: &CaseId, call: &Call) {
        self.cells.push((*id, *call));
    }

    fn wants(&self, _id: &CaseId) -> bool {
        false
    }

    fn measure<I, O>(self, _id: &CaseId, _batch: &[I], _run: impl Fn(&I) -> O + Copy) -> Self {
        self
    }
}

/// Counts every case and returns the line of every cell, in the order of [`suite::visit`].
fn count_all() -> Result<Vec<String>, String> {
    let cells = suite::visit(Cells::default()).cells;
    let valgrind = valgrind()?;
    let program =
        std::env::current_exe().map_err(|error| format!("the bench executable: {error}"))?;
    let directory = std::env::temp_dir().join(format!("biguint-compare-{}", std::process::id()));
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

/// The path of `valgrind` in `PATH`.
fn valgrind() -> Result<PathBuf, String> {
    let path = std::env::var_os("PATH").ok_or("PATH is not set")?;
    std::env::split_paths(&path)
        .map(|directory| directory.join("valgrind"))
        .find(|candidate| candidate.is_file())
        .ok_or_else(|| "the counts need Valgrind, and PATH has no valgrind".to_owned())
}

/// Runs the bench under Callgrind.
struct Counter<'a> {
    valgrind: &'a Path,
    program: &'a Path,
    /// Where Callgrind writes its profiles.
    directory: &'a Path,
}

impl Counter<'_> {
    /// The JSON line of the cell `index`: its operation, width and crate, then either the
    /// function with the instructions of its batch or the reason for its absence.
    fn line(&self, index: usize, id: &CaseId, call: &Call) -> Result<String, String> {
        let outcome = match call {
            Call::Api(function) => format!(
                "\"counted\":{{\"function\":{function:?},\"operations\":{BATCH},\
                 \"instructions\":{}}}",
                self.instructions(index, id)?
            ),
            Call::Absent(reason) => format!("\"absent\":{reason:?}"),
        };
        Ok(format!(
            "{{\"op\":\"{}\",\"bits\":{},\"crate\":\"{}\",{outcome}}}",
            id.op, id.bits, id.lib
        ))
    }

    /// The instructions of the batch of the case `id`, from the entry of [`run_batch`] to its
    /// exit, counted by Callgrind in a run of `--count <id>`.
    fn instructions(&self, index: usize, id: &CaseId) -> Result<u64, String> {
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
struct Count<'a> {
    wanted: &'a str,
    /// The operations that `run_batch` ran.
    operations: Option<usize>,
}

impl Visitor for Count<'_> {
    fn wants(&self, id: &CaseId) -> bool {
        id.to_string() == self.wanted
    }

    fn measure<I, O>(self, _id: &CaseId, batch: &[I], run: impl Fn(&I) -> O + Copy) -> Self {
        Self {
            operations: Some(run_batch(batch, run)),
            ..self
        }
    }
}

/// Runs the case `wanted` for Callgrind; fails unless it ran the whole batch.
fn count(wanted: &str) -> ExitCode {
    let count = suite::visit(Count {
        wanted,
        operations: None,
    });
    match count.operations {
        Some(BATCH) => ExitCode::SUCCESS,
        Some(operations) => {
            eprintln!("compare: {wanted} ran {operations} operations of {BATCH}");
            ExitCode::FAILURE
        }
        None => {
            eprintln!("compare: no case {wanted}");
            ExitCode::FAILURE
        }
    }
}
