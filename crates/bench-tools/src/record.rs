//! The run record: what `bench` measured and how, read back by `report`.

use std::fs;
use std::io::Write as _;
use std::path::Path;

use bench_compare::{Cell, Key, Outcome, Overhead, Sweep};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::tool_error::ToolError;

/// A crate and the version that `Cargo.lock` resolves.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Version {
    /// The crate.
    pub name: String,
    /// Its version.
    pub version: String,
}

/// Everything that `report` needs. `O` is the cost of a count of a bench: an [`Unfinished`] record
/// lacks those of the benches that no run reached yet.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Record<K, O = Overhead> {
    /// The UTC date of the run, `YYYY-MM-DD`.
    pub date: String,
    /// The commit that ran the bench.
    pub head: String,
    /// The last commit that changed the code of the measured library or of a workspace package it
    /// is built from.
    pub library_commit: String,
    /// Whether the measured files differed from `head`.
    pub dirty: bool,
    /// The first line of `rustc -vV`.
    pub rustc: String,
    /// The settings of the workspace's `[profile.bench]`, as `key = value` lines.
    pub profile: Vec<String>,
    /// The LLVM version of the compiler.
    pub llvm: String,
    /// The target triple of the host.
    pub host: String,
    /// The CPU that `-C target-cpu=native` selects.
    pub native_cpu: String,
    /// The CPU's model name in `/proc/cpuinfo`.
    pub cpu_model: String,
    /// The operator's description of the machine.
    pub machine: Option<String>,
    /// The measured package.
    pub package: String,
    /// The features of the measured package that every build enables beside its default ones.
    pub features: Vec<String>,
    /// The versions of the compared crates.
    pub versions: Vec<Version>,
    /// The measurement of every build of the bench.
    pub builds: Vec<Measurement<K, O>>,
}

/// The record that a WebAssembly sweep writes after each run of a bench in a build. `report` and
/// `stamp-wasm` refuse it, and `bench-wasm --changed` measures what it lacks.
pub type Unfinished<K> = Record<K, Option<Overhead>>;

/// One build of the bench, measured.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Measurement<K, O = Overhead> {
    /// The name of the build; empty for the only build of a bench.
    pub name: String,
    /// The `RUSTFLAGS` of the build.
    pub rustflags: String,
    /// The target features that `rustc --print cfg` reports under these flags.
    pub target_features: Vec<String>,
    /// The browser that ran a WebAssembly build; `None` for a native build.
    pub browser: Option<Engine>,
    /// The cost of a count in each bench of a WebAssembly build, in the order of the benches.
    #[serde(default = "Vec::new")]
    pub overheads: Vec<O>,
    /// The hash of the harness sources that measure, for a WebAssembly build with keys.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub harness: Option<String>,
    /// The runs that the cells come from.
    pub runs: Vec<Run>,
    /// Every cell of the comparison.
    pub cells: Vec<Recorded<K>>,
}

/// One run of a build, over all or some of its cells.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Run {
    /// The commit that ran it.
    pub commit: String,
    /// Whether the measured files differed from the commit.
    pub dirty: bool,
    /// The UTC date of the run, `YYYY-MM-DD`.
    pub date: String,
    /// Where it measured, and the load around it.
    pub sweep: Sweep,
}

/// A cell of a record, with where its figures come from.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Recorded<K> {
    /// The cell.
    #[serde(flatten)]
    pub cell: Cell<K>,
    /// The index in [`Measurement::runs`] of the run that counted it; 0 for an absent cell.
    #[serde(default)]
    pub run: usize,
    /// The hash of what its figures depend on in the module, in hexadecimal digits; `None` for a
    /// cell that always counts as changed: one without a key, or that reaches code which the hash
    /// cannot cover.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
}

impl<K> Recorded<K> {
    /// Whether the cell holds counts.
    pub const fn counted(&self) -> bool {
        matches!(self.cell.outcome, Outcome::Counted(_))
    }
}

/// The browser that ran a WebAssembly build.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Engine {
    /// The browser, such as `Chrome`.
    pub name: String,
    /// The version that the browser reports.
    pub version: String,
    /// The flags or preferences of the build, as the setup of the document shows them.
    pub settings: String,
}

/// The field that marks an [`Unfinished`] record.
const UNFINISHED: &str = "unfinished";

/// An [`Unfinished`] record as its file holds it.
#[derive(Serialize)]
struct Marked<'a, K> {
    unfinished: bool,
    #[serde(flatten)]
    record: &'a Unfinished<K>,
}

/// Gives a run to each build of the record `value` that names none: a sweep over all its cells,
/// which the head and the date of the record name.
fn with_runs(value: &mut Value) {
    let origin: Vec<(&str, Value)> = ["head", "dirty", "date"]
        .into_iter()
        .filter_map(|field| Some((field, value.get(field)?.clone())))
        .collect();
    let Some(builds) = value.get_mut("builds").and_then(Value::as_array_mut) else {
        return;
    };
    for build in builds.iter_mut().filter_map(Value::as_object_mut) {
        if build.contains_key("runs") {
            continue;
        }
        let Some(sweep) = build.remove("sweep") else {
            continue;
        };
        let mut run = serde_json::Map::new();
        for (field, origin) in &origin {
            let name = if *field == "head" { "commit" } else { field };
            *run.entry(name).or_insert(Value::Null) = origin.clone();
        }
        *run.entry("sweep").or_insert(Value::Null) = sweep;
        *build.entry("runs").or_insert(Value::Null) = Value::Array(vec![Value::Object(run)]);
    }
}

/// The JSON of the record at `path`, with a run for each build.
fn parsed(path: &Path) -> Result<Value, ToolError> {
    let text = fs::read_to_string(path).map_err(|source| ToolError::io(path, source))?;
    let mut value: Value =
        serde_json::from_str(&text).map_err(|source| ToolError::json(path, source))?;
    with_runs(&mut value);
    Ok(value)
}

/// Writes `value` to `path` as JSON in one step: a kill leaves the file as it was or as it is now.
fn write_json(value: &impl Serialize, path: &Path) -> Result<(), ToolError> {
    let text =
        serde_json::to_string_pretty(value).map_err(|source| ToolError::json(path, source))?;
    let temporary = path.with_extension("json.tmp");
    let mut file =
        fs::File::create(&temporary).map_err(|source| ToolError::io(&temporary, source))?;
    file.write_all((text + "\n").as_bytes())
        .and_then(|()| file.sync_all())
        .map_err(|source| ToolError::io(&temporary, source))?;
    drop(file);
    fs::rename(&temporary, path).map_err(|source| ToolError::io(path, source))
}

impl<K: Key> Record<K> {
    /// Reads the record at `path`, which a sweep finished.
    pub fn read(path: &Path) -> Result<Self, ToolError> {
        let value = parsed(path)?;
        if value.get(UNFINISHED).is_some() {
            return Err(ToolError::Unfinished {
                path: path.to_path_buf(),
            });
        }
        serde_json::from_value(value).map_err(|source| ToolError::json(path, source))
    }

    /// Writes the record to `path`.
    pub fn write(&self, path: &Path) -> Result<(), ToolError> {
        write_json(self, path)
    }
}

impl<K: Key> Unfinished<K> {
    /// Reads the record at `path`, finished or not, to go on from.
    pub fn resume(path: &Path) -> Result<Self, ToolError> {
        serde_json::from_value(parsed(path)?).map_err(|source| ToolError::json(path, source))
    }

    /// Writes the record to `path`, marked unfinished.
    pub fn write_unfinished(&self, path: &Path) -> Result<(), ToolError> {
        write_json(
            &Marked {
                unfinished: true,
                record: self,
            },
            path,
        )
    }
}

impl<K> Unfinished<K> {
    /// The record with the cost of a count of each of `benches` in every build, which it must hold.
    pub(crate) fn finished(self, benches: &[&str]) -> Result<Record<K>, ToolError> {
        Ok(Record {
            date: self.date,
            head: self.head,
            library_commit: self.library_commit,
            dirty: self.dirty,
            rustc: self.rustc,
            profile: self.profile,
            llvm: self.llvm,
            host: self.host,
            native_cpu: self.native_cpu,
            cpu_model: self.cpu_model,
            machine: self.machine,
            package: self.package,
            features: self.features,
            versions: self.versions,
            builds: self
                .builds
                .into_iter()
                .map(|build| build.finished(benches))
                .collect::<Result<_, _>>()?,
        })
    }
}

impl<K> Measurement<K, Option<Overhead>> {
    /// The measurement with the cost of a count of each of `benches`, which it must hold.
    pub(crate) fn finished(self, benches: &[&str]) -> Result<Measurement<K>, ToolError> {
        if self.overheads.len() != benches.len() {
            return Err(ToolError::missing(
                &format!("the cost of a count of each of {} benches", benches.len()),
                &self.name,
            ));
        }
        let overheads = self
            .overheads
            .into_iter()
            .zip(benches)
            .map(|(overhead, bench)| {
                overhead.ok_or_else(|| {
                    ToolError::missing("cost of a count", &format!("the bench {bench}"))
                })
            })
            .collect::<Result<Vec<_>, ToolError>>()?;
        Ok(Measurement {
            name: self.name,
            rustflags: self.rustflags,
            target_features: self.target_features,
            browser: self.browser,
            overheads,
            harness: self.harness,
            runs: self.runs,
            cells: self.cells,
        })
    }
}
