//! The bench executable: it checks every case of a [`Suite`], measures every case with the
//! counters of one pinned CPU, and prints every cell as one JSON line, then the sweep.
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
//! whose `2n` runs count no more cycles than its `n` runs stops the sweep, and so does a chain
//! step under [`FEWEST_STEP_CYCLES`].

use std::fs;
use std::mem::size_of;
use std::path::Path;
use std::process::ExitCode;

#[cfg(all(target_arch = "x86_64", target_feature = "avx2"))]
use crate::barrier::Ymm;
use crate::barrier::{Barrier, Memory, touch};
#[cfg(target_arch = "x86_64")]
use crate::barrier::{Gpr, Xmm};
use crate::counters::{Counters, Counts, pin};
use crate::error::Error;
use crate::record::{Cell, Class, Count, Events, Latency, Line, Load, Outcome, Sibling, Sweep};
use crate::suite::{
    Call, CaseId, Chain, Function, Inputs, Rng, Suite, Visitor, run_batch, run_chain, visit,
};

/// The counted repetitions of every case.
pub const REPETITIONS: usize = 31;

/// The cycles that a short run takes at least, unless its operations reach their cap.
pub const TARGET: u64 = 1 << 20;

/// The fewest cycles that a chain step may count. A step of one instruction takes one cycle, and
/// its figure falls up to 5% below that on a quiet machine; a chain that the compiler regrouped
/// into pairs takes half a cycle per step or less.
pub const FEWEST_STEP_CYCLES: f64 = 0.75;

/// The CPU of the measuring thread, unless `--cpu` names another.
pub(crate) const CPU: usize = 15;

/// The seed of the operand sets of a sweep.
const SEED: u64 = 0x5eed_0fc0_ffee;

/// The most memory that the operand sets of a long run take.
const FOOTPRINT: usize = 256 * 1024;

/// The operations of the first sizing run, and the fewest of a short run.
const FEWEST: usize = 16;

/// The load average of the machine.
const LOADAVG: &str = "/proc/loadavg";

/// The time that each CPU spent in each state, in ticks.
const STAT: &str = "/proc/stat";

/// Runs the bench executable of `S`, named `name` in its messages, with the process arguments:
/// `--cpu <cpu>` names the CPU of the measuring thread.
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

/// Checks and measures every case on `cpu`; returns the line of every cell, in the order of
/// [`visit`], then the line of the sweep.
fn sweep<S: Suite>(cpu: usize) -> Result<Vec<String>, Error> {
    pin(cpu)?;
    let mut counters = Counters::open()?;
    let window = Window::open(cpu)?;
    // Pass 0 checks and sizes the cases; pass `r` runs repetition `r`.
    let mut measuring = Measuring {
        counters: &mut counters,
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
    let sweep = window.close()?;
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
                Outcome::Counted(case.count(function)?)
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
    let line = Line::<S::Key>::Sweep(sweep);
    lines.push(serde_json::to_string(&line).map_err(Error::Cells)?);
    Ok(lines)
}

/// The span of a sweep on one CPU: the load at its start, and the ticks of the CPU's SMT siblings.
pub(crate) struct Window {
    cpu: usize,
    before: f64,
    siblings: Vec<(usize, Ticks)>,
}

impl Window {
    /// Starts the span of a sweep on `cpu`.
    pub(crate) fn open(cpu: usize) -> Result<Self, Error> {
        let siblings = siblings(cpu)?
            .into_iter()
            .map(|sibling| Ok((sibling, ticks(sibling)?)))
            .collect::<Result<Vec<_>, Error>>()?;
        Ok(Self {
            cpu,
            before: load()?,
            siblings,
        })
    }

    /// Ends the span: the sweep with the load around it and the busy share of each sibling.
    pub(crate) fn close(self) -> Result<Sweep, Error> {
        let after = load()?;
        let siblings = self
            .siblings
            .into_iter()
            .map(|(sibling, before)| {
                Ok(Sibling {
                    cpu: sibling,
                    busy: ticks(sibling)?.busy_since(&before, sibling)?,
                })
            })
            .collect::<Result<Vec<_>, Error>>()?;
        Ok(Sweep {
            cpu: self.cpu,
            load: Load {
                before: self.before,
                after,
            },
            siblings,
        })
    }
}

/// The 1-minute load average.
fn load() -> Result<f64, Error> {
    let path = Path::new(LOADAVG);
    let text = fs::read_to_string(path).map_err(|source| Error::io(path, source))?;
    let field = text
        .split_whitespace()
        .next()
        .ok_or_else(|| Error::missing("load average", LOADAVG))?;
    field.parse::<f64>().map_err(|source| Error::Load {
        text: field.to_owned(),
        source,
    })
}

/// The SMT siblings of `cpu`, which share its core.
fn siblings(cpu: usize) -> Result<Vec<usize>, Error> {
    let path = format!("/sys/devices/system/cpu/cpu{cpu}/topology/thread_siblings_list");
    let text = fs::read_to_string(&path).map_err(|source| Error::io(Path::new(&path), source))?;
    let mut siblings = Vec::new();
    // A list such as `15,31` or `14-15`.
    for item in text.trim().split(',') {
        let (first, last) = item.split_once('-').unwrap_or((item, item));
        let parse = |text: &str| {
            text.parse::<usize>()
                .map_err(|source| Error::number("CPU", text, source))
        };
        siblings.extend((parse(first)?..=parse(last)?).filter(|sibling| *sibling != cpu));
    }
    Ok(siblings)
}

/// The ticks of one CPU in `/proc/stat`.
struct Ticks {
    busy: u64,
    total: u64,
}

/// The ticks of `cpu` so far.
fn ticks(cpu: usize) -> Result<Ticks, Error> {
    let path = Path::new(STAT);
    let text = fs::read_to_string(path).map_err(|source| Error::io(path, source))?;
    let name = format!("cpu{cpu}");
    let fields = text
        .lines()
        .find_map(|line| {
            let mut words = line.split_whitespace();
            (words.next() == Some(name.as_str())).then(|| words.collect::<Vec<_>>())
        })
        .ok_or_else(|| Error::missing(&name, STAT))?;
    let numbers = fields
        .iter()
        .map(|field| {
            field
                .parse::<u64>()
                .map_err(|source| Error::number(&name, field, source))
        })
        .collect::<Result<Vec<_>, _>>()?;
    // user, nice, system, idle, iowait, irq, softirq, steal; guest time is part of user time.
    let [user, nice, system, idle, iowait, irq, softirq, steal, ..] = numbers.as_slice() else {
        return Err(Error::missing("eight states", &name));
    };
    let busy = user + nice + system + irq + softirq + steal;
    Ok(Ticks {
        busy,
        total: busy + idle + iowait,
    })
}

impl Ticks {
    /// The busy share of `cpu` since `before`.
    fn busy_since(&self, before: &Self, cpu: usize) -> Result<f64, Error> {
        let missing = || Error::missing("ticks during the sweep", &format!("CPU {cpu}"));
        let busy = self.busy.checked_sub(before.busy).ok_or_else(missing)?;
        match self.total.checked_sub(before.total) {
            None | Some(0) => Err(missing()),
            Some(total) => Ok(busy as f64 / total as f64),
        }
    }
}

/// The most operations of a short run of a case whose operand sets are `I`: as many as fill
/// [`FOOTPRINT`] with the operand sets of a long run, and at least [`FEWEST`].
fn cap<I>() -> usize {
    (FOOTPRINT / (2 * size_of::<I>().max(1))).max(FEWEST)
}

/// The counted runs of one form of a case.
struct Form {
    /// The operations of a short run.
    operations: usize,
    /// The counts of the short run and of the long run of every repetition.
    repetitions: Vec<[Counts; 2]>,
}

impl Form {
    /// The events per operation of the form `name`: the fewest that a long run counts less the
    /// fewest that a short run counts, over the operations of a short run. Long runs whose fewest
    /// cycles are no more than the short runs' are an [`Error::Flat`].
    fn events(&self, name: &str) -> Result<Events, Error> {
        let fewest = |value: &dyn Fn(&[Counts; 2]) -> u64| -> Result<u64, Error> {
            self.repetitions
                .iter()
                .map(value)
                .min()
                .ok_or_else(|| Error::missing("repetitions", name))
        };
        let per = |event: fn(&Counts) -> u64| -> Result<f64, Error> {
            let long = fewest(&|[_, long]| event(long))?;
            let short = fewest(&|[short, _]| event(short))?;
            let difference = long as f64 - short as f64;
            Ok(difference / self.operations as f64)
        };
        let cycles = per(|counts| counts.cycles)?;
        if cycles <= 0.0 {
            return Err(Error::Flat {
                case: name.to_owned(),
                operations: self.operations,
            });
        }
        Ok(Events {
            operations: self.operations,
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
    fn count(&self, function: &Function) -> Result<Count, Error> {
        let latencies = self
            .chains
            .iter()
            .map(|chain| {
                let name = format!("{} chain in {}", self.id, chain.class);
                let events = chain.form.events(&name)?;
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
            batch: self.batch.events(&self.id)?,
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
    /// Runs the form on `count` fresh inputs that `rng` draws, counted by `counters`, right after
    /// an uncounted run on `warm` other fresh inputs; returns the number of uncounted and counted
    /// runs and the counts.
    fn counted(
        &self,
        rng: &mut Rng,
        count: usize,
        warm: usize,
        counters: &mut Counters,
    ) -> Result<([usize; 2], Counts), Error>;
}

/// A form of a case: `region` runs a batch of the inputs that `inputs` draws.
struct Region<'a, I, F> {
    inputs: &'a Inputs<'a, I>,
    region: F,
}

impl<I, F: Fn(&[I]) -> usize> Trial for Region<'_, I, F> {
    /// The uncounted run follows both draws, so the tables and the code of the form are as warm
    /// for every counted run. A GF(2¹⁶) inverse by a 128 KiB table takes 1.7 cycles; when the
    /// uncounted run came before the draws, they evicted more of that table before a long run than
    /// before a short one, and the long runs counted no more cycles.
    fn counted(
        &self,
        rng: &mut Rng,
        count: usize,
        warm: usize,
        counters: &mut Counters,
    ) -> Result<([usize; 2], Counts), Error> {
        let warming = self.inputs.draw(rng, warm);
        let batch = self.inputs.draw(rng, count);
        let warmed = (self.region)(&warming);
        touch(&batch);
        let (runs, counts) = counters.count(&mut || (self.region)(&batch))?;
        Ok(([warmed, runs], counts))
    }
}

impl<O, K> Measuring<'_, O, K> {
    /// The counts of `trial` on `count` fresh inputs, right after an uncounted run on `warm`
    /// others.
    fn counted(
        &mut self,
        trial: &dyn Trial,
        count: usize,
        warm: usize,
        id: &str,
    ) -> Result<Counts, Error> {
        let ([warmed, runs], counts) = trial.counted(&mut self.rng, count, warm, self.counters)?;
        ran(id, warmed, warm)?;
        ran(id, runs, count)?;
        Ok(counts)
    }

    /// The form of a case that `trial` runs, with the operations of its short run: 16, doubled
    /// until a counted run after an uncounted run of as many operations counts [`TARGET`] cycles,
    /// or until `cap`.
    fn size(&mut self, trial: &dyn Trial, cap: usize, id: &str) -> Result<Form, Error> {
        let mut operations = FEWEST;
        loop {
            let counts = self.counted(trial, operations, operations, id)?;
            if counts.cycles >= TARGET || operations >= cap {
                return Ok(Form {
                    operations,
                    repetitions: Vec::new(),
                });
            }
            operations = (2 * operations).min(cap);
        }
    }

    /// Runs one repetition of a form through `trial`: its operations and twice as many counted,
    /// each run on fresh inputs right after an uncounted run of its operations.
    fn repetition(
        &mut self,
        trial: &dyn Trial,
        operations: usize,
        id: &str,
    ) -> Result<[Counts; 2], Error> {
        Ok([
            self.counted(trial, operations, operations, id)?,
            self.counted(trial, 2 * operations, operations, id)?,
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
        let operations = self.cases[index].batch.operations;
        let counts = self.repetition(trial, operations, &id)?;
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
        let operations = self.cases[index]
            .chains
            .iter()
            .find(|chain| chain.class == class)
            .map(|chain| chain.form.operations)
            .ok_or_else(missing)?;
        let counts = self.repetition(trial, operations, id)?;
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
