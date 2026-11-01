//! `bench`: builds the `compare` bench for the native CPU, runs it, and records the counts.

use std::fs;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::record::{Cell, Record, Version};
use crate::{Error, Paths, output};

/// The `RUSTFLAGS` of the build: code for the CPU that runs the bench.
const RUSTFLAGS: &str = "-C target-cpu=native";

/// The crates whose versions the record keeps.
const VERSIONS: [&str; 5] = ["ruint", "crypto-bigint", "bnum", "uint", "num-bigint"];

/// The files that the measured executable is built from, as git pathspecs.
const MEASURED: [&str; 6] = [
    "Cargo.toml",
    "Cargo.lock",
    "rust-toolchain.toml",
    "crates/biguint",
    ":(exclude)crates/biguint/BENCHMARKS.md",
    ":(exclude)crates/biguint/benchmarks",
];

/// Counts every case of the `compare` bench and writes the record. `machine` describes the
/// machine.
pub(crate) fn bench(paths: &Paths, machine: Option<String>) -> Result<(), Error> {
    let started = now()?;
    let rustc = output(Command::new("rustc").arg("-vV").current_dir(&paths.root))?;
    let target_cpus = output(
        Command::new("rustc")
            .args(["--print", "target-cpus"])
            .current_dir(&paths.root),
    )?;
    let valgrind = output(Command::new("valgrind").arg("--version"))?;
    let status_lines = git(paths, &["status", "--porcelain", "--"], &MEASURED)?;
    let record = Record {
        date: date(started),
        head: git(paths, &["rev-parse", "HEAD"], &[])?,
        library_commit: git(
            paths,
            &["log", "-1", "--format=%H", "--", "crates/biguint/src"],
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
        versions: versions(paths)?,
        cells: cells(paths)?,
    };
    record.write(&paths.record())
}

/// Every cell of the comparison with its count: the lines that the `compare` bench prints.
/// `CARGO` names the cargo that runs this tool.
fn cells(paths: &Paths) -> Result<Vec<Cell>, Error> {
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let lines = output(
        Command::new(cargo)
            .args(["bench", "--locked", "-p", "biguint", "--bench", "compare"])
            .current_dir(&paths.root)
            .env("CARGO_TARGET_DIR", paths.results())
            .env("RUSTFLAGS", RUSTFLAGS)
            .env_remove("CARGO_ENCODED_RUSTFLAGS"),
    )?;
    lines
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| serde_json::from_str(line).map_err(Error::Cells))
        .collect()
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

/// The versions of [`VERSIONS`] in `Cargo.lock`.
fn versions(paths: &Paths) -> Result<Vec<Version>, Error> {
    let path = paths.root.join("Cargo.lock");
    let lock = fs::read_to_string(&path).map_err(|source| Error::io(&path, source))?;
    let packages: Vec<(&str, &str)> = lock
        .split("[[package]]")
        .filter_map(|package| {
            let name = field_value(package, "name = ")?;
            let version = field_value(package, "version = ")?;
            Some((name, version))
        })
        .collect();
    VERSIONS
        .iter()
        .map(|wanted| {
            packages
                .iter()
                .find(|(name, _)| name == wanted)
                .map(|(name, version)| Version {
                    name: (*name).to_owned(),
                    version: (*version).to_owned(),
                })
                .ok_or_else(|| Error::missing(wanted, "Cargo.lock"))
        })
        .collect()
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
