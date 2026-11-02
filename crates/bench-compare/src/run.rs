//! `bench`: builds a `compare` bench for the native CPU, runs it, and records the counts.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::error::Error;
use crate::record::{Cell, Key, Record, Version};

/// The `RUSTFLAGS` of the build: code for the CPU that runs the bench, without SSE4a and
/// VPCLMULQDQ, whose `extrq` and 256-bit `vpclmulqdq` Valgrind 3.22 cannot execute.
const RUSTFLAGS: &str = "-C target-cpu=native -C target-feature=-sse4a,-vpclmulqdq";

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
    /// record is `target/benchmarks/<record>` under the workspace root.
    #[must_use]
    pub fn new(manifest_dir: &str, record: &str) -> Self {
        let library = Path::new(manifest_dir).join("..");
        let root = library.join("..").join("..");
        Self {
            record: root.join("target").join("benchmarks").join(record),
            root,
            library,
        }
    }

    /// The target directory of the bench build.
    fn results(&self) -> PathBuf {
        self.root.join("target").join("benchmarks")
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
}

/// Runs the report tool `tool` with the process arguments: `bench [--machine <description>]`
/// counts and records; `report` writes the document from the record.
#[must_use]
pub fn execute<K: Key>(
    tool: &str,
    paths: &Paths,
    measured: &Measured,
    layout: &crate::document::Layout<'_, K>,
) -> std::process::ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = match args.as_slice() {
        [command] if command == "bench" => bench::<K>(paths, measured, None),
        [command, flag, machine] if command == "bench" && flag == "--machine" => {
            bench::<K>(paths, measured, Some(machine.clone()))
        }
        [command] if command == "report" => Record::read(&paths.record)
            .and_then(|record| crate::document::report(paths, layout, &record)),
        _ => Err(Error::Usage {
            tool: tool.to_owned(),
        }),
    };
    match result {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{tool}: {error}");
            std::process::ExitCode::FAILURE
        }
    }
}

/// Counts every case of the bench and writes the record. `machine` describes the machine.
pub fn bench<K: Key>(
    paths: &Paths,
    measured: &Measured,
    machine: Option<String>,
) -> Result<(), Error> {
    let started = now()?;
    let rustc = output(Command::new("rustc").arg("-vV").current_dir(&paths.root))?;
    let target_cpus = output(
        Command::new("rustc")
            .args(["--print", "target-cpus"])
            .current_dir(&paths.root),
    )?;
    let valgrind = output(Command::new("valgrind").arg("--version"))?;
    let status_lines = git(paths, &["status", "--porcelain", "--"], measured.pathspecs)?;
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
        machine,
        rustflags: RUSTFLAGS.to_owned(),
        valgrind: valgrind.trim_end().to_owned(),
        versions: versions(paths, measured.package, measured.versions)?,
        cells: cells(paths, measured)?,
    };
    if let Some(directory) = paths.record.parent() {
        fs::create_dir_all(directory).map_err(|source| Error::io(directory, source))?;
    }
    record.write(&paths.record)
}

/// Every cell of the comparison with its count: the lines that the bench prints.
fn cells<K: Key>(paths: &Paths, measured: &Measured) -> Result<Vec<Cell<K>>, Error> {
    let lines = cargo(
        &paths.root,
        &paths.results(),
        RUSTFLAGS,
        &[
            "bench",
            "--locked",
            "-p",
            measured.package,
            "--bench",
            measured.bench,
        ],
    )?;
    lines
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| serde_json::from_str(line).map_err(Error::Cells))
        .collect()
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

/// The path of `valgrind` in `PATH`.
pub fn valgrind() -> Result<PathBuf, Error> {
    let path = std::env::var_os("PATH").ok_or_else(|| Error::missing("PATH", "the environment"))?;
    std::env::split_paths(&path)
        .map(|directory| directory.join("valgrind"))
        .find(|candidate| candidate.is_file())
        .ok_or_else(|| Error::missing("valgrind", "PATH"))
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
