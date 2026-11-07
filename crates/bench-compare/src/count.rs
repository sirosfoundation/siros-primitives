//! The bench executable: it checks every case of a [`Suite`], measures every case with the
//! counters of one pinned CPU, and prints every cell as one JSON line, then the sweep. A
//! WebAssembly bench sends its lines to the driver that runs it in a browser, which counts and
//! adds the sweep.
//!
//! A sweep first checks every case and sizes its runs: `n` starts at 16 operations and doubles
//! until a run of `n` operations, counted after an uncounted run of as many, takes [`TARGET`]
//! cycles, or until the operand sets of a run of `2n` would outgrow 256 KiB. It then
//! runs [`REPETITIONS`] repetitions of every case, interleaved: repetition `r` of every case runs
//! before repetition `r + 1` of any case. A repetition of a case draws fresh operand sets for
//! every run, outside the counted region, and counts a run of `n` operations and a run of `2n`
//! through [`run_batch`], each right after an uncounted run of `n` operations. A case with a
//! chain form does the same through [`run_chain`] once per [`Barrier`] that holds its result,
//! each with its own `n`. The
//! figure per operation of an event is the fewest that a `2n` run counts less the fewest that an
//! `n` run counts, divided by `n`: the fixed cost of a run cancels. A disturbance only adds
//! counts, so the fewest of each length is its undisturbed run (Chen and Revels, "Robust
//! benchmarking in noisy environments", 2016). A run shares its core with the sibling hardware
//! thread: a one-`add` chain step took 1 cycle while the sibling idled and 2 while it ran, and a
//! median of paired differences fell to 0.06 cycles where the two runs of a pair met different
//! siblings. The latency of a case is its chain with the fewest cycles. A case
//! whose `2n` runs count no more cycles than its `n` runs, or whose chain step counts fewer cycles
//! than [`FEWEST_STEP_CYCLES`], runs [`REPETITIONS`] more repetitions, up to [`RETRIES`] times: a
//! disturbance that hits every short run inflates the fewest of the short runs alone, and more
//! repetitions give the short runs more chances to run undisturbed. A case that stays so stops the
//! sweep. A WebAssembly bench sizes and repeats its runs by its own
//! [`Protocol`].

use std::mem::size_of;
use std::process::ExitCode;

#[cfg(all(target_arch = "x86_64", target_feature = "avx2"))]
use crate::barrier::Ymm;
use crate::barrier::{Barrier, Memory, touch};
#[cfg(target_arch = "x86_64")]
use crate::barrier::{Gpr, Xmm};
use crate::counters::{Counters, Counts};
use crate::error::Error;
#[cfg(target_arch = "wasm32")]
use crate::fingerprint::Fingerprint;
use crate::record::{Cell, Class, Count, Events, Latency, Line, Outcome};
#[cfg(target_arch = "wasm32")]
use crate::record::{Listed, ListedCall, ListedCase, ListedChain, Overhead};
#[cfg(target_arch = "wasm32")]
use crate::selection::Plan;
#[cfg(target_arch = "wasm32")]
use crate::suite::Limbs;
use crate::suite::{Call, CaseId, Chain, Function, Inputs, Rng, Suite, Visitor, visit};
#[cfg(not(target_arch = "wasm32"))]
use crate::suite::{run_batch, run_chain};

/// The counted repetitions of every case.
pub const REPETITIONS: usize = 31;

/// The rounds of [`REPETITIONS`] more repetitions that a case with an implausible figure runs.
pub const RETRIES: usize = 3;

/// The cycles that a native short run takes at least, unless its operations reach their cap.
pub const TARGET: u64 = 1 << 20;

/// The fewest cycles that a chain step may count. A step of one instruction takes one cycle, and
/// its figure falls up to 5% below that on a quiet machine; a chain that the compiler regrouped
/// into pairs takes half a cycle per step or less.
pub const FEWEST_STEP_CYCLES: f64 = 0.75;

/// The CPU of the measuring thread, unless `--cpu` names another.
pub const CPU: usize = 15;

/// The seed of the operand sets of a sweep.
const SEED: u64 = 0x5eed_0fc0_ffee;

/// The most memory that the operand sets of a long run take.
const FOOTPRINT: usize = 256 * 1024;

/// How a sweep sizes its runs.
struct Protocol {
    /// Whether a run passes several times over its operand sets once they fill [`FOOTPRINT`],
    /// until it is long enough.
    passes: bool,
    /// The counted runs of each size that sizes a case, whose fewest count.
    sizing: usize,
    /// The count from which a run is long enough.
    long: Length,
}

/// The count from which a run is long enough.
#[derive(Clone, Copy)]
enum Length {
    /// Core cycles.
    Cycles(u64),
    /// Retired instructions, whatever the cycles that other work adds.
    Instructions(u64),
}

/// The native protocol: one pass over the operand sets, sized until a run takes [`TARGET`]
/// cycles.
const NATIVE_PROTOCOL: Protocol = Protocol {
    passes: false,
    sizing: 1,
    long: Length::Cycles(TARGET),
};

/// The protocol of a WebAssembly bench. A count in a browser costs two requests to the driver,
/// whose cycles varied by 4,000 to 8,000 between counts on a loaded machine, against the 8,192
/// cycles of a long run of 8,192 one-cycle `add`s: a run passes over its operand sets, which stay
/// in the L2 cache as natively, until it counts 4 million instructions, a million cycles at 4
/// instructions per cycle. Cycles sized it badly: at a load of 65 a sizing run of a GF(2⁶⁴) add
/// chain counted 2.3 times the cycles of the runs that followed, and the fewest of nine such runs
/// ended the sizing early.
const WASM_PROTOCOL: Protocol = Protocol {
    passes: true,
    sizing: 3,
    long: Length::Instructions(4 << 20),
};

/// The protocol of this build.
const PROTOCOL: Protocol = if cfg!(target_arch = "wasm32") {
    WASM_PROTOCOL
} else {
    NATIVE_PROTOCOL
};

impl Length {
    /// Whether the fewest of a run's counts reach this length.
    const fn reached(self, fewest: &Counts) -> bool {
        match self {
            Self::Cycles(cycles) => fewest.cycles >= cycles,
            Self::Instructions(instructions) => fewest.instructions >= instructions,
        }
    }
}

/// The fewest times the noise of the fixed cost of a count that a difference of a case counts,
/// where the counts reach the counters through requests: its figure then moves by at most 2%
/// with that cost.
const SIGNAL: f64 = 50.0;

/// The operations of the first sizing run, and the fewest of a short run.
const FEWEST: usize = 16;

/// The empty counts of a WebAssembly bench before its sweep.
#[cfg(target_arch = "wasm32")]
const GLUE_COUNTS: usize = 1000;

/// The sets of [`REPETITIONS`] empty counts whose fewest counts give the noise of a figure, in
/// pairs.
#[cfg(target_arch = "wasm32")]
const NOISE_SETS: usize = 8;

/// Runs the bench executable of `S`, named `name` in its messages, with the process arguments:
/// `--cpu <cpu>` names the CPU of the measuring thread.
#[cfg(not(target_arch = "wasm32"))]
#[must_use]
pub fn main<S: Suite>(name: &str) -> ExitCode {
    // `cargo bench` passes `--bench`.
    let args: Vec<String> = std::env::args()
        .skip(1)
        .filter(|arg| arg != "--bench")
        .collect();
    let cpu = match args.as_slice() {
        [] => Ok(CPU),
        [flag, cpu] if flag == "--cpu" => cpu
            .parse::<usize>()
            .map_err(|source| Error::number("CPU", cpu, source)),
        _ => Err(Error::BenchUsage {
            program: name.to_owned(),
        }),
    };
    match cpu.and_then(sweep::<S>) {
        Ok(lines) => {
            for line in lines {
                println!("{line}");
            }
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{name}: {error}");
            ExitCode::FAILURE
        }
    }
}

/// Runs the WebAssembly bench of `S`, named `name` in its messages, in the page of the driver: it
/// measures the cells of the driver's plan and sends the line of each, or it lists its cells; or
/// it sends the reason for which it stops. A panic sends its message before the module traps.
#[cfg(target_arch = "wasm32")]
#[must_use]
pub fn main<S: Suite>(name: &str) -> ExitCode {
    std::panic::set_hook(Box::new(|info| crate::host::fail(&info.to_string())));
    let plan = crate::host::plan()
        .map_err(Error::Plan)
        .and_then(|text| Plan::parse(&text).map_err(Error::Selection));
    let lines = plan.and_then(|plan| {
        let mut counters = Counters::open()?;
        if plan == Plan::List {
            return listing::<S>(&mut counters);
        }
        let overhead = overhead(&mut counters)?;
        let line = Line::<S::Key>::Overhead(overhead);
        let mut lines = vec![serde_json::to_string(&line).map_err(Error::Cells)?];
        lines.extend(cells::<S>(&mut counters, Some(overhead.noise), plan)?);
        Ok(lines)
    });
    match lines {
        Ok(lines) => {
            for line in lines {
                crate::host::emit(&line);
            }
            ExitCode::SUCCESS
        }
        Err(error) => {
            crate::host::fail(&format!("{name}: {error}"));
            ExitCode::FAILURE
        }
    }
}

/// The cost of a count of an empty region in the page, from the last [`NOISE_SETS`] sets of
/// [`REPETITIONS`] of [`GLUE_COUNTS`] counts. The earlier counts bring the page's code of a count
/// to the engine's optimizing tier: its first count took 1.5 million cycles, its later ones 5,000.
#[cfg(target_arch = "wasm32")]
fn overhead(counters: &mut Counters) -> Result<Overhead, Error> {
    let mut counts = Vec::with_capacity(GLUE_COUNTS);
    for _ in 0..GLUE_COUNTS {
        let (runs, counted) = counters.count(&mut || 0)?;
        ran("an empty count", runs, 0)?;
        counts.push(counted);
    }
    let last = &counts[GLUE_COUNTS - NOISE_SETS * REPETITIONS..];
    let median = |event: fn(&Counts) -> u64| {
        let mut values: Vec<u64> = last.iter().map(event).collect();
        let middle = values.len() / 2;
        let (_, median, _) = values.select_nth_unstable(middle);
        *median as f64
    };
    // A figure takes the fewest counts of two sets of runs; the fixed cost of a count moves it by
    // the difference of the fewest counts of two sets of empty counts.
    let fewest: Vec<f64> = last
        .as_chunks::<REPETITIONS>()
        .0
        .iter()
        .map(|set| set.iter().map(|counts| counts.cycles).min().unwrap_or(0) as f64)
        .collect();
    let noise = fewest
        .as_chunks::<2>()
        .0
        .iter()
        .map(|[first, second]| (second - first).abs())
        .fold(0.0, f64::max);
    Ok(Overhead {
        cycles: median(|counts| counts.cycles),
        instructions: median(|counts| counts.instructions),
        noise,
    })
}

/// Checks and measures every case on `cpu`; returns the line of every cell, in the order of
/// [`visit`], then the line of the sweep.
#[cfg(not(target_arch = "wasm32"))]
fn sweep<S: Suite>(cpu: usize) -> Result<Vec<String>, Error> {
    crate::counters::pin(cpu)?;
    let mut counters = Counters::open()?;
    let window = crate::window::Window::open(cpu)?;
    let mut lines = match cells::<S>(&mut counters, None) {
        Ok(lines) => lines,
        Err(error) => return Err(error.at_load(crate::window::load()?)),
    };
    let line = Line::<S::Key>::Sweep(window.close()?);
    lines.push(serde_json::to_string(&line).map_err(Error::Cells)?);
    Ok(lines)
}

/// The operand sets of a row whose hash a listing sends.
#[cfg(target_arch = "wasm32")]
const LISTED_DRAWS: usize = 16;

/// The line of every cell of `S`, in the order of [`visit`], with what its case runs. No case runs.
#[cfg(target_arch = "wasm32")]
fn listing<S: Suite>(counters: &mut Counters) -> Result<Vec<String>, Error> {
    let mut measuring = visit::<S, _>(Measuring {
        counters,
        pass: 0,
        rng: Rng(SEED),
        cells: Vec::new(),
        cases: Vec::new(),
        settled: Vec::new(),
        next: 0,
        error: None,
        plan: Plan::List,
        listed: Vec::new(),
        operands: S::operands,
    });
    if let Some(error) = measuring.error.take() {
        return Err(error);
    }
    let Measuring { cells, listed, .. } = measuring;
    let mut lines = Vec::with_capacity(cells.len());
    for (id, call) in cells {
        let name = id.to_string();
        let call = match call {
            Call::Case(_) => ListedCall::Case(
                listed
                    .iter()
                    .find(|(listed, _)| *listed == name)
                    .map(|(_, case)| case.clone())
                    .ok_or_else(|| Error::missing(&name, "the listed cases"))?,
            ),
            Call::Absent(reason) => ListedCall::Absent(reason.to_owned()),
        };
        let line = Listed {
            op: id.op.to_string(),
            row: id.key.to_string(),
            key: id.key,
            library: id.lib.to_owned(),
            call,
        };
        lines.push(serde_json::to_string(&line).map_err(Error::Cells)?);
    }
    Ok(lines)
}

/// The hash of the operand sets of the row `key` of `op` that a fixed seed draws.
#[cfg(target_arch = "wasm32")]
fn operands_hash<O, K>(operands: fn(O, &K, &mut Rng) -> Vec<Limbs>, op: &O, key: &K) -> String
where
    O: Clone,
{
    let mut rng = Rng(SEED);
    let mut state = Fingerprint::new();
    for _ in 0..LISTED_DRAWS {
        for operand in operands(op.clone(), key, &mut rng) {
            state.update(&operand.len().to_le_bytes());
            for limb in operand {
                state.update(&limb.to_le_bytes());
            }
        }
    }
    format!("{:032x}", state.finish())
}

/// The index in the module's table of the function `body`, the loop of a case that runs `_run`: the
/// function that [`apart`] calls.
#[cfg(target_arch = "wasm32")]
fn entry<I, F>(body: fn(&[I], F) -> usize, _run: &F) -> usize {
    body as usize
}

/// The operand sets of the pass `pass` of the case `id`: the same whatever else the run measures.
#[cfg(target_arch = "wasm32")]
fn stream(id: &str, pass: usize) -> Rng {
    let mut state = Fingerprint::new();
    state.update(id.as_bytes());
    state.update(&pass.to_le_bytes());
    Rng(SEED ^ state.finish() as u64)
}

/// Checks and measures every case with `counters` that `plan` holds; returns the line of each cell,
/// in the order of [`visit`]. `noise` is the noise of the fixed cost of a count where the counts
/// reach the counters through requests; such a difference counts at least [`SIGNAL`] times as many
/// cycles.
fn cells<S: Suite>(
    counters: &mut Counters,
    noise: Option<f64>,
    #[cfg(target_arch = "wasm32")] plan: Plan,
) -> Result<Vec<String>, Error> {
    // Pass 0 checks and sizes the cases; pass `r` runs repetition `r`.
    let mut measuring = Measuring {
        counters,
        pass: 0,
        rng: Rng(SEED),
        cells: Vec::new(),
        cases: Vec::new(),
        settled: Vec::new(),
        next: 0,
        error: None,
        #[cfg(target_arch = "wasm32")]
        plan,
        #[cfg(target_arch = "wasm32")]
        listed: Vec::new(),
        #[cfg(target_arch = "wasm32")]
        operands: S::operands,
    };
    let mut passes = 0..=REPETITIONS;
    let mut retries = 0;
    loop {
        for pass in passes {
            measuring = visit::<S, _>(Measuring {
                pass,
                next: 0,
                ..measuring
            });
            if let Some(error) = measuring.error.take() {
                return Err(error);
            }
        }
        let settled: Vec<bool> = measuring
            .cases
            .iter()
            .map(|case| case.settled(noise))
            .collect();
        if retries == RETRIES || settled.iter().all(|settled| *settled) {
            break;
        }
        retries += 1;
        measuring.settled = settled;
        passes = retries * REPETITIONS + 1..=(retries + 1) * REPETITIONS;
    }
    #[cfg(target_arch = "wasm32")]
    measuring.check_plan()?;
    let Measuring {
        cells,
        cases,
        #[cfg(target_arch = "wasm32")]
        plan,
        ..
    } = measuring;
    let mut lines = Vec::with_capacity(cells.len() + 1);
    for (id, call) in &cells {
        #[cfg(target_arch = "wasm32")]
        if !plan.holds(id) {
            continue;
        }
        let name = id.to_string();
        let outcome = match call {
            Call::Case(function) => {
                let case = cases
                    .iter()
                    .find(|case| case.id == name)
                    .ok_or_else(|| Error::missing(&name, "the measured cases"))?;
                Outcome::Counted(case.count(function, noise)?)
            }
            Call::Absent(reason) => Outcome::Absent((*reason).to_owned()),
        };
        let line = Line::Cell(Cell {
            op: id.op.to_string(),
            key: id.key.clone(),
            library: id.lib.to_owned(),
            outcome,
        });
        lines.push(serde_json::to_string(&line).map_err(Error::Cells)?);
    }
    Ok(lines)
}

/// The most operand sets of a short run of a case whose operand sets are `I`: as many as fill
/// [`FOOTPRINT`] with the operand sets of a long run, and at least [`FEWEST`].
fn cap<I>() -> usize {
    (FOOTPRINT / (2 * size_of::<I>().max(1))).max(FEWEST)
}

/// The shape of a short run: `passes` passes over `sets` fresh operand sets. A long run passes as
/// many times over twice as many sets.
#[derive(Clone, Copy, Debug)]
struct Run {
    sets: usize,
    passes: usize,
}

impl Run {
    /// The operations of the run.
    const fn operations(self) -> usize {
        self.sets * self.passes
    }

    /// The long run of this short run.
    const fn long(self) -> Self {
        Self {
            sets: 2 * self.sets,
            passes: self.passes,
        }
    }
}

/// The counted runs of one form of a case.
struct Form {
    /// The shape of a short run.
    run: Run,
    /// The counts of the short run and of the long run of every repetition.
    repetitions: Vec<[Counts; 2]>,
}

impl Form {
    /// The events per operation of the form `name`: the fewest that a long run counts less the
    /// fewest that a short run counts, over the operations of a short run. Long runs whose fewest
    /// cycles are no more than the short runs' are an [`Error::Flat`], and a difference under
    /// [`SIGNAL`] times `noise` an [`Error::Noisy`].
    fn events(&self, name: &str, noise: Option<f64>) -> Result<Events, Error> {
        let fewest = |value: &dyn Fn(&[Counts; 2]) -> u64| -> Result<u64, Error> {
            self.repetitions
                .iter()
                .map(value)
                .min()
                .ok_or_else(|| Error::missing("repetitions", name))
        };
        let difference = |event: fn(&Counts) -> u64| -> Result<f64, Error> {
            let long = fewest(&|[_, long]| event(long))?;
            let short = fewest(&|[short, _]| event(short))?;
            Ok(long as f64 - short as f64)
        };
        let operations = self.run.operations();
        let per = |event| Ok::<f64, Error>(difference(event)? / operations as f64);
        let cycles = difference(|counts| counts.cycles)?;
        if cycles <= 0.0 {
            return Err(Error::Flat {
                case: name.to_owned(),
                operations,
            });
        }
        if let Some(noise) = noise.filter(|noise| cycles < SIGNAL * noise) {
            let spread = |run: usize| {
                let mut values: Vec<u64> = self
                    .repetitions
                    .iter()
                    .map(|counts| counts[run].cycles)
                    .collect();
                values.sort_unstable();
                format!(
                    "fewest {}, median {}, most {}",
                    values[0],
                    values[values.len() / 2],
                    values[values.len() - 1]
                )
            };
            return Err(Error::Noisy {
                case: name.to_owned(),
                cycles,
                noise,
                short: spread(0),
                long: spread(1),
            });
        }
        let cycles = cycles / operations as f64;
        Ok(Events {
            operations,
            cycles,
            instructions: per(|counts| counts.instructions)?,
            branch_misses: per(|counts| counts.branch_misses)?,
        })
    }
}

/// The counted runs of one chain form of a case.
struct ChainForm {
    /// Where the chain holds its value between steps.
    class: Class,
    form: Form,
}

/// One measured case.
struct Case {
    id: String,
    batch: Form,
    /// The chain forms, one per barrier; none for a case without a chain form.
    chains: Vec<ChainForm>,
}

impl Case {
    /// Whether every form of the case has a plausible figure: its long runs count more cycles
    /// than its short runs, and each chain step at least [`FEWEST_STEP_CYCLES`].
    fn settled(&self, noise: Option<f64>) -> bool {
        self.batch.events(&self.id, noise).is_ok()
            && self.chains.iter().all(|chain| {
                chain
                    .form
                    .events(&self.id, noise)
                    .is_ok_and(|events| events.cycles >= FEWEST_STEP_CYCLES)
            })
    }

    /// The cell of the case, which runs `function`: its latency is its chain form with the fewest
    /// cycles per step. A chain step under [`FEWEST_STEP_CYCLES`], in any form, is an
    /// [`Error::Reassociated`].
    fn count(&self, function: &Function, noise: Option<f64>) -> Result<Count, Error> {
        let latencies = self
            .chains
            .iter()
            .map(|chain| {
                let name = format!("{} chain in {}", self.id, chain.class);
                let events = chain.form.events(&name, noise)?;
                if events.cycles < FEWEST_STEP_CYCLES {
                    return Err(Error::Reassociated {
                        case: name,
                        cycles: events.cycles,
                        repetitions: chain.form.repetitions.len(),
                        load: None,
                    });
                }
                Ok(Latency {
                    class: chain.class,
                    events,
                })
            })
            .collect::<Result<Vec<_>, Error>>()?;
        Ok(Count {
            function: function.name.to_owned(),
            deviation: function.deviation.map(str::to_owned),
            batch: self.batch.events(&self.id, noise)?,
            chain: latencies
                .into_iter()
                .min_by(|a, b| a.events.cycles.total_cmp(&b.events.cycles)),
        })
    }
}

/// The visitor of one pass of a sweep: pass 0 checks and sizes every case; pass `r` measures
/// repetition `r` of every case.
struct Measuring<'a, O, K> {
    counters: &'a mut Counters,
    pass: usize,
    /// The generator of the sweep's operand sets.
    rng: Rng,
    /// Every cell of the comparison, in the order of [`visit`], noted by pass 0.
    cells: Vec<(CaseId<O, K>, Call)>,
    cases: Vec<Case>,
    /// Per case, whether the passes past the first [`REPETITIONS`] skip it; empty before them.
    settled: Vec<bool>,
    /// The case that the pass visits next.
    next: usize,
    /// The first failure of the pass, which ends it.
    error: Option<Error>,
    /// What this run does.
    #[cfg(target_arch = "wasm32")]
    plan: Plan,
    /// What each case runs, by the name of the case, in a run that lists its cells.
    #[cfg(target_arch = "wasm32")]
    listed: Vec<(String, ListedCase)>,
    /// The operands of a row: [`Suite::operands`].
    #[cfg(target_arch = "wasm32")]
    operands: fn(O, &K, &mut Rng) -> Vec<Limbs>,
}

#[cfg(target_arch = "wasm32")]
impl Plan {
    /// Whether the plan holds the cell `id`.
    fn holds<O: std::fmt::Display, K: std::fmt::Display>(&self, id: &CaseId<O, K>) -> bool {
        match self {
            Self::All | Self::List => true,
            Self::Only(selection) => {
                selection.holds(&id.op.to_string(), &id.key.to_string(), id.lib)
            }
        }
    }
}

#[cfg(target_arch = "wasm32")]
impl<O, K> Measuring<'_, O, K> {
    /// Fails on a pattern of the selection that matched no cell of the bench.
    fn check_plan(&self) -> Result<(), Error> {
        match &self.plan {
            Plan::All | Plan::List => Ok(()),
            Plan::Only(selection) => selection.check_matched().map_err(Error::Selection),
        }
    }

    /// Lists a chain of the case that it listed last, which runs `run` with `x` held by `B`.
    /// Nothing, for a result that `B` does not hold: that chain compiles to nothing.
    fn list_chain<I: Chain<R>, R, B: Barrier, F: Fn(&I) -> R>(
        &mut self,
        id: &str,
        run: &F,
    ) -> Result<(), Error> {
        if !const { B::LEAST <= size_of::<R>() && size_of::<R>() <= B::MOST } {
            return Ok(());
        }
        let entry = entry(crate::suite::run_chain::<I, R, B>, run);
        self.listed
            .last_mut()
            .filter(|(listed, _)| listed == id)
            .ok_or_else(|| Error::missing(id, "the listed cases"))?
            .1
            .chains
            .push(ListedChain {
                class: B::CLASS,
                entry,
            });
        Ok(())
    }
}

#[cfg(target_arch = "wasm32")]
impl<O: Clone + std::fmt::Display, K: Clone + std::fmt::Display> Measuring<'_, O, K> {
    /// Lists the case `id`: its batch loop, which runs `run` on `inputs`. Its chains follow.
    fn list_case<I, R, F: Fn(&I) -> R>(
        &mut self,
        id: &CaseId<O, K>,
        inputs: &Inputs<'_, I>,
        run: &F,
    ) -> Result<(), Error> {
        let function = self
            .cells
            .last()
            .and_then(|(_, call)| match call {
                Call::Case(function) => Some(*function),
                Call::Absent(_) => None,
            })
            .ok_or_else(|| Error::missing(&id.to_string(), "the noted cells"))?;
        let case = ListedCase {
            function: function.name.to_owned(),
            deviation: function.deviation.map(str::to_owned),
            batch: entry(crate::suite::run_batch, run),
            chains: Vec::new(),
            draw: inputs.builders.draw,
            setup: inputs.builders.setup,
            operands: operands_hash(self.operands, &id.op, &id.key),
            captures: crate::host::hex(run).map_err(Error::Plan)?,
        };
        self.listed.push((id.to_string(), case));
        Ok(())
    }
}

/// Fails unless a run of `count` operations of the case `id` ran `runs`.
fn ran(id: &str, runs: usize, count: usize) -> Result<(), Error> {
    if runs == count {
        Ok(())
    } else {
        Err(Error::Runs {
            case: id.to_owned(),
            runs,
            count,
        })
    }
}

/// One form of a case: the code that each case compiles. The rest of a sweep reaches it through
/// `dyn`, so it compiles once.
trait Trial {
    /// Runs the form in the shape `run` on fresh inputs that `rng` draws, counted by `counters`,
    /// right after an uncounted run in the shape `warm` on other fresh inputs; returns the number
    /// of uncounted and counted runs and the counts.
    fn counted(
        &self,
        rng: &mut Rng,
        run: Run,
        warm: Run,
        counters: &mut Counters,
    ) -> Result<([usize; 2], Counts), Error>;
}

/// `body` on `batch` through a function pointer: V8 inlines a direct call of up to 500 wire bytes,
/// so the code of a short loop took the registers its caller left, and the 128-bit product took
/// Chrome 6.6 cycles against 4.7.
#[cfg(target_arch = "wasm32")]
#[inline(always)]
fn apart<I, F>(body: fn(&[I], F) -> usize, batch: &[I], run: F) -> usize {
    core::hint::black_box(body)(batch, run)
}

/// [`crate::suite::run_batch`], apart from the code that counts it.
#[cfg(target_arch = "wasm32")]
#[inline(always)]
fn run_batch<I, R>(batch: &[I], run: impl Fn(&I) -> R) -> usize {
    apart(crate::suite::run_batch, batch, run)
}

/// [`crate::suite::run_chain`], apart from the code that counts it.
#[cfg(target_arch = "wasm32")]
#[inline(always)]
fn run_chain<I: Chain<R>, R, B: Barrier>(batch: &[I], run: impl Fn(&I) -> R) -> usize {
    apart(crate::suite::run_chain::<I, R, B>, batch, run)
}

/// A form of a case: `region` runs a batch of the inputs that `inputs` draws.
struct Region<'a, I, F> {
    inputs: &'a Inputs<'a, I>,
    region: F,
}

impl<I, F: Fn(&[I]) -> usize> Region<'_, I, F> {
    /// Runs `region` `passes` times over `batch`; returns the number of runs.
    fn passes(&self, batch: &[I], passes: usize) -> usize {
        (0..passes).map(|_| (self.region)(batch)).sum()
    }
}

impl<I, F: Fn(&[I]) -> usize> Trial for Region<'_, I, F> {
    /// The uncounted run follows both draws, so the tables and the code of the form are as warm
    /// for every counted run. A GF(2¹⁶) inverse by a 128 KiB table takes 1.7 cycles; when the
    /// uncounted run came before the draws, they evicted more of that table before a long run than
    /// before a short one, and the long runs counted no more cycles.
    fn counted(
        &self,
        rng: &mut Rng,
        run: Run,
        warm: Run,
        counters: &mut Counters,
    ) -> Result<([usize; 2], Counts), Error> {
        let warming = self.inputs.draw(rng, warm.sets);
        let batch = self.inputs.draw(rng, run.sets);
        let warmed = self.passes(&warming, warm.passes);
        #[cfg(target_arch = "wasm32")]
        let batch = crate::barrier::AtPage::of(batch).map_err(Error::OperandSets)?;
        touch(&batch);
        let (runs, counts) = counters.count(&mut || self.passes(&batch, run.passes))?;
        Ok(([warmed, runs], counts))
    }
}

impl<O, K> Measuring<'_, O, K> {
    /// The counts of `trial` in the shape `run` on fresh inputs, right after an uncounted run in
    /// the shape `warm` on others.
    fn counted(
        &mut self,
        trial: &dyn Trial,
        run: Run,
        warm: Run,
        id: &str,
    ) -> Result<Counts, Error> {
        let ([warmed, runs], counts) = trial.counted(&mut self.rng, run, warm, self.counters)?;
        ran(id, warmed, warm.operations())?;
        ran(id, runs, run.operations())?;
        Ok(counts)
    }

    /// The form of a case that `trial` runs, with the shape of its short run: one pass over 16
    /// operand sets, doubled until the fewest counts of the protocol's sizing runs, each after an
    /// uncounted run of the same shape, reach its length, or until `cap` sets. A protocol with
    /// passes then doubles the passes over `cap` sets until the run reaches that length.
    fn size(&mut self, trial: &dyn Trial, cap: usize, id: &str) -> Result<Form, Error> {
        let mut run = Run {
            sets: FEWEST,
            passes: 1,
        };
        loop {
            let mut fewest = Counts {
                cycles: u64::MAX,
                instructions: u64::MAX,
                branch_misses: u64::MAX,
            };
            for _ in 0..PROTOCOL.sizing {
                let counts = self.counted(trial, run, run, id)?;
                fewest = Counts {
                    cycles: fewest.cycles.min(counts.cycles),
                    instructions: fewest.instructions.min(counts.instructions),
                    branch_misses: fewest.branch_misses.min(counts.branch_misses),
                };
            }
            if PROTOCOL.long.reached(&fewest) || (run.sets >= cap && !PROTOCOL.passes) {
                return Ok(Form {
                    run,
                    repetitions: Vec::new(),
                });
            }
            if run.sets < cap {
                run.sets = (2 * run.sets).min(cap);
            } else {
                run.passes *= 2;
            }
        }
    }

    /// Runs one repetition of a form through `trial`: a run of the shape `run` and one of its
    /// long shape counted, each on fresh inputs right after an uncounted run of the shape `run`.
    fn repetition(&mut self, trial: &dyn Trial, run: Run, id: &str) -> Result<[Counts; 2], Error> {
        Ok([
            self.counted(trial, run, run, id)?,
            self.counted(trial, run.long(), run, id)?,
        ])
    }

    /// The index of the case `id` in this pass, which pass 0 took note of.
    fn case(&mut self, id: &str) -> Result<usize, Error> {
        let index = self.next;
        self.next += 1;
        match self.cases.get(index) {
            Some(case) if case.id == id => Ok(index),
            Some(_) | None => Err(Error::missing(id, "the cases of the first pass")),
        }
    }

    /// Whether this pass skips the case `index`: a retry, for a case whose figures are plausible.
    fn skips(&self, index: usize) -> bool {
        self.settled.get(index).copied().unwrap_or(false)
    }

    /// Keeps the first failure.
    fn fail(self, result: Result<(), Error>) -> Self {
        match result {
            Ok(()) => self,
            Err(error) => Self {
                error: Some(error),
                ..self
            },
        }
    }

    /// Sizes, in pass 0, the batch form of the case `id`, which `trial` runs; in pass `r`, runs
    /// its repetition `r`.
    fn batch(&mut self, id: String, trial: &dyn Trial, cap: usize) -> Result<(), Error> {
        #[cfg(target_arch = "wasm32")]
        {
            self.rng = stream(&id, self.pass);
        }
        if self.pass == 0 {
            let batch = self.size(trial, cap, &id)?;
            self.cases.push(Case {
                id,
                batch,
                chains: Vec::new(),
            });
            return Ok(());
        }
        let index = self.case(&id)?;
        if self.skips(index) {
            return Ok(());
        }
        let run = self.cases[index].batch.run;
        let counts = self.repetition(trial, run, &id)?;
        self.cases[index].batch.repetitions.push(counts);
        Ok(())
    }

    /// The index of the case `id`, whose batch form the pass measured just before.
    fn last(&self, id: &str) -> Result<usize, Error> {
        if self.pass == 0 {
            self.cases.len().checked_sub(1)
        } else {
            self.next.checked_sub(1)
        }
        .filter(|index| self.cases.get(*index).is_some_and(|case| case.id == id))
        .ok_or_else(|| Error::missing(id, "the cases of the first pass"))
    }

    /// Sizes, in pass 0, the chain form of the case `index` in `class`, which `trial` runs; in
    /// pass `r`, runs its repetition `r`.
    fn chain_form(
        &mut self,
        index: usize,
        class: Class,
        trial: &dyn Trial,
        cap: usize,
        id: &str,
    ) -> Result<(), Error> {
        if self.pass == 0 {
            let form = self.size(trial, cap, id)?;
            self.cases[index].chains.push(ChainForm { class, form });
            return Ok(());
        }
        if self.skips(index) {
            return Ok(());
        }
        let missing = || Error::missing(&format!("chain in {class}"), id);
        let run = self.cases[index]
            .chains
            .iter()
            .find(|chain| chain.class == class)
            .map(|chain| chain.form.run)
            .ok_or_else(missing)?;
        let counts = self.repetition(trial, run, id)?;
        self.cases[index]
            .chains
            .iter_mut()
            .find(|chain| chain.class == class)
            .ok_or_else(missing)?
            .form
            .repetitions
            .push(counts);
        Ok(())
    }
}

impl<O: Clone + std::fmt::Display, K: Clone + std::fmt::Display> Visitor<O, K>
    for Measuring<'_, O, K>
{
    fn note(&mut self, id: &CaseId<O, K>, call: &Call) {
        if self.pass == 0 {
            self.cells.push((id.clone(), *call));
            #[cfg(target_arch = "wasm32")]
            if let Plan::Only(selection) = &mut self.plan {
                selection.note(&id.op.to_string(), &id.key.to_string(), id.lib);
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn wants(&self, _id: &CaseId<O, K>) -> bool {
        self.error.is_none()
    }

    #[cfg(target_arch = "wasm32")]
    fn wants(&self, id: &CaseId<O, K>) -> bool {
        self.error.is_none() && self.plan.holds(id)
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn checks(&self, _id: &CaseId<O, K>) -> bool {
        self.pass == 0
    }

    #[cfg(target_arch = "wasm32")]
    fn checks(&self, _id: &CaseId<O, K>) -> bool {
        self.pass == 0 && self.plan != Plan::List
    }

    fn measure<I, R>(
        mut self,
        id: &CaseId<O, K>,
        inputs: &Inputs<'_, I>,
        run: impl Fn(&I) -> R + Copy,
    ) -> Self {
        #[cfg(target_arch = "wasm32")]
        if self.plan == Plan::List {
            let result = self.list_case(id, inputs, &run);
            return self.fail(result);
        }
        let trial = Region {
            inputs,
            region: |batch: &[I]| run_batch(batch, run),
        };
        let result = self.batch(id.to_string(), &trial, cap::<I>());
        self.fail(result)
    }

    fn measure_chain<I: Chain<R>, R>(
        mut self,
        id: &CaseId<O, K>,
        inputs: &Inputs<'_, I>,
        run: impl Fn(&I) -> R + Copy,
    ) -> Self {
        if self.error.is_some() {
            return self;
        }
        let name = id.to_string();
        #[cfg(target_arch = "wasm32")]
        if self.plan == Plan::List {
            let result = self
                .list_chain::<I, R, crate::barrier::Locals, _>(&name, &run)
                .and_then(|()| self.list_chain::<I, R, Memory, _>(&name, &run));
            return self.fail(result);
        }
        let result = self
            .last(&name)
            .and_then(|index| self.chains(index, inputs, run, &name));
        self.fail(result)
    }
}

impl<O, K> Measuring<'_, O, K> {
    /// Sizes or runs one repetition of the chain forms of the case `index`: one per [`Barrier`]
    /// that holds its result. A result that no barrier holds is an [`Error::Missing`].
    fn chains<I: Chain<R>, R>(
        &mut self,
        index: usize,
        inputs: &Inputs<'_, I>,
        run: impl Fn(&I) -> R + Copy,
        name: &str,
    ) -> Result<(), Error> {
        #[cfg(target_arch = "x86_64")]
        {
            self.chain::<I, R, Gpr>(index, inputs, run, name)?;
            self.chain::<I, R, Xmm>(index, inputs, run, name)?;
        }
        #[cfg(all(target_arch = "x86_64", target_feature = "avx2"))]
        self.chain::<I, R, Ymm>(index, inputs, run, name)?;
        #[cfg(target_arch = "wasm32")]
        self.chain::<I, R, crate::barrier::Locals>(index, inputs, run, name)?;
        self.chain::<I, R, Memory>(index, inputs, run, name)?;
        if self.cases[index].chains.is_empty() {
            return Err(Error::missing(
                &format!("barrier that holds {} bytes", size_of::<R>()),
                name,
            ));
        }
        Ok(())
    }

    /// Sizes, in pass 0, the chain form of the case `index` whose value `B` holds; in pass `r`,
    /// runs its repetition `r`. Nothing, for a result that `B` does not hold: that chain compiles
    /// to nothing.
    fn chain<I: Chain<R>, R, B: Barrier>(
        &mut self,
        index: usize,
        inputs: &Inputs<'_, I>,
        run: impl Fn(&I) -> R + Copy,
        name: &str,
    ) -> Result<(), Error> {
        if !const { B::LEAST <= size_of::<R>() && size_of::<R>() <= B::MOST } {
            return Ok(());
        }
        let trial = Region {
            inputs,
            region: |batch: &[I]| run_chain::<I, R, B>(batch, run),
        };
        self.chain_form(index, B::CLASS, &trial, cap::<I>(), name)
    }
}
