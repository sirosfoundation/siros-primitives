//! `bench-wasm`, `stamp-wasm` and `modules`: the WebAssembly builds of a comparison, in two browsers.
//!
//! `bench-wasm --changed` is a content-addressed action cache over the cells: see
//! [`crate::changed`].

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use crate::browser::{
    Browser, CHROME, CHROME_WIDE, Cpus, FIREFOX, FIREFOX_WIDE, LISTING, Threads, list,
    run as run_in_browser, version as browser_version,
};
use crate::changed::{CellId, code, compact, harness, merge, overtaken, select};
use crate::count::CPU;
use crate::document::Layout;
use crate::error::{Error, ToolError};
use crate::fewest::fewest;
use crate::identity::Identity;
use crate::module::{custom_section, used_features};
use crate::record::{
    Cell, Engine, Key, Line, Listed, ListedCall, Measurement, Outcome, Overhead, Record, Recorded,
    Run,
};
use crate::run::{Measured, Options, Paths, cargo, check_features, git, parse_cpu};
use crate::selection::{Plan, Selection};
use crate::window::{Window, load};
use crate::workspace::Sources;

/// The target of a WebAssembly build.
const WASM_TARGET: &str = "wasm32-unknown-unknown";

/// The rounds of re-measuring of one build, of which a row needs at most one per crate. Past them a
/// re-measured figure keeps trailing a kept one: [`ToolError::Unsettled`].
const ROUNDS: usize = 12;

/// The runs of a bench that measures a selection, unless `--runs` says otherwise. The repetitions
/// of a selected cell come within seconds and meet one burst of load, where those of a full run
/// spread over minutes. See the README for the figures that chose it.
pub const SELECTED_RUNS: usize = 3;

/// The linker flag that keeps in each module the relocations that [`crate::identity`] reads. Only
/// custom sections and the zeros of `.bss` differ from a module without it.
pub const KEEP_RELOCATIONS: &str = "-C link-arg=--emit-relocs";

/// One compilation of the benches for WebAssembly.
#[derive(Debug)]
pub struct Flavour {
    /// The suffix of its target directory.
    pub slug: &'static str,
    /// The `RUSTFLAGS` that decide its code, and that a record keeps. The build adds
    /// [`KEEP_RELOCATIONS`].
    pub rustflags: &'static str,
    /// The word that names its modules: `<bench>.<word>.wasm`.
    pub word: &'static str,
}

/// The default features of the target, which every current browser runs.
pub const PLAIN: Flavour = Flavour {
    slug: "wasm",
    rustflags: "-C codegen-units=1",
    word: "plain",
};

/// The instructions of the wide-arithmetic proposal: `i64.add128`, `i64.sub128`, `i64.mul_wide_s`
/// and `i64.mul_wide_u`.
pub const WIDE: Flavour = Flavour {
    slug: "wasm-wide",
    rustflags: "-C codegen-units=1 -C target-feature=+wide-arithmetic",
    word: "wide",
};

/// The compilations of every comparison.
pub const FLAVOURS: [Flavour; 2] = [PLAIN, WIDE];

/// A WebAssembly build of a comparison: a compilation in a browser.
#[derive(Debug)]
pub struct WasmBuild {
    /// The name of the build in the document.
    pub name: &'static str,
    /// The compilation: an index of [`FLAVOURS`].
    pub flavour: usize,
    /// The browser, with its settings.
    pub browser: Browser,
}

/// The WebAssembly builds of every comparison, which `bench-wasm` measures: with and without the
/// instructions of the wide-arithmetic proposal, in Chrome and in Firefox.
pub const WASM: [WasmBuild; 4] = [
    WasmBuild {
        name: "in WebAssembly, Chrome",
        flavour: 0,
        browser: CHROME,
    },
    WasmBuild {
        name: "in WebAssembly with wide arithmetic, Chrome",
        flavour: 1,
        browser: CHROME_WIDE,
    },
    WasmBuild {
        name: "in WebAssembly, Firefox",
        flavour: 0,
        browser: FIREFOX,
    },
    WasmBuild {
        name: "in WebAssembly with wide arithmetic, Firefox",
        flavour: 1,
        browser: FIREFOX_WIDE,
    },
];

/// The modules of one compilation, in the order of [`Measured::benches`].
struct Built {
    modules: Vec<PathBuf>,
    /// The target features that the modules use.
    target_features: Vec<String>,
}

/// The target directory of `flavour`, under `paths.root/target` or `base`.
fn results(paths: &Paths, base: Option<&Path>, flavour: &Flavour) -> PathBuf {
    base.map_or_else(|| paths.root.join("target"), Path::to_path_buf)
        .join(format!("benchmarks-{}", flavour.slug))
}

/// The features that the WebAssembly `module` at `path` uses: its `target_features` section, which
/// the linker writes. `rustc --print cfg` leaves out the features that rustc keeps unstable, wide
/// arithmetic among them.
fn module_features(module: &[u8], path: &Path) -> Result<Vec<String>, ToolError> {
    let section = custom_section(module, b"target_features")?
        .ok_or_else(|| Error::missing("target_features section", &path.display().to_string()))?;
    Ok(used_features(section)?)
}

/// Builds the benches of one compilation.
fn build_flavour(
    paths: &Paths,
    measured: &Measured,
    base: Option<&Path>,
    flavour: &Flavour,
) -> Result<Built, ToolError> {
    let features = measured.features.join(",");
    let mut args = vec!["bench", "--locked", "-p", measured.package];
    if !features.is_empty() {
        args.extend(["--features", &features]);
    }
    args.extend(["--target", WASM_TARGET]);
    for bench in measured.benches {
        args.extend(["--bench", bench]);
    }
    args.extend(["--no-run", "--message-format=json"]);
    let target = results(paths, base, flavour);
    let rustflags = format!("{} {KEEP_RELOCATIONS}", flavour.rustflags);
    let messages = cargo(&paths.root, &target, &rustflags, &args)?;
    let modules = check_features(measured, &messages)?;
    let mut features: Option<Vec<String>> = None;
    for path in &modules {
        let module = fs::read(path).map_err(|source| Error::io(path, source))?;
        let found = module_features(&module, path)?;
        match &features {
            Some(first) if *first != found => {
                return Err(Error::missing(
                    &format!("the target features of the first module, {first:?}"),
                    &path.display().to_string(),
                )
                .into());
            }
            Some(_) => {}
            None => features = Some(found),
        }
    }
    let target_features = features.ok_or_else(|| Error::missing("bench module", "the build"))?;
    Ok(Built {
        modules,
        target_features,
    })
}

/// Builds the benches of every compilation, at once: the two take the same time.
fn build_all(
    paths: &Paths,
    measured: &Measured,
    base: Option<&Path>,
) -> Result<Vec<Built>, ToolError> {
    std::thread::scope(|scope| {
        let builds: Vec<_> = FLAVOURS
            .iter()
            .map(|flavour| scope.spawn(move || build_flavour(paths, measured, base, flavour)))
            .collect();
        builds
            .into_iter()
            .map(|build| {
                build
                    .join()
                    .map_err(|_payload| Error::missing("result", "a thread of the build"))?
            })
            .collect()
    })
}

/// A cell of a module.
struct ListedCell<K> {
    id: CellId<K>,
    /// The reason for its absence, or what its figures depend on.
    state: CellState,
}

/// What a module says of a cell.
enum CellState {
    /// It has no case; the reason.
    Absent(String),
    /// It has a case, whose code is this; `None` for a cell that always counts as changed.
    Case(Option<String>),
}

/// A module with its cells.
struct Listing<K> {
    bench: String,
    bytes: Vec<u8>,
    cells: Vec<ListedCell<K>>,
}

impl<K> Listing<K> {
    /// The number of cells that hold a case.
    fn cases(&self) -> usize {
        self.cells
            .iter()
            .filter(|cell| matches!(cell.state, CellState::Case(_)))
            .count()
    }
}

/// The cells of the module at `path`, which `chrome` lists.
fn list_module<K: Key>(path: &Path, bench: &str, chrome: &Path) -> Result<Listing<K>, ToolError> {
    let bytes = fs::read(path).map_err(|source| Error::io(path, source))?;
    let lines = list(&bytes, &LISTING, chrome)?;
    let mut identity = Identity::of(&bytes)?;
    let mut cells = Vec::with_capacity(lines.len());
    for line in lines {
        let listed: Listed<K> = serde_json::from_str(&line).map_err(Error::Cells)?;
        let state = match listed.call {
            ListedCall::Absent(reason) => CellState::Absent(reason),
            ListedCall::Case(case) => CellState::Case(code(&mut identity, &case)?),
        };
        cells.push(ListedCell {
            id: CellId {
                op: listed.op,
                key: listed.key,
                library: listed.library,
            },
            state,
        });
    }
    Ok(Listing {
        bench: bench.to_owned(),
        bytes,
        cells,
    })
}

/// The listings of every module of every compilation.
fn list_all<K: Key>(
    measured: &Measured,
    built: &[Built],
    chrome: &Path,
) -> Result<Vec<Vec<Listing<K>>>, ToolError> {
    built
        .iter()
        .map(|built| {
            built
                .modules
                .iter()
                .zip(measured.benches)
                .map(|(path, bench)| list_module(path, bench, chrome))
                .collect()
        })
        .collect()
}

/// `options`' number of runs, if it names one.
fn parse_runs(options: &Options) -> Result<Option<usize>, Error> {
    options
        .runs
        .as_deref()
        .map(|text| {
            text.parse::<usize>()
                .map_err(|source| Error::number("runs", text, source))
        })
        .transpose()
}

/// What stays the same for every build of a sweep.
struct Context<'a, K> {
    layout: &'a Layout<'a, K>,
    options: &'a Options,
    cpus: Cpus,
    /// The number of runs of a bench, if `--runs` names one.
    runs: Option<usize>,
    /// The record before the sweep.
    old: Option<&'a Record<K>>,
    /// The record's fields of this sweep, whose head and state name a new run.
    fresh: &'a Record<K>,
    /// Whether a sweep over all cells, whatever the record holds.
    full: bool,
}

/// What a bench counted.
struct Counted<K> {
    /// The cost of a count.
    overhead: Overhead,
    cells: Vec<Cell<K>>,
}

/// The state of a build before its first round.
struct Started<K> {
    /// What each module lists, with the figures that the record holds.
    cells: Vec<Recorded<K>>,
    /// The cells whose key differs from the record.
    changed: Vec<CellId<K>>,
    /// The runs that the kept figures come from.
    runs: Vec<Run>,
    /// The cost of a count per bench, where the record's holds.
    overheads: Vec<Option<Overhead>>,
}

/// What the bench `bench` counted, from its `lines`.
fn parse_lines<K: Key>(lines: Vec<String>, bench: &str) -> Result<Counted<K>, Error> {
    let wrong = || {
        Error::missing(
            "the cost of a count, then cells",
            &format!("the lines of the WebAssembly bench {bench}"),
        )
    };
    let mut overhead = None;
    let mut cells = Vec::with_capacity(lines.len());
    for line in lines {
        match serde_json::from_str(&line).map_err(Error::Cells)? {
            Line::Overhead(cost) if overhead.is_none() => overhead = Some(cost),
            Line::Cell(cell) if overhead.is_some() => cells.push(cell),
            Line::Overhead(_) | Line::Cell(_) | Line::Sweep(_) => return Err(wrong()),
        }
    }
    overhead
        .map(|overhead| Counted { overhead, cells })
        .ok_or_else(wrong)
}

impl<K: Key> Context<'_, K> {
    /// Whether a figure of the build `old` in the record could hold for `build`: everything in the
    /// key beside the code of the cell. The cost of a count of each bench goes with its figures, so
    /// a record of other benches counts too.
    fn same_settings(
        &self,
        old: &Measurement<K>,
        record: &Record<K>,
        build: &WasmBuild,
        engine: &Engine,
        benches: usize,
    ) -> bool {
        old.browser.as_ref() == Some(engine)
            && old.rustflags == FLAVOURS[build.flavour].rustflags
            && old.harness.as_deref() == Some(harness().as_str())
            && record.cpu_model == self.fresh.cpu_model
            && old.overheads.len() == benches
    }

    /// The state of `build` before its first round: what each module lists, with the figures that
    /// the record holds.
    fn start(&self, build: &WasmBuild, engine: &Engine, listings: &[Listing<K>]) -> Started<K> {
        let old = self.old.filter(|_| !self.full).and_then(|record| {
            Some((
                record,
                record.builds.iter().find(|old| old.name == build.name)?,
            ))
        });
        let settings = old.is_some_and(|(record, measurement)| {
            self.same_settings(measurement, record, build, engine, listings.len())
        });
        let by_id: HashMap<CellId<K>, &Recorded<K>> = old
            .iter()
            .flat_map(|(_, measurement)| &measurement.cells)
            .map(|cell| (CellId::of(&cell.cell), cell))
            .collect();
        let mut cells = Vec::new();
        let mut changed = Vec::new();
        for listed in listings.iter().flat_map(|listing| &listing.cells) {
            let cell = |outcome| Cell {
                op: listed.id.op.clone(),
                key: listed.id.key.clone(),
                library: listed.id.library.clone(),
                outcome,
            };
            let recorded = match &listed.state {
                CellState::Absent(reason) => Recorded {
                    cell: cell(Outcome::Absent(reason.clone())),
                    run: 0,
                    code: None,
                },
                CellState::Case(code) => {
                    let kept = by_id.get(&listed.id).filter(|kept| kept.counted());
                    let same =
                        settings && code.is_some() && kept.is_some_and(|kept| kept.code == *code);
                    if !same {
                        changed.push(listed.id.clone());
                    }
                    match kept {
                        Some(kept) => Recorded {
                            code: code.clone(),
                            ..(*kept).clone()
                        },
                        // It holds no figure until a round measures it.
                        None => Recorded {
                            cell: cell(Outcome::Absent("not measured".to_owned())),
                            run: 0,
                            code: code.clone(),
                        },
                    }
                }
            };
            cells.push(recorded);
        }
        let (runs, overheads) = old.map_or_else(
            || (Vec::new(), vec![None; listings.len()]),
            |(_, measurement)| {
                let overheads = if settings {
                    measurement.overheads.iter().copied().map(Some).collect()
                } else {
                    vec![None; listings.len()]
                };
                (measurement.runs.clone(), overheads)
            },
        );
        Started {
            cells,
            changed,
            runs,
            overheads,
        }
    }

    /// Measures `build` by rounds until every verdict compares figures of one run. Returns the
    /// measurement, and the number of cells of its first round.
    fn measure(
        &self,
        build: &WasmBuild,
        listings: &[Listing<K>],
        target_features: &[String],
    ) -> Result<(Measurement<K>, usize), ToolError> {
        let executable = self.options.executable(&build.browser)?;
        let engine = Engine {
            name: build.browser.name().to_owned(),
            version: browser_version(executable)?,
            settings: build.browser.settings(),
        };
        let Started {
            mut cells,
            changed,
            mut runs,
            mut overheads,
        } = self.start(build, &engine, listings);
        let total = cells
            .iter()
            .filter(|cell| cell.code.is_some() || cell.counted())
            .count();
        let mut seeds = changed;
        let mut first = None;
        for _ in 0..ROUNDS {
            if seeds.is_empty() {
                break;
            }
            let selected = select(self.layout, &cells, std::mem::take(&mut seeds))?;
            first = first.or(Some(selected.len()));
            println!(
                "{}: measuring {} of {total} cells; load {}",
                build.name,
                selected.len(),
                load()?
            );
            let window = Window::open(self.cpus.measuring)?;
            let mut fresh = Vec::with_capacity(selected.len());
            for (index, listing) in listings.iter().enumerate() {
                let Some(counted) = self.run_module(build, executable, listing, &selected)? else {
                    continue;
                };
                overheads[index] = Some(counted.overhead);
                fresh.extend(counted.cells);
            }
            let sweep = window.close()?;
            runs.push(Run {
                commit: self.fresh.head.clone(),
                dirty: self.fresh.dirty,
                date: self.fresh.date.clone(),
                sweep,
            });
            merge(&mut cells, fresh, runs.len() - 1)?;
            seeds = overtaken(self.layout, &cells)?;
        }
        if let Some(seed) = seeds.first() {
            return Err(ToolError::Unsettled {
                build: build.name.to_owned(),
                rows: seeds.len(),
                rounds: ROUNDS,
                first: seed.pattern(),
            });
        }
        let runs = compact(runs, &mut cells);
        let overheads = overheads
            .into_iter()
            .zip(listings)
            .map(|(overhead, listing)| {
                overhead.ok_or_else(|| {
                    Error::missing("cost of a count", &format!("the bench {}", listing.bench))
                })
            })
            .collect::<Result<Vec<_>, Error>>()?;
        Ok((
            Measurement {
                name: build.name.to_owned(),
                rustflags: FLAVOURS[build.flavour].rustflags.to_owned(),
                target_features: target_features.to_vec(),
                browser: Some(engine),
                overheads,
                harness: Some(harness()),
                runs,
                cells,
            },
            first.unwrap_or(0),
        ))
    }

    /// Runs the cells of `listing` in `selected`, if it has any, as many times as the options say.
    fn run_module(
        &self,
        build: &WasmBuild,
        executable: &Path,
        listing: &Listing<K>,
        selected: &HashSet<CellId<K>>,
    ) -> Result<Option<Counted<K>>, ToolError> {
        let patterns: Vec<String> = listing
            .cells
            .iter()
            .filter(|cell| matches!(cell.state, CellState::Case(_)))
            .filter(|cell| selected.contains(&cell.id))
            .map(|cell| cell.id.pattern())
            .collect();
        if patterns.is_empty() {
            return Ok(None);
        }
        let (plan, runs) = if patterns.len() == listing.cases() {
            (Plan::All, self.runs.unwrap_or(1))
        } else {
            (
                Plan::Only(Selection::of(patterns)?),
                self.runs.unwrap_or(SELECTED_RUNS),
            )
        };
        let lines = (0..runs)
            .map(|_| {
                run_in_browser(
                    &listing.bytes,
                    &build.browser,
                    executable,
                    self.cpus,
                    Threads::Measuring,
                    &plan,
                )
            })
            .collect::<Result<Vec<_>, Error>>()?;
        Ok(Some(parse_lines(fewest(lines)?, &listing.bench)?))
    }
}

/// Prints what the first round of each build would measure, by name.
fn print_selection<K: Key>(
    layout: &Layout<'_, K>,
    context: &Context<'_, K>,
    listings: &[Vec<Listing<K>>],
) -> Result<(), ToolError> {
    for build in &WASM {
        let executable = context.options.executable(&build.browser)?;
        let engine = Engine {
            name: build.browser.name().to_owned(),
            version: browser_version(executable)?,
            settings: build.browser.settings(),
        };
        let Started { cells, changed, .. } =
            context.start(build, &engine, &listings[build.flavour]);
        let selected = select(layout, &cells, changed.iter().cloned())?;
        let changed: HashSet<CellId<K>> = changed.into_iter().collect();
        let total = listings[build.flavour]
            .iter()
            .map(Listing::cases)
            .sum::<usize>();
        println!(
            "{}: {} of {total} cells: {} changed, {} compared with them",
            build.name,
            selected.len(),
            changed.len(),
            selected.len() - changed.len()
        );
        let mut lines: Vec<String> = selected
            .iter()
            .map(|id| {
                let reason = if changed.contains(id) {
                    "changed "
                } else {
                    "compared"
                };
                format!("  {reason} {}", id.pattern())
            })
            .collect();
        lines.sort_unstable();
        for line in lines {
            println!("{line}");
        }
    }
    Ok(())
}

/// Measures the WebAssembly builds of the bench and writes their record. With `--changed` it measures
/// the cells whose key differs from the record, what they are compared with, and what overtakes those
/// by a round of its own; with `--dry-run` it stops after printing them. Without it, every cell.
pub(crate) fn bench_wasm<K: Key>(
    paths: &Paths,
    measured: &Measured,
    layout: &Layout<'_, K>,
    options: &Options,
) -> Result<(), ToolError> {
    let chrome = options.executable(&CHROME)?;
    let measuring = parse_cpu(options.cpu.as_deref())?.unwrap_or(CPU);
    let cpus = Cpus {
        measuring,
        browser: parse_cpu(options.browser_cpu.as_deref())?.unwrap_or(measuring),
    };
    let started = crate::run::now()?;
    let fresh = crate::run::skeleton::<K>(paths, measured, options, started)?;
    let load_before = load()?;
    let clock = Instant::now();
    let built = build_all(paths, measured, options.target_dir.as_deref())?;
    let building = clock.elapsed();
    let clock = Instant::now();
    let listings = list_all::<K>(measured, &built, chrome)?;
    let listing = clock.elapsed();
    let old = if options.changed && paths.wasm_record.exists() {
        Some(Record::read(&paths.wasm_record)?)
    } else {
        None
    };
    let context = Context {
        layout,
        options,
        cpus,
        runs: parse_runs(options)?,
        old: old.as_ref(),
        fresh: &fresh,
        full: !options.changed,
    };
    if options.dry_run {
        print_selection(layout, &context, &listings)?;
        println!(
            "built in {:.1} s, listed in {:.1} s; load {load_before} before, {} after",
            building.as_secs_f64(),
            listing.as_secs_f64(),
            load()?
        );
        return Ok(());
    }
    let clock = Instant::now();
    let mut builds = Vec::with_capacity(WASM.len());
    let mut measuring_cells = 0;
    for build in &WASM {
        let (measurement, cells) = context.measure(
            build,
            &listings[build.flavour],
            &built[build.flavour].target_features,
        )?;
        measuring_cells += cells;
        builds.push(measurement);
    }
    let running = clock.elapsed();
    let date = builds
        .iter()
        .flat_map(|build| &build.runs)
        .map(|run| run.date.clone())
        .max()
        .unwrap_or_else(|| fresh.date.clone());
    let written = Record {
        builds,
        date,
        ..fresh
    };
    written.write(&paths.wasm_record)?;
    println!(
        "built in {:.1} s, listed in {:.1} s, measured {measuring_cells} cells in {:.1} s; load \
         {load_before} before, {} after",
        building.as_secs_f64(),
        listing.as_secs_f64(),
        running.as_secs_f64(),
        load()?
    );
    Ok(())
}

/// Writes the keys of the tree into the WebAssembly record, without measuring: its figures keep
/// holding for a tree whose measured sources the record's commit holds. The harness stays out of
/// this check: `bench-browser diff` shows that a new harness leaves the measured code as it was.
pub(crate) fn stamp_wasm<K: Key>(
    paths: &Paths,
    measured: &Measured,
    options: &Options,
) -> Result<(), ToolError> {
    let refused = |reason: String| ToolError::Unstamped { reason };
    let chrome = options.executable(&CHROME)?;
    let mut record = Record::<K>::read(&paths.wasm_record)?;
    if record.dirty {
        return Err(refused(format!(
            "the measured files differed from {} when it ran",
            record.head
        )));
    }
    let sources = Sources::of(paths, measured.package)?;
    let harness_files = "crates/bench-compare";
    let pathspecs: Vec<&str> = sources
        .executable()
        .into_iter()
        .filter(|path| !path.starts_with(harness_files))
        .collect();
    let status = git(paths, &["status", "--porcelain", "--"], &pathspecs)?;
    if !status.is_empty() {
        return Err(refused(format!(
            "the tree has uncommitted changes:\n{status}"
        )));
    }
    let moved = git(
        paths,
        &["diff", "--name-only", record.head.as_str(), "--"],
        &pathspecs,
    )?;
    if !moved.is_empty() {
        return Err(refused(format!(
            "these files differ from {}:\n{moved}",
            record.head
        )));
    }
    let built = build_all(paths, measured, options.target_dir.as_deref())?;
    let listings = list_all::<K>(measured, &built, chrome)?;
    for wasm in &WASM {
        let rustflags = FLAVOURS[wasm.flavour].rustflags;
        let build = record
            .builds
            .iter_mut()
            .find(|build| build.name == wasm.name)
            .ok_or_else(|| refused(format!("it has no build `{}`", wasm.name)))?;
        if build.rustflags != rustflags {
            return Err(refused(format!(
                "the build `{}` has RUSTFLAGS `{}`, and this tool builds with `{rustflags}`",
                wasm.name, build.rustflags
            )));
        }
        let mut codes: HashMap<CellId<K>, Option<String>> = HashMap::new();
        for listed in listings[wasm.flavour]
            .iter()
            .flat_map(|listing| &listing.cells)
        {
            if let CellState::Case(code) = &listed.state {
                *codes.entry(listed.id.clone()).or_insert(None) = code.clone();
            }
        }
        let counted = build.cells.iter().filter(|cell| cell.counted()).count();
        if counted != codes.len() {
            return Err(refused(format!(
                "the build `{}` holds {counted} counted cells, and the modules {}",
                wasm.name,
                codes.len()
            )));
        }
        for cell in build.cells.iter_mut().filter(|cell| cell.counted()) {
            let id = CellId::of(&cell.cell);
            let code = codes
                .get(&id)
                .ok_or_else(|| refused(format!("the modules have no cell {}", id.pattern())))?;
            cell.code.clone_from(code);
        }
        build.harness = Some(harness());
    }
    record.write(&paths.wasm_record)?;
    Ok(())
}

/// Builds the modules of every compilation, and copies each to `<out>/<bench>.<word>.wasm`, for
/// `bench-browser decide`.
pub(crate) fn write_modules(
    paths: &Paths,
    measured: &Measured,
    options: &Options,
    out: &Path,
) -> Result<(), ToolError> {
    let built = build_all(paths, measured, options.target_dir.as_deref())?;
    fs::create_dir_all(out).map_err(|source| Error::io(out, source))?;
    for (flavour, built) in FLAVOURS.iter().zip(&built) {
        for (module, bench) in built.modules.iter().zip(measured.benches) {
            let target = out.join(format!("{bench}.{}.wasm", flavour.word));
            let copied = fs::copy(module, &target).map_err(|source| Error::io(&target, source))?;
            println!("{}: {copied} bytes", target.display());
        }
    }
    Ok(())
}
