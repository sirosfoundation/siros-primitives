//! The span of a sweep on the measuring CPU: the load of the machine around it, and the busy
//! share of the CPU's SMT siblings during it, read from `/proc` and `/sys`.

use std::fs;
use std::path::Path;

use crate::error::Error;
use crate::record::{Load, Sibling, Sweep};

/// The load average of the machine.
const LOADAVG: &str = "/proc/loadavg";

/// The time that each CPU spent in each state, in ticks.
const STAT: &str = "/proc/stat";

/// The span of a sweep on one CPU: the load at its start, and the ticks of the CPU's SMT siblings.
pub struct Window {
    cpu: usize,
    before: f64,
    siblings: Vec<(usize, Ticks)>,
}

impl Window {
    /// Starts the span of a sweep on `cpu`.
    pub fn open(cpu: usize) -> Result<Self, Error> {
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
    pub fn close(self) -> Result<Sweep, Error> {
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
