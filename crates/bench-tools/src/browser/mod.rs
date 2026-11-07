//! The browsers that run a WebAssembly bench, with the settings of each build, and the driver that
//! counts the bench in them.

use std::fs;
use std::path::Path;
use std::process::Command;

#[cfg(not(target_os = "linux"))]
use bench_compare::{Error, Plan};

use crate::tool_error::ToolError;

#[cfg(target_os = "linux")]
mod driver;

#[cfg(target_os = "linux")]
pub use driver::{list, run};

/// The error of a thread that panicked with `payload`.
pub(crate) fn panicked(payload: Box<dyn std::any::Any + Send>) -> ToolError {
    ToolError::Browser {
        reason: format!(
            "a thread panicked: {}",
            payload
                .downcast_ref::<&str>()
                .copied()
                .or_else(|| payload.downcast_ref::<String>().map(String::as_str))
                .unwrap_or("a payload of another type")
        ),
    }
}

/// Runs a bench in a browser: the counters need Linux.
#[cfg(not(target_os = "linux"))]
pub fn run(
    _module: &[u8],
    _browser: &Browser,
    _executable: &Path,
    _cpus: Cpus,
    _threads: Threads,
    _plan: &Plan,
) -> Result<Vec<String>, ToolError> {
    Err(Error::NoPerfEvent.into())
}

/// Lists the cells of a bench in a browser: the driver needs Linux.
#[cfg(not(target_os = "linux"))]
pub fn list(
    _module: &[u8],
    _browser: &Browser,
    _executable: &Path,
) -> Result<Vec<String>, ToolError> {
    Err(Error::NoPerfEvent.into())
}

/// A browser engine, with the settings of a build.
#[derive(Clone, Copy, Debug)]
pub enum Browser {
    /// Chrome or Chromium, with the flags of its V8 engine.
    Chrome {
        /// The V8 flags, which `--js-flags` passes.
        js_flags: &'static [&'static str],
    },
    /// Firefox, with the preferences of its profile.
    Firefox {
        /// The preferences, which the profile's `user.js` sets.
        prefs: &'static [Pref],
    },
}

/// Chrome with every function compiled by TurboFan, the optimizing tier, when the module loads.
pub const CHROME: Browser = Browser::Chrome {
    js_flags: &["--no-liftoff"],
};

/// [`CHROME`] with the instructions of the wide-arithmetic proposal, which V8 gates behind a flag.
pub const CHROME_WIDE: Browser = Browser::Chrome {
    js_flags: &["--no-liftoff", "--wasm-wide-arithmetic"],
};

/// The preference of Firefox that disables the baseline compiler: Ion, the optimizing tier,
/// compiles every function when the module loads.
const ION_ONLY: Pref = Pref {
    name: "javascript.options.wasm_baselinejit",
    value: "false",
};

/// Firefox with every function compiled by Ion, without the instructions of the wide-arithmetic
/// proposal.
pub const FIREFOX: Browser = Browser::Firefox {
    prefs: &[
        ION_ONLY,
        Pref {
            name: "javascript.options.wasm_wide_arithmetic",
            value: "false",
        },
    ],
};

/// Firefox with every function compiled by Ion and the instructions of the wide-arithmetic
/// proposal.
pub const FIREFOX_WIDE: Browser = Browser::Firefox {
    prefs: &[
        ION_ONLY,
        Pref {
            name: "javascript.options.wasm_wide_arithmetic",
            value: "true",
        },
    ],
};

/// Chrome with the settings that list the cells of a module of any build at once: it compiles a
/// function when its first call comes.
pub const LISTING: Browser = Browser::Chrome {
    js_flags: &["--wasm-wide-arithmetic"],
};

/// A preference of Firefox: its name and its value as JavaScript, such as `false`.
#[derive(Clone, Copy, Debug)]
pub struct Pref {
    /// The name.
    pub name: &'static str,
    /// The value, as a JavaScript literal.
    pub value: &'static str,
}

/// The threads of the browser that a figure counts.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Threads {
    /// The thread that runs the module.
    Measuring,
    /// Every thread of the process that runs the module, at the start of the bench.
    Process,
}

impl Browser {
    /// The browser's name.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Chrome { .. } => "Chrome",
            Self::Firefox { .. } => "Firefox",
        }
    }

    /// The settings of the build, as the setup of a document names them.
    #[must_use]
    pub fn settings(&self) -> String {
        match self {
            Self::Chrome { js_flags } => format!("`--js-flags={}`", js_flags.join(" ")),
            Self::Firefox { prefs } => prefs
                .iter()
                .map(|pref| format!("`{}={}`", pref.name, pref.value))
                .collect::<Vec<_>>()
                .join(", "),
        }
    }

    /// The arguments that start the browser headless, with the profile `profile`, on the page
    /// `url`. A Firefox profile gets its preferences.
    fn arguments(&self, profile: &Path, url: &str) -> Result<Vec<String>, ToolError> {
        match self {
            Self::Chrome { js_flags } => Ok(vec![
                "--headless".to_owned(),
                // Without it, Chrome 156 stops with "No usable sandbox" where AppArmor forbids
                // unprivileged user namespaces, as Ubuntu 24.04 does.
                "--no-sandbox".to_owned(),
                format!("--user-data-dir={}", profile.display()),
                "--no-first-run".to_owned(),
                "--no-default-browser-check".to_owned(),
                "--disable-extensions".to_owned(),
                "--disable-background-networking".to_owned(),
                "--disable-component-update".to_owned(),
                "--disable-sync".to_owned(),
                "--disable-gpu".to_owned(),
                "--mute-audio".to_owned(),
                format!("--js-flags={}", js_flags.join(" ")),
                url.to_owned(),
            ]),
            Self::Firefox { prefs } => {
                let mut lines: Vec<String> = QUIET_FIREFOX
                    .iter()
                    .chain(prefs.iter())
                    .map(|pref| format!("user_pref(\"{}\", {});", pref.name, pref.value))
                    .collect();
                lines.push(String::new());
                let path = profile.join("user.js");
                fs::write(&path, lines.join("\n"))
                    .map_err(|source| ToolError::io(&path, source))?;
                Ok(vec![
                    "--headless".to_owned(),
                    "--no-remote".to_owned(),
                    "--profile".to_owned(),
                    profile.display().to_string(),
                    url.to_owned(),
                ])
            }
        }
    }
}

/// The preferences of every Firefox profile: no first-run pages, updates, telemetry or
/// background requests.
const QUIET_FIREFOX: &[Pref] = &[
    Pref {
        name: "browser.shell.checkDefaultBrowser",
        value: "false",
    },
    Pref {
        name: "browser.startup.homepage_override.mstone",
        value: "\"ignore\"",
    },
    Pref {
        name: "browser.aboutwelcome.enabled",
        value: "false",
    },
    Pref {
        name: "datareporting.policy.dataSubmissionEnabled",
        value: "false",
    },
    Pref {
        name: "toolkit.telemetry.reportingpolicy.firstRun",
        value: "false",
    },
    Pref {
        name: "app.update.disabledForTesting",
        value: "true",
    },
    Pref {
        name: "network.captive-portal-service.enabled",
        value: "false",
    },
    Pref {
        name: "browser.safebrowsing.update.enabled",
        value: "false",
    },
    Pref {
        name: "extensions.update.enabled",
        value: "false",
    },
];

/// The version that the browser at `executable` reports, such as `Mozilla Firefox 158.0.1`.
pub fn version(executable: &Path) -> Result<String, ToolError> {
    let text = crate::run::output(Command::new(executable).arg("--version"))?;
    text.lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .map(str::to_owned)
        .ok_or_else(|| {
            ToolError::missing("version", &format!("{} --version", executable.display()))
        })
}

/// The CPUs of a run in a browser.
#[derive(Clone, Copy, Debug)]
pub struct Cpus {
    /// The CPU of the measuring thread, alone.
    pub measuring: usize,
    /// The CPU of the driver and of every other thread of the browser.
    pub browser: usize,
}
