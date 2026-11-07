//! The `BENCHMARKS.md` of a library, and the odd cells of its tables.

use std::collections::HashMap;
use std::fs;

use bench_compare::{
    Count, FEWEST_STEP_CYCLES, Key, Outcome, Overhead, REPETITIONS, RETRIES, Sweep,
};

use crate::changed::CellId;
use crate::record::{Measurement, Record, Recorded};
use crate::run::Paths;
use crate::tool_error::{ToolError, UnmatchedVerdict};
use crate::wasm::SELECTED_RUNS;

/// The 1-minute load average from which a sweep ran under load.
pub const LOAD: f64 = 2.0;

/// A crate's figure on a row is odd above this many times its figure on a row of more bits.
const ODD: f64 = 1.1;

/// The subject trails a row from this many cycles above the row's fewest.
const TRAIL: f64 = 0.5;

/// An operation of a comparison, as the document names it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Operation<'a> {
    /// The operation in the record, such as `add_vartime`.
    pub op: &'a str,
    /// The operation in a row of its `Functions` table, such as `add`.
    pub name: &'a str,
    /// What it computes, such as `a + b`.
    pub formula: &'a str,
    /// Whether its functions run in time that depends on no value.
    pub constant_time: bool,
}

impl<'a> Operation<'a> {
    /// The default, variable-time operation `op` of the record, named `name`, computing
    /// `formula`.
    #[must_use]
    pub const fn new(op: &'a str, name: &'a str, formula: &'a str) -> Self {
        Self {
            op,
            name,
            formula,
            constant_time: false,
        }
    }

    /// The constant-time operation `op`, named `name` and marked in its table's title.
    #[must_use]
    pub const fn constant_time(op: &'a str, name: &'a str, formula: &'a str) -> Self {
        Self {
            op,
            name,
            formula,
            constant_time: true,
        }
    }

    /// The operation as its table's title names it: `add`, or `add, constant time`.
    fn title(&self) -> String {
        if self.constant_time {
            format!("{}, constant time", self.name)
        } else {
            self.name.to_owned()
        }
    }
}

/// The words of a comparison: what its rows are and how the document names them.
pub struct Layout<'a, K> {
    /// The crate under test.
    pub subject: &'a str,
    /// The package of the report tool, for the `Reproduce` section.
    pub tool: &'a str,
    /// Every operation, in the order of the result tables.
    pub operations: &'a [Operation<'a>],
    /// The heading of the row column of the result tables.
    pub row_header: &'a str,
    /// A phrase that names rows, after an operation or a function: `at 64 and 128 bits`.
    pub rows_phrase: fn(&[K]) -> String,
    /// The size of a row in bits, which orders the rows of a crate for the odd cells.
    pub bits: fn(&K) -> u32,
    /// Facts of the `Method` section that reading the figures needs, one short line each.
    pub method: &'a [&'a str],
    /// The name of the native build that the WebAssembly figures compare with.
    pub wasm_native: &'a str,
}

/// A build and its table.
type MeasuredTable<'r, K> = (&'r Measurement<K>, Table<'r, K>);

/// The table of each build of `record` and `wasm`, which hold the same cells; a verdict that compares
/// figures of different runs is a [`ToolError::Unmatched`].
fn checked_tables<'r, K: Key>(
    layout: &'r Layout<'_, K>,
    record: &'r Record<K>,
    wasm: Option<&'r Record<K>>,
) -> Result<Vec<MeasuredTable<'r, K>>, ToolError> {
    let tables = record
        .builds
        .iter()
        .chain(wasm.iter().flat_map(|wasm| wasm.builds.iter()))
        .map(|build| Ok((build, Table::new(&build.cells, layout)?)))
        .collect::<Result<Vec<_>, ToolError>>()?;
    let [(_, first), ..] = tables.as_slice() else {
        return Err(ToolError::missing("build", "the run record"));
    };
    for (build, table) in &tables {
        if table.ops != first.ops || table.keys != first.keys || table.crates != first.crates {
            return Err(ToolError::missing(
                &format!("the cells of the first build in the build {}", build.name),
                "the run records",
            ));
        }
        check_matched(layout, table, build)?;
    }
    Ok(tables)
}

/// Whether [`report`] takes `record` and `wasm`.
pub fn check_records<K: Key>(
    layout: &Layout<'_, K>,
    record: &Record<K>,
    wasm: Option<&Record<K>>,
) -> Result<(), ToolError> {
    checked_tables(layout, record, wasm).map(drop)
}

/// Writes the document from `record` and from `wasm`, the record of the WebAssembly builds when
/// there is one, and prints to the terminal every odd cell: a crate's figure on a row that exceeds
/// its figure on a row of more bits, in the same table, by more than 10%. Then it prints every
/// trailing cell of the native and the WebAssembly builds: a figure of the subject 0.5 cycles or
/// more above the fewest figure of its row and part among the functions that keep the row's
/// definition. It refuses a record where a figure of the subject and the figure it is compared with
/// come from different runs: [`ToolError::Unmatched`].
pub fn report<K: Key>(
    paths: &Paths,
    layout: &Layout<'_, K>,
    record: &Record<K>,
    wasm: Option<&Record<K>>,
) -> Result<(), ToolError> {
    let tables = checked_tables(layout, record, wasm)?;
    let [(_, first), ..] = tables.as_slice() else {
        return Err(ToolError::missing("build", "the run record"));
    };
    let (native, browsers) = tables.split_at(record.builds.len());
    let mut lines = header(record, layout, first);
    lines.extend(setup(record, "## Setup"));
    lines.extend(method(layout, record));
    lines.extend(functions(layout, first));
    for (build, table) in native {
        lines.extend(results(layout, table, build));
    }
    if let Some(wasm) = wasm {
        let (_, reference) = native
            .iter()
            .find(|(build, _)| build.name == layout.wasm_native)
            .ok_or_else(|| {
                ToolError::missing(
                    &format!("the native build `{}`", layout.wasm_native),
                    "the run record",
                )
            })?;
        lines.extend(webassembly(wasm, layout, first));
        lines.extend(against_competitors(layout, browsers));
        lines.extend(against_native(layout, reference, browsers));
    }
    lines.extend(reproduce(layout.tool, wasm.is_some()));
    let path = paths.library.join("BENCHMARKS.md");
    fs::write(&path, lines.join("\n") + "\n").map_err(|source| ToolError::io(&path, source))?;
    let odd: Vec<String> = tables
        .iter()
        .flat_map(|(build, table)| odd_cells(layout, table, build))
        .collect();
    println!(
        "{}: {} odd cells, where a crate's figure exceeds its figure on a row of more bits by \
         more than {:.0}%",
        path.display(),
        odd.len(),
        100.0 * (ODD - 1.0)
    );
    for cell in odd {
        println!("{cell}");
    }
    let trailing: Vec<String> = tables
        .iter()
        .flat_map(|(build, table)| trailing_cells(layout, table, build))
        .collect();
    println!(
        "{}: {} trailing cells, where `{}` is {TRAIL} cycles or more above the fewest figure",
        path.display(),
        trailing.len(),
        layout.subject
    );
    for cell in trailing {
        println!("{cell}");
    }
    Ok(())
}

/// Fails on the first verdict of `table`, the cells of `build`, that compares figures of different
/// runs.
fn check_matched<K: Key>(
    layout: &Layout<'_, K>,
    table: &Table<'_, K>,
    build: &Measurement<K>,
) -> Result<(), ToolError> {
    let run = |cell: &Recorded<K>| {
        build.runs.get(cell.run).map_or_else(
            || format!("unknown run {}", cell.run),
            |run| format!("run {} at {}", cell.run, run.commit),
        )
    };
    for verdict in table.verdicts(layout) {
        if let Some(rival) = verdict.unmatched() {
            return Err(ToolError::Unmatched(Box::new(UnmatchedVerdict {
                build: build.name.clone(),
                op: verdict.operation.op.to_owned(),
                row: verdict.key.to_string(),
                part: PARTS[verdict.part].1,
                subject: layout.subject.to_owned(),
                subject_run: run(verdict.subject),
                rival: rival.cell.library.clone(),
                rival_run: run(rival),
            })));
        }
    }
    Ok(())
}

/// The title of the table of `operation` in `build`.
fn title<K>(operation: &Operation<'_>, build: &Measurement<K>) -> String {
    let (name, formula) = (operation.title(), operation.formula);
    if build.name.is_empty() {
        format!("{name}: {formula}")
    } else {
        format!("{name}: {formula}, {}", build.name)
    }
}

/// Whether `sweep` ran under load.
fn loaded(sweep: &Sweep) -> bool {
    sweep.load.before >= LOAD || sweep.load.after >= LOAD
}

/// The cells of a build, indexed by operation, row and crate.
pub(crate) struct Table<'a, K> {
    /// Each operation, in the order of the layout.
    ops: Vec<Operation<'a>>,
    keys: Vec<K>,
    crates: Vec<&'a str>,
    cells: HashMap<(&'a str, &'a K, &'a str), &'a Recorded<K>>,
}

/// What a verdict on the subject compares in one part of one row.
pub(crate) struct Verdict<'a, K> {
    /// The operation.
    pub(crate) operation: Operation<'a>,
    /// The row.
    pub(crate) key: &'a K,
    /// The part of the row, an index of [`parts`].
    pub(crate) part: usize,
    /// The cell of the subject's figure.
    pub(crate) subject: &'a Recorded<K>,
    /// The cells of the row's fewest figure, then of the fewest figure of the other crates; each
    /// with every tie.
    pub(crate) rivals: [Vec<&'a Recorded<K>>; 2],
}

impl<'a, K> Verdict<'a, K> {
    /// A cell that the subject is compared with, where no cell tied with it comes from the subject's
    /// run.
    pub(crate) fn unmatched(&self) -> Option<&'a Recorded<K>> {
        self.rivals
            .iter()
            .filter(|tied| !tied.iter().any(|cell| cell.run == self.subject.run))
            .find_map(|tied| tied.first().copied())
    }
}

/// The cells of one row as a round of re-measuring sees them.
pub(crate) struct ComparedRow<K> {
    /// The cells that the row holds or shows.
    pub(crate) members: Vec<CellId<K>>,
    /// The cells that a verdict on the row compares: the columns of the subject and the leaders.
    pub(crate) compared: Vec<CellId<K>>,
}

impl<'a, K: Key> Table<'a, K> {
    /// Fails unless `cells` hold exactly one cell for every operation, row and crate, and the
    /// layout names every operation.
    pub(crate) fn new(cells: &'a [Recorded<K>], layout: &Layout<'a, K>) -> Result<Self, ToolError> {
        let recorded = unique(cells.iter().map(|cell| cell.cell.op.as_str()));
        if let Some(op) = recorded.iter().find(|op| {
            !layout
                .operations
                .iter()
                .any(|operation| operation.op == **op)
        }) {
            return Err(ToolError::missing(&format!("operation {op}"), "the layout"));
        }
        let ops = layout
            .operations
            .iter()
            .filter(|operation| recorded.contains(&operation.op))
            .copied()
            .collect();
        let table = Self {
            ops,
            keys: unique(cells.iter().map(|cell| cell.cell.key.clone())),
            crates: unique(cells.iter().map(|cell| cell.cell.library.as_str())),
            cells: cells
                .iter()
                .map(|cell| {
                    (
                        (
                            cell.cell.op.as_str(),
                            &cell.cell.key,
                            cell.cell.library.as_str(),
                        ),
                        cell,
                    )
                })
                .collect(),
        };
        if table.cells.len() != cells.len() {
            return Err(ToolError::missing(
                "single cell per operation, row and crate",
                "the run record",
            ));
        }
        for Operation { op, .. } in &table.ops {
            for key in &table.keys {
                for library in &table.crates {
                    if !table.cells.contains_key(&(*op, key, *library)) {
                        return Err(ToolError::missing(
                            &format!("cell {op}/{key}/{library}"),
                            "the run record",
                        ));
                    }
                }
            }
        }
        Ok(table)
    }

    /// The crates, in table order, with a counted cell of any of `ops` on any row; a table shows
    /// no column of a crate that has none.
    fn measured(&self, ops: &[Operation<'_>]) -> Vec<&'a str> {
        self.crates
            .iter()
            .copied()
            .filter(|library| {
                ops.iter().any(|operation| {
                    self.keys
                        .iter()
                        .any(|key| self.figure(operation, key, library).is_some())
                })
            })
            .collect()
    }

    /// The figure a table of `operation` shows on the row `key` for `library`: its own count, or,
    /// in a variable-time table without one, the count of the constant-time twin of the same name.
    fn figure(&self, operation: &Operation<'_>, key: &K, library: &str) -> Option<Figure<'a, K>> {
        if let Some(figure) = self.count(operation.op, key, library) {
            return Some(figure);
        }
        if operation.constant_time {
            return None;
        }
        self.ops
            .iter()
            .find(|twin| twin.constant_time && twin.name == operation.name)
            .and_then(|twin| self.count(twin.op, key, library))
    }

    /// The figure of `op` on the row `key` in `library`; `None` for an absent cell.
    fn count(&self, op: &str, key: &K, library: &str) -> Option<Figure<'a, K>> {
        let cell = *self.cells.get(&(op, key, library))?;
        match &cell.cell.outcome {
            Outcome::Counted(count) => Some(Figure { count, cell }),
            Outcome::Absent(_) => None,
        }
    }

    /// The cells whose figures tie for the fewest of `part` on the row `key` of `operation`, among
    /// the crates that `wanted` takes and the functions that keep the row's definition.
    fn tied(
        &self,
        operation: &Operation<'_>,
        key: &K,
        part: usize,
        wanted: impl Fn(&str) -> bool,
    ) -> Vec<&'a Recorded<K>> {
        let figures: Vec<(f64, &'a Recorded<K>)> = self
            .crates
            .iter()
            .filter(|library| wanted(library))
            .filter_map(|library| self.figure(operation, key, library))
            .filter(|figure| figure.count.deviation.is_none())
            .filter_map(|figure| Some((shown(parts(figure.count)[part]?), figure.cell)))
            .collect();
        let fewest = figures.iter().map(|(number, _)| *number).reduce(f64::min);
        figures
            .into_iter()
            .filter(|(number, _)| Some(*number) == fewest)
            .map(|(_, cell)| cell)
            .collect()
    }

    /// What the verdicts on the subject compare: per operation, row and part where the subject has
    /// a figure, that figure, the row's fewest and the fewest of the other crates.
    pub(crate) fn verdicts(&self, layout: &Layout<'_, K>) -> Vec<Verdict<'a, K>> {
        let mut verdicts = Vec::new();
        for operation in &self.ops {
            for key in &self.keys {
                let Some(ours) = self.figure(operation, key, layout.subject) else {
                    continue;
                };
                for (part, _) in PARTS {
                    if parts(ours.count)[part].is_none() {
                        continue;
                    }
                    let fewest = self.tied(operation, key, part, |_| true);
                    let others = |library: &str| !library.starts_with(layout.subject);
                    let best = self.tied(operation, key, part, others);
                    verdicts.push(Verdict {
                        operation: *operation,
                        key: &ours.cell.cell.key,
                        part,
                        subject: ours.cell,
                        rivals: [fewest, best],
                    });
                }
            }
        }
        verdicts
    }

    /// The cells of the row of `verdict`, as its table shows them.
    pub(crate) fn row_cells<'s>(
        &'s self,
        verdict: &'s Verdict<'_, K>,
    ) -> impl Iterator<Item = &'a Recorded<K>> + 's {
        self.crates.iter().filter_map(move |library| {
            self.figure(&verdict.operation, verdict.key, library)
                .map(|figure| figure.cell)
        })
    }

    /// Every row, as a round of re-measuring sees it.
    pub(crate) fn rows(&self, layout: &Layout<'_, K>) -> Vec<ComparedRow<K>> {
        let id = |cell: &Recorded<K>| CellId::of(&cell.cell);
        let mut rows = Vec::new();
        for operation in &self.ops {
            for key in &self.keys {
                let mut members = Vec::new();
                for library in &self.crates {
                    members.extend(
                        self.cells
                            .get(&(operation.op, key, *library))
                            .map(|cell| id(cell)),
                    );
                    members.extend(
                        self.figure(operation, key, library)
                            .map(|figure| id(figure.cell)),
                    );
                }
                let mut compared: Vec<CellId<K>> = self
                    .crates
                    .iter()
                    .filter(|library| library.starts_with(layout.subject))
                    .filter_map(|library| self.figure(operation, key, library))
                    .map(|figure| id(figure.cell))
                    .collect();
                let others = |library: &str| !library.starts_with(layout.subject);
                for (part, _) in PARTS {
                    compared.extend(
                        self.tied(operation, key, part, |_| true)
                            .into_iter()
                            .map(id),
                    );
                    compared.extend(self.tied(operation, key, part, others).into_iter().map(id));
                }
                rows.push(ComparedRow { members, compared });
            }
        }
        rows
    }
}

/// A count as a table shows it, with the cell that holds it.
struct Figure<'a, K> {
    count: &'a Count,
    cell: &'a Recorded<K>,
}

impl<K> Clone for Figure<'_, K> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<K> Copy for Figure<'_, K> {}

impl<K> Figure<'_, K> {
    /// The function as a `Functions` cell names it.
    fn function(self) -> String {
        format!("`{}`", self.count.function)
    }
}

/// The odd cells of `table` in `build`, one line each: a crate's figure, as the table shows it,
/// above [`ODD`] times its figure on a row of more bits.
fn odd_cells<K: Key>(
    layout: &Layout<'_, K>,
    table: &Table<'_, K>,
    build: &Measurement<K>,
) -> Vec<String> {
    let mut lines = Vec::new();
    for operation in &table.ops {
        let title = title(operation, build);
        for library in &table.crates {
            for (part, name) in ["throughput", "latency"].into_iter().enumerate() {
                let figure = |key: &K| {
                    table
                        .figure(operation, key, library)
                        .and_then(|figure| parts(figure.count)[part])
                        .map(shown)
                };
                for small in &table.keys {
                    for large in &table.keys {
                        let (Some(less), Some(more)) = (figure(small), figure(large)) else {
                            continue;
                        };
                        if (layout.bits)(small) < (layout.bits)(large) && less > ODD * more {
                            lines.push(format!(
                                "{title} | {library} | {name} {}: {} > {}: {}",
                                (layout.rows_phrase)(std::slice::from_ref(small)),
                                value(less),
                                (layout.rows_phrase)(std::slice::from_ref(large)),
                                value(more)
                            ));
                        }
                    }
                }
            }
        }
    }
    lines
}

/// The trailing cells of `table` in `build`, one line each: a figure of the subject, as the table
/// shows it, [`TRAIL`] cycles or more above the fewest figure of its row and part among the
/// figures without a deviation.
fn trailing_cells<K: Key>(
    layout: &Layout<'_, K>,
    table: &Table<'_, K>,
    build: &Measurement<K>,
) -> Vec<String> {
    let mut lines = Vec::new();
    for operation in &table.ops {
        let title = title(operation, build);
        for key in &table.keys {
            let Some(ours) = table.figure(operation, key, layout.subject) else {
                continue;
            };
            for (part, name) in ["throughput", "latency"].into_iter().enumerate() {
                let Some(own) = parts(ours.count)[part].map(shown) else {
                    continue;
                };
                let best = table
                    .crates
                    .iter()
                    .filter_map(|library| {
                        table
                            .figure(operation, key, library)
                            .filter(|figure| figure.count.deviation.is_none())
                            .and_then(|figure| parts(figure.count)[part])
                            .map(|figure| (shown(figure), *library))
                    })
                    .reduce(|least, other| if other.0 < least.0 { other } else { least });
                if let Some((fewest, library)) = best
                    && own - fewest >= TRAIL
                {
                    lines.push(format!(
                        "{title} | {} | {name} {} against {library} {}",
                        (layout.rows_phrase)(core::slice::from_ref(key)),
                        value(own),
                        value(fewest)
                    ));
                }
            }
        }
    }
    lines
}

/// The values of `items` in first-seen order, without repeats.
fn unique<T: PartialEq>(items: impl Iterator<Item = T>) -> Vec<T> {
    let mut values = Vec::new();
    for item in items {
        if !values.contains(&item) {
            values.push(item);
        }
    }
    values
}

/// `values` joined as in a sentence: `a, b and c`.
#[must_use]
pub fn sentence<T: ToString>(values: &[T]) -> String {
    let words: Vec<String> = values.iter().map(ToString::to_string).collect();
    match words.split_last() {
        Some((last, [])) => last.clone(),
        Some((last, rest)) => format!("{} and {last}", rest.join(", ")),
        None => String::new(),
    }
}

/// A Markdown table; `align` holds `l` or `r` per column.
#[must_use]
pub fn markdown_table(header: &[String], rows: &[Vec<String>], align: &str) -> Vec<String> {
    let rule: Vec<&str> = align
        .chars()
        .map(|side| if side == 'r' { "---:" } else { "---" })
        .collect();
    let mut lines = vec![
        format!("| {} |", header.join(" | ")),
        format!("|{}|", rule.join("|")),
    ];
    lines.extend(rows.iter().map(|row| format!("| {} |", row.join(" | "))));
    lines
}

fn header<K: Key>(record: &Record<K>, layout: &Layout<'_, K>, table: &Table<'_, K>) -> Vec<String> {
    vec![
        "# Benchmarks".to_owned(),
        String::new(),
        summary(record, layout, table, ""),
        String::new(),
    ]
}

/// The sentence that names what `record` measured: the commit, the compared crates, `how` and the
/// date.
fn summary<K: Key>(
    record: &Record<K>,
    layout: &Layout<'_, K>,
    table: &Table<'_, K>,
    how: &str,
) -> String {
    let others: Vec<String> = table
        .crates
        .iter()
        .filter(|library| **library != layout.subject)
        .map(|library| format!("`{library}`"))
        .collect();
    let state = if record.dirty {
        " The measured files had uncommitted changes."
    } else {
        ""
    };
    format!(
        "`{}` at commit `{}` against {}{how}, in cycles per operation, measured on {}.{state}",
        layout.subject,
        short(&record.library_commit),
        sentence(&others),
        record.date
    )
}

/// The section of the WebAssembly builds of `record`, before their results: how they ran and
/// their setup.
fn webassembly<K: Key>(
    record: &Record<K>,
    layout: &Layout<'_, K>,
    table: &Table<'_, K>,
) -> Vec<String> {
    let mut lines = vec![
        "## WebAssembly".to_owned(),
        String::new(),
        summary(
            record,
            layout,
            table,
            ", built for `wasm32-unknown-unknown` and run in headless browsers",
        ),
        String::new(),
    ];
    lines.extend(WEBASSEMBLY.iter().map(|line| format!("- {line}")));
    lines.push(String::new());
    lines.extend(setup(record, "### Setup, WebAssembly"));
    lines
}

/// How the WebAssembly builds ran, one short line each.
const WEBASSEMBLY: &[&str] = &[
    "The bench module runs in a dedicated worker of the browser, on the measuring CPU. A driver \
     counts that thread with perf_event, and holds the CPU while the thread sleeps.",
    "A second worker carries the start and the end of each count to the driver while the \
     measuring worker sleeps.",
    "The browser's flags compile every function with its optimizing tier when the module loads.",
    "A counted loop runs behind a function pointer, which keeps V8 from inlining it into the code \
     around the count.",
    "A batch loop reads its step and its end by volatile loads, so LLVM unrolls it for no \
     operation, whatever the operation's size. A chain through locals over operand sets of more \
     than 32 bytes reads its step the same way.",
    "A run passes over its operand sets, of up to 256 KiB, as many times as it takes to count 4 \
     million instructions. They start at a 4 KiB boundary in every case.",
    "A figure of cycles is the lower rate of the fewest run of n operations and the fewest of 2n, \
     each less the fewest empty count: under load, a run of each length seldom meets an idle \
     sibling hardware thread.",
    "A case whose 2n runs counted no more than its n runs, whose difference counted under 50 times \
     the noise of the fixed cost of a count, or whose chain step counted under 0.75 cycles ran 31 \
     more repetitions, up to 3 times. A case that stays so stops the sweep.",
    "Latency: `x` in WebAssembly locals, with a copy stored after each step, or in memory, \
     whichever is faster.",
];

/// The first seven characters of a commit hash.
fn short(commit: &str) -> &str {
    commit.get(..7).unwrap_or(commit)
}

/// The setup table of `record` under `heading`.
fn setup<K>(record: &Record<K>, heading: &str) -> Vec<String> {
    let mut cpu = format!("{} (`{}`)", record.cpu_model, record.native_cpu);
    if let Some(machine) = &record.machine {
        cpu += &format!("; {machine}");
    }
    let crates: Vec<String> = record
        .versions
        .iter()
        .map(|entry| format!("{} {}", entry.name, entry.version))
        .collect();
    let mut rows = vec![
        vec!["CPU".to_owned(), cpu],
        vec![
            "Compiler".to_owned(),
            format!("{}, LLVM {}", record.rustc, record.llvm),
        ],
        vec![
            "Profile".to_owned(),
            format!("`bench`: `{}`", record.profile.join("`, `")),
        ],
        vec!["Features".to_owned(), features(record)],
    ];
    let label = |what: &str, build: &Measurement<K>| {
        if build.name.is_empty() {
            what.to_owned()
        } else {
            format!("{what}, {}", build.name)
        }
    };
    for build in &record.builds {
        rows.push(vec![
            label("RUSTFLAGS", build),
            format!("`{}`", build.rustflags),
        ]);
    }
    for build in &record.builds {
        if let Some(browser) = &build.browser {
            rows.push(vec![
                label("Browser", build),
                format!("{}; {}", browser.version, browser.settings),
            ]);
            rows.push(vec![
                label("Target features", build),
                format!("`{}`", build.target_features.join("`, `")),
            ]);
            rows.push(vec![label("Cost of a count", build), overheads(build)]);
        }
    }
    for build in &record.builds {
        for (index, run) in build.runs.iter().enumerate() {
            let counted = build
                .cells
                .iter()
                .filter(|cell| cell.counted() && cell.run == index)
                .count();
            let state = if run.dirty {
                ", with uncommitted changes"
            } else {
                ""
            };
            rows.push(vec![
                label(&format!("Run {}", index + 1), build),
                format!(
                    "`{}`{state}, {}: {counted} cells; {}",
                    short(&run.commit),
                    run.date,
                    load(&run.sweep)
                ),
            ]);
        }
    }
    rows.push(vec!["Crates".to_owned(), crates.join(", ")]);
    let mut lines = vec![heading.to_owned(), String::new()];
    lines.extend(markdown_table(&[String::new(), String::new()], &rows, "ll"));
    lines.push(String::new());
    lines
}

/// The cost of an empty count in the benches of `build`, from the fewest to the most of them.
fn overheads<K>(build: &Measurement<K>) -> String {
    let range = |part: fn(&Overhead) -> f64| {
        let values: Vec<f64> = build.overheads.iter().map(part).collect();
        let least = values.iter().copied().fold(f64::INFINITY, f64::min);
        let most = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        if value(least) == value(most) {
            value(least)
        } else {
            format!("{} to {}", value(least), value(most))
        }
    };
    format!(
        "{} cycles and {} instructions; its noise in a figure, {} cycles",
        range(|overhead| overhead.cycles),
        range(|overhead| overhead.instructions),
        range(|overhead| overhead.noise)
    )
}

/// The features that the builds enable beside the default ones of the measured package.
fn features<K>(record: &Record<K>) -> String {
    if record.features.is_empty() {
        format!("`{}` with its default features", record.package)
    } else {
        let names: Vec<String> = record
            .features
            .iter()
            .map(|feature| format!("`{feature}`"))
            .collect();
        format!("`{}` built with {}", record.package, sentence(&names))
    }
}

/// The measuring CPU, the load around `sweep` and the busy share of the CPU's SMT siblings.
fn load(sweep: &Sweep) -> String {
    let mut text = format!(
        "CPU {}; 1-minute load {} before, {} after",
        sweep.cpu, sweep.load.before, sweep.load.after
    );
    for sibling in &sweep.siblings {
        text += &format!(
            "; its SMT sibling, CPU {}, {:.0}% busy",
            sibling.cpu,
            100.0 * sibling.busy
        );
    }
    text
}

fn method<K>(layout: &Layout<'_, K>, record: &Record<K>) -> Vec<String> {
    let mut lines = vec![
        "## Method".to_owned(),
        String::new(),
        "- Each cell: throughput / latency, in user-mode cycles per operation.".to_owned(),
        "- Throughput: independent operations on fresh operands.".to_owned(),
        "- Latency: the chain `x = op(x, y)`, with `x` in registers or in memory, whichever is \
         faster."
            .to_owned(),
        format!(
            "- Each native figure: over {REPETITIONS} repetitions, the fewest cycles of 2n operations \
             less the fewest of n, over n. A case whose 2n runs counted no more than its n runs, \
             or whose chain step counted under {FEWEST_STEP_CYCLES} cycles, ran {REPETITIONS} \
             more, up to {RETRIES} times."
        ),
    ];
    if record
        .builds
        .iter()
        .flat_map(|build| &build.runs)
        .any(|run| loaded(&run.sweep))
    {
        lines.push(format!(
            "- A sweep ran at a 1-minute load of {LOAD} or more: compare figures within a table \
             only."
        ));
    }
    if record.builds.iter().any(|build| build.runs.len() > 1) {
        lines.push(format!(
            "- A sweep over part of a build gives each selected cell the fewest figures of \
             {SELECTED_RUNS} runs of its bench by default, or the figures of one run of the whole \
             bench where the selection holds 1/{SELECTED_RUNS} of its cells or more. The setup table \
             names each run, and a verdict compares figures of one run."
        ));
    }
    lines.extend(layout.method.iter().map(|line| format!("- {line}")));
    lines.push(String::new());
    lines
}

/// `value` with a comma between groups of three digits.
fn grouped(value: u64) -> String {
    let digits = value.to_string();
    let mut text = String::new();
    for (index, digit) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index).is_multiple_of(3) {
            text.push(',');
        }
        text.push(digit);
    }
    text
}

/// A figure as the tables show it: one decimal below 100, whole from 100 up.
fn shown(figure: f64) -> f64 {
    if figure >= 100.0 {
        figure.round()
    } else {
        (10.0 * figure).round() / 10.0
    }
}

/// The text of a figure: one decimal below 100, whole and grouped from 100 up.
fn value(figure: f64) -> String {
    if figure >= 100.0 {
        grouped(figure.round() as u64)
    } else {
        format!("{figure:.1}")
    }
}

/// The `Functions` sections: one table for the variable-time operations, then one for the
/// constant-time operations, each with the crates that run a function of its class. A cell names
/// the crate's functions; a line under the table names the rows of a crate that runs on some
/// rows only.
fn functions<K: Key>(layout: &Layout<'_, K>, table: &Table<'_, K>) -> Vec<String> {
    let mut lines = Vec::new();
    for (constant_time, heading) in [
        (false, "## Functions"),
        (true, "## Functions, constant time"),
    ] {
        let ops: Vec<Operation<'_>> = table
            .ops
            .iter()
            .filter(|operation| operation.constant_time == constant_time)
            .copied()
            .collect();
        let crates = table.measured(&ops);
        if crates.is_empty() {
            continue;
        }
        let rows: Vec<Vec<String>> = ops
            .iter()
            .map(|operation| {
                let mut row = vec![operation.name.to_owned()];
                row.extend(crates.iter().map(|library| {
                    let figures: Vec<Figure<'_, K>> = table
                        .keys
                        .iter()
                        .filter_map(|key| table.figure(operation, key, library))
                        .collect();
                    let names = unique(figures.iter().map(|figure| figure.function()));
                    if names.is_empty() {
                        "–".to_owned()
                    } else {
                        names.join(", ")
                    }
                }));
                row
            })
            .collect();
        let mut header = vec!["Operation".to_owned()];
        header.extend(crates.iter().map(|library| (*library).to_owned()));
        lines.extend([heading.to_owned(), String::new()]);
        lines.extend(markdown_table(&header, &rows, &"l".repeat(header.len())));
        let coverage: Vec<String> = crates
            .iter()
            .filter_map(|library| coverage(layout, table, &ops, library))
            .collect();
        if !coverage.is_empty() {
            lines.push(String::new());
            lines.extend(coverage);
        }
        lines.push(String::new());
    }
    lines
}

/// The line under a `Functions` table for `library` when it runs `ops` on some rows only: its
/// rows, per operation where they differ.
fn coverage<K: Key>(
    layout: &Layout<'_, K>,
    table: &Table<'_, K>,
    ops: &[Operation<'_>],
    library: &str,
) -> Option<String> {
    let rows: Vec<(&str, Vec<K>)> = ops
        .iter()
        .map(|operation| {
            let keys = table
                .keys
                .iter()
                .filter(|key| table.figure(operation, key, library).is_some())
                .cloned()
                .collect();
            (operation.name, keys)
        })
        .filter(|(_, keys): &(&str, Vec<K>)| !keys.is_empty())
        .collect();
    if rows.iter().all(|(_, keys)| *keys == table.keys) {
        return None;
    }
    // The operations of each set of rows, in table order; the operations on every row go unnamed.
    let mut groups: Vec<(Vec<&str>, &Vec<K>)> = Vec::new();
    for (name, keys) in &rows {
        match groups.iter_mut().find(|(_, other)| *other == keys) {
            Some((names, _)) => names.push(name),
            None => groups.push((vec![name], keys)),
        }
    }
    let text = match groups.as_slice() {
        [(_, keys)] => (layout.rows_phrase)(keys),
        _ => groups
            .iter()
            .filter(|(_, keys)| **keys != table.keys)
            .map(|(names, keys)| format!("{} {}", sentence(names), (layout.rows_phrase)(keys)))
            .collect::<Vec<_>>()
            .join("; "),
    };
    Some(format!("- `{library}`: {text}."))
}

/// The legend above every results table.
const LEGEND: &str = "Each cell: throughput cycles / latency cycles per operation. \
                      Bold: the fewest of the row, every tie bold. –: no such function or no \
                      chain form.";

/// The two parts of a cell are joined by a slash between no-break spaces, so a cell never wraps.
const SEPARATOR: &str = "\u{a0}/\u{a0}";

/// The figures of one cell, in the order of its two parts.
fn parts(count: &Count) -> [Option<f64>; 2] {
    [
        Some(count.batch.cycles),
        count.chain.map(|latency| latency.events.cycles),
    ]
}

/// The results of `build`: per operation, its table.
fn results<K: Key>(
    layout: &Layout<'_, K>,
    table: &Table<'_, K>,
    build: &Measurement<K>,
) -> Vec<String> {
    let heading = if build.name.is_empty() {
        "## Results".to_owned()
    } else {
        format!("## Results {}", build.name)
    };
    let mut lines = vec![heading];
    for operation in &table.ops {
        let crates = table.measured(core::slice::from_ref(operation));
        let mut rows = Vec::new();
        // Each starred cell: its crate, its deviation and its row.
        let mut starred: Vec<(&str, &str, &K)> = Vec::new();
        for key in &table.keys {
            let figures: Vec<Option<Figure<'_, K>>> = crates
                .iter()
                .map(|library| table.figure(operation, key, library))
                .collect();
            let ours = table
                .figure(operation, key, layout.subject)
                .map(|figure| parts(figure.count));
            // The fewest of each part as the table shows it, so that every tie is bold.
            let fewest: [Option<f64>; 2] = std::array::from_fn(|part| {
                figures
                    .iter()
                    .flatten()
                    .filter(|figure| figure.count.deviation.is_none())
                    .filter_map(|figure| parts(figure.count)[part])
                    .map(shown)
                    .reduce(f64::min)
            });
            let mut row = vec![key.to_string()];
            for (library, figure) in crates.iter().zip(&figures) {
                let Some(figure) = figure else {
                    row.push("–".to_owned());
                    continue;
                };
                let cell: Vec<String> = parts(figure.count)
                    .iter()
                    .enumerate()
                    .map(|(part, number)| match (number, &figure.count.deviation) {
                        (None, _) => "–".to_owned(),
                        (Some(number), Some(deviation))
                            if ours.is_some_and(|ours| {
                                ours[part].is_some_and(|ours| *number < ours)
                            }) =>
                        {
                            if !starred.contains(&(library, deviation.as_str(), key)) {
                                starred.push((library, deviation, key));
                            }
                            format!("{}\\*", value(*number))
                        }
                        (Some(number), None) if Some(shown(*number)) == fewest[part] => {
                            format!("**{}**", value(*number))
                        }
                        (Some(number), _) => value(*number),
                    })
                    .collect();
                row.push(cell.join(SEPARATOR));
            }
            rows.push(row);
        }
        let mut header = vec![layout.row_header.to_owned()];
        header.extend(crates.iter().map(|library| (*library).to_owned()));
        let title = title(operation, build);
        let legend = if starred.is_empty() {
            LEGEND.to_owned()
        } else {
            format!(
                "{LEGEND} A `\\*` marks the figure of a function that deviates from the row's \
                 definition where the figure is below `{}`'s; the note under the table says how \
                 the function deviates.",
                layout.subject
            )
        };
        lines.extend([
            String::new(),
            format!("### {title}"),
            String::new(),
            legend,
            String::new(),
        ]);
        lines.extend(markdown_table(
            &header,
            &rows,
            &("l".to_owned() + &"r".repeat(crates.len())),
        ));
        lines.extend(deviations(layout, &starred));
    }
    lines.push(String::new());
    lines
}

/// The two parts of a cell, by name.
const PARTS: [(usize, &str); 2] = [(0, "throughput"), (1, "latency")];

/// The heading of the column of a WebAssembly build: its browser, and whether it has wide
/// arithmetic.
fn browser_column<K>(build: &Measurement<K>) -> String {
    let browser = build
        .browser
        .as_ref()
        .map_or(build.name.as_str(), |engine| engine.name.as_str());
    if build.rustflags.contains("+wide-arithmetic") {
        format!("{browser}, wide arithmetic")
    } else {
        browser.to_owned()
    }
}

/// The figure of `part` of `library` on the row `key` of `operation`, as the tables show it.
fn shown_part<K: Key>(
    table: &Table<'_, K>,
    operation: &Operation<'_>,
    key: &K,
    library: &str,
    part: usize,
) -> Option<f64> {
    table
        .figure(operation, key, library)
        .and_then(|figure| parts(figure.count)[part])
        .map(shown)
}

/// The fewest figure of `part` on the row `key` of `operation` among the other crates whose
/// functions keep the row's definition, with its crate; the first of a tie.
fn best_competitor<'a, K: Key>(
    layout: &Layout<'_, K>,
    table: &Table<'a, K>,
    operation: &Operation<'_>,
    key: &K,
    part: usize,
) -> Option<(f64, &'a str)> {
    table
        .crates
        .iter()
        .filter(|library| !library.starts_with(layout.subject))
        .filter_map(|&library| {
            let figure = table.figure(operation, key, library)?;
            let number = parts(figure.count)[part]?;
            figure
                .count
                .deviation
                .is_none()
                .then_some((shown(number), library))
        })
        .reduce(|best, next| if next.0 < best.0 { next } else { best })
}

/// The operations and rows of the WebAssembly tables of `part`: those where the subject has a
/// figure of `part` in the first build.
fn subject_rows<'t, K: Key>(
    layout: &Layout<'_, K>,
    table: &'t Table<'_, K>,
    part: usize,
) -> Vec<(Operation<'t>, &'t K)> {
    table
        .ops
        .iter()
        .flat_map(|operation| table.keys.iter().map(move |key| (*operation, key)))
        .filter(|(operation, key)| {
            shown_part(table, operation, key, layout.subject, part).is_some()
        })
        .collect()
}

/// `ours − best` with its sign, and `ours ÷ best`.
fn difference(ours: f64, best: f64) -> String {
    let delta = ours - best;
    let sign = if delta < 0.0 { "−" } else { "+" };
    format!("{sign}{} ({:.2}×)", value(delta.abs()), ours / best)
}

/// The tables of the WebAssembly builds against the best competitor, throughput then latency:
/// per operation and row, each build's figure of the subject against the fewest figure of the
/// crates that keep the row's definition.
fn against_competitors<K: Key>(
    layout: &Layout<'_, K>,
    browsers: &[(&Measurement<K>, Table<'_, K>)],
) -> Vec<String> {
    let Some((_, first)) = browsers.first() else {
        return Vec::new();
    };
    let mut lines = Vec::new();
    for (part, name) in PARTS {
        let mut header = vec!["Operation".to_owned(), layout.row_header.to_owned()];
        for (build, _) in browsers {
            header.extend([browser_column(build), "Δ".to_owned()]);
        }
        let rows: Vec<Vec<String>> = subject_rows(layout, first, part)
            .into_iter()
            .map(|(operation, key)| {
                let mut row = vec![operation.title(), key.to_string()];
                for (_, table) in browsers {
                    let ours = shown_part(table, &operation, key, layout.subject, part);
                    match (ours, best_competitor(layout, table, &operation, key, part)) {
                        (Some(ours), Some((best, library))) => row.extend([
                            format!("{} vs {library} {}", value(ours), value(best)),
                            difference(ours, best),
                        ]),
                        (Some(ours), None) => row.extend([value(ours), "–".to_owned()]),
                        (None, _) => row.extend(["–".to_owned(), "–".to_owned()]),
                    }
                }
                row
            })
            .collect();
        lines.extend([
            String::new(),
            format!("### Against the best competitor, {name}"),
            String::new(),
            format!(
                "Cycles per operation of `{}` against the fewest of the crates whose functions \
                 keep the row's definition. Δ: ours − best, then ours ÷ best; below 1 we lead.",
                layout.subject
            ),
            String::new(),
        ]);
        lines.extend(markdown_table(
            &header,
            &rows,
            &("ll".to_owned() + &"lr".repeat(browsers.len())),
        ));
    }
    lines
}

/// The tables of the WebAssembly builds against the native build `reference`, throughput then
/// latency: per operation and row, the subject's figure in each build and its ratio to the
/// native figure.
fn against_native<K: Key>(
    layout: &Layout<'_, K>,
    reference: &Table<'_, K>,
    browsers: &[(&Measurement<K>, Table<'_, K>)],
) -> Vec<String> {
    let Some((_, first)) = browsers.first() else {
        return Vec::new();
    };
    let native_column = if layout.wasm_native.is_empty() {
        "Native".to_owned()
    } else {
        format!("Native, {}", layout.wasm_native)
    };
    let mut lines = Vec::new();
    for (part, name) in PARTS {
        let mut header = vec![
            "Operation".to_owned(),
            layout.row_header.to_owned(),
            native_column.clone(),
        ];
        header.extend(browsers.iter().map(|(build, _)| browser_column(build)));
        let rows: Vec<Vec<String>> = subject_rows(layout, first, part)
            .into_iter()
            .map(|(operation, key)| {
                let native = shown_part(reference, &operation, key, layout.subject, part);
                let mut row = vec![
                    operation.title(),
                    key.to_string(),
                    native.map_or("–".to_owned(), value),
                ];
                row.extend(browsers.iter().map(|(_, table)| {
                    match (
                        shown_part(table, &operation, key, layout.subject, part),
                        native,
                    ) {
                        (Some(ours), Some(native)) => {
                            format!("{} ({:.1}×)", value(ours), ours / native)
                        }
                        (Some(ours), None) => value(ours),
                        (None, _) => "–".to_owned(),
                    }
                }));
                row
            })
            .collect();
        lines.extend([
            String::new(),
            format!("### Against native, {name}"),
            String::new(),
            format!(
                "Cycles per operation of `{}` in each build, then its ratio to the native figure.",
                layout.subject
            ),
            String::new(),
        ]);
        lines.extend(markdown_table(
            &header,
            &rows,
            &("llr".to_owned() + &"r".repeat(browsers.len())),
        ));
    }
    lines.push(String::new());
    lines
}

/// The notes under a table on its starred cells: per crate and deviation, the rows of its stars.
fn deviations<K: Key>(layout: &Layout<'_, K>, starred: &[(&str, &str, &K)]) -> Vec<String> {
    let mut groups: Vec<(&str, &str, Vec<K>)> = Vec::new();
    for (library, deviation, key) in starred {
        match groups
            .iter_mut()
            .find(|(other, reason, _)| other == library && reason == deviation)
        {
            Some((_, _, keys)) => keys.push((*key).clone()),
            None => groups.push((library, deviation, vec![(*key).clone()])),
        }
    }
    groups
        .into_iter()
        .flat_map(|(library, deviation, keys)| {
            [
                String::new(),
                format!(
                    "\\* `{library}` {}: {deviation}.",
                    (layout.rows_phrase)(&keys)
                ),
            ]
        })
        .collect()
}

fn reproduce(tool: &str, wasm: bool) -> Vec<String> {
    let mut lines = vec![
        "## Reproduce".to_owned(),
        String::new(),
        "```sh".to_owned(),
        format!("cargo run -p {tool} -- bench    # the CPU's counts of the native build"),
    ];
    if wasm {
        lines.push(format!(
            "cargo run -p {tool} -- bench-wasm --chrome <chrome> --firefox <firefox>   # the \
             WebAssembly builds"
        ));
        lines.push(format!(
            "cargo run -p {tool} -- bench-wasm --changed --chrome <chrome> --firefox <firefox>   # \
             the cells whose code changed, with the figures they are compared with"
        ));
    }
    lines.extend([
        format!("cargo run -p {tool} -- report   # this file"),
        "```".to_owned(),
    ]);
    lines
}
