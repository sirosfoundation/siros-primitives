//! The lines of several runs of one bench as the lines of one: per cell, the fewest cycles of each
//! form over the runs. A disturbance only adds counts, so each run's figure bounds the undisturbed
//! one from above.

use serde_json::Value;

use crate::error::Error;
use crate::record::{Count, Latency};

/// The field of a line that holds a cell.
const CELL: &str = "cell";

/// The field of a cell that holds its counts.
const COUNTED: &str = "counted";

/// `first` with the form of `other` wherever `other` counts fewer cycles. A chain that one of them
/// lacks is an [`Error::Missing`].
fn fewer(first: Count, other: Count, name: &Value) -> Result<Count, Error> {
    let chain = match (first.chain, other.chain) {
        (Some(one), Some(two)) => Some(fewer_latency(one, two)),
        (None, None) => None,
        (Some(_), None) | (None, Some(_)) => {
            return Err(Error::missing(
                &format!("a chain of {name}"),
                "each run of the bench",
            ));
        }
    };
    Ok(Count {
        batch: if other.batch.cycles < first.batch.cycles {
            other.batch
        } else {
            first.batch
        },
        chain,
        ..first
    })
}

/// The chain with fewer cycles per step.
fn fewer_latency(one: Latency, two: Latency) -> Latency {
    if two.events.cycles < one.events.cycles {
        two
    } else {
        one
    }
}

/// The cell of `line` without its counts, which names it, and its counts; `None` for another line.
fn cell(line: &Value) -> Result<Option<(Value, Option<Count>)>, Error> {
    let Some(cell) = line.get(CELL) else {
        return Ok(None);
    };
    let mut name = cell.clone();
    let counted = name.as_object_mut().and_then(|name| name.remove(COUNTED));
    let count = counted
        .map(serde_json::from_value)
        .transpose()
        .map_err(Error::Cells)?;
    Ok(Some((name, count)))
}

/// The lines of the first of `runs`, each the lines of one run of one bench, with the counts of each
/// cell the fewest over the runs. A cell that a run lacks is an [`Error::Missing`].
pub fn fewest(runs: Vec<Vec<String>>) -> Result<Vec<String>, Error> {
    let mut runs = runs.into_iter();
    let first = runs
        .next()
        .ok_or_else(|| Error::missing("run", "the runs of a bench"))?;
    let mut lines = first
        .iter()
        .map(|line| serde_json::from_str::<Value>(line).map_err(Error::Cells))
        .collect::<Result<Vec<_>, Error>>()?;
    for run in runs {
        let others = run
            .iter()
            .map(|line| serde_json::from_str::<Value>(line).map_err(Error::Cells))
            .collect::<Result<Vec<_>, Error>>()?;
        let others = others
            .iter()
            .map(cell)
            .filter_map(Result::transpose)
            .collect::<Result<Vec<_>, Error>>()?;
        for line in &mut lines {
            let Some((name, count)) = cell(line)? else {
                continue;
            };
            let (_, other) = others
                .iter()
                .find(|(other, _)| *other == name)
                .ok_or_else(|| Error::missing(&name.to_string(), "a run of the bench"))?;
            let merged = match (count, other.clone()) {
                (Some(count), Some(other)) => fewer(count, other, &name)?,
                (None, None) => continue,
                (Some(_), None) | (None, Some(_)) => {
                    return Err(Error::missing(
                        &format!("the same outcome of {name}"),
                        "the runs of the bench",
                    ));
                }
            };
            line[CELL][COUNTED] = serde_json::to_value(merged).map_err(Error::Cells)?;
        }
    }
    lines
        .iter()
        .map(|line| serde_json::to_string(line).map_err(Error::Cells))
        .collect()
}
