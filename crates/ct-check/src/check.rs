//! The check: builds the operations for each target CPU and runs them under memcheck.

use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

use bench_compare::{cargo, output, valgrind};
use serde::Deserialize;

/// A build of the operations: the code that a target CPU and its features select.
struct Build {
    /// The target CPU, which names the build's directory under `target/ct-check`.
    name: &'static str,
    /// The `RUSTFLAGS` of the build.
    rustflags: &'static str,
}

/// The baseline of `x86_64`, and AVX2 with the ADX carries and the PCLMULQDQ carry-less products.
/// The flags name the CPU: the CPU of the build machine changes no code.
const BUILDS: [Build; 2] = [
    Build {
        name: "x86-64",
        rustflags: "-C target-cpu=x86-64",
    },
    Build {
        name: "x86-64-v3",
        rustflags: "-C target-cpu=x86-64-v3 -C target-feature=+adx,+pclmulqdq",
    },
];

/// The exit status of Valgrind when memcheck reported an error.
const REPORTED: i32 = 42;

/// The option that names the exemptions of the check.
const SUPPRESSIONS: &str = concat!(
    "--suppressions=",
    env!("CARGO_MANIFEST_DIR"),
    "/memcheck.supp"
);

/// The argument that runs the operations.
pub(crate) const OPERATIONS: &str = "--operations";

/// Why the check stopped before a verdict.
#[derive(Debug, thiserror::Error)]
enum Failure {
    /// A command could not run or failed.
    #[error(transparent)]
    Command(#[from] bench_compare::Error),
    /// A line of `cargo build` is no JSON message.
    #[error("a message of cargo build: {0}")]
    Message(serde_json::Error),
    /// `cargo build` named no executable of this package.
    #[error("cargo build named no executable of {}", env!("CARGO_PKG_NAME"))]
    Executable,
}

/// Checks every build; fails when memcheck reports an error in one.
pub(crate) fn main() -> ExitCode {
    let checked = valgrind().map_err(Failure::from).and_then(|valgrind| {
        BUILDS
            .iter()
            .try_fold(true, |clean, build| Ok(check(&valgrind, build)? && clean))
    });
    match checked {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::FAILURE,
        Err(error) => {
            eprintln!("ct-check: {error}");
            ExitCode::FAILURE
        }
    }
}

/// Builds the operations for `build` and runs them under memcheck, which prints its errors;
/// whether it reported none.
fn check(valgrind: &Path, build: &Build) -> Result<bool, Failure> {
    let program = compile(build)?;
    let checked = output(
        Command::new(valgrind)
            .args([
                "--tool=memcheck",
                "--quiet",
                &format!("--error-exitcode={REPORTED}"),
                "--track-origins=yes",
                "--read-inline-info=yes",
                "--num-callers=16",
                "--leak-check=no",
                SUPPRESSIONS,
            ])
            .arg(program)
            .arg(OPERATIONS),
    );
    match checked {
        Ok(report) => {
            println!("{}: {}, no error", build.name, report.trim_end());
            Ok(true)
        }
        Err(bench_compare::Error::Failed { status, .. }) if status.code() == Some(REPORTED) => {
            println!(
                "{}: memcheck reported a branch or a memory address that depends on a secret \
                 operand",
                build.name
            );
            Ok(false)
        }
        Err(error) => Err(error.into()),
    }
}

/// Builds the operations for `build` in `target/ct-check/<name>`, with the line tables that name
/// the inlined functions in memcheck's reports; the executable.
fn compile(build: &Build) -> Result<PathBuf, Failure> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..");
    let messages = cargo(
        &root,
        &root.join("target").join("ct-check").join(build.name),
        build.rustflags,
        &[
            "build",
            "--locked",
            "--release",
            "-p",
            env!("CARGO_PKG_NAME"),
            "--message-format=json-render-diagnostics",
            "--config",
            "profile.release.debug=\"line-tables-only\"",
        ],
    )?;
    messages
        .lines()
        .map(|line| serde_json::from_str::<Message>(line).map_err(Failure::Message))
        .find_map(|message| message.map(Message::executable).transpose())
        .unwrap_or(Err(Failure::Executable))
}

/// The fields of a message of `cargo build --message-format=json` that name an executable.
#[derive(Deserialize)]
struct Message {
    target: Option<Target>,
    executable: Option<PathBuf>,
}

/// The target that a message is about.
#[derive(Deserialize)]
struct Target {
    name: String,
}

impl Message {
    /// The executable of this package that the message names.
    fn executable(self) -> Option<PathBuf> {
        match self.target {
            Some(Target { name }) if name == env!("CARGO_PKG_NAME") => self.executable,
            Some(_) | None => None,
        }
    }
}
