//! `bench`: builds the benches of a comparison for the native CPU, runs them, and records what the
//! CPU's counters counted. [`execute`] also runs the commands of [`crate::wasm`].

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

use bench_compare::{CPU, Cell, Error, Key, Line, Sweep, Window, load};
use serde::Deserialize;

use crate::browser::Browser;
use crate::record::{Measurement, Record, Recorded, Run, Version};
use crate::tool_error::ToolError;
use crate::wasm::{RUN_BENCH, bench_wasm, run_bench, stamp_wasm, write_modules};
use crate::workspace::Sources;

/// A build of the bench.
pub struct Build {
    /// The name of the build in the document; empty for the only build of a bench.
    pub name: &'static str,
    /// The suffix of the build's target directory; empty for the first build.
    pub slug: &'static str,
    /// The `RUSTFLAGS` of the build.
    pub rustflags: &'static str,
}

/// The build for the CPU that runs the bench. One codegen unit makes the code of each crate
/// independent of the rest of the bench: with sixteen, a change to the measured crate moved the
/// counts of others by up to 14%. Every loop starts on a 64-byte line, so a loop of up to 64 bytes
/// never straddles two: two identical one-`add` chain loops took 1.0 and 2.0 cycles per step where
/// the second's branch crossed a line.
pub const NATIVE: Build = Build {
    name: "",
    slug: "",
    rustflags: "-C target-cpu=native -C codegen-units=1 -C llvm-args=-align-loops=64",
};

/// The CPUs of the machine.
const CPUINFO: &str = "/proc/cpuinfo";

/// The attempts of one run of a bench: load can stop a bench, and another attempt meets other load.
/// Past them the sweep stops.
pub(crate) const ATTEMPTS: usize = 3;

/// What `attempt` returns, in up to [`ATTEMPTS`] attempts of the run `run`. Each refused attempt
/// before the last prints the run, the load and the error.
pub(crate) fn attempts<T>(
    run: &str,
    mut attempt: impl FnMut() -> Result<T, ToolError>,
) -> Result<T, ToolError> {
    let mut number = 1;
    loop {
        match attempt() {
            Err(error) if number < ATTEMPTS => {
                println!(
                    "{run}: attempt {number} of {ATTEMPTS} refused, load {}: {error}",
                    load()?
                );
                number += 1;
            }
            Err(error) => return Err(error),
            Ok(value) => return Ok(value),
        }
    }
}

/// The locations that a report tool reads and writes.
pub struct Paths {
    /// The workspace root.
    pub root: PathBuf,
    /// The crate whose benchmarks the document describes.
    pub library: PathBuf,
    /// The run record of the native builds.
    pub record: PathBuf,
    /// The run record of the WebAssembly builds.
    pub wasm_record: PathBuf,
}

impl Paths {
    /// The paths of the tool at `manifest_dir`, which lives in the library's directory; the
    /// records are `benchmarks/record.json` and `benchmarks/record-wasm.json` there.
    #[must_use]
    pub fn new(manifest_dir: &str) -> Self {
        let library = Path::new(manifest_dir).join("..");
        let root = library.join("..").join("..");
        Self {
            record: library.join("benchmarks").join("record.json"),
            wasm_record: library.join("benchmarks").join("record-wasm.json"),
            root,
            library,
        }
    }

    /// The target directory of `build`.
    fn results(&self, build: &Build) -> PathBuf {
        let name = if build.slug.is_empty() {
            "benchmarks".to_owned()
        } else {
            format!("benchmarks-{}", build.slug)
        };
        self.root.join("target").join(name)
    }
}

/// What a bench measures: the package and its bench, the files that the measured executable is
/// built from, and the compared crates.
pub struct Measured {
    /// The package that holds the bench.
    pub package: &'static str,
    /// The bench targets, which measure disjoint operations, in measurement order. Each compiles
    /// one part of the comparison, so the targets build in parallel.
    pub benches: &'static [&'static str],
    /// The crates whose versions the record keeps.
    pub versions: &'static [&'static str],
    /// The features of the package that every build enables beside its default ones.
    pub features: &'static [&'static str],
    /// The features that no crate of a build may have.
    pub forbidden: &'static [&'static str],
    /// The builds of the bench, measured one after the other.
    pub builds: &'static [Build],
}

/// The options of `bench` and `bench-wasm`.
#[derive(Default)]
pub struct Options {
    /// The CPU of the measuring thread; the bench's own default when absent. `bench-wasm` takes a
    /// list, separated by commas: one stream per CPU.
    pub cpu: Option<String>,
    /// The operator's description of the machine.
    pub machine: Option<String>,
    /// The CPU of the browser's other threads, one per stream; the measuring CPU when absent.
    pub browser_cpu: Option<String>,
    /// The executable of Chrome or Chromium.
    pub chrome: Option<PathBuf>,
    /// The executable of Firefox.
    pub firefox: Option<PathBuf>,
    /// Whether `bench-wasm` measures only what changed.
    pub changed: bool,
    /// Whether `bench-wasm --changed` stops after it names what it would measure.
    pub dry_run: bool,
    /// The runs of a bench, whose fewest figures count.
    pub runs: Option<String>,
    /// The directory of the targets of the WebAssembly builds, for `target` of the workspace.
    pub target_dir: Option<PathBuf>,
}

impl Options {
    /// The options of `args`: `--cpu <cpu>` and `--machine <description>`, and with `browsers`
    /// `--browser-cpu <cpu>`, `--chrome <executable>`, `--firefox <executable>`, `--runs <runs>`,
    /// `--target-dir <directory>`, `--changed` and `--dry-run`, each at most once.
    fn parse(args: &[String], browsers: bool) -> Option<Self> {
        match args {
            [] => Some(Self::default()),
            [flag, rest @ ..] if browsers && flag == "--changed" => {
                let options = Self::parse(rest, browsers)?;
                (!options.changed).then_some(Self {
                    changed: true,
                    ..options
                })
            }
            [flag, rest @ ..] if browsers && flag == "--dry-run" => {
                let options = Self::parse(rest, browsers)?;
                (!options.dry_run).then_some(Self {
                    dry_run: true,
                    ..options
                })
            }
            [flag, value, rest @ ..] => {
                let options = Self::parse(rest, browsers)?;
                match flag.as_str() {
                    "--cpu" if options.cpu.is_none() => Some(Self {
                        cpu: Some(value.clone()),
                        ..options
                    }),
                    "--machine" if options.machine.is_none() => Some(Self {
                        machine: Some(value.clone()),
                        ..options
                    }),
                    "--browser-cpu" if browsers && options.browser_cpu.is_none() => Some(Self {
                        browser_cpu: Some(value.clone()),
                        ..options
                    }),
                    "--chrome" if browsers && options.chrome.is_none() => Some(Self {
                        chrome: Some(PathBuf::from(value)),
                        ..options
                    }),
                    "--firefox" if browsers && options.firefox.is_none() => Some(Self {
                        firefox: Some(PathBuf::from(value)),
                        ..options
                    }),
                    "--runs" if browsers && options.runs.is_none() => Some(Self {
                        runs: Some(value.clone()),
                        ..options
                    }),
                    "--target-dir" if browsers && options.target_dir.is_none() => Some(Self {
                        target_dir: Some(PathBuf::from(value)),
                        ..options
                    }),
                    _ => None,
                }
            }
            [_] => None,
        }
    }

    /// The executable of `browser`, which the options must name.
    pub(crate) fn executable(&self, browser: &Browser) -> Result<&Path, ToolError> {
        let (executable, flag) = match browser {
            Browser::Chrome { .. } => (&self.chrome, "--chrome"),
            Browser::Firefox { .. } => (&self.firefox, "--firefox"),
        };
        executable
            .as_deref()
            .ok_or_else(|| ToolError::missing(&format!("{flag} <executable>"), "the options"))
    }
}

/// Runs the report tool `tool` with the process arguments:
/// - `bench [--cpu <cpu>] [--machine <description>]` measures the native builds and records them;
/// - `bench-wasm --chrome <executable> --firefox <executable> [--changed [--dry-run]] [--runs <runs>]
///   [--target-dir <directory>] [--cpu <cpu>[,<cpu>...]] [--browser-cpu <cpu>[,<cpu>...]]
///   [--machine <description>]` measures the WebAssembly builds and records them, on one stream per
///   measuring CPU;
/// - `stamp-wasm --chrome <executable> [--target-dir <directory>]` writes the keys of the tree into
///   the WebAssembly record;
/// - `modules --out <directory> [--target-dir <directory>]` builds the modules for
///   `bench-browser decide`;
/// - `report` writes the document from the records.
#[must_use]
pub fn execute<K: Key>(
    tool: &str,
    paths: &Paths,
    measured: &Measured,
    layout: &crate::document::Layout<'_, K>,
) -> std::process::ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let usage = || ToolError::Usage {
        tool: tool.to_owned(),
    };
    let result = match args.as_slice() {
        [command, options @ ..] if command == "bench" => Options::parse(options, false)
            .ok_or_else(usage)
            .and_then(|options| {
                bench::<K>(paths, measured, measured.builds, &paths.record, options)
            }),
        [command, options @ ..] if command == "bench-wasm" => Options::parse(options, true)
            .ok_or_else(usage)
            .and_then(|options| bench_wasm::<K>(paths, measured, layout, &options)),
        [command, options @ ..] if command == "stamp-wasm" => Options::parse(options, true)
            .ok_or_else(usage)
            .and_then(|options| stamp_wasm::<K>(paths, measured, &options)),
        [command, flag, out, options @ ..] if command == "modules" && flag == "--out" => {
            Options::parse(options, true)
                .ok_or_else(usage)
                .and_then(|options| write_modules(paths, measured, &options, Path::new(out)))
        }
        [command] if command == "report" => report_records(paths, layout),
        [command, arguments @ ..] if command == RUN_BENCH => run_bench(arguments),
        _ => Err(usage()),
    };
    match result {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{tool}: {error}");
            std::process::ExitCode::FAILURE
        }
    }
}

/// Writes the document from the records.
fn report_records<K: Key>(
    paths: &Paths,
    layout: &crate::document::Layout<'_, K>,
) -> Result<(), ToolError> {
    let record = Record::read(&paths.record)?;
    let wasm = if paths.wasm_record.exists() {
        Some(Record::read(&paths.wasm_record)?)
    } else {
        None
    };
    crate::document::report(paths, layout, &record, wasm.as_ref())
}

/// The fields of a record of a run that starts at `started` seconds after the Unix epoch, with no
/// build yet.
pub(crate) fn skeleton<K: Key, O>(
    paths: &Paths,
    measured: &Measured,
    options: &Options,
    started: u64,
) -> Result<Record<K, O>, ToolError> {
    let rustc = output(Command::new("rustc").arg("-vV").current_dir(&paths.root))?;
    let target_cpus = output(
        Command::new("rustc")
            .args(["--print", "target-cpus"])
            .current_dir(&paths.root),
    )?;
    let sources = Sources::of(paths, measured.package)?;
    let status_lines = git(
        paths,
        &["status", "--porcelain", "--"],
        &sources.executable(),
    )?;
    Ok(Record {
        date: date(started),
        head: git(paths, &["rev-parse", "HEAD"], &[])?,
        library_commit: git(
            paths,
            &["log", "-1", "--format=%H", "--"],
            &sources.library(),
        )?,
        dirty: !status_lines.is_empty(),
        rustc: rustc
            .lines()
            .next()
            .ok_or_else(|| ToolError::missing("version", "rustc -vV"))?
            .to_owned(),
        profile: bench_profile(paths)?,
        llvm: field(&rustc, "LLVM version: ", "rustc -vV")?,
        host: field(&rustc, "host: ", "rustc -vV")?,
        native_cpu: native_cpu(&target_cpus)?,
        cpu_model: cpu_model()?,
        machine: options.machine.clone(),
        package: measured.package.to_owned(),
        features: measured
            .features
            .iter()
            .map(|&feature| feature.to_owned())
            .collect(),
        versions: versions(paths, measured.package, measured.versions)?,
        builds: Vec::new(),
    })
}

/// Measures every case of the bench in each of the native `builds` and writes the record at
/// `record`.
pub(crate) fn bench<K: Key>(
    paths: &Paths,
    measured: &Measured,
    builds: &[Build],
    record: &Path,
    options: Options,
) -> Result<(), ToolError> {
    let started = now()?;
    let fresh: Record<K> = skeleton(paths, measured, &options, started)?;
    let measurements = builds
        .iter()
        .map(|build| {
            let counted = cells(paths, measured, build, &options)?;
            Ok(Measurement {
                name: build.name.to_owned(),
                rustflags: build.rustflags.to_owned(),
                target_features: counted.target_features,
                browser: None,
                overheads: Vec::new(),
                harness: None,
                runs: vec![Run {
                    commit: fresh.head.clone(),
                    dirty: fresh.dirty,
                    date: fresh.date.clone(),
                    sweep: counted.sweep,
                }],
                cells: counted
                    .cells
                    .into_iter()
                    .map(|cell| Recorded {
                        cell,
                        run: 0,
                        code: None,
                    })
                    .collect(),
            })
        })
        .collect::<Result<Vec<_>, ToolError>>()?;
    let written = Record {
        builds: measurements,
        ..fresh
    };
    if let Some(directory) = record.parent() {
        fs::create_dir_all(directory).map_err(|source| ToolError::io(directory, source))?;
    }
    written.write(record)
}

/// Every cell of the comparison in the native `build` with its counts, and the sweep: the lines
/// that the benches print, measuring on the CPU of `options`, the bench's default CPU when absent.
/// The benches are built first, in parallel, and the features of the build must pass
/// [`check_features`]. They then run one after the other, in the order of
/// [`Measured::benches`], each in up to [`ATTEMPTS`] attempts; the sweep spans them all.
fn cells<K: Key>(
    paths: &Paths,
    measured: &Measured,
    build: &Build,
    options: &Options,
) -> Result<Counted<K>, ToolError> {
    let features = measured.features.join(",");
    let mut args = vec!["bench", "--locked", "-p", measured.package];
    if !features.is_empty() {
        args.extend(["--features", &features]);
    }
    let mut build_args = args.clone();
    for bench in measured.benches {
        build_args.extend(["--bench", bench]);
    }
    build_args.extend(["--no-run", "--message-format=json"]);
    let messages = cargo(
        &paths.root,
        &paths.results(build),
        build.rustflags,
        &build_args,
    )?;
    drop(check_features(measured, &messages)?);
    let cpu = options.cpu.as_deref();
    let cpu_number = parse_cpu(cpu)?.unwrap_or(CPU);
    let target_features = target_features(paths, build)?;
    let window = Window::open(cpu_number)?;
    let mut cells = Vec::new();
    for bench in measured.benches {
        let mut run_args = args.clone();
        run_args.extend(["--bench", bench]);
        if let Some(cpu) = cpu {
            run_args.extend(["--", "--cpu", cpu]);
        }
        let run = if build.name.is_empty() {
            (*bench).to_owned()
        } else {
            format!("{}: {bench}", build.name)
        };
        let text = attempts(&run, || {
            cargo(
                &paths.root,
                &paths.results(build),
                build.rustflags,
                &run_args,
            )
        })?;
        let mut sweeps = Vec::new();
        for line in text.lines().filter(|line| !line.trim().is_empty()) {
            match serde_json::from_str(line).map_err(Error::Cells)? {
                Line::Cell(cell) => cells.push(cell),
                Line::Sweep(sweep) => sweeps.push(sweep),
                Line::Overhead(_) => {
                    return Err(ToolError::missing(
                        "cells and a sweep, where it sent the cost of a count",
                        &format!("the lines of the bench {bench}"),
                    ));
                }
            }
        }
        match <[Sweep; 1]>::try_from(sweeps) {
            Ok([sweep]) if sweep.cpu == cpu_number => {}
            Ok([sweep]) => {
                return Err(ToolError::missing(
                    &format!(
                        "sweep on CPU {cpu_number}, where it ran on CPU {}",
                        sweep.cpu
                    ),
                    &format!("the lines of the bench {bench}"),
                ));
            }
            Err(sweeps) => {
                return Err(ToolError::missing(
                    &format!("single sweep among its {}", sweeps.len()),
                    &format!("the lines of the bench {bench}"),
                ));
            }
        }
    }
    Ok(Counted {
        cells,
        sweep: window.close()?,
        target_features,
    })
}

/// One JSON message of a cargo build: the fields that [`check_features`] reads.
#[derive(Deserialize)]
struct Message {
    reason: String,
    package_id: Option<String>,
    target: Option<Target>,
    #[serde(default)]
    features: Vec<String>,
    /// The executable of a bench, a `.wasm` module for a WebAssembly build.
    executable: Option<PathBuf>,
}

/// The target of a compiled unit.
#[derive(Deserialize)]
struct Target {
    name: String,
    kind: Vec<String>,
}

/// Checks the units of a build from its JSON `messages`: every bench of `measured` has every
/// feature in [`Measured::features`], and no unit has a feature in [`Measured::forbidden`].
/// Returns the executable of every bench, in the order of [`Measured::benches`].
pub(crate) fn check_features(
    measured: &Measured,
    messages: &str,
) -> Result<Vec<PathBuf>, ToolError> {
    let mut benches = Vec::new();
    for line in messages.lines().filter(|line| !line.trim().is_empty()) {
        let message: Message = serde_json::from_str(line).map_err(ToolError::Messages)?;
        if message.reason != "compiler-artifact" {
            continue;
        }
        let unit = message
            .package_id
            .ok_or_else(|| ToolError::missing("package_id", "a compiler-artifact message"))?;
        if let Some(feature) = message
            .features
            .iter()
            .find(|feature| measured.forbidden.contains(&feature.as_str()))
        {
            return Err(ToolError::Forbidden {
                unit,
                feature: feature.clone(),
            });
        }
        if let Some(target) = message
            .target
            .filter(|target| target.kind.iter().any(|kind| kind == "bench"))
        {
            benches.push((target.name, message.features, message.executable));
        }
    }
    let mut executables = Vec::with_capacity(measured.benches.len());
    for bench in measured.benches {
        let (_, built, executable) = benches
            .iter()
            .find(|(name, _, _)| name == bench)
            .ok_or_else(|| {
                ToolError::missing(&format!("bench {bench}"), "the messages of the build")
            })?;
        if let Some(feature) = measured
            .features
            .iter()
            .find(|feature| !built.iter().any(|built| built == *feature))
        {
            return Err(ToolError::Unbuilt {
                package: measured.package.to_owned(),
                feature: (*feature).to_owned(),
            });
        }
        executables.push(executable.clone().ok_or_else(|| {
            ToolError::missing(
                &format!("executable of the bench {bench}"),
                "the messages of the build",
            )
        })?);
    }
    Ok(executables)
}

/// The CPU that `cpu` names, if any.
pub(crate) fn parse_cpu(cpu: Option<&str>) -> Result<Option<usize>, ToolError> {
    cpu.map(|cpu| {
        cpu.parse::<usize>()
            .map_err(|source| ToolError::number("CPU", cpu, source))
    })
    .transpose()
}

/// What the benches of one build counted.
struct Counted<K> {
    /// Every cell of the comparison.
    cells: Vec<Cell<K>>,
    /// The sweep that spans the benches.
    sweep: Sweep,
    /// The target features of the build, of `rustc --print cfg`.
    target_features: Vec<String>,
}

/// The standard output of `cargo` with `args` in the workspace at `root`, which must succeed. The
/// build takes exactly `rustflags`, whatever the environment and the configuration set, and keeps its
/// artifacts in `target_dir`. `CARGO` names the cargo that runs this tool.
pub(crate) fn cargo(
    root: &Path,
    target_dir: &Path,
    rustflags: &str,
    args: &[&str],
) -> Result<String, ToolError> {
    output(
        Command::new(std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into()))
            .args(args)
            .current_dir(root)
            .env("CARGO_TARGET_DIR", target_dir)
            .env("RUSTFLAGS", rustflags)
            .env_remove("CARGO_ENCODED_RUSTFLAGS"),
    )
}

/// The target features that `rustc --print cfg` reports under the flags of the native `build`. A
/// build without AVX2 is a [`ToolError::NoAvx2`].
fn target_features(paths: &Paths, build: &Build) -> Result<Vec<String>, ToolError> {
    let text = output(
        Command::new("rustc")
            .args(["--print", "cfg"])
            .args(build.rustflags.split_whitespace())
            .current_dir(&paths.root),
    )?;
    let features: Vec<String> = text
        .lines()
        .filter_map(|line| line.strip_prefix("target_feature=\""))
        .filter_map(|rest| rest.strip_suffix('"'))
        .map(str::to_owned)
        .collect();
    if features.iter().any(|feature| feature == "avx2") {
        Ok(features)
    } else {
        Err(ToolError::NoAvx2 {
            rustflags: build.rustflags.to_owned(),
        })
    }
}

/// The `key = value` lines of the `[profile.bench]` section of the workspace's `Cargo.toml`. A
/// profile without `lto = "fat"` is an [`ToolError::NoFatLto`].
fn bench_profile(paths: &Paths) -> Result<Vec<String>, ToolError> {
    let path = paths.root.join("Cargo.toml");
    let manifest = fs::read_to_string(&path).map_err(|source| ToolError::io(&path, source))?;
    let settings: Vec<String> = manifest
        .lines()
        .skip_while(|line| line.trim() != "[profile.bench]")
        .skip(1)
        .map(str::trim)
        .take_while(|line| !line.starts_with('['))
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(str::to_owned)
        .collect();
    if settings.iter().any(|line| line == "lto = \"fat\"") {
        Ok(settings)
    } else {
        Err(ToolError::NoFatLto { manifest: path })
    }
}

/// The model name of the machine's CPU.
pub(crate) fn cpu_model() -> Result<String, ToolError> {
    let path = Path::new(CPUINFO);
    let text = fs::read_to_string(path).map_err(|source| ToolError::io(path, source))?;
    text.lines()
        .find_map(|line| line.strip_prefix("model name"))
        .and_then(|rest| rest.split_once(':'))
        .map(|(_, name)| name.trim().to_owned())
        .ok_or_else(|| ToolError::missing("model name", CPUINFO))
}

/// The standard output of `command`, which must succeed. Its standard error goes to the terminal.
pub(crate) fn output(command: &mut Command) -> Result<String, ToolError> {
    let line = format!("{command:?}");
    let output = command
        .stderr(Stdio::inherit())
        .output()
        .map_err(|source| ToolError::Spawn {
            command: line.clone(),
            source,
        })?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    } else {
        Err(ToolError::Failed {
            command: line,
            status: output.status,
        })
    }
}

/// The output of `git` with `args` and then `pathspecs`, without its trailing newline.
pub(crate) fn git(paths: &Paths, args: &[&str], pathspecs: &[&str]) -> Result<String, ToolError> {
    let text = output(
        Command::new("git")
            .arg("-C")
            .arg(&paths.root)
            .args(args)
            .args(pathspecs),
    )?;
    Ok(text.trim_end().to_owned())
}

/// The rest of the line of `text` that starts with `prefix`.
fn field(text: &str, prefix: &str, origin: &str) -> Result<String, ToolError> {
    text.lines()
        .find_map(|line| line.strip_prefix(prefix))
        .map(str::to_owned)
        .ok_or_else(|| ToolError::missing(prefix.trim_end_matches([':', ' ']), origin))
}

/// The CPU that `rustc --print target-cpus` names as the host's: `(currently <cpu>)`.
fn native_cpu(target_cpus: &str) -> Result<String, ToolError> {
    target_cpus
        .split("(currently ")
        .nth(1)
        .and_then(|rest| rest.split(')').next())
        .map(str::to_owned)
        .ok_or_else(|| ToolError::missing("native CPU", "rustc --print target-cpus"))
}

/// The versions of `wanted` in `Cargo.lock`.
fn versions(paths: &Paths, package: &str, wanted: &[&str]) -> Result<Vec<Version>, ToolError> {
    let path = paths.root.join("Cargo.lock");
    let lock = fs::read_to_string(&path).map_err(|source| ToolError::io(&path, source))?;
    let packages: Vec<Locked<'_>> = lock
        .split("[[package]]")
        .filter_map(Locked::parse)
        .collect();
    let measured = packages
        .iter()
        .find(|locked| locked.name == package)
        .ok_or_else(|| ToolError::missing(package, "Cargo.lock"))?;
    wanted
        .iter()
        .map(|wanted| {
            let entry = measured
                .dependencies
                .iter()
                .find(|entry| entry.split(' ').next() == Some(*wanted))
                .ok_or_else(|| {
                    ToolError::missing(&format!("dependency {wanted} of {package}"), "Cargo.lock")
                })?;
            Ok(Version {
                name: (*wanted).to_owned(),
                version: resolved(&packages, entry)?.version.to_owned(),
            })
        })
        .collect()
}

/// The package that the dependency `entry` of a lock names. The entry names a version where the lock
/// holds several of the crate.
fn resolved<'l, 'a>(packages: &'l [Locked<'a>], entry: &str) -> Result<&'l Locked<'a>, ToolError> {
    let mut parts = entry.split(' ');
    let name = parts.next();
    let version = parts.next();
    let mut named = packages.iter().filter(|locked| {
        Some(locked.name) == name && version.is_none_or(|version| version == locked.version)
    });
    match (named.next(), named.next()) {
        (Some(locked), None) => Ok(locked),
        (Some(_), Some(_)) | (None, _) => Err(ToolError::missing(
            &format!("single version of {entry}"),
            "Cargo.lock",
        )),
    }
}

/// The name and the version of `package` and of every package it reaches in the text `lock` of a
/// `Cargo.lock`.
pub(crate) fn locked_closure(
    lock: &str,
    package: &str,
) -> Result<HashSet<(String, String)>, ToolError> {
    let packages: Vec<Locked<'_>> = lock
        .split("[[package]]")
        .filter_map(Locked::parse)
        .collect();
    let mut queue = vec![resolved(&packages, package)?];
    let mut reached = HashSet::new();
    while let Some(locked) = queue.pop() {
        if reached.insert((locked.name.to_owned(), locked.version.to_owned())) {
            for entry in &locked.dependencies {
                queue.push(resolved(&packages, entry)?);
            }
        }
    }
    Ok(reached)
}

/// A `[[package]]` entry of `Cargo.lock`: its name, its version, and its dependencies as the lock
/// writes them, `name` or `name version`.
struct Locked<'a> {
    name: &'a str,
    version: &'a str,
    dependencies: Vec<&'a str>,
}

impl<'a> Locked<'a> {
    fn parse(entry: &'a str) -> Option<Self> {
        let dependencies = entry
            .split_once("dependencies = [")
            .and_then(|(_, rest)| rest.split_once(']'))
            .map(|(list, _)| {
                list.lines()
                    .map(|line| line.trim().trim_end_matches(',').trim_matches('"'))
                    .filter(|line| !line.is_empty())
                    .collect()
            })
            .unwrap_or_default();
        Some(Self {
            name: field_value(entry, "name = ")?,
            version: field_value(entry, "version = ")?,
            dependencies,
        })
    }
}

/// The quoted value of the line of `package` that starts with `prefix`.
fn field_value<'a>(package: &'a str, prefix: &str) -> Option<&'a str> {
    package
        .lines()
        .find_map(|line| line.strip_prefix(prefix))
        .map(|value| value.trim_matches('"'))
}

/// Seconds since the Unix epoch.
pub(crate) fn now() -> Result<u64, ToolError> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs())
        .map_err(ToolError::Clock)
}

/// The UTC date of `seconds` after the Unix epoch, as `YYYY-MM-DD`, by the civil-from-days
/// algorithm of Howard Hinnant.
fn date(seconds: u64) -> String {
    let days = seconds / 86_400 + 719_468;
    let era = days / 146_097;
    let day_of_era = days % 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let shifted_month = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * shifted_month + 2) / 5 + 1;
    let month = if shifted_month < 10 {
        shifted_month + 3
    } else {
        shifted_month - 9
    };
    let year = year_of_era + era * 400 + u64::from(month <= 2);
    format!("{year:04}-{month:02}-{day:02}")
}
