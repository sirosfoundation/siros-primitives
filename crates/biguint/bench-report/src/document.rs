//! `crates/biguint/BENCHMARKS.md` and the charts in `crates/biguint/benchmarks/`.

use std::collections::HashMap;
use std::fs;

use crate::chart::{self, Chart, Point, instructions};
use crate::record::{Cell, Count, Outcome, Record};
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

/// The line under the title of every chart.
const CHART_SUBTITLE: &str =
    "Instructions per operation, counted by Callgrind. BENCHMARKS.md lists the values.";

/// Writes the document and one chart per operation from `record`.
pub(crate) fn write(paths: &Paths, record: &Record) -> Result<(), Error> {
    let table = Table::new(record)?;
    let charts = paths.library().join("benchmarks");
    fs::create_dir_all(&charts).map_err(|source| Error::io(&charts, source))?;
    for (op, formula) in &table.ops {
        let svg = chart::svg(&Chart {
            title: format!("{op}: {formula}"),
            subtitle: CHART_SUBTITLE,
            widths: &table.widths,
            crates: &table.crates,
            points: table.points(op),
        })?;
        let path = charts.join(format!("{op}.svg"));
        fs::write(&path, svg).map_err(|source| Error::io(&path, source))?;
    }
    let mut lines = header(record, &table);
    lines.extend(setup(record));
    lines.extend(method(&table));
    lines.extend(functions(&table));
    lines.extend(results(&table));
    lines.extend(reproduce());
    let path = paths.library().join("BENCHMARKS.md");
    fs::write(&path, lines.join("\n") + "\n").map_err(|source| Error::io(&path, source))
}

/// The cells of a run, indexed by operation, width and crate.
struct Table<'a> {
    /// Each operation with its formula.
    ops: Vec<(&'a str, &'static str)>,
    widths: Vec<u32>,
    crates: Vec<&'a str>,
    cells: HashMap<(&'a str, u32, &'a str), &'a Cell>,
    /// The operand sets of every counted batch.
    batch: u32,
}

impl<'a> Table<'a> {
    /// Fails unless the record has exactly one cell for every operation, width and crate, its
    /// batches have one size, and every operation has a formula.
    fn new(record: &'a Record) -> Result<Self, Error> {
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
                FORMULAS
                    .iter()
                    .find(|(name, _)| *name == op)
                    .map(|(_, formula)| (op, *formula))
                    .ok_or_else(|| Error::missing(&format!("formula for {op}"), "the report"))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let table = Self {
            ops,
            widths: unique(record.cells.iter().map(|cell| cell.bits)),
            crates: unique(record.cells.iter().map(|cell| cell.library.as_str())),
            cells: record
                .cells
                .iter()
                .map(|cell| ((cell.op.as_str(), cell.bits, cell.library.as_str()), cell))
                .collect(),
            batch: *batch,
        };
        if table.cells.len() != record.cells.len() {
            return Err(Error::missing(
                "single cell per operation, width and crate",
                "the run record",
            ));
        }
        for (op, _) in &table.ops {
            for bits in &table.widths {
                for library in &table.crates {
                    if !table.cells.contains_key(&(*op, *bits, *library)) {
                        return Err(Error::missing(
                            &format!("cell {op}/{bits}/{library}"),
                            "the run record",
                        ));
                    }
                }
            }
        }
        Ok(table)
    }

    /// The count of `op` at `bits` in `library`; `None` for an absent cell.
    fn count(&self, op: &str, bits: u32, library: &str) -> Option<&'a Count> {
        match self
            .cells
            .get(&(op, bits, library))
            .map(|cell| &cell.outcome)
        {
            Some(Outcome::Counted(count)) => Some(count),
            Some(Outcome::Absent(_)) | None => None,
        }
    }

    /// The chart points of `op`: one per counted case.
    fn points(&self, op: &str) -> Vec<Point<'a>> {
        let mut points = Vec::new();
        for library in &self.crates {
            for &bits in &self.widths {
                if let Some(count) = self.count(op, bits, library) {
                    points.push(Point {
                        library,
                        bits,
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

fn header(record: &Record, table: &Table<'_>) -> Vec<String> {
    let others: Vec<String> = table
        .crates
        .iter()
        .filter(|library| **library != SUBJECT)
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
            "`{SUBJECT}` at commit `{}` against {}: {} at {} bits, in instructions per \
             operation, counted on {}.{state}",
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

fn setup(record: &Record) -> Vec<String> {
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
            "Each case runs one crate's function over {batch} operand sets, the same for every \
             crate in a row, with operands and results behind `core::hint::black_box`. Callgrind \
             counts the instructions of the loop over the batch, including the functions it \
             calls, such as the allocator of `num-bigint`; each count is that total divided by \
             {batch}. The loop's own loads and stores are part of every count. Every instruction \
             counts as one, whatever its latency: a hardware divide weighs as much as an \
             addition. The counts are exact at any machine load. Every result equals \
             `num-bigint`'s, reduced modulo 2^N for the crates that wrap."
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
                let Some(count) = table.count(op, bits, library) else {
                    continue;
                };
                match groups
                    .iter_mut()
                    .find(|(function, _)| *function == count.function)
                {
                    Some((_, widths)) => widths.push(bits),
                    None => groups.push((&count.function, vec![bits])),
                }
            }
            let entries: Vec<String> = groups
                .iter()
                .map(|(function, widths)| {
                    if *widths == table.widths {
                        format!("`{function}`")
                    } else {
                        format!("`{function}` at {} bits", sentence(widths))
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

/// Per crate, its absent cells as (operation, width).
type AbsentCells<'a> = Vec<(&'a str, Vec<(&'a str, u32)>)>;

/// One line per reason for an absent cell: per crate, the operations and widths it covers, with
/// the operations left out when they are all of them.
fn absences(table: &Table<'_>) -> Vec<String> {
    let mut reasons: Vec<(&str, AbsentCells<'_>)> = Vec::new();
    for library in &table.crates {
        for (op, _) in &table.ops {
            for &bits in &table.widths {
                let Some(Outcome::Absent(reason)) = table
                    .cells
                    .get(&(*op, bits, *library))
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

/// The absent `cells` of `library`: its name, the operations unless all of them, and the widths
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
        "Each cell: instructions per operation. Bold marks the fewest in each row.".to_owned(),
    ];
    for (op, formula) in &table.ops {
        let mut rows = Vec::new();
        for &bits in &table.widths {
            let counts: Vec<Option<f64>> = table
                .crates
                .iter()
                .map(|library| table.count(op, bits, library).map(Count::per_operation))
                .collect();
            let fewest = counts.iter().flatten().copied().reduce(f64::min);
            let mut row = vec![bits.to_string()];
            row.extend(counts.iter().map(|count| match count {
                Some(count) if Some(*count) == fewest => format!("**{}**", instructions(*count)),
                Some(count) => instructions(*count),
                None => "–".to_owned(),
            }));
            rows.push(row);
        }
        let mut header = vec!["Bits".to_owned()];
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

fn reproduce() -> Vec<String> {
    [
        "",
        "## Reproduce",
        "",
        "```sh",
        "cargo run -p biguint-bench-report -- bench    # Callgrind counts of the native build",
        "cargo run -p biguint-bench-report -- report   # this file and benchmarks/*.svg",
        "```",
    ]
    .map(str::to_owned)
    .to_vec()
}
