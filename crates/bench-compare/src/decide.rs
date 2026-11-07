//! `bench-browser decide`: a matched decision between two directories of modules, which the tool
//! `modules` writes. The owner's rule: a fresh figure is compared only with one of the same
//! conditions, so baseline and change run back to back in each build.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{Map, Value};

use crate::browser::{Cpus, LISTING, Threads, list, run as run_in_browser};
use crate::error::Error;
use crate::fewest::fewest;
use crate::fingerprint::Fingerprint;
use crate::record::Count;
use crate::selection::{Plan, Selection};
use crate::tool_error::ToolError;
use crate::wasm::{FLAVOURS, WASM, WasmBuild};
use crate::window::load;

/// What a decision runs.
pub struct Decision<'a> {
    /// The directory of the baseline's modules.
    pub baseline: &'a Path,
    /// The directory of the change's modules.
    pub change: &'a Path,
    /// The cells.
    pub only: &'a str,
    /// The runs of each side, back to back.
    pub runs: usize,
    /// The executable of Chrome or Chromium.
    pub chrome: &'a Path,
    /// The executable of Firefox.
    pub firefox: &'a Path,
    /// The CPUs of the runs.
    pub cpus: Cpus,
}

/// The fields of a line that name no part of its row.
const NOT_ROW: [&str; 6] = ["op", "crate", "counted", "case", "absent", "row"];

/// A module with the names of its cells.
struct Named {
    bytes: Vec<u8>,
    /// Per cell: its operation, its row and its crate, as a selection names them.
    cells: Vec<(String, String, String)>,
    /// The name of each row, as a selection names it, by the fields that the lines of a run hold.
    rows: HashMap<String, String>,
}

/// The fields of the row of the cell `line`, as one text.
fn row_fields(line: &Map<String, Value>) -> String {
    line.iter()
        .filter(|(field, _)| !NOT_ROW.contains(&field.as_str()))
        .map(|(_, value)| value.to_string())
        .collect::<Vec<_>>()
        .join(" ")
}

/// The lines that the module at `path` lists, which the file `<path>.list` keeps per content of the
/// module: a browser takes seconds to start.
fn listing(path: &Path, bytes: &[u8], chrome: &Path) -> Result<Vec<String>, Error> {
    let mut cache = path.as_os_str().to_owned();
    cache.push(".list");
    let cache = PathBuf::from(cache);
    let hash = format!("{:032x}", Fingerprint::of(bytes));
    if cache.exists() {
        let text = fs::read_to_string(&cache).map_err(|source| Error::io(&cache, source))?;
        if let Some(rest) = text.strip_prefix(&hash) {
            return Ok(rest
                .lines()
                .filter(|line| !line.is_empty())
                .map(str::to_owned)
                .collect());
        }
    }
    let lines = list(bytes, &LISTING, chrome)?;
    let text = format!("{hash}\n{}\n", lines.join("\n"));
    fs::write(&cache, text).map_err(|source| Error::io(&cache, source))?;
    Ok(lines)
}

/// The string field `field` of the cell `line`.
fn text(line: &Map<String, Value>, field: &str) -> Result<String, Error> {
    line.get(field)
        .and_then(Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| Error::missing(field, "a line of a listing"))
}

impl Named {
    /// The module at `path`; `None` when there is no such file.
    fn read(path: &Path, chrome: &Path) -> Result<Option<Self>, Error> {
        if !path.exists() {
            return Ok(None);
        }
        let bytes = fs::read(path).map_err(|source| Error::io(path, source))?;
        let mut cells = Vec::new();
        let mut rows = HashMap::new();
        for line in listing(path, &bytes, chrome)? {
            let line: Map<String, Value> = serde_json::from_str(&line).map_err(Error::Cells)?;
            let row = text(&line, "row")?;
            drop(rows.insert(row_fields(&line), row.clone()));
            cells.push((text(&line, "op")?, row, text(&line, "crate")?));
        }
        Ok(Some(Self { bytes, cells, rows }))
    }
}

/// The name of each bench in `directory`: `<bench>.<word>.wasm`.
fn benches(directory: &Path) -> Result<Vec<String>, Error> {
    let mut names = Vec::new();
    for entry in fs::read_dir(directory).map_err(|source| Error::io(directory, source))? {
        let entry = entry.map_err(|source| Error::io(directory, source))?;
        let name = entry.file_name();
        let Some(bench) = name.to_str().and_then(|name| {
            FLAVOURS
                .iter()
                .find_map(|flavour| name.strip_suffix(&format!(".{}.wasm", flavour.word)))
        }) else {
            continue;
        };
        if !names.iter().any(|name| name == bench) {
            names.push(bench.to_owned());
        }
    }
    names.sort_unstable();
    Ok(names)
}

/// A cell of the lines of one side.
struct Figures {
    /// The operation, the row and the crate.
    name: String,
    count: Option<Count>,
}

/// The cells in `lines`, the lines of a run of `module`.
fn figures(lines: Vec<String>, module: &Named) -> Result<Vec<Figures>, Error> {
    let mut cells = Vec::new();
    for line in lines {
        let line: Value = serde_json::from_str(&line).map_err(Error::Cells)?;
        let Some(cell) = line.get("cell").and_then(Value::as_object) else {
            continue;
        };
        let fields = row_fields(cell);
        let row = module
            .rows
            .get(&fields)
            .ok_or_else(|| Error::missing(&format!("row {fields}"), "the listing of the module"))?;
        cells.push(Figures {
            name: format!("{}/{row}/{}", text(cell, "op")?, text(cell, "crate")?),
            count: cell
                .get("counted")
                .cloned()
                .map(serde_json::from_value)
                .transpose()
                .map_err(Error::Cells)?,
        });
    }
    Ok(cells)
}

/// A figure, or a dash without one.
fn figure(value: Option<f64>) -> String {
    value.map_or_else(|| "–".to_owned(), |value| format!("{value:.2}"))
}

/// The change of a figure from `before` to `after`.
fn delta(before: Option<f64>, after: Option<f64>) -> String {
    match (before, after) {
        (Some(before), Some(after)) => format!("{:+.2}", after - before),
        _ => "–".to_owned(),
    }
}

/// The line of a cell of both sides.
fn row(name: &str, before: Option<&Count>, after: Option<&Count>) -> String {
    let batch = |count: Option<&Count>| count.map(|count| count.batch.cycles);
    let chain = |count: Option<&Count>| {
        count.and_then(|count| count.chain.map(|chain| chain.events.cycles))
    };
    let instructions = |count: Option<&Count>| count.map(|count| count.batch.instructions);
    format!(
        "  {name}: throughput {} -> {} ({}), latency {} -> {} ({}), instructions {} -> {}",
        figure(batch(before)),
        figure(batch(after)),
        delta(batch(before), batch(after)),
        figure(chain(before)),
        figure(chain(after)),
        delta(chain(before), chain(after)),
        figure(instructions(before)),
        figure(instructions(after)),
    )
}

/// Runs the cells of `selection` in `module` as `build` measures them; `None` for no cell of it.
fn run_side(
    module: Option<&Named>,
    selection: &Selection,
    build: &WasmBuild,
    decision: &Decision<'_>,
    executable: &Path,
) -> Result<Option<Vec<String>>, Error> {
    let Some(module) = module else {
        return Ok(None);
    };
    let Some(narrowed) = selection.narrowed(&module.cells) else {
        return Ok(None);
    };
    run_in_browser(
        &module.bytes,
        &build.browser,
        executable,
        decision.cpus,
        Threads::Measuring,
        &Plan::Only(narrowed),
    )
    .map(Some)
}

/// Runs the bench `bench` of one build on both sides, back to back, and prints each cell's fewest
/// figures over the runs and the load around them.
fn decide_bench(
    decision: &Decision<'_>,
    selection: &Selection,
    build: &WasmBuild,
    bench: &str,
    sides: [Option<&Named>; 2],
) -> Result<(), Error> {
    let executable = match build.browser.name() {
        "Chrome" => decision.chrome,
        _ => decision.firefox,
    };
    let before = load()?;
    let mut runs: [Vec<Vec<String>>; 2] = [Vec::new(), Vec::new()];
    for _ in 0..decision.runs {
        for (side, lines) in sides.iter().zip(&mut runs) {
            if let Some(line) = run_side(*side, selection, build, decision, executable)? {
                lines.push(line);
            }
        }
    }
    let after = load()?;
    let side = |lines: Vec<Vec<String>>, module: Option<&Named>| match module {
        Some(module) if !lines.is_empty() => fewest(lines).and_then(|lines| figures(lines, module)),
        _ => Ok(Vec::new()),
    };
    let [baseline, change] = runs;
    let baseline = side(baseline, sides[0])?;
    let change = side(change, sides[1])?;
    println!(
        "{}, {bench}: {} runs each; load {before} before, {after} after",
        build.name, decision.runs
    );
    for cell in &baseline {
        let other = change.iter().find(|other| other.name == cell.name);
        println!(
            "{}",
            row(
                &cell.name,
                cell.count.as_ref(),
                other.and_then(|other| other.count.as_ref())
            )
        );
    }
    for cell in change
        .iter()
        .filter(|cell| !baseline.iter().any(|other| other.name == cell.name))
    {
        println!("{}", row(&cell.name, None, cell.count.as_ref()));
    }
    Ok(())
}

/// Runs the cells of `decision` of each bench in each build, baseline and change back to back, and
/// prints their figures. A pattern that no module holds stops it before any run.
pub fn decide(decision: &Decision<'_>) -> Result<(), ToolError> {
    let mut selection = Selection::parse(decision.only)?;
    let names = benches(decision.baseline)?;
    let mut modules = Vec::new();
    for flavour in &FLAVOURS {
        let mut benches = Vec::new();
        for bench in &names {
            let file = format!("{bench}.{}.wasm", flavour.word);
            let base = Named::read(&decision.baseline.join(&file), decision.chrome)?;
            let change = Named::read(&decision.change.join(&file), decision.chrome)?;
            for (op, row, library) in base.iter().chain(&change).flat_map(|side| &side.cells) {
                selection.note(op, row, library);
            }
            benches.push((bench, [base, change]));
        }
        modules.push(benches);
    }
    selection.check_matched()?;
    for build in &WASM {
        for (bench, [base, change]) in &modules[build.flavour] {
            decide_bench(
                decision,
                &selection,
                build,
                bench,
                [base.as_ref(), change.as_ref()],
            )?;
        }
    }
    Ok(())
}
