//! `bench`: builds a `compare` bench for the native CPU, runs it, and records what the CPU's
//! counters counted.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Deserialize;

use crate::error::Error;
use crate::record::{Cell, Key, Line, Measurement, Record, Sweep, Version};

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
/// counts of others by up to 14%.
pub const NATIVE: Build = Build {
    name: "",
    slug: "",
    rustflags: "-C target-cpu=native -C codegen-units=1",
};

/// The CPUs of the machine.
const CPUINFO: &str = "/proc/cpuinfo";

/// The locations that a report tool reads and writes.
pub struct Paths {
    /// The workspace root.
    pub root: PathBuf,
    /// The crate whose benchmarks the document describes.
    pub library: PathBuf,
    /// The run record.
    pub record: PathBuf,
}

impl Paths {
    /// The paths of the tool at `manifest_dir`, which lives in the library's directory; the
    /// record is `benchmarks/record.json` there.
    #[must_use]
    pub fn new(manifest_dir: &str) -> Self {
        let library = Path::new(manifest_dir).join("..");
        let root = library.join("..").join("..");
        Self {
            record: library.join("benchmarks").join("record.json"),
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
    /// The bench target.
    pub bench: &'static str,
    /// The sources of the measured library, whose last commit the record names.
    pub sources: &'static str,
    /// The files that the measured executable is built from, as git pathspecs.
    pub pathspecs: &'static [&'static str],
    /// The crates whose versions the record keeps.
    pub versions: &'static [&'static str],
    /// The features of the package that every build enables beside its default ones.
    pub features: &'static [&'static str],
    /// The features that no crate of a build may have.
    pub forbidden: &'static [&'static str],
    /// The builds of the bench, measured one after the other.
    pub builds: &'static [Build],
}

/// The options of `bench`.
#[derive(Default)]
pub struct Options {
    /// The CPU of the measuring thread; the bench's own default when absent.
    pub cpu: Option<String>,
    /// The operator's description of the machine.
    pub machine: Option<String>,
}

impl Options {
    /// The options of `args`: `--cpu <cpu>` and `--machine <description>`, each at most once.
    fn parse(args: &[String]) -> Option<Self> {
        match args {
            [] => Some(Self::default()),
            [flag, value, rest @ ..] => {
                let options = Self::parse(rest)?;
                match flag.as_str() {
                    "--cpu" if options.cpu.is_none() => Some(Self {
                        cpu: Some(value.clone()),
                        ..options
                    }),
                    "--machine" if options.machine.is_none() => Some(Self {
                        machine: Some(value.clone()),
                        ..options
                    }),
                    _ => None,
                }
            }
            [_] => None,
        }
    }
}

/// Runs the report tool `tool` with the process arguments: `bench [--cpu <cpu>] [--machine
/// <description>]` measures and records; `report` writes the document from the record.
#[must_use]
pub fn execute<K: Key>(
    tool: &str,
    paths: &Paths,
    measured: &Measured,
    layout: &crate::document::Layout<'_, K>,
) -> std::process::ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let usage = || Error::Usage {
        tool: tool.to_owned(),
    };
    let result = match args.as_slice() {
        [command, options @ ..] if command == "bench" => Options::parse(options)
            .ok_or_else(usage)
            .and_then(|options| bench::<K>(paths, measured, options)),
        [command] if command == "report" => Record::read(&paths.record)
            .and_then(|record| crate::document::report(paths, layout, &record)),
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

/// Measures every case of the bench and writes the record.
pub fn bench<K: Key>(paths: &Paths, measured: &Measured, options: Options) -> Result<(), Error> {
    let started = now()?;
    let rustc = output(Command::new("rustc").arg("-vV").current_dir(&paths.root))?;
    let target_cpus = output(
        Command::new("rustc")
            .args(["--print", "target-cpus"])
            .current_dir(&paths.root),
    )?;
    let status_lines = git(paths, &["status", "--porcelain", "--"], measured.pathspecs)?;
    let builds = measured
        .builds
        .iter()
        .map(|build| {
            let target_features = target_features(paths, build)?;
            let (cells, sweep) = cells(paths, measured, build, options.cpu.as_deref())?;
            Ok(Measurement {
                name: build.name.to_owned(),
                rustflags: build.rustflags.to_owned(),
                target_features,
                sweep,
                cells,
            })
        })
        .collect::<Result<Vec<_>, Error>>()?;
    let record = Record::<K> {
        date: date(started),
        head: git(paths, &["rev-parse", "HEAD"], &[])?,
        library_commit: git(
            paths,
            &["log", "-1", "--format=%H", "--", measured.sources],
            &[],
        )?,
        dirty: !status_lines.is_empty(),
        rustc: rustc
            .lines()
            .next()
            .ok_or_else(|| Error::missing("version", "rustc -vV"))?
            .to_owned(),
        llvm: field(&rustc, "LLVM version: ", "rustc -vV")?,
        host: field(&rustc, "host: ", "rustc -vV")?,
        native_cpu: native_cpu(&target_cpus)?,
        cpu_model: cpu_model()?,
        machine: options.machine,
        package: measured.package.to_owned(),
        features: measured
            .features
            .iter()
            .map(|&feature| feature.to_owned())
            .collect(),
        versions: versions(paths, measured.package, measured.versions)?,
        builds,
    };
    if let Some(directory) = paths.record.parent() {
        fs::create_dir_all(directory).map_err(|source| Error::io(directory, source))?;
    }
    record.write(&paths.record)
}

/// Every cell of the comparison in `build` with its counts, and the sweep: the lines that the
/// bench prints, measuring on `cpu`. The bench is built first, and the features of its build
/// must pass [`check_features`].
fn cells<K: Key>(
    paths: &Paths,
    measured: &Measured,
    build: &Build,
    cpu: Option<&str>,
) -> Result<(Vec<Cell<K>>, Sweep), Error> {
    let features = measured.features.join(",");
    let mut args = vec![
        "bench",
        "--locked",
        "-p",
        measured.package,
        "--bench",
        measured.bench,
    ];
    if !features.is_empty() {
        args.extend(["--features", &features]);
    }
    let mut build_args = args.clone();
    build_args.extend(["--no-run", "--message-format=json"]);
    let messages = cargo(
        &paths.root,
        &paths.results(build),
        build.rustflags,
        &build_args,
    )?;
    check_features(measured, &messages)?;
    if let Some(cpu) = cpu {
        args.extend(["--", "--cpu", cpu]);
    }
    let text = cargo(&paths.root, &paths.results(build), build.rustflags, &args)?;
    let mut cells = Vec::new();
    let mut sweeps = Vec::new();
    for line in text.lines().filter(|line| !line.trim().is_empty()) {
        match serde_json::from_str(line).map_err(Error::Cells)? {
            Line::Cell(cell) => cells.push(cell),
            Line::Sweep(sweep) => sweeps.push(sweep),
        }
    }
    match <[Sweep; 1]>::try_from(sweeps) {
        Ok([sweep]) => Ok((cells, sweep)),
        Err(sweeps) => Err(Error::missing(
            &format!("single sweep among its {}", sweeps.len()),
            "the lines of the bench",
        )),
    }
}

/// One JSON message of a cargo build: the fields that [`check_features`] reads.
#[derive(Deserialize)]
struct Message {
    reason: String,
    package_id: Option<String>,
    target: Option<Target>,
    #[serde(default)]
    features: Vec<String>,
}

/// The target of a compiled unit.
#[derive(Deserialize)]
struct Target {
    name: String,
    kind: Vec<String>,
}

/// Checks the units of a build from its JSON `messages`: the bench of `measured` has every feature
/// in [`Measured::features`], and no unit has a feature in [`Measured::forbidden`].
fn check_features(measured: &Measured, messages: &str) -> Result<(), Error> {
    let mut bench = None;
    for line in messages.lines().filter(|line| !line.trim().is_empty()) {
        let message: Message = serde_json::from_str(line).map_err(Error::Messages)?;
        if message.reason != "compiler-artifact" {
            continue;
        }
        let unit = message
            .package_id
            .ok_or_else(|| Error::missing("package_id", "a compiler-artifact message"))?;
        if let Some(feature) = message
            .features
            .iter()
            .find(|feature| measured.forbidden.contains(&feature.as_str()))
        {
            return Err(Error::Forbidden {
                unit,
                feature: feature.clone(),
            });
        }
        if message.target.as_ref().is_some_and(|target| {
            target.name == measured.bench && target.kind.iter().any(|kind| kind == "bench")
        }) {
            bench = Some(message.features);
        }
    }
    let bench = bench.ok_or_else(|| {
        Error::missing(
            &format!("bench {}", measured.bench),
            "the messages of the build",
        )
    })?;
    match measured
        .features
        .iter()
        .find(|feature| !bench.iter().any(|built| built == *feature))
    {
        Some(feature) => Err(Error::Unbuilt {
            package: measured.package.to_owned(),
            feature: (*feature).to_owned(),
        }),
        None => Ok(()),
    }
}

/// The standard output of `cargo` with `args` in the workspace at `root`, which must succeed. The
/// build takes exactly `rustflags`, whatever the environment and the configuration set, and keeps
/// its artifacts in `target_dir`. `CARGO` names the cargo that runs this tool.
pub fn cargo(
    root: &Path,
    target_dir: &Path,
    rustflags: &str,
    args: &[&str],
) -> Result<String, Error> {
    output(
        Command::new(std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into()))
            .args(args)
            .current_dir(root)
            .env("CARGO_TARGET_DIR", target_dir)
            .env("RUSTFLAGS", rustflags)
            .env_remove("CARGO_ENCODED_RUSTFLAGS"),
    )
}

/// The target features that `rustc --print cfg` reports under the flags of `build`. A build
/// without AVX2 is a [`Error::NoAvx2`].
fn target_features(paths: &Paths, build: &Build) -> Result<Vec<String>, Error> {
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
        Err(Error::NoAvx2 {
            rustflags: build.rustflags.to_owned(),
        })
    }
}

/// The model name of the machine's CPU.
fn cpu_model() -> Result<String, Error> {
    let path = Path::new(CPUINFO);
    let text = fs::read_to_string(path).map_err(|source| Error::io(path, source))?;
    text.lines()
        .find_map(|line| line.strip_prefix("model name"))
        .and_then(|rest| rest.split_once(':'))
        .map(|(_, name)| name.trim().to_owned())
        .ok_or_else(|| Error::missing("model name", CPUINFO))
}

/// The standard output of `command`, which must succeed. Its standard error goes to the terminal.
pub fn output(command: &mut Command) -> Result<String, Error> {
    let line = format!("{command:?}");
    let output = command
        .stderr(Stdio::inherit())
        .output()
        .map_err(|source| Error::Spawn {
            command: line.clone(),
            source,
        })?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    } else {
        Err(Error::Failed {
            command: line,
            status: output.status,
        })
    }
}

/// The output of `git` with `args` and then `pathspecs`, without its trailing newline.
fn git(paths: &Paths, args: &[&str], pathspecs: &[&str]) -> Result<String, Error> {
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
fn field(text: &str, prefix: &str, origin: &str) -> Result<String, Error> {
    text.lines()
        .find_map(|line| line.strip_prefix(prefix))
        .map(str::to_owned)
        .ok_or_else(|| Error::missing(prefix.trim_end_matches([':', ' ']), origin))
}

/// The CPU that `rustc --print target-cpus` names as the host's: `(currently <cpu>)`.
fn native_cpu(target_cpus: &str) -> Result<String, Error> {
    target_cpus
        .split("(currently ")
        .nth(1)
        .and_then(|rest| rest.split(')').next())
        .map(str::to_owned)
        .ok_or_else(|| Error::missing("native CPU", "rustc --print target-cpus"))
}

/// The versions of `wanted` in `Cargo.lock`.
fn versions(paths: &Paths, package: &str, wanted: &[&str]) -> Result<Vec<Version>, Error> {
    let path = paths.root.join("Cargo.lock");
    let lock = fs::read_to_string(&path).map_err(|source| Error::io(&path, source))?;
    let packages: Vec<Locked<'_>> = lock
        .split("[[package]]")
        .filter_map(Locked::parse)
        .collect();
    let measured = packages
        .iter()
        .find(|locked| locked.name == package)
        .ok_or_else(|| Error::missing(package, "Cargo.lock"))?;
    wanted
        .iter()
        .map(|wanted| {
            let entry = measured
                .dependencies
                .iter()
                .find(|entry| entry.split(' ').next() == Some(*wanted))
                .ok_or_else(|| {
                    Error::missing(&format!("dependency {wanted} of {package}"), "Cargo.lock")
                })?;
            // The entry names a version where the lock holds several of the crate.
            let version = match entry.split(' ').nth(1) {
                Some(version) => version,
                None => {
                    let mut named = packages.iter().filter(|locked| locked.name == *wanted);
                    match (named.next(), named.next()) {
                        (Some(locked), None) => locked.version,
                        (Some(_), Some(_)) | (None, _) => {
                            return Err(Error::missing(
                                &format!("single version of {wanted}"),
                                "Cargo.lock",
                            ));
                        }
                    }
                }
            };
            Ok(Version {
                name: (*wanted).to_owned(),
                version: version.to_owned(),
            })
        })
        .collect()
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
fn now() -> Result<u64, Error> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs())
        .map_err(Error::Clock)
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
