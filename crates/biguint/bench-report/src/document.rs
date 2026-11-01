//! `crates/biguint/BENCHMARKS.md` and the charts in `crates/biguint/benchmarks/`.

use std::collections::HashMap;
use std::fs;

use crate::chart::{self, Chart, Panel, Point, grouped};
use crate::record::{Cell, Kind, Record};
use crate::results::Measured;
use crate::{Error, Paths};

/// The crate under test.
const SUBJECT: &str = "biguint";

/// The formula of each operation.
const FORMULAS: [(&str, &str); 5] = [
    ("add", "a + b"),
    ("mul", "a × b"),
    ("div", "a / b"),
    ("mod", "a mod b"),
    ("mulmod", "a × b mod m"),
];

/// Writes the document and one chart per operation from `record` and the times of its builds.
pub(crate) fn write(
    paths: &Paths,
    record: &Record,
    measured: &[Measured<'_>],
) -> Result<(), Error> {
    let table = Table::new(record, measured)?;
    let charts = paths.library().join("benchmarks");
    fs::create_dir_all(&charts).map_err(|source| Error::io(&charts, source))?;
    for (op, formula) in &table.ops {
        let svg = chart::svg(&Chart {
            title: format!("{op}: {formula}"),
            notes: chart_notes(record),
            widths: &table.widths,
            crates: &table.crates,
            panels: table
                .builds
                .iter()
                .map(|build| table.panel(build, op))
                .collect(),
        })?;
        let path = charts.join(format!("{op}.svg"));
        fs::write(&path, svg).map_err(|source| Error::io(&path, source))?;
    }
    let mut lines = header(record, &table);
    lines.extend(summary(&table));
    lines.extend(setup(record)?);
    lines.extend(method(&table));
    lines.extend(functions(&table));
    lines.extend(results(&table));
    lines.extend(reproduce());
    let path = paths.library().join("BENCHMARKS.md");
    fs::write(&path, lines.join("\n") + "\n").map_err(|source| Error::io(&path, source))
}

/// The cells and times of a run, indexed by operation, width and crate.
struct Table<'a> {
    builds: &'a [Measured<'a>],
    /// Each operation with its formula.
    ops: Vec<(&'a str, &'static str)>,
    widths: Vec<u32>,
    crates: Vec<&'a str>,
    cells: HashMap<(&'a str, u32, &'a str), &'a Cell>,
    /// The operations in one iteration of every case.
    batch: u32,
}

impl<'a> Table<'a> {
    /// Fails when the record has no cells, cells of different batch sizes, or an operation
    /// without a formula.
    fn new(record: &'a Record, builds: &'a [Measured<'a>]) -> Result<Self, Error> {
        let batches = unique(record.cells.iter().map(|cell| cell.batch));
        let [batch] = batches.as_slice() else {
            return Err(Error::missing(
                "single batch size",
                "the cells of the run record",
            ));
        };
        let ops = unique(record.cells.iter().map(|cell| cell.op.as_str()))
            .into_iter()
            .map(|op| {
                FORMULAS
                    .iter()
                    .find(|(name, _)| *name == op)
                    .map(|(_, formula)| (op, *formula))
                    .ok_or_else(|| Error::missing(&format!("formula for {op}"), "the report"))
            })
            .collect::<Result<_, _>>()?;
        Ok(Self {
            builds,
            ops,
            widths: unique(record.cells.iter().map(|cell| cell.bits)),
            crates: unique(record.cells.iter().map(|cell| cell.library.as_str())),
            cells: record
                .cells
                .iter()
                .map(|cell| ((cell.op.as_str(), cell.bits, cell.library.as_str()), cell))
                .collect(),
            batch: *batch,
        })
    }

    /// The time of `op` at `bits` in `library` in `build`; `None` without a case.
    fn time(build: &Measured<'_>, op: &str, bits: u32, library: &str) -> Option<f64> {
        build.times.get(&format!("{op}/{bits}/{library}")).copied()
    }

    /// The chart panel of `op` in `build`: one point per case.
    fn panel(&self, build: &'a Measured<'a>, op: &str) -> Panel<'a> {
        let mut points = Vec::new();
        for library in &self.crates {
            for &bits in &self.widths {
                if let Some(nanoseconds) = Self::time(build, op, bits, library) {
                    points.push(Point {
                        library,
                        bits,
                        nanoseconds,
                    });
                }
            }
        }
        Panel {
            build: &build.build.name,
            points,
        }
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

fn chart_notes(record: &Record) -> Vec<&'static str> {
    let mut notes =
        vec!["Each point is one crate at one width. Log scale; BENCHMARKS.md lists the values."];
    if record.provisional.is_some() {
        notes.push("The times are provisional upper bounds; BENCHMARKS.md gives the reason.");
    }
    notes
}

/// `values` joined as in a sentence: `a, b and c`.
fn sentence<T: ToString>(values: &[T]) -> String {
    let words: Vec<String> = values.iter().map(ToString::to_string).collect();
    match words.split_last() {
        Some((last, [])) => last.clone(),
        Some((last, rest)) => format!("{} and {last}", rest.join(", ")),
        None => String::new(),
    }
}

/// A Markdown table; `align` holds `l` or `r` per column.
fn markdown_table(header: &[String], rows: &[Vec<String>], align: &str) -> Vec<String> {
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

/// Three significant digits; whole nanoseconds from 100 up.
fn nanoseconds(value: f64) -> String {
    if value >= 100.0 {
        grouped(value.round() as u64)
    } else {
        let decimals = if value >= 10.0 {
            1
        } else if value >= 1.0 {
            2
        } else {
            3
        };
        format!("{value:.decimals$}")
    }
}

fn header(record: &Record, table: &Table<'_>) -> Vec<String> {
    let others: Vec<String> = table
        .crates
        .iter()
        .filter(|library| **library != SUBJECT)
        .map(|library| format!("`{library}`"))
        .collect();
    let state = if record.dirty {
        " The measured files had uncommitted changes."
    } else {
        ""
    };
    vec![
        "# Benchmarks".to_owned(),
        String::new(),
        format!(
            "`{SUBJECT}` at commit `{}` against {}: {} at {} bits, measured on {}.{state}",
            short(&record.library_commit),
            sentence(&others),
            sentence(&table.ops.iter().map(|(op, _)| *op).collect::<Vec<_>>()),
            sentence(&table.widths),
            record.date
        ),
        String::new(),
    ]
}

/// The first seven characters of a commit hash.
fn short(commit: &str) -> &str {
    commit.get(..7).unwrap_or(commit)
}

fn summary(table: &Table<'_>) -> Vec<String> {
    let mut rows = Vec::new();
    for (op, _) in &table.ops {
        for build in table.builds {
            let mut row = vec![(*op).to_owned(), build.build.name.clone()];
            for &bits in &table.widths {
                let own = Table::time(build, op, bits, SUBJECT);
                let fastest_other = table
                    .crates
                    .iter()
                    .filter(|library| **library != SUBJECT)
                    .filter_map(|library| Table::time(build, op, bits, library))
                    .reduce(f64::min);
                row.push(match (own, fastest_other) {
                    (Some(own), Some(other)) => format!("{:.2}", own / other),
                    (None, _) | (_, None) => "–".to_owned(),
                });
            }
            rows.push(row);
        }
    }
    let mut header = vec!["Operation".to_owned(), "Build".to_owned()];
    header.extend(table.widths.iter().map(|bits| format!("{bits} bits")));
    let mut lines = vec![
        "## Summary".to_owned(),
        String::new(),
        format!(
            "Each cell: `{SUBJECT}`'s time per operation over the fastest other crate's. Below 1, \
             `{SUBJECT}` is faster."
        ),
        String::new(),
    ];
    lines.extend(markdown_table(
        &header,
        &rows,
        &("ll".to_owned() + &"r".repeat(table.widths.len())),
    ));
    lines.push(String::new());
    lines
}

fn setup(record: &Record) -> Result<Vec<String>, Error> {
    let cpu = match &record.machine {
        Some(machine) => format!("{machine} (`{}`)", record.native_cpu),
        None => format!("`{}`", record.native_cpu),
    };
    let builds: Vec<String> = record
        .builds
        .iter()
        .map(|build| {
            if build.rustflags.is_empty() {
                format!("`{}`: the baseline of the host target.", build.name)
            } else {
                format!("`{}`: `{}`.", build.name, build.rustflags)
            }
        })
        .collect();
    let criterion = record
        .versions
        .iter()
        .find(|entry| entry.name == "criterion")
        .ok_or_else(|| Error::missing("criterion version", "the run record"))?;
    let crates: Vec<String> = record
        .versions
        .iter()
        .filter(|entry| entry.name != "criterion")
        .map(|entry| format!("{} {}", entry.name, entry.version))
        .collect();
    let timing = match record.criterion_args.as_slice() {
        [] => format!("criterion {}", criterion.version),
        args => format!("criterion {} `{}`", criterion.version, args.join(" ")),
    };
    let rows = vec![
        vec!["CPU".to_owned(), cpu],
        vec![
            "Compiler".to_owned(),
            format!("{}, LLVM {}, `bench` profile", record.rustc, record.llvm),
        ],
        vec!["Builds".to_owned(), builds.join(" ")],
        vec!["Timing".to_owned(), timing],
        vec!["Crates".to_owned(), crates.join(", ")],
    ];
    let mut lines = vec!["## Setup".to_owned(), String::new()];
    lines.extend(markdown_table(&[String::new(), String::new()], &rows, "ll"));
    if let Some(reason) = &record.provisional {
        lines.push(String::new());
        lines.push(format!(
            "**The times are provisional.** {reason}. Each time is an upper bound, and the times \
             compare only within one table."
        ));
    }
    lines.push(String::new());
    Ok(lines)
}

fn method(table: &Table<'_>) -> Vec<String> {
    let batch = table.batch;
    let operands = [
        ["add, mul", "`a` and `b` uniform below 2^N"],
        [
            "div, mod",
            "a dividend with bit N − 1 set; a divisor below 2^(N/2) with bit N/2 − 1 set, held \
             in an N-bit integer",
        ],
        [
            "mulmod",
            "a modulus `m` with bit N − 1 set; `a` and `b` uniform below `m`",
        ],
    ];
    let rows: Vec<Vec<String>> = operands
        .iter()
        .map(|row| row.iter().map(|text| (*text).to_owned()).collect())
        .collect();
    let mut lines = vec![
        "## Method".to_owned(),
        String::new(),
        format!(
            "Each case times one crate's function over {batch} operand sets, the same for every \
             crate in a row, with operands and results behind `core::hint::black_box`. The time \
             per operation is criterion's median for the batch divided by {batch}. Every result \
             equals `num-bigint`'s, reduced modulo 2^N for the crates that wrap."
        ),
        String::new(),
    ];
    lines.extend(markdown_table(
        &["Operation".to_owned(), "Operands at width N".to_owned()],
        &rows,
        "ll",
    ));
    lines.push(String::new());
    lines
}

fn functions(table: &Table<'_>) -> Vec<String> {
    let mut rows = Vec::new();
    for (op, _) in &table.ops {
        let mut row = vec![(*op).to_owned()];
        for library in &table.crates {
            // Each function with the widths it runs at.
            let mut groups: Vec<(&str, Vec<u32>)> = Vec::new();
            for &bits in &table.widths {
                let Some(cell) = table.cells.get(&(*op, bits, *library)) else {
                    continue;
                };
                if cell.kind == Kind::Absent {
                    continue;
                }
                match groups.iter_mut().find(|(detail, _)| *detail == cell.detail) {
                    Some((_, widths)) => widths.push(bits),
                    None => groups.push((&cell.detail, vec![bits])),
                }
            }
            let entries: Vec<String> = groups
                .iter()
                .map(|(detail, widths)| {
                    if *widths == table.widths {
                        format!("`{detail}`")
                    } else {
                        format!("`{detail}` at {} bits", sentence(widths))
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
        "Each cell times the crate's own function:".to_owned(),
        String::new(),
    ];
    lines.extend(markdown_table(&header, &rows, &"l".repeat(header.len())));
    lines.push(String::new());
    lines.extend(
        [
            "- `native` is `u64` at 64 bits and `u128` at 128 bits.",
            "- `crypto-bigint` is constant-time; its div, mod and mulmod cases run its \
             `*_vartime` functions.",
            "- `num-bigint` allocates each result.",
            "- `uint` is `construct_uint!`, as in `primitive-types`.",
        ]
        .map(str::to_owned),
    );
    lines.extend(absences(table));
    lines.push(String::new());
    lines
}

/// Per crate, its empty cells as (operation, width).
type EmptyCells<'a> = Vec<(&'a str, Vec<(&'a str, u32)>)>;

/// One line per reason for an empty cell: per crate, the operations and widths it covers, with
/// the operations left out when they are all of them.
fn absences(table: &Table<'_>) -> Vec<String> {
    let mut reasons: Vec<(&str, EmptyCells<'_>)> = Vec::new();
    for library in &table.crates {
        for (op, _) in &table.ops {
            for &bits in &table.widths {
                let Some(cell) = table.cells.get(&(*op, bits, *library)) else {
                    continue;
                };
                if cell.kind != Kind::Absent {
                    continue;
                }
                let position = match reasons
                    .iter()
                    .position(|(reason, _)| *reason == cell.detail)
                {
                    Some(position) => position,
                    None => {
                        reasons.push((&cell.detail, Vec::new()));
                        reasons.len() - 1
                    }
                };
                let Some((_, crates)) = reasons.get_mut(position) else {
                    continue;
                };
                match crates.iter_mut().find(|(other, _)| other == library) {
                    Some((_, cells)) => cells.push((op, bits)),
                    None => crates.push((library, vec![(op, bits)])),
                }
            }
        }
    }
    reasons
        .iter()
        .map(|(reason, crates)| {
            let cases: Vec<String> = crates
                .iter()
                .map(|(library, cells)| describe(table, library, cells))
                .collect();
            let mut reason = (*reason).to_owned();
            if let Some(first) = reason.get_mut(..1) {
                first.make_ascii_uppercase();
            }
            format!("- {reason}: {}.", sentence(&cases))
        })
        .collect()
}

/// The empty `cells` of `library`: its name, the operations unless all of them, and the widths
/// unless all of them.
fn describe(table: &Table<'_>, library: &str, cells: &[(&str, u32)]) -> String {
    let ops = unique(cells.iter().map(|(op, _)| *op));
    let widths = unique(cells.iter().map(|(_, bits)| *bits));
    let mut text = format!("`{library}`");
    if ops.len() < table.ops.len() {
        text += &format!(" {}", sentence(&ops));
    }
    if widths.len() < table.widths.len() {
        text += &format!(" at {} bits", sentence(&widths));
    }
    text
}

fn results(table: &Table<'_>) -> Vec<String> {
    let mut lines = vec![
        "## Results".to_owned(),
        String::new(),
        "Each cell: nanoseconds per operation. Bold marks the fastest time in each row.".to_owned(),
    ];
    for (op, formula) in &table.ops {
        let mut rows = Vec::new();
        for &bits in &table.widths {
            for build in table.builds {
                let times: Vec<Option<f64>> = table
                    .crates
                    .iter()
                    .map(|library| Table::time(build, op, bits, library))
                    .collect();
                let fastest = times.iter().flatten().copied().reduce(f64::min);
                let mut row = vec![bits.to_string(), build.build.name.clone()];
                row.extend(times.iter().map(|time| match time {
                    Some(time) if Some(*time) == fastest => {
                        format!("**{} ns**", nanoseconds(*time))
                    }
                    Some(time) => format!("{} ns", nanoseconds(*time)),
                    None => "–".to_owned(),
                }));
                rows.push(row);
            }
        }
        let mut header = vec!["Bits".to_owned(), "Build".to_owned()];
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
            &("rl".to_owned() + &"r".repeat(table.crates.len())),
        ));
    }
    lines
}

fn reproduce() -> Vec<String> {
    [
        "",
        "## Reproduce",
        "",
        "```sh",
        "cargo run -p biguint-bench-report -- bench    # criterion, default and native builds",
        "cargo run -p biguint-bench-report -- report   # this file and benchmarks/*.svg",
        "```",
    ]
    .map(str::to_owned)
    .to_vec()
}
