//! The driver of a WebAssembly bench: it serves the module to a headless browser, finds the thread
//! that runs it, and counts that thread with its perf_event group while the module runs.
//!
//! The driver pins itself, and with it the browser, to one CPU. The page starts two dedicated
//! workers on shared memory: the measuring worker instantiates the module, and the helper passes
//! the module's `begin` and `end` to the driver as synchronous requests while the measuring
//! worker sleeps in `Atomics.wait`. The driver resets and enables the counters on `/begin` and
//! reads them on `/end`. The module's other imports are synchronous requests of the measuring
//! worker. Before the bench starts, the measuring worker spins for two seconds: the thread
//! whose CPU time grows the most meanwhile is the measuring thread, which the driver then pins to
//! the measuring CPU.
//!
//! A thread of the driver holds the measuring CPU meanwhile: it yields to the measuring thread
//! whenever that one runs, and keeps the CPU from idling while it sleeps, which drew other work
//! onto the CPU between counts. Firefox's one-`add` chain step took 1.06 to 1.40 cycles with the
//! holder and 1.34 to 1.64 without, three runs each at a load of 21 to 31.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant};

use bench_compare::{Counters, Counts, Plan, pin, pin_thread};
use nix::sys::signal::{Signal, kill};
use nix::sys::wait::waitpid;
use nix::unistd::Pid;
use tiny_http::{Header, Method, Request, Response, Server};

use super::{Browser, Cpus, Threads, panicked};
use crate::tool_error::ToolError;

/// The page, which starts the two workers on the memory they share.
const PAGE: &str = include_str!("page.html");

/// The scripts of the page, by path.
const SCRIPTS: [(&str, &str); 3] = [
    ("/worker.js", include_str!("worker.js")),
    ("/helper.js", include_str!("helper.js")),
    ("/shared.js", include_str!("shared.js")),
];

/// The headers of every response: they isolate the page from other origins, which shared memory
/// needs.
const ISOLATION: [(&str, &str); 2] = [
    ("Cross-Origin-Opener-Policy", "same-origin"),
    ("Cross-Origin-Embedder-Policy", "require-corp"),
];

/// The longest that the driver waits for the next request of the page.
const IDLE: Duration = Duration::from_secs(300);

/// The fewest ticks of CPU time that the measuring thread spends while the worker spins for two
/// seconds: a quarter of them at 100 ticks per second. The browser shares its CPU with the worker
/// while it loads the page: the worker of Firefox got 17 of the 50 ticks of half a second.
const CALIBRATION_TICKS: u64 = 50;

/// The spins of the worker that may pass before one thread stands out.
const CALIBRATIONS: usize = 5;

/// Runs the cells of `plan` of the bench `module` in `browser` at `executable` on `cpus`, counting
/// `threads`; returns the lines that the bench sends. A thread of its own drives the browser, and
/// another holds the measuring CPU: their pins leave the calling thread as it was.
pub fn run(
    module: &[u8],
    browser: &Browser,
    executable: &Path,
    cpus: Cpus,
    threads: Threads,
    plan: &Plan,
) -> Result<Vec<String>, ToolError> {
    std::thread::scope(|scope| {
        let (stop, stopped) = mpsc::channel::<()>();
        let holder = scope.spawn(move || hold(cpus.measuring, &stopped));
        let placement = Some(Placement { cpus, threads });
        let driven = scope
            .spawn(move || drive(module, browser, executable, placement, plan))
            .join()
            .map_err(panicked);
        // The holder stops once the channel is closed.
        drop(stop);
        holder.join().map_err(panicked)??;
        driven?
    })
}

/// The lines that the bench `module` sends when it lists its cells in `browser` at `executable`,
/// on the CPUs of the calling thread: nothing counts, so nothing is pinned.
pub fn list(module: &[u8], browser: &Browser, executable: &Path) -> Result<Vec<String>, ToolError> {
    drive(module, browser, executable, None, &Plan::List)
}

/// Where a run that counts stands.
#[derive(Clone, Copy)]
struct Placement {
    /// The CPUs of the run.
    cpus: Cpus,
    /// The threads that it counts.
    threads: Threads,
}

/// Holds `cpu` until `stopped` closes: yields it to any thread of equal priority, the measuring
/// thread among them, and keeps it from idling while the measuring thread sleeps.
fn hold(cpu: usize, stopped: &mpsc::Receiver<()>) -> Result<(), ToolError> {
    pin(cpu)?;
    while let Err(mpsc::TryRecvError::Empty) = stopped.try_recv() {
        std::thread::yield_now();
    }
    Ok(())
}

/// Starts the browser on the calling thread, pinned where `placement` says, and answers the page.
fn drive(
    module: &[u8],
    browser: &Browser,
    executable: &Path,
    placement: Option<Placement>,
    plan: &Plan,
) -> Result<Vec<String>, ToolError> {
    if let Some(Placement { cpus, .. }) = placement {
        pin(cpus.browser)?;
    }
    nix::sys::prctl::set_child_subreaper(true).map_err(|source| ToolError::Browser {
        reason: format!("becoming the subreaper of the browser: {source}"),
    })?;
    let server = Server::http("127.0.0.1:0").map_err(|source| ToolError::Browser {
        reason: format!("serving the page: {source}"),
    })?;
    let port = server
        .server_addr()
        .to_ip()
        .ok_or_else(|| ToolError::missing("port", "the page server"))?
        .port();
    let profile = Profile::create()?;
    let log = profile.path.join("browser.log");
    let output = fs::File::create(&log).map_err(|source| ToolError::io(&log, source))?;
    let errors = output
        .try_clone()
        .map_err(|source| ToolError::io(&log, source))?;
    let arguments = browser.arguments(&profile.path, &format!("http://127.0.0.1:{port}/"))?;
    let child = Command::new(executable)
        .args(&arguments)
        .current_dir(&profile.path)
        .stdin(Stdio::null())
        .stdout(output)
        .stderr(errors)
        .spawn()
        .map_err(|source| ToolError::Spawn {
            command: executable.display().to_string(),
            source,
        })?;
    let mut session = Session {
        module,
        plan: plan.text(),
        placement,
        browser: child,
        calibration: None,
        calibrations: 0,
        counters: Vec::new(),
        lines: Vec::new(),
        failure: None,
    };
    let result = session.serve(&server);
    let stopped = session.stop();
    let result = result.map_err(|error| with_log(error, &log));
    let removed = profile.remove();
    let lines = result?;
    stopped?;
    removed?;
    Ok(lines)
}

/// `error` with the last lines of the browser's log at `log`.
fn with_log(error: ToolError, log: &Path) -> ToolError {
    let tail = match fs::read_to_string(log) {
        Ok(text) => {
            let lines: Vec<&str> = text.lines().collect();
            lines[lines.len().saturating_sub(20)..].join("\n")
        }
        Err(source) => format!("unreadable: {source}"),
    };
    ToolError::Browser {
        reason: format!("{error}\nthe last lines of the browser's log:\n{tail}"),
    }
}

/// The start of the name of a profile directory, which the process id of its driver ends.
const PROFILE: &str = "bench-tools-";

/// Kills the browsers whose driver a kill took away, and removes their profiles. They wait for
/// requests that never come, and hold their memory.
fn reap_orphans() -> Result<(), ToolError> {
    let directory = std::env::temp_dir();
    let entries = fs::read_dir(&directory).map_err(|source| ToolError::io(&directory, source))?;
    for entry in entries {
        let entry = entry.map_err(|source| ToolError::io(&directory, source))?;
        let Some(driver) = entry
            .file_name()
            .to_str()
            .and_then(|name| name.strip_prefix(PROFILE))
            .and_then(|pid| pid.parse::<i32>().ok())
        else {
            continue;
        };
        if Path::new(&format!("/proc/{driver}")).exists() {
            continue;
        }
        let profile = entry.path();
        for pid in naming(&profile)? {
            match kill(Pid::from_raw(pid), Signal::SIGKILL) {
                Ok(()) | Err(nix::Error::ESRCH) => {}
                Err(source) => {
                    return Err(ToolError::Browser {
                        reason: format!("stopping the orphaned browser {pid}: {source}"),
                    });
                }
            }
        }
        match fs::remove_dir_all(&profile) {
            Ok(()) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(source) => return Err(ToolError::io(&profile, source)),
        }
    }
    Ok(())
}

/// The processes whose command line names `profile`.
fn naming(profile: &Path) -> Result<Vec<i32>, ToolError> {
    let name = profile.as_os_str().as_encoded_bytes();
    let proc = Path::new("/proc");
    let mut found = Vec::new();
    for entry in fs::read_dir(proc).map_err(|source| ToolError::io(proc, source))? {
        let entry = entry.map_err(|source| ToolError::io(proc, source))?;
        let Some(pid) = number(&entry.file_name()) else {
            continue;
        };
        let path = entry.path().join("cmdline");
        let Some(command) = gone(fs::read(&path), &path)? else {
            continue;
        };
        // Each argument ends in a zero byte; a profile of a driver whose number continues is another.
        if command.windows(name.len() + 1).any(|window| {
            window.starts_with(name)
                && window
                    .get(name.len())
                    .is_some_and(|byte| !byte.is_ascii_digit())
        }) {
            found.push(pid);
        }
    }
    Ok(found)
}

/// A fresh profile directory of the browser.
struct Profile {
    path: PathBuf,
}

impl Profile {
    /// An empty directory in the temporary directory, named after this process.
    fn create() -> Result<Self, ToolError> {
        reap_orphans()?;
        let path = std::env::temp_dir().join(format!("{PROFILE}{}", std::process::id()));
        if path.exists() {
            fs::remove_dir_all(&path).map_err(|source| ToolError::io(&path, source))?;
        }
        fs::create_dir_all(&path).map_err(|source| ToolError::io(&path, source))?;
        Ok(Self { path })
    }

    fn remove(self) -> Result<(), ToolError> {
        fs::remove_dir_all(&self.path).map_err(|source| ToolError::io(&self.path, source))
    }
}

/// One run of a bench in the browser.
struct Session<'a> {
    module: &'a [u8],
    /// The text of the plan of the run, which the page passes to the bench.
    plan: String,
    /// Where the calibration pins the measuring thread, and what it counts; `None` for a run that
    /// counts nothing.
    placement: Option<Placement>,
    browser: Child,
    /// The CPU time of every thread of the browser when the worker started to spin.
    calibration: Option<Vec<Task>>,
    /// The spins of the worker so far.
    calibrations: usize,
    /// The counters of the counted threads, which the calibration opens.
    counters: Vec<Counters>,
    lines: Vec<String>,
    /// The reason for which the bench stopped, as it sent it.
    failure: Option<String>,
}

/// What the page asks.
enum Asked {
    /// More requests follow.
    More,
    /// The bench returned its exit status.
    Done(String),
}

impl Session<'_> {
    /// Answers the page until the bench returns; returns its lines.
    fn serve(&mut self, server: &Server) -> Result<Vec<String>, ToolError> {
        let mut deadline = Instant::now() + IDLE;
        loop {
            if let Some(status) = self
                .browser
                .try_wait()
                .map_err(|source| ToolError::Browser {
                    reason: format!("waiting for the browser: {source}"),
                })?
            {
                return Err(ToolError::Browser {
                    reason: format!("the browser exited with {status} before the bench returned"),
                });
            }
            let request = server
                .recv_timeout(Duration::from_millis(500))
                .map_err(|source| ToolError::Browser {
                    reason: format!("receiving a request of the page: {source}"),
                })?;
            let Some(request) = request else {
                if Instant::now() > deadline {
                    return Err(ToolError::Browser {
                        reason: format!("no request of the page for {} seconds", IDLE.as_secs()),
                    });
                }
                continue;
            };
            deadline = Instant::now() + IDLE;
            match self.answer(request)? {
                Asked::More => {}
                Asked::Done(status) => {
                    return match (status.as_str(), self.failure.take()) {
                        ("0", None) => Ok(core::mem::take(&mut self.lines)),
                        (_, Some(reason)) => Err(ToolError::Bench { reason }),
                        (status, None) => Err(ToolError::Bench {
                            reason: format!("the bench returned {status}"),
                        }),
                    };
                }
            }
        }
    }

    /// Answers one request of the page.
    fn answer(&mut self, mut request: Request) -> Result<Asked, ToolError> {
        let path = request.url().to_owned();
        let body =
            io::read_to_string(request.as_reader()).map_err(|source| ToolError::Browser {
                reason: format!("reading the request {path}: {source}"),
            })?;
        let script = SCRIPTS.iter().find(|(name, _)| *name == path);
        let (reply, asked) = match (request.method(), path.as_str(), script) {
            (Method::Get, "/", _) => (Reply::of(PAGE, "text/html"), Asked::More),
            (Method::Get, "/bench.wasm", _) => {
                (Reply::of(self.module, "application/wasm"), Asked::More)
            }
            (Method::Get, _, Some((_, source))) => {
                (Reply::of(source, "text/javascript"), Asked::More)
            }
            (Method::Post, "/plan", _) => (Reply::of(&self.plan, "text/plain"), Asked::More),
            (Method::Post, "/calibrate", _) => {
                self.calibration = Some(tasks()?);
                (Reply::empty(), Asked::More)
            }
            (Method::Post, "/calibrated", _) => {
                let found = self.calibrated()?;
                let text = if found { "found" } else { "again" };
                (Reply::of(text, "text/plain"), Asked::More)
            }
            (Method::Post, "/begin", _) => {
                for counters in &mut self.counters {
                    counters.begin()?;
                }
                (Reply::empty(), Asked::More)
            }
            (Method::Post, "/end", _) => {
                let counts = self.end()?;
                let text = format!(
                    "{} {} {}",
                    counts.cycles, counts.instructions, counts.branch_misses
                );
                (Reply::of(text, "text/plain"), Asked::More)
            }
            (Method::Post, "/emit", _) => {
                self.lines.push(body);
                (Reply::empty(), Asked::More)
            }
            (Method::Post, "/fail", _) => {
                self.failure = Some(body);
                (Reply::empty(), Asked::More)
            }
            (Method::Post, "/done", _) => (Reply::empty(), Asked::Done(body)),
            (Method::Post, "/error", _) => {
                let reason = match self.failure.take() {
                    Some(failure) => format!("{failure}\n{body}"),
                    None => body,
                };
                return Err(ToolError::Bench { reason });
            }
            _ => (
                Reply {
                    status: 404,
                    ..Reply::empty()
                },
                Asked::More,
            ),
        };
        request
            .respond(reply.response()?)
            .map_err(|source| ToolError::Browser {
                reason: format!("answering the request {path}: {source}"),
            })?;
        Ok(asked)
    }

    /// The counts of every counted thread since `/begin`, summed.
    fn end(&mut self) -> Result<Counts, ToolError> {
        let mut sum = Counts {
            cycles: 0,
            instructions: 0,
            branch_misses: 0,
        };
        if self.counters.is_empty() {
            return Err(ToolError::missing("counted thread", "the browser"));
        }
        for counters in &mut self.counters {
            let counts = counters.end()?;
            sum.cycles += counts.cycles;
            sum.instructions += counts.instructions;
            sum.branch_misses += counts.branch_misses;
        }
        Ok(sum)
    }

    /// Finds the measuring thread, the thread whose CPU time grew the most since `/calibrate`,
    /// and opens the counters of the threads to count; returns whether one thread stood out. The
    /// page spins again until one does, [`CALIBRATIONS`] times at most.
    fn calibrated(&mut self) -> Result<bool, ToolError> {
        let Placement { cpus, threads } = self
            .placement
            .ok_or_else(|| ToolError::missing("place to count", "a run that lists its cells"))?;
        let before = self
            .calibration
            .take()
            .ok_or_else(|| ToolError::missing("/calibrate before /calibrated", "the page"))?;
        let after = tasks()?;
        let mut grown: Vec<(u64, &Task)> = after
            .iter()
            .map(|task| {
                let earlier = before
                    .iter()
                    .find(|other| other.tid == task.tid)
                    .map_or(0, |other| other.ticks);
                (task.ticks.saturating_sub(earlier), task)
            })
            .collect();
        grown.sort_by_key(|(ticks, _)| core::cmp::Reverse(*ticks));
        self.calibrations += 1;
        let measuring = match grown.as_slice() {
            [(ticks, task), rest @ ..]
                if *ticks >= CALIBRATION_TICKS
                    && rest.first().is_none_or(|(other, _)| 2 * other < *ticks) =>
            {
                *task
            }
            _ if self.calibrations < CALIBRATIONS => return Ok(false),
            _ => {
                return Err(ToolError::Browser {
                    reason: format!(
                        "no single thread spent {CALIBRATION_TICKS} ticks, twice as many as any \
                         other, in {CALIBRATIONS} spins of the worker: the most grown in the \
                         last {:?}",
                        grown
                            .iter()
                            .take(3)
                            .map(|(ticks, task)| (task.tid, task.name.as_str(), *ticks))
                            .collect::<Vec<_>>()
                    ),
                });
            }
        };
        pin_thread(measuring.tid, cpus.measuring)?;
        let counted: Vec<i32> = match threads {
            Threads::Measuring => vec![measuring.tid],
            Threads::Process => after
                .iter()
                .filter(|task| task.pid == measuring.pid)
                .map(|task| task.tid)
                .collect(),
        };
        self.counters = counted
            .into_iter()
            .map(Counters::observe)
            .collect::<Result<_, _>>()?;
        Ok(true)
    }

    /// Kills every process of the browser and reaps it.
    fn stop(&mut self) -> Result<(), ToolError> {
        let failure = |source: nix::Error| ToolError::Browser {
            reason: format!("stopping the browser: {source}"),
        };
        for pid in descendants()? {
            match kill(Pid::from_raw(pid), Signal::SIGKILL) {
                Ok(()) | Err(nix::Error::ESRCH) => {}
                Err(source) => return Err(failure(source)),
            }
        }
        #[expect(
            unused_results,
            reason = "the browser exits by the kill; its status tells nothing"
        )]
        self.browser.wait().map_err(|source| ToolError::Browser {
            reason: format!("waiting for the browser: {source}"),
        })?;
        // The killed descendants that this process adopted as their subreaper, until none is left.
        loop {
            match waitpid(None, None) {
                Ok(_) => {}
                Err(nix::Error::ECHILD) => return Ok(()),
                Err(source) => return Err(failure(source)),
            }
        }
    }
}

/// A response of the driver.
struct Reply {
    status: u16,
    content_type: &'static str,
    body: Vec<u8>,
}

impl Reply {
    /// A response of `body`, of `content_type`.
    fn of(body: impl AsRef<[u8]>, content_type: &'static str) -> Self {
        Self {
            status: 200,
            content_type,
            body: body.as_ref().to_vec(),
        }
    }

    /// An empty response.
    fn empty() -> Self {
        Self::of([], "text/plain")
    }

    /// The response, with the headers of its content type and of [`ISOLATION`].
    fn response(self) -> Result<Response<io::Cursor<Vec<u8>>>, ToolError> {
        let header = |name: &str, value: &str| {
            Header::from_bytes(name, value).map_err(|()| {
                ToolError::missing(&format!("header {name}: {value}"), "the page server")
            })
        };
        let mut response = Response::from_data(self.body)
            .with_status_code(self.status)
            .with_header(header("Content-Type", self.content_type)?);
        for (name, value) in ISOLATION {
            response = response.with_header(header(name, value)?);
        }
        Ok(response)
    }
}

/// A thread of the browser and the CPU time it spent in user mode.
struct Task {
    pid: i32,
    tid: i32,
    ticks: u64,
    /// The name of the thread.
    name: String,
}

/// Every thread of the browser's processes, with its user-mode CPU time.
fn tasks() -> Result<Vec<Task>, ToolError> {
    let mut tasks = Vec::new();
    for pid in descendants()? {
        let directory = PathBuf::from(format!("/proc/{pid}/task"));
        let Some(entries) = gone(fs::read_dir(&directory), &directory)? else {
            continue;
        };
        for entry in entries {
            let entry = entry.map_err(|source| ToolError::io(&directory, source))?;
            let Some(tid) = number(&entry.file_name()) else {
                continue;
            };
            let path = entry.path().join("stat");
            let Some(stat) = gone(fs::read_to_string(&path), &path)? else {
                continue;
            };
            // `utime` is the 14th field; the name in parentheses, the 2nd, may hold spaces.
            let ticks = stat
                .rsplit_once(')')
                .and_then(|(_, rest)| rest.split_whitespace().nth(11))
                .and_then(|field| field.parse::<u64>().ok())
                .ok_or_else(|| ToolError::missing("utime", &path.display().to_string()))?;
            let comm = entry.path().join("comm");
            let Some(name) = gone(fs::read_to_string(&comm), &comm)? else {
                continue;
            };
            tasks.push(Task {
                pid,
                tid,
                ticks,
                name: name.trim().to_owned(),
            });
        }
    }
    Ok(tasks)
}

/// Every process that descends from this one, the subreaper of the browser: the browser and every
/// process it started, whatever exited between them.
fn descendants() -> Result<Vec<i32>, ToolError> {
    let root = nix::unistd::getpid().as_raw();
    let proc = Path::new("/proc");
    let mut parents = Vec::new();
    for entry in fs::read_dir(proc).map_err(|source| ToolError::io(proc, source))? {
        let entry = entry.map_err(|source| ToolError::io(proc, source))?;
        let Some(pid) = number(&entry.file_name()) else {
            continue;
        };
        let path = entry.path().join("stat");
        let Some(stat) = gone(fs::read_to_string(&path), &path)? else {
            continue;
        };
        let parent = stat
            .rsplit_once(')')
            .and_then(|(_, rest)| rest.split_whitespace().nth(1))
            .and_then(|field| field.parse::<i32>().ok())
            .ok_or_else(|| ToolError::missing("parent", &path.display().to_string()))?;
        parents.push((pid, parent));
    }
    let mut found = vec![root];
    let mut index = 0;
    while let Some(&parent) = found.get(index) {
        found.extend(
            parents
                .iter()
                .filter(|(_, other)| *other == parent)
                .map(|(pid, _)| *pid),
        );
        index += 1;
    }
    Ok(found.split_off(1))
}

/// The value of a read under `/proc`; `None` for a process or thread that exited meanwhile.
fn gone<T>(result: io::Result<T>, path: &Path) -> Result<Option<T>, ToolError> {
    match result {
        Ok(value) => Ok(Some(value)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(source) => Err(ToolError::io(path, source)),
    }
}

/// The number that names a directory under `/proc`.
fn number(name: &std::ffi::OsStr) -> Option<i32> {
    name.to_str().and_then(|name| name.parse().ok())
}
