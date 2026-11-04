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
//! every run, outside the counted region; it runs `n` operations uncounted, then draws the
//! operand sets of a run of `n` operations and of a run of `2n`, and counts both runs back to back
//! through [`run_batch`]. A case with a chain form does the same through [`run_chain`] once per
//! [`Barrier`] that holds its result, each with its own `n`. The figure per operation of an event
//! is the difference of the `2n` count less the `n` count of rank [`RANK`] among the repetitions,
//! from the fewest, divided by `n`: the fixed cost of a run cancels. A run shares its core with
//! the sibling hardware thread, which only adds counts: a one-`add` chain step took 1 cycle while
//! the sibling idled and 2 while it ran. Two runs counted back to back mostly meet the same
//! sibling, so a low difference is a pair that met an idle one (Chen and Revels, "Robust
//! benchmarking in noisy environments", 2016, for the minimum). The rank passes over a few pairs
//! whose runs met different siblings, the case where a median of differences fell to 0.06 cycles
//! per step, and over a few long runs whose operands ran a fast path: a GF(2⁸) square chain that
//! started at 1 took 19 instructions per step against 218. The latency of a case is its chain
//! with the fewest cycles. A case
//! whose `2n` runs count no more cycles than its `n` runs stops the sweep, and so does a chain
//! step under [`FEWEST_STEP_CYCLES`].

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
use crate::record::Overhead;
use crate::record::{Cell, Class, Count, Events, Latency, Line, Outcome};
use crate::suite::{
    Call, CaseId, Chain, Function, Inputs, Rng, Suite, Visitor, run_batch, run_chain, visit,
};

/// The counted repetitions of every case.
pub const REPETITIONS: usize = 31;

/// The rank, from the fewest, of the repetition whose difference a figure takes: the fourth of
/// 31. It passes over three pairs that the sibling thread or a fast path disturbed.
pub const RANK: usize = REPETITIONS / 8;

/// The cycles that a short run takes at least, unless its operations reach their cap.
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

/// Whether a run passes several times over its operand sets once they fill [`FOOTPRINT`], until
/// it takes [`TARGET`] cycles: in a WebAssembly bench. A count in a browser costs two requests to
/// the driver, whose cycles varied by 4,000 to 8,000 between counts on a loaded machine, against
/// the 8,192 cycles of a long run of 8,192 one-cycle `add`s. Passes keep the operand sets in the
/// L2 cache, as natively.
const PASSES: bool = cfg!(target_arch = "wasm32");

/// The counted runs of each size that sizes a case, whose fewest cycles count: one natively. A
/// count in a browser met an interruption of up to 800,000 cycles, which ended the sizing of a
/// case at 16 operations.
const SIZING: usize = if cfg!(target_arch = "wasm32") { 3 } else { 1 };

/// Whether a figure takes the difference of the two runs of each repetition, which run back to
/// back and mostly meet the same sibling thread: natively. In a browser, the two requests of a
/// count part the runs, and a busy sibling slowed a one-`add` chain step to 1.7 cycles in either
/// run alone: the difference of rank 4 gave 0.54 cycles per step, and the difference of the
/// counts of rank 4 of each length 1.01.
const PAIRED: bool = !cfg!(target_arch = "wasm32");

/// The fewest times the noise of a difference of two empty counts that a difference of a case
/// counts, where the counts reach the counters through requests: its figure then moves by at most
/// 2% with the cost of a count.
const SIGNAL: f64 = 50.0;

/// The operations of the first sizing run, and the fewest of a short run.
const FEWEST: usize = 16;

/// The empty counts of a WebAssembly bench before its sweep.
#[cfg(target_arch = "wasm32")]
const GLUE_COUNTS: usize = 1000;

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

/// Runs the WebAssembly bench of `S`, named `name` in its messages, in the page of the driver:
/// it sends the line of every cell to the driver, or the reason for which it stops. A panic sends
/// its message before the module traps.
#[cfg(target_arch = "wasm32")]
#[must_use]
pub fn main<S: Suite>(name: &str) -> ExitCode {
    std::panic::set_hook(Box::new(|info| crate::host::fail(&info.to_string())));
    let measured = Counters::open().and_then(|mut counters| {
        let overhead = overhead(&mut counters)?;
        let line = Line::<S::Key>::Overhead(overhead);
        let mut lines = vec![serde_json::to_string(&line).map_err(Error::Cells)?];
        lines.extend(cells::<S>(&mut counters, Some(overhead.noise))?);
        Ok(lines)
    });
    match measured {
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

/// The cost of a count of an empty region in the page, from the last `2 REPETITIONS` of
/// [`GLUE_COUNTS`] counts. The earlier counts bring the page's code of a count to the engine's
/// optimizing tier: its first count took 1.5 million cycles, its later ones 5,000.
#[cfg(target_arch = "wasm32")]
fn overhead(counters: &mut Counters) -> Result<Overhead, Error> {
    let mut counts = Vec::with_capacity(GLUE_COUNTS);
    for _ in 0..GLUE_COUNTS {
        let (runs, counted) = counters.count(&mut || 0)?;
        ran("an empty count", runs, 0)?;
        counts.push(counted);
    }
    let last = &counts[GLUE_COUNTS - 2 * REPETITIONS..];
    let median = |event: fn(&Counts) -> u64| {
        let mut values: Vec<u64> = last.iter().map(event).collect();
        let (_, median, _) = values.select_nth_unstable(REPETITIONS);
        *median as f64
    };
    let mut differences: Vec<f64> = last
        .as_chunks::<2>()
        .0
        .iter()
        .map(|[first, second]| second.cycles as f64 - first.cycles as f64)
        .collect();
    differences.sort_by(f64::total_cmp);
    Ok(Overhead {
        cycles: median(|counts| counts.cycles),
        instructions: median(|counts| counts.instructions),
        noise: differences[RANK]
            .abs()
            .max(differences[REPETITIONS - 1 - RANK].abs()),
    })
}

/// Checks and measures every case on `cpu`; returns the line of every cell, in the order of
/// [`visit`], then the line of the sweep.
#[cfg(not(target_arch = "wasm32"))]
fn sweep<S: Suite>(cpu: usize) -> Result<Vec<String>, Error> {
    crate::counters::pin(cpu)?;
    let mut counters = Counters::open()?;
    let window = crate::window::Window::open(cpu)?;
    let mut lines = cells::<S>(&mut counters, None)?;
    let line = Line::<S::Key>::Sweep(window.close()?);
    lines.push(serde_json::to_string(&line).map_err(Error::Cells)?);
    Ok(lines)
}

/// Checks and measures every case with `counters`; returns the line of every cell, in the order
/// of [`visit`]. `noise` is the noise of a difference of two counts where the counts reach the
/// counters through requests; such a difference counts at least [`SIGNAL`] times as many cycles.
fn cells<S: Suite>(counters: &mut Counters, noise: Option<f64>) -> Result<Vec<String>, Error> {
    // Pass 0 checks and sizes the cases; pass `r` runs repetition `r`.
    let mut measuring = Measuring {
        counters,
        pass: 0,
        rng: Rng(SEED),
        cells: Vec::new(),
        cases: Vec::new(),
        next: 0,
        error: None,
    };
    for pass in 0..=REPETITIONS {
        measuring = visit::<S, _>(Measuring {
            pass,
            next: 0,
            ..measuring
        });
        if let Some(error) = measuring.error.take() {
            return Err(error);
        }
    }
    let Measuring { cells, cases, .. } = measuring;
    let mut lines = Vec::with_capacity(cells.len() + 1);
    for (id, call) in &cells {
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
    /// The events per operation of the form `name`, over the operations of a short run: the
    /// difference of rank [`RANK`], from the fewest, of the long run less the short run of each
    /// repetition; without [`PAIRED`], the count of rank [`RANK`] of the long runs less that of
    /// the short runs. A cycle difference of zero or less is an [`Error::Flat`], and one under
    /// [`SIGNAL`] times `noise` an [`Error::Noisy`].
    fn events(&self, name: &str, noise: Option<f64>) -> Result<Events, Error> {
        let difference = |event: fn(&Counts) -> u64| -> Result<f64, Error> {
            if self.repetitions.len() <= RANK {
                return Err(Error::missing("repetitions", name));
            }
            let ranked = |mut values: Vec<f64>| {
                let (_, value, _) = values.select_nth_unstable_by(RANK, f64::total_cmp);
                *value
            };
            let count = |run: usize| {
                self.repetitions
                    .iter()
                    .map(|counts| event(&counts[run]) as f64)
                    .collect::<Vec<_>>()
            };
            Ok(if PAIRED {
                ranked(
                    self.repetitions
                        .iter()
                        .map(|[short, long]| event(long) as f64 - event(short) as f64)
                        .collect(),
                )
            } else {
                ranked(count(1)) - ranked(count(0))
            })
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
            return Err(Error::Noisy {
                case: name.to_owned(),
                cycles,
                noise,
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
    /// The case that the pass visits next.
    next: usize,
    /// The first failure of the pass, which ends it.
    error: Option<Error>,
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
    /// Runs the form in the shape `run` on fresh inputs that `rng` draws, outside the counted
    /// region; returns the number of runs.
    fn warm(&self, rng: &mut Rng, run: Run) -> usize;

    /// Runs the form in the shape `run` on fresh inputs that `rng` draws, counted by `counters`;
    /// returns the number of runs and the counts.
    fn counted(
        &self,
        rng: &mut Rng,
        run: Run,
        counters: &mut Counters,
    ) -> Result<(usize, Counts), Error>;

    /// Runs the form in the shape `run` on fresh inputs, then in its long shape, counted by
    /// `counters` back to back: both runs draw their inputs first. Returns the number of runs and
    /// the counts of each.
    fn pair(
        &self,
        rng: &mut Rng,
        run: Run,
        counters: &mut Counters,
    ) -> Result<[(usize, Counts); 2], Error>;
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
    fn warm(&self, rng: &mut Rng, run: Run) -> usize {
        let batch = self.inputs.draw(rng, run.sets);
        touch(&batch);
        self.passes(&batch, run.passes)
    }

    fn counted(
        &self,
        rng: &mut Rng,
        run: Run,
        counters: &mut Counters,
    ) -> Result<(usize, Counts), Error> {
        let batch = self.inputs.draw(rng, run.sets);
        touch(&batch);
        counters.count(&mut || self.passes(&batch, run.passes))
    }

    fn pair(
        &self,
        rng: &mut Rng,
        run: Run,
        counters: &mut Counters,
    ) -> Result<[(usize, Counts); 2], Error> {
        let short = self.inputs.draw(rng, run.sets);
        let long = self.inputs.draw(rng, run.long().sets);
        touch(&long);
        touch(&short);
        Ok([
            counters.count(&mut || self.passes(&short, run.passes))?,
            counters.count(&mut || self.passes(&long, run.passes))?,
        ])
    }
}

impl<O, K> Measuring<'_, O, K> {
    /// Runs `trial` in the shape `run` on fresh inputs, uncounted.
    fn warm(&mut self, trial: &dyn Trial, run: Run, id: &str) -> Result<(), Error> {
        ran(id, trial.warm(&mut self.rng, run), run.operations())
    }

    /// The counts of `trial` in the shape `run` on fresh inputs.
    fn counted(&mut self, trial: &dyn Trial, run: Run, id: &str) -> Result<Counts, Error> {
        let (runs, counts) = trial.counted(&mut self.rng, run, self.counters)?;
        ran(id, runs, run.operations())?;
        Ok(counts)
    }

    /// The form of a case that `trial` runs, with the shape of its short run: one pass over 16
    /// operand sets, doubled until the fewest cycles of [`SIZING`] counted runs after an uncounted
    /// run of the same shape reach [`TARGET`], or until `cap` sets. A WebAssembly bench then
    /// doubles the passes over `cap` sets until the run reaches [`TARGET`].
    fn size(&mut self, trial: &dyn Trial, cap: usize, id: &str) -> Result<Form, Error> {
        let mut run = Run {
            sets: FEWEST,
            passes: 1,
        };
        loop {
            self.warm(trial, run, id)?;
            let mut cycles = u64::MAX;
            for _ in 0..SIZING {
                cycles = cycles.min(self.counted(trial, run, id)?.cycles);
            }
            if cycles >= TARGET || (run.sets >= cap && !PASSES) {
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

    /// Runs one repetition of a form through `trial`: a run of the shape `run` uncounted, then a
    /// run of that shape and one of its long shape counted back to back, each run on fresh
    /// inputs.
    fn repetition(&mut self, trial: &dyn Trial, run: Run, id: &str) -> Result<[Counts; 2], Error> {
        self.warm(trial, run, id)?;
        let [(short_runs, short), (long_runs, long)] =
            trial.pair(&mut self.rng, run, self.counters)?;
        ran(id, short_runs, run.operations())?;
        ran(id, long_runs, run.long().operations())?;
        Ok([short, long])
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
        }
    }

    fn wants(&self, _id: &CaseId<O, K>) -> bool {
        self.error.is_none()
    }

    fn checks(&self, _id: &CaseId<O, K>) -> bool {
        self.pass == 0
    }

    fn measure<I, R>(
        mut self,
        id: &CaseId<O, K>,
        inputs: &Inputs<'_, I>,
        run: impl Fn(&I) -> R + Copy,
    ) -> Self {
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
