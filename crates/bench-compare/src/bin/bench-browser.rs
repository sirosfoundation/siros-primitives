//! Runs one WebAssembly bench in a browser, pinned to one CPU, and prints its lines, then the
//! sweep.

use std::process::ExitCode;

#[cfg(target_os = "linux")]
fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match linux::run(&args) {
        Ok(lines) => {
            for line in lines {
                println!("{line}");
            }
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("bench-browser: {error}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(not(target_os = "linux"))]
fn main() -> ExitCode {
    eprintln!("bench-browser: {}", bench_compare::Error::NoPerfEvent);
    ExitCode::FAILURE
}

#[cfg(target_os = "linux")]
mod linux {
    use std::fs;
    use std::path::Path;

    use bench_compare::{
        Browser, CHROME, CHROME_WIDE, CPU, Cpus, Error, FIREFOX, FIREFOX_WIDE, Line, Threads,
        Window, run_in_browser,
    };

    /// The usage of the tool.
    const USAGE: &str = "usage: bench-browser <module.wasm> (--chrome | --firefox) <executable> \
                         [--cpu <cpu>] [--browser-cpu <cpu>] [--build baseline | wide] \
                         [--tiering optimizing | default] [--threads measuring | process]";

    /// What one run measures.
    struct Arguments<'a> {
        module: &'a Path,
        firefox: bool,
        executable: &'a Path,
        cpu: usize,
        browser_cpu: Option<usize>,
        wide: bool,
        optimizing: bool,
        threads: Threads,
    }

    impl Arguments<'_> {
        /// The browser with the settings of the build: the settings of the standard builds,
        /// which compile every function with the optimizing tier, or the browser's defaults with
        /// the one flag that V8 needs for wide arithmetic.
        fn browser(&self) -> Browser {
            match (self.firefox, self.wide, self.optimizing) {
                (false, false, true) => CHROME,
                (false, true, true) => CHROME_WIDE,
                (true, false, true) => FIREFOX,
                (true, true, true) => FIREFOX_WIDE,
                (false, false, false) => Browser::Chrome { js_flags: &[] },
                (false, true, false) => Browser::Chrome {
                    js_flags: &["--wasm-wide-arithmetic"],
                },
                // Firefox enables wide arithmetic by default from version 155.
                (true, false | true, false) => Browser::Firefox { prefs: &[] },
            }
        }
    }

    fn usage() -> Error {
        Error::Browser {
            reason: USAGE.to_owned(),
        }
    }

    /// The CPU that `text` names.
    fn cpu(text: &str) -> Result<usize, Error> {
        text.parse().map_err(|source| Error::Number {
            what: "CPU".to_owned(),
            text: text.to_owned(),
            source,
        })
    }

    fn parse(args: &[String]) -> Result<Arguments<'_>, Error> {
        let [module, engine, executable, options @ ..] = args else {
            return Err(usage());
        };
        let firefox = match engine.as_str() {
            "--chrome" => false,
            "--firefox" => true,
            _ => return Err(usage()),
        };
        let mut arguments = Arguments {
            module: Path::new(module),
            firefox,
            executable: Path::new(executable),
            cpu: CPU,
            browser_cpu: None,
            wide: false,
            optimizing: true,
            threads: Threads::Measuring,
        };
        let (pairs, []) = options.as_chunks::<2>() else {
            return Err(usage());
        };
        for [flag, value] in pairs {
            match (flag.as_str(), value.as_str()) {
                ("--cpu", text) => arguments.cpu = cpu(text)?,
                ("--browser-cpu", text) => {
                    arguments.browser_cpu = Some(cpu(text)?);
                }
                ("--build", "baseline") => arguments.wide = false,
                ("--build", "wide") => arguments.wide = true,
                ("--tiering", "optimizing") => arguments.optimizing = true,
                ("--tiering", "default") => arguments.optimizing = false,
                ("--threads", "measuring") => arguments.threads = Threads::Measuring,
                ("--threads", "process") => arguments.threads = Threads::Process,
                _ => return Err(usage()),
            }
        }
        Ok(arguments)
    }

    /// Runs the bench that `args` name; returns its lines and the line of the sweep.
    pub(super) fn run(args: &[String]) -> Result<Vec<String>, Error> {
        let arguments = parse(args)?;
        let module = fs::read(arguments.module).map_err(|source| Error::Io {
            path: arguments.module.to_path_buf(),
            source,
        })?;
        let window = Window::open(arguments.cpu)?;
        let mut lines = run_in_browser(
            &module,
            &arguments.browser(),
            arguments.executable,
            Cpus {
                measuring: arguments.cpu,
                browser: arguments.browser_cpu.unwrap_or(arguments.cpu),
            },
            arguments.threads,
        )?;
        let sweep = Line::<serde_json::Value>::Sweep(window.close()?);
        lines.push(serde_json::to_string(&sweep).map_err(Error::Cells)?);
        Ok(lines)
    }
}
