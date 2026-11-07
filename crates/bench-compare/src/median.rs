//! The lines of several runs of one bench as the lines of one: per cell, the median figure of each
//! form over the runs. A figure is the difference of two fewest counts, so a run's error goes either
//! way, and the fewest figure over the runs would pick the run that erred low. See the README of the
//! crate for the figures.

use serde_json::Value;

use crate::error::Error;
use crate::record::Count;

/// The field of a line that holds a cell.
const CELL: &str = "cell";

/// The field of a cell that holds its counts.
const COUNTED: &str = "counted";

/// The form of `forms` with the median cycles: the lower of the two in the middle of an even number.
fn median<T>(mut forms: Vec<T>, cycles: fn(&T) -> f64) -> Option<T> {
    forms.sort_by(|one, two| cycles(one).total_cmp(&cycles(two)));
    let middle = forms.len().checked_sub(1)? / 2;
    forms.into_iter().nth(middle)
}

/// `first`, with its counts over `runs` of the cell `name`: the median figure of each form.
fn merged(first: Count, others: Vec<Count>, name: &Value) -> Result<Count, Error> {
    let runs = 1 + others.len();
    let batches = others
        .iter()
        .map(|other| other.batch)
        .chain([first.batch])
        .collect();
    let chains: Vec<_> = others
        .iter()
        .filter_map(|other| other.chain)
        .chain(first.chain)
        .collect();
    if !chains.is_empty() && chains.len() != runs {
        return Err(Error::missing(
            &format!("a chain of {name}"),
            "each run of the bench",
        ));
    }
    let batch = median(batches, |events| events.cycles)
        .ok_or_else(|| Error::missing("run", "the runs of a bench"))?;
    Ok(Count {
        batch,
        chain: median(chains, |latency| latency.events.cycles),
        ..first
    })
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
/// cell the median over the runs. A cell that a run lacks is an [`Error::Missing`].
pub fn median_of_runs(runs: Vec<Vec<String>>) -> Result<Vec<String>, Error> {
    let mut runs = runs.into_iter();
    let first = runs
        .next()
        .ok_or_else(|| Error::missing("run", "the runs of a bench"))?;
    let mut lines = first
        .iter()
        .map(|line| serde_json::from_str::<Value>(line).map_err(Error::Cells))
        .collect::<Result<Vec<_>, Error>>()?;
    let others = runs
        .map(|run| {
            run.iter()
                .map(|line| {
                    let line = serde_json::from_str::<Value>(line).map_err(Error::Cells)?;
                    cell(&line)
                })
                .filter_map(Result::transpose)
                .collect::<Result<Vec<_>, Error>>()
        })
        .collect::<Result<Vec<_>, Error>>()?;
    for line in &mut lines {
        let Some((name, count)) = cell(line)? else {
            continue;
        };
        let outcomes = others
            .iter()
            .map(|run| {
                run.iter()
                    .find(|(other, _)| *other == name)
                    .map(|(_, count)| count.clone())
                    .ok_or_else(|| Error::missing(&name.to_string(), "a run of the bench"))
            })
            .collect::<Result<Vec<_>, Error>>()?;
        let same_outcome = || Error::missing(&format!("the same outcome of {name}"), "the runs");
        match count {
            Some(count) => {
                let counts = outcomes
                    .into_iter()
                    .map(|outcome| outcome.ok_or_else(same_outcome))
                    .collect::<Result<Vec<_>, Error>>()?;
                let counted = merged(count, counts, &name)?;
                line[CELL][COUNTED] = serde_json::to_value(counted).map_err(Error::Cells)?;
            }
            None if outcomes.iter().any(Option::is_some) => return Err(same_outcome()),
            None => {}
        }
    }
    lines
        .iter()
        .map(|line| serde_json::to_string(line).map_err(Error::Cells))
        .collect()
}
