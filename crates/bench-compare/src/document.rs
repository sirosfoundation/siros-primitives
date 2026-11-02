//! The `BENCHMARKS.md` of a library and the charts in its `benchmarks/` directory.

use std::collections::HashMap;
use std::fs;

use crate::chart::{self, CHARTED, Category, Chart, Point, instructions};
use crate::error::Error;
use crate::record::{Cell, Count, Key, Outcome, Record};
use crate::run::Paths;

/// The line under the title of every chart.
const CHART_SUBTITLE: &str =
    "Instructions per operation, counted by Callgrind. BENCHMARKS.md lists the values.";

/// The words of a comparison: what its rows are and how the document names them.
pub struct Layout<'a, K> {
    /// The crate under test.
    pub subject: &'a str,
    /// The package of the report tool, for the `Reproduce` section.
    pub tool: &'a str,
    /// The formula of each operation, by the operation's name.
    pub formulas: &'a [(&'a str, &'a str)],
    /// The heading of the row column of the result tables.
    pub row_header: &'a str,
    /// A phrase that names rows, after an operation or a function: `at 64 and 128 bits`.
    pub rows_phrase: fn(&[K]) -> String,
    /// The body of the `Method` section for a batch of the given size.
    pub method: fn(u32) -> Vec<String>,
    /// The bullet lines under the `Functions` table, before the absences.
    pub notes: &'a [&'a str],
    /// The title of the horizontal axis of the charts.
    pub axis_title: &'a str,
    /// The name of a row in a chart tooltip: `64 bits`.
    pub category_name: fn(&K) -> String,
    /// Whether a line joins the markers of adjacent rows in a chart.
    pub lines: bool,
}

/// Writes the document and one chart per operation from `record`.
pub fn report<K: Key>(
    paths: &Paths,
    layout: &Layout<'_, K>,
    record: &Record<K>,
) -> Result<(), Error> {
    let table = Table::new(record, layout)?;
    let charts = paths.library.join("benchmarks");
    fs::create_dir_all(&charts).map_err(|source| Error::io(&charts, source))?;
    let categories: Vec<Category> = table
        .keys
        .iter()
        .map(|key| Category {
            label: key.to_string(),
            name: (layout.category_name)(key),
        })
        .collect();
    // The palette has one slot per crate up to `CHARTED`; further crates stay in the tables.
    let charted = &table.crates[..table.crates.len().min(CHARTED)];
    for (op, formula) in &table.ops {
        let svg = chart::svg(&Chart {
            title: format!("{op}: {formula}"),
            subtitle: CHART_SUBTITLE,
            categories: &categories,
            axis_title: layout.axis_title,
            crates: charted,
            points: table.points(op, charted),
            lines: layout.lines,
        })?;
        let path = charts.join(format!("{op}.svg"));
        fs::write(&path, svg).map_err(|source| Error::io(&path, source))?;
    }
    let mut lines = header(record, layout, &table);
    lines.extend(setup(record));
    lines.extend(method(layout, &table));
    lines.extend(functions(layout, &table));
    lines.extend(results(layout, &table));
    lines.extend(reproduce(layout.tool));
    let path = paths.library.join("BENCHMARKS.md");
    fs::write(&path, lines.join("\n") + "\n").map_err(|source| Error::io(&path, source))
}

/// The cells of a run, indexed by operation, row and crate.
struct Table<'a, K> {
    /// Each operation with its formula.
    ops: Vec<(&'a str, &'a str)>,
    keys: Vec<K>,
    crates: Vec<&'a str>,
    cells: HashMap<(&'a str, &'a K, &'a str), &'a Cell<K>>,
    /// The operand sets of every counted batch.
    batch: u32,
}

impl<'a, K: Key> Table<'a, K> {
    /// Fails unless the record has exactly one cell for every operation, row and crate, its
    /// batches have one size, and every operation has a formula.
    fn new(record: &'a Record<K>, layout: &Layout<'a, K>) -> Result<Self, Error> {
        let batches = unique(record.cells.iter().filter_map(|cell| match &cell.outcome {
            Outcome::Counted(count) => Some(count.operations),
            Outcome::Absent(_) => None,
        }));
        let [batch] = batches.as_slice() else {
            return Err(Error::missing(
                "single batch size",
                "the cells of the run record",
            ));
        };
        let ops = unique(record.cells.iter().map(|cell| cell.op.as_str()))
            .into_iter()
            .map(|op| {
                layout
                    .formulas
                    .iter()
                    .find(|(name, _)| *name == op)
                    .map(|(_, formula)| (op, *formula))
                    .ok_or_else(|| Error::missing(&format!("formula for {op}"), "the report"))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let table = Self {
            ops,
            keys: unique(record.cells.iter().map(|cell| cell.key.clone())),
            crates: unique(record.cells.iter().map(|cell| cell.library.as_str())),
            cells: record
                .cells
                .iter()
                .map(|cell| ((cell.op.as_str(), &cell.key, cell.library.as_str()), cell))
                .collect(),
            batch: *batch,
        };
        if table.cells.len() != record.cells.len() {
            return Err(Error::missing(
                "single cell per operation, row and crate",
                "the run record",
            ));
        }
        for (op, _) in &table.ops {
            for key in &table.keys {
                for library in &table.crates {
                    if !table.cells.contains_key(&(*op, key, *library)) {
                        return Err(Error::missing(
                            &format!("cell {op}/{key}/{library}"),
                            "the run record",
                        ));
                    }
                }
            }
        }
        Ok(table)
    }

    /// The count of `op` on the row `key` in `library`; `None` for an absent cell.
    fn count(&self, op: &str, key: &K, library: &str) -> Option<&'a Count> {
        match self
            .cells
            .get(&(op, key, library))
            .map(|cell| &cell.outcome)
        {
            Some(Outcome::Counted(count)) => Some(count),
            Some(Outcome::Absent(_)) | None => None,
        }
    }

    /// The chart points of `op` for `crates`: one per counted case.
    fn points(&self, op: &str, crates: &[&'a str]) -> Vec<Point<'a>> {
        let mut points = Vec::new();
        for library in crates {
            for (category, key) in self.keys.iter().enumerate() {
                if let Some(count) = self.count(op, key, library) {
                    points.push(Point {
                        library,
                        category,
                        instructions: count.per_operation(),
                    });
                }
            }
        }
        points
    }
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
    let others: Vec<String> = table
        .crates
        .iter()
        .filter(|library| **library != layout.subject)
        .map(|library| format!("`{library}`"))
        .collect();
    let state = if record.dirty {
        " The counted files had uncommitted changes."
    } else {
        ""
    };
    vec![
        "# Benchmarks".to_owned(),
        String::new(),
        format!(
            "`{}` at commit `{}` against {}: {} {}, in instructions per operation, counted on \
             {}.{state}",
            layout.subject,
            short(&record.library_commit),
            sentence(&others),
            sentence(&table.ops.iter().map(|(op, _)| *op).collect::<Vec<_>>()),
            (layout.rows_phrase)(&table.keys),
            record.date
        ),
        String::new(),
    ]
}

/// The first seven characters of a commit hash.
fn short(commit: &str) -> &str {
    commit.get(..7).unwrap_or(commit)
}

fn setup<K>(record: &Record<K>) -> Vec<String> {
    let cpu = match &record.machine {
        Some(machine) => format!("{machine} (`{}`)", record.native_cpu),
        None => format!("`{}`", record.native_cpu),
    };
    let counter = match record.valgrind.strip_prefix("valgrind-") {
        Some(version) => format!("Valgrind {version}, Callgrind"),
        None => format!("{}, Callgrind", record.valgrind),
    };
    let crates: Vec<String> = record
        .versions
        .iter()
        .map(|entry| format!("{} {}", entry.name, entry.version))
        .collect();
    let rows = vec![
        vec!["CPU".to_owned(), cpu],
        vec![
            "Compiler".to_owned(),
            format!(
                "{}, LLVM {}, `bench` profile, `{}`",
                record.rustc, record.llvm, record.rustflags
            ),
        ],
        vec!["Counter".to_owned(), counter],
        vec!["Crates".to_owned(), crates.join(", ")],
    ];
    let mut lines = vec!["## Setup".to_owned(), String::new()];
    lines.extend(markdown_table(&[String::new(), String::new()], &rows, "ll"));
    lines.push(String::new());
    lines
}

fn method<K>(layout: &Layout<'_, K>, table: &Table<'_, K>) -> Vec<String> {
    let mut lines = vec!["## Method".to_owned(), String::new()];
    lines.extend((layout.method)(table.batch));
    lines.push(String::new());
    lines
}

fn functions<K: Key>(layout: &Layout<'_, K>, table: &Table<'_, K>) -> Vec<String> {
    let mut rows = Vec::new();
    for (op, _) in &table.ops {
        let mut row = vec![(*op).to_owned()];
        for library in &table.crates {
            // Each function with the rows it runs on.
            let mut groups: Vec<(&str, Vec<K>)> = Vec::new();
            for key in &table.keys {
                let Some(count) = table.count(op, key, library) else {
                    continue;
                };
                match groups
                    .iter_mut()
                    .find(|(function, _)| *function == count.function)
                {
                    Some((_, keys)) => keys.push(key.clone()),
                    None => groups.push((&count.function, vec![key.clone()])),
                }
            }
            let entries: Vec<String> = groups
                .iter()
                .map(|(function, keys)| {
                    if *keys == table.keys {
                        format!("`{function}`")
                    } else {
                        format!("`{function}` {}", (layout.rows_phrase)(keys))
                    }
                })
                .collect();
            row.push(if entries.is_empty() {
                "–".to_owned()
            } else {
                entries.join("; ")
            });
        }
        rows.push(row);
    }
    let mut header = vec!["Operation".to_owned()];
    header.extend(table.crates.iter().map(|library| (*library).to_owned()));
    let mut lines = vec![
        "## Functions".to_owned(),
        String::new(),
        "Each cell counts the crate's own function:".to_owned(),
        String::new(),
    ];
    lines.extend(markdown_table(&header, &rows, &"l".repeat(header.len())));
    lines.push(String::new());
    lines.extend(layout.notes.iter().map(|note| (*note).to_owned()));
    lines.extend(absences(layout, table));
    lines.push(String::new());
    lines
}

/// Per crate, its absent cells as (operation, row).
type AbsentCells<'a, K> = Vec<(&'a str, Vec<(&'a str, &'a K)>)>;

/// One line per reason for an absent cell: per crate, the operations and rows it covers, with
/// the operations left out when they are all of them.
fn absences<'a, K: Key>(layout: &Layout<'_, K>, table: &'a Table<'a, K>) -> Vec<String> {
    let mut reasons: Vec<(&str, AbsentCells<'a, K>)> = Vec::new();
    for library in &table.crates {
        for (op, _) in &table.ops {
            for key in &table.keys {
                let Some(Outcome::Absent(reason)) = table
                    .cells
                    .get(&(*op, key, *library))
                    .map(|cell| &cell.outcome)
                else {
                    continue;
                };
                let position = match reasons.iter().position(|(other, _)| other == reason) {
                    Some(position) => position,
                    None => {
                        reasons.push((reason, Vec::new()));
                        reasons.len() - 1
                    }
                };
                let Some((_, crates)) = reasons.get_mut(position) else {
                    continue;
                };
                match crates.iter_mut().find(|(other, _)| other == library) {
                    Some((_, cells)) => cells.push((op, key)),
                    None => crates.push((library, vec![(op, key)])),
                }
            }
        }
    }
    reasons
        .iter()
        .map(|(reason, crates)| {
            let cases: Vec<String> = crates
                .iter()
                .map(|(library, cells)| describe(layout, table, library, cells))
                .collect();
            let mut reason = (*reason).to_owned();
            if let Some(first) = reason.get_mut(..1) {
                first.make_ascii_uppercase();
            }
            format!("- {reason}: {}.", sentence(&cases))
        })
        .collect()
}

/// The absent `cells` of `library`: its name, the operations unless all of them, and the rows
/// unless all of them.
fn describe<K: Key>(
    layout: &Layout<'_, K>,
    table: &Table<'_, K>,
    library: &str,
    cells: &[(&str, &K)],
) -> String {
    let ops = unique(cells.iter().map(|(op, _)| *op));
    let keys: Vec<K> = unique(cells.iter().map(|(_, key)| (*key).clone()));
    let mut text = format!("`{library}`");
    if ops.len() < table.ops.len() {
        text += &format!(" {}", sentence(&ops));
    }
    if keys.len() < table.keys.len() {
        text += &format!(" {}", (layout.rows_phrase)(&keys));
    }
    text
}

fn results<K: Key>(layout: &Layout<'_, K>, table: &Table<'_, K>) -> Vec<String> {
    let mut lines = vec![
        "## Results".to_owned(),
        String::new(),
        "Each cell: instructions per operation. Bold marks the fewest in each row.".to_owned(),
    ];
    if let Some(unplotted) = table.crates.get(CHARTED..).filter(|rest| !rest.is_empty()) {
        let names: Vec<String> = unplotted.iter().map(|name| format!("`{name}`")).collect();
        let verb = if names.len() == 1 {
            "appears"
        } else {
            "appear"
        };
        lines.push(format!(
            "The charts show the first {CHARTED} crates; {} {verb} in the tables only.",
            sentence(&names)
        ));
    }
    for (op, formula) in &table.ops {
        let mut rows = Vec::new();
        for key in &table.keys {
            let counts: Vec<Option<f64>> = table
                .crates
                .iter()
                .map(|library| table.count(op, key, library).map(Count::per_operation))
                .collect();
            let fewest = counts.iter().flatten().copied().reduce(f64::min);
            let mut row = vec![key.to_string()];
            row.extend(counts.iter().map(|count| match count {
                Some(count) if Some(*count) == fewest => format!("**{}**", instructions(*count)),
                Some(count) => instructions(*count),
                None => "–".to_owned(),
            }));
            rows.push(row);
        }
        let mut header = vec![layout.row_header.to_owned()];
        header.extend(table.crates.iter().map(|library| (*library).to_owned()));
        let title = format!("{op}: {formula}");
        lines.extend([
            String::new(),
            format!("### {title}"),
            String::new(),
            format!("![{title}](benchmarks/{op}.svg)"),
            String::new(),
        ]);
        lines.extend(markdown_table(
            &header,
            &rows,
            &("r".to_owned() + &"r".repeat(table.crates.len())),
        ));
    }
    lines
}

fn reproduce(tool: &str) -> Vec<String> {
    vec![
        String::new(),
        "## Reproduce".to_owned(),
        String::new(),
        "```sh".to_owned(),
        format!("cargo run -p {tool} -- bench    # Callgrind counts of the native build"),
        format!("cargo run -p {tool} -- report   # this file and benchmarks/*.svg"),
        "```".to_owned(),
    ]
}
