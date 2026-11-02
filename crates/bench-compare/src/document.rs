//! The `BENCHMARKS.md` of a library and the charts in its `benchmarks/` directory.

use std::collections::HashMap;
use std::fs;

use crate::barrier::REGISTERS;
use crate::chart::{self, CHARTED, Category, Chart, Point, value};
use crate::count::{REPETITIONS, TARGET};
use crate::error::Error;
use crate::record::{Cell, Count, Key, Measurement, Outcome, Record, Sweep};
use crate::run::Paths;

/// The 1-minute load average from which a sweep ran under load.
pub const LOAD: f64 = 2.0;

/// The target features that name no vector or arithmetic extension: state saving, atomics and
/// random numbers.
const SYSTEM_FEATURES: [&str; 8] = [
    "cmpxchg16b",
    "fxsr",
    "rdrand",
    "rdseed",
    "xsave",
    "xsavec",
    "xsaveopt",
    "xsaves",
];

/// The line under the title of every chart.
const CHART_SUBTITLE: &str = "Cycles per operation in a loop of independent operations, counted by \
                              the CPU. BENCHMARKS.md lists the values.";

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
    /// The lines of the `Method` section that describe the operands and the checks.
    pub method: fn() -> Vec<String>,
    /// The bullet lines under the `Functions` table, before the absences.
    pub notes: &'a [&'a str],
    /// The title of the horizontal axis of the charts.
    pub axis_title: &'a str,
    /// The name of a row in a chart tooltip: `64 bits`.
    pub category_name: fn(&K) -> String,
    /// Whether a line joins the markers of adjacent rows in a chart.
    pub lines: bool,
}

/// Writes the document and one chart per operation and build from `record`.
pub fn report<K: Key>(
    paths: &Paths,
    layout: &Layout<'_, K>,
    record: &Record<K>,
) -> Result<(), Error> {
    let tables = record
        .builds
        .iter()
        .map(|build| Ok((build, Table::new(&build.cells, layout)?)))
        .collect::<Result<Vec<_>, Error>>()?;
    let [(_, first), ..] = tables.as_slice() else {
        return Err(Error::missing("build", "the run record"));
    };
    for (build, table) in &tables {
        if table.ops != first.ops || table.keys != first.keys || table.crates != first.crates {
            return Err(Error::missing(
                &format!("the cells of the first build in the build {}", build.name),
                "the run record",
            ));
        }
    }
    let charts = paths.library.join("benchmarks");
    fs::create_dir_all(&charts).map_err(|source| Error::io(&charts, source))?;
    let categories: Vec<Category> = first
        .keys
        .iter()
        .map(|key| Category {
            label: key.to_string(),
            name: (layout.category_name)(key),
        })
        .collect();
    // The palette has one slot per crate up to `CHARTED`; further crates stay in the tables.
    let charted = &first.crates[..first.crates.len().min(CHARTED)];
    for (build, table) in &tables {
        for (op, formula) in &table.ops {
            let svg = chart::svg(&Chart {
                title: title(op, formula, build),
                subtitle: CHART_SUBTITLE,
                categories: &categories,
                axis_title: layout.axis_title,
                crates: charted,
                points: table.points(op, charted),
                lines: layout.lines,
            })?;
            let path = charts.join(chart_file(op, build));
            fs::write(&path, svg).map_err(|source| Error::io(&path, source))?;
        }
    }
    let mut lines = header(record, layout, first);
    lines.extend(setup(record));
    lines.extend(method(layout, record));
    lines.extend(functions(layout, first));
    for (build, table) in &tables {
        lines.extend(results(layout, table, build));
    }
    lines.extend(reproduce(layout.tool));
    let path = paths.library.join("BENCHMARKS.md");
    fs::write(&path, lines.join("\n") + "\n").map_err(|source| Error::io(&path, source))
}

/// The title of the chart and the table of `op` in `build`.
fn title<K>(op: &str, formula: &str, build: &Measurement<K>) -> String {
    if build.name.is_empty() {
        format!("{op}: {formula}")
    } else {
        format!("{op}: {formula}, {}", build.name)
    }
}

/// The file of the chart of `op` in `build`, in `benchmarks/`.
fn chart_file<K>(op: &str, build: &Measurement<K>) -> String {
    if build.slug.is_empty() {
        format!("{op}.svg")
    } else {
        format!("{op}-{}.svg", build.slug)
    }
}

/// Whether `sweep` ran under load.
fn loaded(sweep: &Sweep) -> bool {
    sweep.load.before >= LOAD || sweep.load.after >= LOAD
}

/// The cells of a build, indexed by operation, row and crate.
struct Table<'a, K> {
    /// Each operation with its formula.
    ops: Vec<(&'a str, &'a str)>,
    keys: Vec<K>,
    crates: Vec<&'a str>,
    cells: HashMap<(&'a str, &'a K, &'a str), &'a Cell<K>>,
}

/// A figure per operation that the result tables show, by its row label.
type Metric = (&'static str, fn(&Count) -> Option<f64>);

impl<'a, K: Key> Table<'a, K> {
    /// Fails unless `cells` hold exactly one cell for every operation, row and crate, and every
    /// operation has a formula.
    fn new(cells: &'a [Cell<K>], layout: &Layout<'a, K>) -> Result<Self, Error> {
        let ops = unique(cells.iter().map(|cell| cell.op.as_str()))
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
            keys: unique(cells.iter().map(|cell| cell.key.clone())),
            crates: unique(cells.iter().map(|cell| cell.library.as_str())),
            cells: cells
                .iter()
                .map(|cell| ((cell.op.as_str(), &cell.key, cell.library.as_str()), cell))
                .collect(),
        };
        if table.cells.len() != cells.len() {
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

    /// Whether a case of `op` has a chain form.
    fn chained(&self, op: &str) -> bool {
        self.keys.iter().any(|key| {
            self.crates.iter().any(|library| {
                self.count(op, key, library)
                    .is_some_and(|count| count.chain.is_some())
            })
        })
    }

    /// The chart points of `op` for `crates`: the throughput cycles of each counted case.
    fn points(&self, op: &str, crates: &[&'a str]) -> Vec<Point<'a>> {
        let mut points = Vec::new();
        for library in crates {
            for (category, key) in self.keys.iter().enumerate() {
                if let Some(count) = self.count(op, key, library) {
                    points.push(Point {
                        library,
                        category,
                        cycles: count.batch.cycles,
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
        " The measured files had uncommitted changes."
    } else {
        ""
    };
    let cpus: Vec<String> = unique(record.builds.iter().map(|build| build.sweep.cpu))
        .iter()
        .map(ToString::to_string)
        .collect();
    vec![
        "# Benchmarks".to_owned(),
        String::new(),
        format!(
            "`{}` at commit `{}` against {}: {} {}, in cycles and instructions per operation, \
             measured on {}.{state}",
            layout.subject,
            short(&record.library_commit),
            sentence(&others),
            sentence(&table.ops.iter().map(|(op, _)| *op).collect::<Vec<_>>()),
            (layout.rows_phrase)(&table.keys),
            record.date
        ),
        String::new(),
        format!(
            "The CPU is an {}, measured on its CPU {}.",
            record.cpu_model,
            sentence(&cpus)
        ),
        format!(
            "Its counters give each figure per operation as the median, over {REPETITIONS} \
             repetitions, of a run of 2n operations on fresh operands less a run of n, over n."
        ),
        String::new(),
    ]
}

/// The first seven characters of a commit hash.
fn short(commit: &str) -> &str {
    commit.get(..7).unwrap_or(commit)
}

fn setup<K>(record: &Record<K>) -> Vec<String> {
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
            format!("{}, LLVM {}, `bench` profile", record.rustc, record.llvm),
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
        let extensions: Vec<&str> = build
            .target_features
            .iter()
            .map(String::as_str)
            .filter(|feature| !SYSTEM_FEATURES.contains(feature))
            .collect();
        rows.push(vec![label("Extensions", build), extensions.join(", ")]);
    }
    rows.push(vec![
        "Counters".to_owned(),
        "one Linux perf_event group: user-mode cycles, instructions and branch misses".to_owned(),
    ]);
    for build in &record.builds {
        rows.push(vec![label("Load", build), load(&build.sweep)]);
    }
    rows.push(vec!["Crates".to_owned(), crates.join(", ")]);
    let mut lines = vec!["## Setup".to_owned(), String::new()];
    lines.extend(markdown_table(&[String::new(), String::new()], &rows, "ll"));
    lines.push(String::new());
    lines
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

/// The load around `sweep` and the busy share of the measuring CPU's siblings.
fn load(sweep: &Sweep) -> String {
    let mut text = format!(
        "1-minute load average {} before the sweep and {} after",
        sweep.load.before, sweep.load.after
    );
    for sibling in &sweep.siblings {
        text += &format!(
            "; CPU {}, the SMT sibling of CPU {}, busy {:.0}% of the sweep",
            sibling.cpu,
            sweep.cpu,
            100.0 * sibling.busy
        );
    }
    text
}

fn method<K>(layout: &Layout<'_, K>, record: &Record<K>) -> Vec<String> {
    let mut lines = vec!["## Method".to_owned(), String::new()];
    lines.extend(protocol());
    let loaded = record
        .builds
        .iter()
        .filter(|build| loaded(&build.sweep))
        .count();
    let sentence = match (loaded, record.builds.len()) {
        (0, _) => None,
        (1, 1) => Some("The sweep ran at a 1-minute load of {LOAD} or more, so its cycles"),
        (loaded, builds) if loaded == builds => {
            Some("The sweeps ran at a 1-minute load of {LOAD} or more, so their cycles")
        }
        (_, _) => {
            Some("A sweep ran at a 1-minute load of {LOAD} or more, so the cycles of its build")
        }
    };
    if let Some(sentence) = sentence {
        lines.extend([
            String::new(),
            format!(
                "{} compare within each table only.",
                sentence.replace("{LOAD}", &LOAD.to_string())
            ),
        ]);
    }
    lines.push(String::new());
    lines.extend((layout.method)());
    lines.push(String::new());
    lines
}

/// How the counts are taken.
fn protocol() -> Vec<String> {
    vec![
        format!(
            "A sweep measures every case in {REPETITIONS} repetitions; repetition r of every case \
             runs before repetition r + 1 of any case, so a drift of the machine reaches all \
             cases alike. Each run draws fresh operand sets from the case's distribution, outside \
             the counted region: no operand set runs twice, so the branch predictor never learns \
             a sequence. A repetition runs n operations uncounted, then counts a run of n \
             operations and a run of 2n. Before the repetitions, n starts at 16 operations and \
             doubles until a run of n, counted after an uncounted run of as many, takes {} \
             cycles; n stops earlier where the operand sets of 2n operations fill 256 KiB.",
            chart::grouped(TARGET)
        ),
        String::new(),
        format!(
            "One perf_event group on the measuring thread, pinned to one CPU, counts user-mode \
             cycles, instructions and branch misses; a run during which the kernel shares the \
             counters with other groups stops the sweep. A figure per operation is the median, \
             over the {REPETITIONS} repetitions, of the long run less the short run, divided by \
             n: the fixed cost of a run cancels, as in nanoBench's differencing, and each \
             difference pairs two runs taken back to back. A case whose long runs count no more \
             cycles than its short runs stops the sweep, and so does a chain step under one \
             cycle, the latency of a single instruction."
        ),
        String::new(),
        format!(
            "Throughput runs independent operations, with operands and results behind \
             `core::hint::black_box`. Latency runs the chain `x = op(x, y)`: from the second \
             operand set on, each result is the first operand of the next operation, so each \
             operation waits for the one before. Each result passes through an empty \
             inline-assembly block, which keeps the compiler from regrouping the chain. The block \
             holds a result of up to {REGISTERS} bytes in general-purpose registers in one chain \
             and in vector registers in another, each chain a case of its own in the sweep; it \
             can only add moves between the two, so the latency is the chain with fewer cycles. \
             A larger result passes through memory, in a single chain. The chain applies where \
             the result has the type of the first operand."
        ),
    ]
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
        "Each cell measures the crate's own function:".to_owned(),
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
/// unless all of them. Operations whose rows differ name their rows each.
fn describe<K: Key>(
    layout: &Layout<'_, K>,
    table: &Table<'_, K>,
    library: &str,
    cells: &[(&str, &K)],
) -> String {
    let ops = unique(cells.iter().map(|(op, _)| *op));
    let rows_of = |op: &str| -> Vec<K> {
        unique(
            cells
                .iter()
                .filter(|(other, _)| *other == op)
                .map(|(_, key)| (*key).clone()),
        )
    };
    let rows_phrase = |keys: &[K]| {
        if keys.len() < table.keys.len() {
            format!(" {}", (layout.rows_phrase)(keys))
        } else {
            String::new()
        }
    };
    let per_op: Vec<Vec<K>> = ops.iter().map(|op| rows_of(op)).collect();
    let mut text = format!("`{library}`");
    match per_op.split_first() {
        Some((first, rest)) if rest.iter().any(|keys| keys != first) => {
            let parts: Vec<String> = ops
                .iter()
                .zip(&per_op)
                .map(|(op, keys)| format!("{op}{}", rows_phrase(keys)))
                .collect();
            text += &format!(" {}", sentence(&parts));
        }
        _ => {
            if ops.len() < table.ops.len() {
                text += &format!(" {}", sentence(&ops));
            }
            let keys: Vec<K> = unique(cells.iter().map(|(_, key)| (*key).clone()));
            text += &rows_phrase(&keys);
        }
    }
    text
}

/// The results of `build`: per operation, its chart and its table.
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
    let mut lines = vec![
        heading,
        String::new(),
        "Per operation, each row gives throughput cycles, latency cycles for the operations that \
         have chains, and instructions. Bold marks the fewest in each row."
            .to_owned(),
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
        let mut metrics: Vec<Metric> =
            vec![("throughput cycles", |count| Some(count.batch.cycles))];
        if table.chained(op) {
            metrics.push(("latency cycles", |count| {
                count.chain.map(|latency| latency.events.cycles)
            }));
        }
        metrics.push(("instructions", |count| Some(count.batch.instructions)));
        let mut rows = Vec::new();
        for key in &table.keys {
            for (index, (label, metric)) in metrics.iter().enumerate() {
                let values: Vec<Option<f64>> = table
                    .crates
                    .iter()
                    .map(|library| table.count(op, key, library).and_then(metric))
                    .collect();
                let fewest = values.iter().flatten().copied().reduce(f64::min);
                let mut row = vec![
                    if index == 0 {
                        key.to_string()
                    } else {
                        String::new()
                    },
                    (*label).to_owned(),
                ];
                row.extend(values.iter().map(|figure| match figure {
                    Some(figure) if Some(*figure) == fewest => format!("**{}**", value(*figure)),
                    Some(figure) => value(*figure),
                    None => "–".to_owned(),
                }));
                rows.push(row);
            }
        }
        let mut header = vec![layout.row_header.to_owned(), "Per operation".to_owned()];
        header.extend(table.crates.iter().map(|library| (*library).to_owned()));
        let title = title(op, formula, build);
        lines.extend([
            String::new(),
            format!("### {title}"),
            String::new(),
            format!("![{title}](benchmarks/{})", chart_file(op, build)),
            String::new(),
        ]);
        lines.extend(markdown_table(
            &header,
            &rows,
            &("rl".to_owned() + &"r".repeat(table.crates.len())),
        ));
    }
    lines.push(String::new());
    lines
}

fn reproduce(tool: &str) -> Vec<String> {
    vec![
        "## Reproduce".to_owned(),
        String::new(),
        "```sh".to_owned(),
        format!("cargo run -p {tool} -- bench    # the CPU's counts of the native build"),
        format!("cargo run -p {tool} -- report   # this file and benchmarks/*.svg"),
        "```".to_owned(),
    ]
}
