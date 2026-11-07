//! Runs one WebAssembly bench in a browser, pinned to one CPU, and prints its lines, then the sweep;
//! decides between two sets of modules; and reads what a module holds.

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
    use std::collections::HashMap;
    use std::fs;
    use std::path::Path;

    use bench_compare::{
        Browser, CHROME, CHROME_WIDE, CPU, Cpus, Decision, Error, FIREFOX, FIREFOX_WIDE, Identity,
        LISTING, Line, Plan, SELECTED_RUNS, Selection, Threads, ToolError, Window, code, decide,
        fewest, list_in_browser, run_in_browser,
    };
    use serde_json::Value;

    /// The usage of the tool.
    const USAGE: &str = "usage: bench-browser <module.wasm> (--chrome | --firefox) <executable> \
                         [--only <op/key/crate,...>] [--runs <runs>] [--cpu <cpu>] [--browser-cpu \
                         <cpu>] [--build baseline | wide] [--tiering optimizing | default] \
                         [--threads measuring | process]\n\
                         | bench-browser decide --baseline <directory> --change <directory> --only \
                         <op/key/crate,...> --chrome <executable> --firefox <executable> [--runs \
                         <runs>] [--cpu <cpu>] [--browser-cpu <cpu>]\n\
                         | bench-browser list <module.wasm> --chrome <executable>\n\
                         | bench-browser diff <module.wasm> <module.wasm> [<word in a name>]\n\
                         | bench-browser edges <module.wasm>";

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
        only: Option<Selection>,
        runs: Option<usize>,
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

    /// The whole number that `text` names, which counts `what`.
    fn number(what: &str, text: &str) -> Result<usize, Error> {
        text.parse().map_err(|source| Error::Number {
            what: what.to_owned(),
            text: text.to_owned(),
            source,
        })
    }

    /// The CPU that `text` names.
    fn cpu(text: &str) -> Result<usize, Error> {
        number("CPU", text)
    }

    fn parse(args: &[String]) -> Result<Arguments<'_>, ToolError> {
        let [module, engine, executable, options @ ..] = args else {
            return Err(usage().into());
        };
        let firefox = match engine.as_str() {
            "--chrome" => false,
            "--firefox" => true,
            _ => return Err(usage().into()),
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
            only: None,
            runs: None,
        };
        let (pairs, []) = options.as_chunks::<2>() else {
            return Err(usage().into());
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
                ("--only", text) => arguments.only = Some(Selection::parse(text)?),
                ("--runs", text) => arguments.runs = Some(number("runs", text)?),
                _ => return Err(usage().into()),
            }
        }
        Ok(arguments)
    }

    /// Runs the cells of the bench that `args` name, as many times as they say; returns its lines
    /// and the line of the sweep.
    fn measure(args: &[String]) -> Result<Vec<String>, ToolError> {
        let arguments = parse(args)?;
        let module = fs::read(arguments.module).map_err(|source| Error::Io {
            path: arguments.module.to_path_buf(),
            source,
        })?;
        let (plan, runs) = match arguments.only.clone() {
            Some(selection) => (
                Plan::Only(selection),
                arguments.runs.unwrap_or(SELECTED_RUNS),
            ),
            None => (Plan::All, arguments.runs.unwrap_or(1)),
        };
        let window = Window::open(arguments.cpu)?;
        let mut all = Vec::with_capacity(runs);
        for _ in 0..runs {
            all.push(run_in_browser(
                &module,
                &arguments.browser(),
                arguments.executable,
                Cpus {
                    measuring: arguments.cpu,
                    browser: arguments.browser_cpu.unwrap_or(arguments.cpu),
                },
                arguments.threads,
                &plan,
            )?);
        }
        let mut lines = fewest(all)?;
        let sweep = Line::<Value>::Sweep(window.close()?);
        lines.push(serde_json::to_string(&sweep).map_err(Error::Cells)?);
        Ok(lines)
    }

    /// Runs a decision that `options`, pairs of a flag and its value, name.
    fn decision(options: &[String]) -> Result<Vec<String>, ToolError> {
        let (pairs, []) = options.as_chunks::<2>() else {
            return Err(usage().into());
        };
        let value = |wanted: &str| {
            pairs
                .iter()
                .find(|[flag, _]| flag == wanted)
                .map(|[_, value]| value.as_str())
        };
        let required = |wanted: &str| value(wanted).ok_or_else(usage);
        if let Some([flag, _]) = pairs.iter().find(|[flag, _]| {
            ![
                "--baseline",
                "--change",
                "--only",
                "--runs",
                "--chrome",
                "--firefox",
                "--cpu",
                "--browser-cpu",
            ]
            .contains(&flag.as_str())
        }) {
            return Err(Error::Browser {
                reason: format!("the flag {flag}\n{USAGE}"),
            }
            .into());
        }
        let measuring = value("--cpu").map_or(Ok(CPU), cpu)?;
        decide(&Decision {
            baseline: Path::new(required("--baseline")?),
            change: Path::new(required("--change")?),
            only: required("--only")?,
            runs: value("--runs").map_or(Ok(SELECTED_RUNS), |text| number("runs", text))?,
            chrome: Path::new(required("--chrome")?),
            firefox: Path::new(required("--firefox")?),
            cpus: Cpus {
                measuring,
                browser: value("--browser-cpu").map_or(Ok(measuring), cpu)?,
            },
        })?;
        Ok(Vec::new())
    }

    /// The file at `path`.
    fn read(path: &str) -> Result<Vec<u8>, Error> {
        fs::read(path).map_err(|source| Error::Io {
            path: path.into(),
            source,
        })
    }

    /// The index of every function of the module of `identity` whose name holds `word`, by name.
    fn named(identity: &Identity<'_>, word: &str) -> HashMap<String, Vec<u32>> {
        let mut functions: HashMap<String, Vec<u32>> = HashMap::new();
        for (index, name) in identity.names() {
            if name.contains(word) {
                functions
                    .entry((*name).to_owned())
                    .or_default()
                    .push(*index);
            }
        }
        functions
    }

    /// The hashes of `functions`, sorted, with the function of each.
    fn hashes(
        identity: &mut Identity<'_>,
        functions: &[u32],
    ) -> Result<Vec<(u128, u32)>, ToolError> {
        let mut hashes = functions
            .iter()
            .map(|function| Ok((identity.function(*function)?.hash, *function)))
            .collect::<Result<Vec<_>, ToolError>>()?;
        hashes.sort_unstable();
        Ok(hashes)
    }

    /// The functions whose name holds `word` and that differ between two modules, by name, where
    /// each differs, and how many stayed: what each reaches counts, and what layout moves does not.
    fn diff(first: &str, second: &str, word: &str) -> Result<Vec<String>, ToolError> {
        let (first, second) = (read(first)?, read(second)?);
        let mut first = Identity::of(&first)?;
        let mut second = Identity::of(&second)?;
        let ours = named(&first, word);
        let theirs = named(&second, word);
        let mut lines = Vec::new();
        let mut same = 0_usize;
        for (name, functions) in &ours {
            let Some(others) = theirs.get(name) else {
                lines.push(format!("only in the first {name}"));
                continue;
            };
            let (mine, yours) = (hashes(&mut first, functions)?, hashes(&mut second, others)?);
            for (mine, yours) in mine.iter().zip(&yours) {
                match first.difference(mine.1, &mut second, yours.1)? {
                    None => same += 1,
                    Some(why) => lines.push(format!("differs {name}: {why}")),
                }
            }
            if mine.len() != yours.len() {
                lines.push(format!("of another number {name}"));
            }
        }
        for name in theirs.keys().filter(|name| !ours.contains_key(*name)) {
            lines.push(format!("only in the second {name}"));
        }
        lines.sort_unstable();
        lines.push(format!("same={same} differ={}", lines.len()));
        Ok(lines)
    }

    /// The functions that each defined function of the module at `path` calls, by function index:
    /// a line of the function and its callees in order.
    fn edges(path: &str) -> Result<Vec<String>, ToolError> {
        let bytes = read(path)?;
        let identity = Identity::of(&bytes)?;
        Ok(identity
            .edges()
            .into_iter()
            .map(|(function, callees)| {
                let callees: Vec<String> = callees.iter().map(u32::to_string).collect();
                format!("{function} {}", callees.join(" "))
            })
            .collect())
    }

    /// The cells of the module at `path`: for each, its name, its code, what it captures and the
    /// names of its loops, apart by tabs.
    fn listing(path: &str, chrome: &str) -> Result<Vec<String>, ToolError> {
        let bytes = read(path)?;
        let lines = list_in_browser(&bytes, &LISTING, Path::new(chrome))?;
        let mut identity = Identity::of(&bytes)?;
        let names: HashMap<u32, String> = identity
            .names()
            .iter()
            .map(|(index, name)| (*index, (*name).to_owned()))
            .collect();
        let mut listed = Vec::with_capacity(lines.len());
        for line in lines {
            let value: Value = serde_json::from_str(&line).map_err(Error::Cells)?;
            let text = |field: &str| {
                value
                    .get(field)
                    .and_then(Value::as_str)
                    .ok_or_else(|| Error::Missing {
                        what: field.to_owned(),
                        origin: "a line of the listing".to_owned(),
                    })
            };
            let cell = format!("{}/{}/{}", text("op")?, text("row")?, text("crate")?);
            let Some(case) = value.get("case") else {
                listed.push(format!("{cell}\tabsent"));
                continue;
            };
            let case = serde_json::from_value(case.clone()).map_err(Error::Cells)?;
            let hash = code(&mut identity, &case)?.unwrap_or_else(|| "always".to_owned());
            let mut entries = vec![case.batch];
            entries.extend(case.chains.iter().map(|chain| chain.entry));
            let mut functions = Vec::with_capacity(entries.len());
            for entry in entries {
                let slot = u32::try_from(entry).map_err(|_error| Error::Missing {
                    what: "table index of 32 bits".to_owned(),
                    origin: "a line of the listing".to_owned(),
                })?;
                let function = identity.in_table(slot)?;
                functions.push(format!(
                    "{function}:{}",
                    names.get(&function).map_or("", String::as_str)
                ));
            }
            listed.push(format!(
                "{cell}\t{hash}\t{}\t{}",
                case.captures,
                functions.join(" ")
            ));
        }
        Ok(listed)
    }

    /// Runs the command that `args` name; returns what it prints last.
    pub(super) fn run(args: &[String]) -> Result<Vec<String>, ToolError> {
        match args {
            [command, options @ ..] if command == "decide" => decision(options),
            [command, module, flag, chrome] if command == "list" && flag == "--chrome" => {
                listing(module, chrome)
            }
            [command, first, second] if command == "diff" => diff(first, second, ""),
            [command, first, second, word] if command == "diff" => diff(first, second, word),
            [command, module] if command == "edges" => edges(module),
            _ => measure(args),
        }
    }
}
