//! A content-addressed action cache of WebAssembly figures, as in Bazel: the key of a cell is every
//! input that determines its figures, and a cell whose key the record holds keeps its figures. A
//! cell without a key counts as changed. The known failure of such a cache is an input missing from
//! the key, which returns a stale hit.
//!
//! The key of a cell has its code, which [`code`] hashes from the module, and the settings of its
//! build: the browser with its version and settings, the `RUSTFLAGS`, the model of the CPU, and
//! [`harness`], the hash of the sources that measure.

use std::collections::HashSet;

use crate::document::{Layout, Table};
use crate::error::Error;
use crate::fingerprint::Fingerprint;
use crate::identity::Identity;
use crate::module::ModuleError;
use crate::record::{Cell, Key, ListedCase, Recorded, Run};

/// The sources that measure, whose text is part of the key of every cell.
const HARNESS: [&str; 11] = [
    include_str!("count.rs"),
    include_str!("counters.rs"),
    include_str!("barrier.rs"),
    include_str!("suite.rs"),
    include_str!("host.rs"),
    include_str!("browser/mod.rs"),
    include_str!("browser/driver.rs"),
    include_str!("browser/worker.js"),
    include_str!("browser/helper.js"),
    include_str!("browser/shared.js"),
    include_str!("browser/page.html"),
];

/// The hash of the sources that measure, in hexadecimal digits.
#[must_use]
pub fn harness() -> String {
    let mut state = Fingerprint::new();
    for source in HARNESS {
        take(&mut state, source.as_bytes());
    }
    format!("{:032x}", state.finish())
}

/// Takes `bytes` into `state` with their length, so that the parts of a key do not run together.
fn take(state: &mut Fingerprint, bytes: &[u8]) {
    state.update(&(bytes.len() as u64).to_le_bytes());
    state.update(bytes);
}

/// The index in the table of the function that the module sent as `slot`.
fn slot(slot: usize, what: &str) -> Result<u32, ModuleError> {
    u32::try_from(slot).map_err(|_error| ModuleError {
        offset: 0,
        reason: format!("{what} at the index {slot} of the table"),
    })
}

/// The bytes of `digits`, two hexadecimal digits each.
fn bytes(digits: &str) -> Result<Vec<u8>, ModuleError> {
    let malformed = || ModuleError {
        offset: 0,
        reason: format!("the captures {digits:?} of a case"),
    };
    let (pairs, []) = digits.as_bytes().as_chunks::<2>() else {
        return Err(malformed());
    };
    pairs
        .iter()
        .map(|pair| {
            str::from_utf8(pair)
                .ok()
                .and_then(|pair| u8::from_str_radix(pair, 16).ok())
                .ok_or_else(malformed)
        })
        .collect()
}

/// The code of the cell that runs `case` in the module of `identity`, in hexadecimal digits: the
/// hashes of its loops, of what draws its inputs, of the operands of its row and of what its
/// operation captures. `None` for a loop that reaches code decided at run time, which no hash covers:
/// such a cell always counts as changed. What draws the inputs always reaches its row's operand
/// function through a pointer: the hash of the operands covers it by value.
pub fn code(identity: &mut Identity<'_>, case: &ListedCase) -> Result<Option<String>, ModuleError> {
    let batch = identity.entry(slot(case.batch, "the batch loop")?)?;
    if batch.indirect {
        return Ok(None);
    }
    let mut state = Fingerprint::new();
    take(&mut state, case.function.as_bytes());
    take(
        &mut state,
        case.deviation.as_deref().unwrap_or_default().as_bytes(),
    );
    state.update_hash(batch.hash);
    for chain in &case.chains {
        let entry = identity.entry(slot(chain.entry, "a chain loop")?)?;
        if entry.indirect {
            return Ok(None);
        }
        take(&mut state, format!("{:?}", chain.class).as_bytes());
        state.update_hash(entry.hash);
    }
    state.update_hash(identity.entry(slot(case.draw, "the draw")?)?.hash);
    state.update_hash(identity.entry(slot(case.setup, "the setup")?)?.hash);
    take(&mut state, case.operands.as_bytes());
    state.update_hash(identity.captures(&bytes(&case.captures)?));
    Ok(Some(format!("{:032x}", state.finish())))
}

/// The name of a cell.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct CellId<K> {
    /// The operation.
    pub op: String,
    /// The row.
    pub key: K,
    /// The crate.
    pub library: String,
}

impl<K: Key> CellId<K> {
    /// The name of `cell`.
    #[must_use]
    pub fn of(cell: &Cell<K>) -> Self {
        Self {
            op: cell.op.clone(),
            key: cell.key.clone(),
            library: cell.library.clone(),
        }
    }

    /// The name as a pattern of a [`Selection`](crate::Selection).
    #[must_use]
    pub fn pattern(&self) -> String {
        format!("{}/{}/{}", self.op, self.key, self.library)
    }
}

/// The cells that a round re-measures, given the cells in `seeds`: per row of a seed, the seed, the
/// columns of the subject, and the cells of the row's fewest figure and of the best competitor's in
/// each part. A leader may be the constant-time twin of a variable-time row, whose own row then
/// counts too.
pub fn select<K: Key>(
    layout: &Layout<'_, K>,
    cells: &[Recorded<K>],
    seeds: impl IntoIterator<Item = CellId<K>>,
) -> Result<HashSet<CellId<K>>, Error> {
    let rows = Table::new(cells, layout)?.rows(layout);
    let mut selected: HashSet<CellId<K>> = seeds.into_iter().collect();
    loop {
        let mut grown = false;
        for row in &rows {
            if row.members.iter().any(|member| selected.contains(member)) {
                for cell in &row.compared {
                    grown |= selected.insert(cell.clone());
                }
            }
        }
        if !grown {
            return Ok(selected);
        }
    }
}

/// The seeds of the next round after a round of re-measuring: per row whose subject and leader come
/// from different runs, the cells of the row in the subject's run, and the leaders that were kept.
/// Empty when every verdict is matched.
pub fn overtaken<K: Key>(
    layout: &Layout<'_, K>,
    cells: &[Recorded<K>],
) -> Result<Vec<CellId<K>>, Error> {
    let table = Table::new(cells, layout)?;
    let mut seeds = Vec::new();
    for verdict in table.verdicts(layout) {
        let Some(rival) = verdict.unmatched() else {
            continue;
        };
        seeds.push(CellId::of(&rival.cell));
        seeds.extend(
            table
                .row_cells(&verdict)
                .filter(|cell| cell.run == verdict.subject.run)
                .map(|cell| CellId::of(&cell.cell)),
        );
    }
    Ok(seeds)
}

/// Replaces the figures of the cells of `cells` that `fresh` names by those of `fresh`, which come
/// from the run `run`. It takes no minimum with the old figures: a figure of an earlier run met other
/// conditions. A fresh cell that `cells` lacks is an [`Error::Missing`].
pub fn merge<K: Key>(
    cells: &mut [Recorded<K>],
    fresh: Vec<Cell<K>>,
    run: usize,
) -> Result<(), Error> {
    for cell in fresh {
        let id = CellId::of(&cell);
        let kept = cells
            .iter_mut()
            .find(|kept| CellId::of(&kept.cell) == id)
            .ok_or_else(|| Error::missing(&id.pattern(), "the listed cells"))?;
        kept.cell = cell;
        kept.run = run;
    }
    Ok(())
}

/// `runs` without those that no counted cell of `cells` comes from, with the cells' indices moved.
pub fn compact<K>(runs: Vec<Run>, cells: &mut [Recorded<K>]) -> Vec<Run> {
    let used: Vec<bool> = (0..runs.len())
        .map(|index| cells.iter().any(|cell| cell.counted() && cell.run == index))
        .collect();
    let moved: Vec<usize> = used
        .iter()
        .scan(0, |next, used| {
            let index = *next;
            *next += usize::from(*used);
            Some(index)
        })
        .collect();
    for cell in cells {
        cell.run = if cell.counted() {
            moved.get(cell.run).copied().unwrap_or(cell.run)
        } else {
            0
        };
    }
    runs.into_iter()
        .zip(used)
        .filter_map(|(run, used)| used.then_some(run))
        .collect()
}
