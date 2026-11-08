//! `bench-wasm`, `stamp-wasm` and `modules`: the WebAssembly builds of a comparison, in two browsers.
//!
//! `bench-wasm --changed` is a content-addressed action cache over the cells: see
//! [`crate::changed`].

use std::collections::{HashMap, HashSet, VecDeque};
use std::fs;
use std::io::{self, Read as _, Write as _};
use std::panic::{self, AssertUnwindSafe};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::time::Instant;

use bench_compare::{
    CPU, Cell, Error, Key, Line, Listed, ListedCall, Outcome, Overhead, Plan, Selection, Sweep,
    Window, load,
};

use crate::browser::{
    Browser, CHROME, CHROME_WIDE, Cpus, FIREFOX, FIREFOX_WIDE, LISTING, Threads, die_with_parent,
    list, panicked, run as run_in_browser, version as browser_version,
};
use crate::changed::{CellId, code, compact, harness, merge, overtaken, select};
use crate::document::Layout;
use crate::fewest::fewest;
use crate::identity::Identity;
use crate::module::{custom_section, used_features};
use crate::record::{Engine, Measurement, Record, Recorded, Run, Unfinished};
use crate::run::{
    Measured, Options, Paths, SELECTED_RUNS, attempts, cargo, check_features, git, locked_closure,
};
use crate::tool_error::ToolError;
use crate::workspace::Sources;

/// The target of a WebAssembly build.
const WASM_TARGET: &str = "wasm32-unknown-unknown";

/// The rounds of re-measuring of one build, of which a row needs at most one per crate. Past them a
/// re-measured figure keeps trailing a kept one: [`ToolError::Unsettled`].
const ROUNDS: usize = 12;

/// The command of a report tool that runs one bench once, which [`Unit::attempt`] starts.
pub(crate) const RUN_BENCH: &str = "run-bench";

/// The reason of a cell that no run of this sweep or of the record measured yet.
const NOT_MEASURED: &str = "not measured";

/// The linker flag that keeps in each module the relocations that [`Identity`] reads. Only custom
/// sections and the zeros of `.bss` differ from a module without it.
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
    let section = custom_section(module, b"target_features")?.ok_or_else(|| {
        ToolError::missing("target_features section", &path.display().to_string())
    })?;
    Ok(used_features(section)?)
}

/// Builds the benches of one compilation, and checks what it built.
fn build(
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
        let module = fs::read(path).map_err(|source| ToolError::io(path, source))?;
        let found = module_features(&module, path)?;
        match &features {
            Some(first) if *first != found => {
                return Err(ToolError::missing(
                    &format!("the target features of the first module, {first:?}"),
                    &path.display().to_string(),
                ));
            }
            Some(_) => {}
            None => features = Some(found),
        }
    }
    let target_features =
        features.ok_or_else(|| ToolError::missing("bench module", "the build"))?;
    Ok(Built {
        modules,
        target_features,
    })
}

/// Builds the benches of every compilation, in parallel. Every build ends before the first failure
/// returns.
fn build_all(
    paths: &Paths,
    measured: &Measured,
    base: Option<&Path>,
) -> Result<Vec<Built>, ToolError> {
    std::thread::scope(|scope| {
        let builds: Vec<_> = FLAVOURS
            .iter()
            .map(|flavour| scope.spawn(move || build(paths, measured, base, flavour)))
            .collect();
        builds
            .into_iter()
            .map(|build| build.join().map_err(panicked)?)
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

impl<K: Key> Listing<K> {
    /// The number of cells that hold a case.
    fn cases(&self) -> usize {
        self.cells
            .iter()
            .filter(|cell| matches!(cell.state, CellState::Case(_)))
            .count()
    }

    /// The code of the cell `id`; `None` for an absent cell and for one that always counts as
    /// changed.
    fn code(&self, id: &CellId<K>) -> Result<Option<String>, ToolError> {
        let listed = self
            .cells
            .iter()
            .find(|cell| cell.id == *id)
            .ok_or_else(|| {
                ToolError::missing(&id.pattern(), &format!("the cells of {}", self.bench))
            })?;
        Ok(match &listed.state {
            CellState::Absent(_) => None,
            CellState::Case(code) => code.clone(),
        })
    }
}

/// The cells of the module at `path`, which `chrome` lists.
fn list_module<K: Key>(path: &Path, bench: &str, chrome: &Path) -> Result<Listing<K>, ToolError> {
    let bytes = fs::read(path).map_err(|source| ToolError::io(path, source))?;
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
fn parse_runs(options: &Options) -> Result<Option<usize>, ToolError> {
    options
        .runs
        .as_deref()
        .map(|text| {
            text.parse::<usize>()
                .map_err(|source| ToolError::number("runs", text, source))
        })
        .transpose()
}

/// The streams of a sweep: one per measuring CPU in `cpus`, each with the CPU of the same place in
/// `browser_cpus`, or the measuring CPU itself where that is absent. Without `cpus`, one stream on
/// [`CPU`]. No CPU serves two streams.
pub fn streams(cpus: Option<&str>, browser_cpus: Option<&str>) -> Result<Vec<Cpus>, ToolError> {
    let list = |text: &str| {
        text.split(',')
            .map(|cpu| {
                cpu.parse::<usize>()
                    .map_err(|source| ToolError::number("CPU", cpu, source))
            })
            .collect::<Result<Vec<_>, _>>()
    };
    let measuring = cpus.map_or_else(|| Ok(vec![CPU]), list)?;
    let browser = browser_cpus
        .map(list)
        .transpose()?
        .unwrap_or_else(|| measuring.clone());
    if browser.len() != measuring.len() {
        return Err(ToolError::Streams {
            reason: format!(
                "{} measuring CPUs and {} browser CPUs",
                measuring.len(),
                browser.len()
            ),
        });
    }
    let streams: Vec<Cpus> = measuring
        .into_iter()
        .zip(browser)
        .map(|(measuring, browser)| Cpus { measuring, browser })
        .collect();
    for (index, stream) in streams.iter().enumerate() {
        let shared = streams[..index].iter().find(|other| {
            [other.measuring, other.browser]
                .iter()
                .any(|cpu| *cpu == stream.measuring || *cpu == stream.browser)
        });
        if let Some(other) = shared {
            return Err(ToolError::Streams {
                reason: format!(
                    "the streams on CPUs {} and {}, and on {} and {}, share a CPU",
                    other.measuring, other.browser, stream.measuring, stream.browser
                ),
            });
        }
    }
    Ok(streams)
}

/// What stays the same for every build of a sweep.
struct Context<'a, K> {
    layout: &'a Layout<'a, K>,
    options: &'a Options,
    /// The number of runs of a bench, if `--runs` names one.
    runs: Option<usize>,
    /// The record before the sweep, finished or not.
    old: Option<&'a Unfinished<K>>,
    /// The record's fields of this sweep, whose head and state name a new run.
    fresh: &'a Unfinished<K>,
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
    /// What each module lists, with the figures and the costs of a count that the record holds and
    /// the runs that they come from. A figure whose key differs holds no code: a sweep that stops
    /// before it measures the cell again leaves it changed.
    measurement: Measurement<K, Option<Overhead>>,
    /// The cells whose key differs from the record.
    changed: Vec<CellId<K>>,
}

impl<K: Key> Started<K> {
    /// The cells that the first round measures again, before what their rows compare them with:
    /// the changed cells, and the rows whose verdict compares figures of different runs, which a
    /// sweep that stopped between two rounds leaves.
    fn seeds(&self, layout: &Layout<'_, K>) -> Result<Vec<CellId<K>>, ToolError> {
        let mut seeds = self.changed.clone();
        seeds.extend(overtaken(layout, &self.measurement.cells)?);
        Ok(seeds)
    }
}

/// What the bench `bench` counted, from its `lines`.
fn parse_lines<K: Key>(lines: Vec<String>, bench: &str) -> Result<Counted<K>, ToolError> {
    let wrong = || {
        ToolError::missing(
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

/// The runs of one bench in one build, back to back on one stream, which takes it from the queue of
/// the sweep.
struct Unit<'a> {
    /// The build: an index of [`WASM`].
    build: usize,
    /// The bench: an index of the build's listings.
    bench: usize,
    /// The name of the bench.
    name: &'a str,
    module: &'a [u8],
    /// The browser's executable.
    executable: &'a Path,
    plan: Plan,
    runs: usize,
    /// The cells that it measures.
    cells: usize,
    /// The round of its build, from 0.
    round: usize,
}

impl Unit<'_> {
    /// The lines of the runs on `stream`, with the fewest figures of each cell, and the span of the
    /// runs.
    fn measure(&self, stream: Cpus) -> Result<(Vec<String>, Sweep), ToolError> {
        println!(
            "{}: {}: {} cells, {} runs, on CPUs {} and {}; load {}",
            WASM[self.build].name,
            self.name,
            self.cells,
            self.runs,
            stream.measuring,
            stream.browser,
            load()?
        );
        let window = Window::open(stream.measuring)?;
        let lines = (0..self.runs)
            .map(|_| self.attempts(stream))
            .collect::<Result<Vec<_>, ToolError>>()?;
        Ok((fewest(lines)?, window.close()?))
    }

    /// The lines of one run, in up to [`ATTEMPTS`](crate::run::ATTEMPTS) attempts.
    fn attempts(&self, stream: Cpus) -> Result<Vec<String>, ToolError> {
        attempts(&format!("{}: {}", WASM[self.build].name, self.name), || {
            self.attempt(stream)
        })
    }

    /// The lines of one run, in a process of its own: the driver's subreaper and the processes it
    /// finds and stops are the whole process's, so two streams need two processes. The process
    /// runs the executable of this one, whatever replaced its file since, and reads the module from
    /// its standard input.
    fn attempt(&self, stream: Cpus) -> Result<Vec<String>, ToolError> {
        let command = format!("{RUN_BENCH} of {}", self.name);
        let mut child = Command::new("/proc/self/exe")
            .arg(RUN_BENCH)
            .arg(self.build.to_string())
            .arg(self.executable)
            .arg(stream.measuring.to_string())
            .arg(stream.browser.to_string())
            .arg(self.plan.text())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|source| ToolError::Spawn {
                command: command.clone(),
                source,
            })?;
        let written = child
            .stdin
            .take()
            .ok_or_else(|| ToolError::missing("standard input", &command))
            .and_then(|mut input| {
                input
                    .write_all(self.module)
                    .map_err(|source| ToolError::Refused {
                        reason: format!("{command}: writing the module: {source}"),
                    })
            });
        let output = child
            .wait_with_output()
            .map_err(|source| ToolError::Refused {
                reason: format!("{command}: waiting: {source}"),
            })?;
        if !output.status.success() {
            return Err(ToolError::Refused {
                reason: format!(
                    "{}: {}: {}: {}",
                    WASM[self.build].name,
                    self.name,
                    output.status,
                    String::from_utf8_lossy(&output.stderr).trim_end()
                ),
            });
        }
        written?;
        Ok(String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(str::to_owned)
            .collect())
    }
}

/// Runs one bench once, in the process of [`RUN_BENCH`] that [`Unit::attempt`] starts: `args` name
/// the build, the browser's executable, the measuring CPU, the browser's CPU and the plan, and the
/// standard input holds the module. It prints the lines of the bench.
pub(crate) fn run_bench(args: &[String]) -> Result<(), ToolError> {
    die_with_parent()?;
    let [build, executable, measuring, browser, plan] = args else {
        return Err(ToolError::Refused {
            reason: format!("the arguments of {RUN_BENCH}: {args:?}"),
        });
    };
    let number = |what: &str, text: &str| {
        text.parse::<usize>()
            .map_err(|source| ToolError::number(what, text, source))
    };
    let build = WASM
        .get(number("build", build)?)
        .ok_or_else(|| ToolError::missing(&format!("build {build}"), "the WebAssembly builds"))?;
    let cpus = Cpus {
        measuring: number("CPU", measuring)?,
        browser: number("CPU", browser)?,
    };
    let mut module = Vec::new();
    let read = io::stdin()
        .lock()
        .read_to_end(&mut module)
        .map_err(|source| ToolError::Refused {
            reason: format!("reading the module: {source}"),
        })?;
    if read == 0 {
        return Err(ToolError::missing("module", "the standard input"));
    }
    let lines = run_in_browser(
        &module,
        &build.browser,
        Path::new(executable),
        cpus,
        Threads::Measuring,
        &Plan::parse(plan)?,
    )?;
    let mut out = io::stdout().lock();
    for line in lines {
        writeln!(out, "{line}").map_err(|source| ToolError::Refused {
            reason: format!("writing the lines: {source}"),
        })?;
    }
    Ok(())
}

impl<'a, K: Key> Context<'a, K> {
    /// Whether a figure of the build `old` in the record could hold for `build`: everything in the
    /// key beside the code of the cell. The cost of a count of each bench goes with its figures, so
    /// a record of other benches counts too.
    fn same_settings(
        &self,
        old: &Measurement<K, Option<Overhead>>,
        record: &Unfinished<K>,
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

    /// The state of `build` before its first round: what each of its `listings` lists, with the
    /// figures that the record holds.
    fn start(
        &self,
        build: &WasmBuild,
        listings: &[Listing<K>],
        target_features: &[String],
    ) -> Result<Started<K>, ToolError> {
        let executable = self.options.executable(&build.browser)?;
        let engine = Engine {
            name: build.browser.name().to_owned(),
            version: browser_version(executable)?,
            settings: build.browser.settings(),
        };
        let old = self.old.filter(|_| !self.full).and_then(|record| {
            Some((
                record,
                record.builds.iter().find(|old| old.name == build.name)?,
            ))
        });
        let settings = old.is_some_and(|(record, measurement)| {
            self.same_settings(measurement, record, build, &engine, listings.len())
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
                            code: if same { code.clone() } else { None },
                            ..(*kept).clone()
                        },
                        // It holds no figure until a round measures it.
                        None => Recorded {
                            cell: cell(Outcome::Absent(NOT_MEASURED.to_owned())),
                            run: 0,
                            code: None,
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
                    measurement.overheads.clone()
                } else {
                    vec![None; listings.len()]
                };
                (measurement.runs.clone(), overheads)
            },
        );
        Ok(Started {
            measurement: Measurement {
                name: build.name.to_owned(),
                rustflags: FLAVOURS[build.flavour].rustflags.to_owned(),
                target_features: target_features.to_vec(),
                browser: Some(engine),
                overheads,
                harness: Some(harness()),
                runs,
                cells,
            },
            changed,
        })
    }

    /// The units of a round of `build` that measures `selected` among the cells of its `listings`:
    /// one per bench with a selected cell. A unit runs `--runs` times, or by default
    /// [`SELECTED_RUNS`] times, where one run of the whole bench costs more. That run takes the
    /// bench's other cells with it.
    fn units<'l>(
        &self,
        build: usize,
        round: usize,
        listings: &'l [Listing<K>],
        selected: &HashSet<CellId<K>>,
    ) -> Result<Vec<Unit<'l>>, ToolError>
    where
        'a: 'l,
    {
        let executable = self.options.executable(&WASM[build].browser)?;
        let mut units = Vec::new();
        for (bench, listing) in listings.iter().enumerate() {
            let patterns: Vec<String> = listing
                .cells
                .iter()
                .filter(|cell| matches!(cell.state, CellState::Case(_)))
                .filter(|cell| selected.contains(&cell.id))
                .map(|cell| cell.id.pattern())
                .collect();
            if patterns.is_empty() {
                continue;
            }
            let cases = listing.cases();
            let whole = patterns.len() == cases
                || (self.runs.is_none() && patterns.len() * SELECTED_RUNS >= cases);
            let (plan, cells, runs) = if whole {
                (Plan::All, cases, self.runs.unwrap_or(1))
            } else {
                let cells = patterns.len();
                (
                    Plan::Only(Selection::of(patterns)?),
                    cells,
                    self.runs.unwrap_or(SELECTED_RUNS),
                )
            };
            units.push(Unit {
                build,
                bench,
                name: &listing.bench,
                module: &listing.bytes,
                executable,
                plan,
                runs,
                cells,
                round,
            });
        }
        Ok(units)
    }
}

/// A sweep under way: the record so far, and the units that remain. The main thread owns it, and
/// the streams take its units.
struct Progress<'a, K> {
    context: &'a Context<'a, K>,
    /// The listings of each compilation.
    listings: &'a [Vec<Listing<K>>],
    /// Where the record goes.
    path: &'a Path,
    /// The record so far, with every build in the order of [`WASM`].
    record: Unfinished<K>,
    queue: VecDeque<Unit<'a>>,
    /// The build and the bench of each unit that a stream runs. A build runs one round at a time,
    /// which runs a bench once at most.
    running: Vec<(usize, usize)>,
    /// The cells of the units of this sweep that ended.
    measured: usize,
    /// The cells of each build whose key differs from the record and that no run of this sweep has
    /// measured yet.
    pending: Vec<HashSet<CellId<K>>>,
}

impl<'a, K: Key> Progress<'a, K> {
    /// The sweep of the `started` builds, with the units of their first rounds in the queue.
    fn new(
        context: &'a Context<'a, K>,
        listings: &'a [Vec<Listing<K>>],
        path: &'a Path,
        started: Vec<Started<K>>,
    ) -> Result<Self, ToolError> {
        let mut queue = VecDeque::new();
        let mut builds = Vec::with_capacity(started.len());
        let mut pending = Vec::with_capacity(started.len());
        for (index, started) in started.into_iter().enumerate() {
            let seeds = started.seeds(context.layout)?;
            if !seeds.is_empty() {
                queue.extend(Self::round_units(
                    context,
                    listings,
                    index,
                    0,
                    &started.measurement,
                    seeds,
                )?);
            }
            builds.push(started.measurement);
            pending.push(started.changed.into_iter().collect());
        }
        Ok(Self {
            context,
            listings,
            path,
            record: Record {
                builds,
                ..context.fresh.clone()
            },
            queue,
            running: Vec::new(),
            measured: 0,
            pending,
        })
    }

    /// The units of the round `round` of the build `build`, which `measurement` holds, given its
    /// `seeds`.
    fn round_units(
        context: &Context<'a, K>,
        listings: &'a [Vec<Listing<K>>],
        build: usize,
        round: usize,
        measurement: &Measurement<K, Option<Overhead>>,
        seeds: Vec<CellId<K>>,
    ) -> Result<Vec<Unit<'a>>, ToolError> {
        let listings = &listings[WASM[build].flavour];
        let selected = select(context.layout, &measurement.cells, seeds)?;
        let total = listings.iter().map(Listing::cases).sum::<usize>();
        println!(
            "{}: round {}: {} of {total} cells; load {}",
            WASM[build].name,
            round + 1,
            selected.len(),
            load()?
        );
        context.units(build, round, listings, &selected)
    }

    /// Runs the queue on `streams`, and writes the record after each unit. On the first failure it
    /// hands out no more units, and returns the failure once the running units end.
    fn run(&mut self, streams: &[Cpus]) -> Result<(), ToolError> {
        let mut free: Vec<Cpus> = streams.iter().rev().copied().collect();
        let mut failure = None;
        std::thread::scope(|scope| {
            let (sender, received) = mpsc::channel();
            loop {
                let starting = if failure.is_none() {
                    free.len().min(self.queue.len())
                } else {
                    0
                };
                let streams = free.split_off(free.len() - starting);
                for (unit, stream) in self.queue.drain(..starting).zip(streams.into_iter().rev()) {
                    self.running.push((unit.build, unit.bench));
                    let sender = sender.clone();
                    drop(scope.spawn(move || {
                        let result = panic::catch_unwind(AssertUnwindSafe(|| unit.measure(stream)))
                            .unwrap_or_else(|payload| Err(panicked(payload)));
                        if sender.send((unit, stream, result)).is_err() {
                            eprintln!("bench-wasm: a run ended after its sweep");
                        }
                    }));
                }
                if self.running.is_empty() {
                    return;
                }
                let Ok((unit, stream, result)) = received.recv() else {
                    failure = Some(ToolError::missing("the end of a run", "the streams"));
                    return;
                };
                free.push(stream);
                self.running
                    .retain(|running| *running != (unit.build, unit.bench));
                let ended = result.and_then(|(lines, sweep)| self.ended(&unit, lines, sweep));
                if let Err(error) = ended {
                    if failure.is_none() {
                        failure = Some(error);
                    } else {
                        println!(
                            "{}: {}: this run stopped too: {error}",
                            WASM[unit.build].name, unit.name
                        );
                    }
                }
            }
        });
        failure.map_or(Ok(()), Err)
    }

    /// Takes the figures of `unit`, its `lines` over the span `sweep`, into the record and writes
    /// it; queues the next round of its build where its round ended.
    fn ended(
        &mut self,
        unit: &Unit<'a>,
        lines: Vec<String>,
        sweep: Sweep,
    ) -> Result<(), ToolError> {
        let wasm = &WASM[unit.build];
        let listing = &self.listings[wasm.flavour][unit.bench];
        let counted = parse_lines::<K>(lines, unit.name)?;
        let fresh = self.context.fresh;
        let measurement = &mut self.record.builds[unit.build];
        let run = measurement.runs.len();
        measurement.runs.push(Run {
            commit: fresh.head.clone(),
            dirty: fresh.dirty,
            date: fresh.date.clone(),
            sweep,
        });
        let cells = counted
            .cells
            .into_iter()
            .map(|cell| {
                let code = listing.code(&CellId::of(&cell))?;
                Ok(Recorded { cell, run, code })
            })
            .collect::<Result<Vec<_>, ToolError>>()?;
        let ids: HashSet<CellId<K>> = cells.iter().map(|cell| CellId::of(&cell.cell)).collect();
        self.pending
            .get_mut(unit.build)
            .ok_or_else(|| ToolError::missing("the changed cells", wasm.name))?
            .retain(|id| !ids.contains(id));
        merge(&mut measurement.cells, cells)?;
        *measurement
            .overheads
            .get_mut(unit.bench)
            .ok_or_else(|| ToolError::missing("cost of a count", unit.name))? =
            Some(counted.overhead);
        let runs = std::mem::take(&mut measurement.runs);
        measurement.runs = compact(runs, &mut measurement.cells);
        self.measured += unit.cells;
        self.record.date = latest(&self.record.builds, &fresh.date);
        self.record.write_unfinished(self.path)?;
        let busy = self.running.iter().any(|(build, _)| *build == unit.build)
            || self.queue.iter().any(|queued| queued.build == unit.build);
        if busy {
            return Ok(());
        }
        let seeds = overtaken(self.context.layout, &self.record.builds[unit.build].cells)?;
        let Some(first) = seeds.first() else {
            return Ok(());
        };
        if unit.round + 1 == ROUNDS {
            return Err(ToolError::Unsettled {
                build: wasm.name.to_owned(),
                rows: seeds.len(),
                rounds: ROUNDS,
                first: first.pattern(),
            });
        }
        let units = Self::round_units(
            self.context,
            self.listings,
            unit.build,
            unit.round + 1,
            &self.record.builds[unit.build],
            seeds,
        )?;
        for unit in units.into_iter().rev() {
            self.queue.push_front(unit);
        }
        Ok(())
    }

    /// The finished record: every listed case of every build has figures of its code, from a run of
    /// this sweep or kept with an equal key, and every bench its cost of a count.
    fn finish(self, benches: &[&str]) -> Result<Record<K>, ToolError> {
        for (measurement, pending) in self.record.builds.iter().zip(&self.pending) {
            if let Some(id) = pending.iter().next() {
                return Err(ToolError::missing(
                    &format!("figures of {} at its code", id.pattern()),
                    &measurement.name,
                ));
            }
        }
        self.record.finished(benches)
    }
}

/// The date of the latest run of `builds`; `otherwise` where they hold no run.
fn latest<K, O>(builds: &[Measurement<K, O>], otherwise: &str) -> String {
    builds
        .iter()
        .flat_map(|build| &build.runs)
        .map(|run| run.date.clone())
        .max()
        .unwrap_or_else(|| otherwise.to_owned())
}

/// Prints what the first round of each of the `started` builds would measure, by name.
fn print_selection<K: Key>(
    layout: &Layout<'_, K>,
    started: &[Started<K>],
    listings: &[Vec<Listing<K>>],
) -> Result<(), ToolError> {
    for (build, started) in WASM.iter().zip(started) {
        let selected = select(layout, &started.measurement.cells, started.seeds(layout)?)?;
        let changed: HashSet<&CellId<K>> = started.changed.iter().collect();
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

/// Measures the WebAssembly builds of the bench and writes their record after each run of a bench in
/// a build. With `--changed` it measures the cells whose key differs from the record, what they are
/// compared with, and what overtakes those by a round of its own; with `--dry-run` it stops after
/// printing them. Without it, every cell. A run takes up to [`ATTEMPTS`](crate::run::ATTEMPTS);
/// past them the sweep stops with the record unfinished, and `--changed` measures the rest.
pub(crate) fn bench_wasm<K: Key>(
    paths: &Paths,
    measured: &Measured,
    layout: &Layout<'_, K>,
    options: &Options,
) -> Result<(), ToolError> {
    let chrome = options.executable(&CHROME)?;
    let streams = streams(options.cpu.as_deref(), options.browser_cpu.as_deref())?;
    let runs = parse_runs(options)?;
    let started = crate::run::now()?;
    let fresh: Unfinished<K> = crate::run::skeleton(paths, measured, options, started)?;
    if let Some(directory) = paths.wasm_record.parent() {
        fs::create_dir_all(directory).map_err(|source| ToolError::io(directory, source))?;
    }
    let load_before = load()?;
    let clock = Instant::now();
    let built = build_all(paths, measured, options.target_dir.as_deref())?;
    let building = clock.elapsed();
    let clock = Instant::now();
    let listings = list_all::<K>(measured, &built, chrome)?;
    let listing = clock.elapsed();
    let old = if options.changed && paths.wasm_record.exists() {
        Some(Unfinished::resume(&paths.wasm_record)?)
    } else {
        None
    };
    let context = Context {
        layout,
        options,
        runs,
        old: old.as_ref(),
        fresh: &fresh,
        full: !options.changed,
    };
    let started = WASM
        .iter()
        .map(|build| {
            context.start(
                build,
                &listings[build.flavour],
                &built[build.flavour].target_features,
            )
        })
        .collect::<Result<Vec<_>, ToolError>>()?;
    if options.dry_run {
        print_selection(layout, &started, &listings)?;
        println!(
            "built in {:.1} s, listed in {:.1} s; load {load_before} before, {} after",
            building.as_secs_f64(),
            listing.as_secs_f64(),
            load()?
        );
        return Ok(());
    }
    let clock = Instant::now();
    let mut progress = Progress::new(&context, &listings, &paths.wasm_record, started)?;
    let swept = progress.run(&streams);
    let running = clock.elapsed();
    let summary = format!(
        "built in {:.1} s, listed in {:.1} s, measured {} cells in {:.1} s on {} stream{}",
        building.as_secs_f64(),
        listing.as_secs_f64(),
        progress.measured,
        running.as_secs_f64(),
        streams.len(),
        if streams.len() == 1 { "" } else { "s" }
    );
    if let Err(error) = swept {
        println!(
            "{summary}, then stopped; {} holds what ended, and `bench-wasm --changed` measures the \
             rest; load {load_before} before, {} after",
            paths.wasm_record.display(),
            load()?
        );
        return Err(error);
    }
    progress
        .finish(measured.benches)?
        .write(&paths.wasm_record)?;
    println!("{summary}; load {load_before} before, {} after", load()?);
    Ok(())
}

/// Refuses a tree whose root manifests build `package` otherwise than `head` does: a `Cargo.toml` that
/// differs beyond the list of members, or a `Cargo.lock` where `package` reaches a package or a
/// version that it reaches nowhere at `head`.
fn check_manifests(paths: &Paths, package: &str, head: &str) -> Result<(), ToolError> {
    let refused = |reason: String| ToolError::Unstamped { reason };
    let read = |file: &str| {
        let path = paths.root.join(file);
        fs::read_to_string(&path).map_err(|source| ToolError::io(&path, source))
    };
    let without_members = |manifest: &str| -> Result<String, ToolError> {
        let (before, rest) = manifest
            .split_once("members = [")
            .ok_or_else(|| ToolError::missing("members", "Cargo.toml"))?;
        let (_, after) = rest
            .split_once(']')
            .ok_or_else(|| ToolError::missing("the end of members", "Cargo.toml"))?;
        Ok(format!("{before}{after}"))
    };
    let manifest = git(paths, &["show", &format!("{head}:Cargo.toml")], &[])? + "\n";
    if without_members(&manifest)? != without_members(&read("Cargo.toml")?)? {
        return Err(refused(format!(
            "Cargo.toml differs from {head} beyond its members"
        )));
    }
    let lock = git(paths, &["show", &format!("{head}:Cargo.lock")], &[])? + "\n";
    let locked = locked_closure(&lock, package)?;
    let mut added: Vec<String> = locked_closure(&read("Cargo.lock")?, package)?
        .difference(&locked)
        .map(|(name, version)| format!("{name} {version}"))
        .collect();
    added.sort_unstable();
    if added.is_empty() {
        Ok(())
    } else {
        Err(refused(format!(
            "{package} reaches packages that it reaches nowhere at {head}: {}",
            added.join(", ")
        )))
    }
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
    let compared: Vec<&str> = pathspecs
        .iter()
        .copied()
        .filter(|path| !["Cargo.toml", "Cargo.lock"].contains(path))
        .collect();
    let moved = git(
        paths,
        &["diff", "--name-only", record.head.as_str(), "--"],
        &compared,
    )?;
    if !moved.is_empty() {
        return Err(refused(format!(
            "these files differ from {}:\n{moved}",
            record.head
        )));
    }
    check_manifests(paths, measured.package, &record.head)?;
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
    fs::create_dir_all(out).map_err(|source| ToolError::io(out, source))?;
    for (flavour, built) in FLAVOURS.iter().zip(&built) {
        for (module, bench) in built.modules.iter().zip(measured.benches) {
            let target = out.join(format!("{bench}.{}.wasm", flavour.word));
            let copied =
                fs::copy(module, &target).map_err(|source| ToolError::io(&target, source))?;
            println!("{}: {copied} bytes", target.display());
        }
    }
    Ok(())
}
