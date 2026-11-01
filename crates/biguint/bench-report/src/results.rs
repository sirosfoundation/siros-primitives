//! Criterion's results: the median time of each case, per operation.

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::UNIX_EPOCH;

use serde::Deserialize;
use serde::de::DeserializeOwned;

use crate::Error;
use crate::record::{Build, Cell, Kind};

/// The identity of a criterion benchmark, from its `benchmark.json`.
#[derive(Deserialize)]
struct Benchmark {
    full_id: String,
}

/// Criterion's `estimates.json`.
#[derive(Deserialize)]
struct Estimates {
    median: Estimate,
}

/// One statistic of `estimates.json`, in nanoseconds per iteration.
#[derive(Deserialize)]
struct Estimate {
    point_estimate: f64,
}

/// The times of one build.
pub(crate) struct Measured<'a> {
    /// The build.
    pub(crate) build: &'a Build,
    /// Nanoseconds per operation, by case.
    pub(crate) times: HashMap<String, f64>,
}

/// A median time, and when criterion wrote it.
struct Measurement {
    nanoseconds: f64,
    written: u64,
}

/// Nanoseconds per operation of every case of `cells` in `build`: criterion's median time of
/// one iteration in `directory`, divided by the operations of the iteration.
///
/// Fails unless every case has a result that criterion wrote during the timing run of `build`.
pub(crate) fn times(
    directory: &Path,
    build: &Build,
    cells: &[Cell],
) -> Result<HashMap<String, f64>, Error> {
    let measurements = measurements(directory)?;
    cells
        .iter()
        .filter(|cell| cell.kind != Kind::Absent)
        .map(|cell| {
            let measurement = measurements
                .get(&cell.case)
                .ok_or_else(|| Error::NoResult {
                    directory: directory.to_path_buf(),
                    case: cell.case.clone(),
                })?;
            if measurement.written < build.started {
                return Err(Error::Stale {
                    directory: directory.to_path_buf(),
                    case: cell.case.clone(),
                });
            }
            Ok((
                cell.case.clone(),
                measurement.nanoseconds / f64::from(cell.batch),
            ))
        })
        .collect()
}

/// Every result in criterion's output `directory`, by benchmark id.
fn measurements(directory: &Path) -> Result<HashMap<String, Measurement>, Error> {
    let entries = fs::read_dir(directory).map_err(|source| Error::io(directory, source))?;
    let mut measurements = HashMap::new();
    for entry in entries {
        let latest = entry
            .map_err(|source| Error::io(directory, source))?
            .path()
            .join("new");
        let benchmark = latest.join("benchmark.json");
        // Criterion keeps reports and other data beside the benchmarks.
        if !benchmark.is_file() {
            continue;
        }
        let id = read::<Benchmark>(&benchmark)?.full_id;
        let estimates = latest.join("estimates.json");
        let measurement = Measurement {
            nanoseconds: read::<Estimates>(&estimates)?.median.point_estimate,
            written: written(&estimates)?,
        };
        if measurements.insert(id.clone(), measurement).is_some() {
            return Err(Error::Duplicate {
                directory: directory.to_path_buf(),
                case: id,
            });
        }
    }
    Ok(measurements)
}

/// The JSON file at `path`.
fn read<T: DeserializeOwned>(path: &Path) -> Result<T, Error> {
    let text = fs::read_to_string(path).map_err(|source| Error::io(path, source))?;
    serde_json::from_str(&text).map_err(|source| Error::json(path, source))
}

/// Seconds since the Unix epoch when `path` was last written.
fn written(path: &Path) -> Result<u64, Error> {
    let modified = fs::metadata(path)
        .and_then(|metadata| metadata.modified())
        .map_err(|source| Error::io(path, source))?;
    let elapsed = modified.duration_since(UNIX_EPOCH).map_err(Error::Clock)?;
    Ok(elapsed.as_secs())
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::fs;

    use super::times;
    use crate::Error;
    use crate::record::{Build, Cell, Kind};

    fn cell(case: &str) -> Cell {
        Cell {
            case: case.to_owned(),
            op: "add".to_owned(),
            bits: 64,
            library: "biguint".to_owned(),
            kind: Kind::Api,
            detail: "+".to_owned(),
            batch: 256,
        }
    }

    fn build(started: u64) -> Build {
        Build {
            name: "default".to_owned(),
            rustflags: String::new(),
            started,
            finished: started,
        }
    }

    /// A case reads its median over its batch; a case without a result, or with a result
    /// older than its run, is refused by name.
    #[test]
    fn times_refuse_missing_and_stale_results() {
        let directory = std::env::temp_dir().join(format!("bench-report-{}", std::process::id()));
        let latest = directory.join("add_64_biguint").join("new");
        fs::create_dir_all(&latest).expect("a fixture directory");
        fs::write(
            latest.join("benchmark.json"),
            r#"{"full_id":"add/64/biguint"}"#,
        )
        .expect("a fixture benchmark.json");
        fs::write(
            latest.join("estimates.json"),
            r#"{"median":{"point_estimate":512.0}}"#,
        )
        .expect("a fixture estimates.json");

        let accepted = times(&directory, &build(0), &[cell("add/64/biguint")]);
        let missing = times(&directory, &build(0), &[cell("add/128/biguint")]);
        let stale = times(&directory, &build(u64::MAX), &[cell("add/64/biguint")]);
        fs::remove_dir_all(&directory).expect("the fixture removed");

        let expected = HashMap::from([("add/64/biguint".to_owned(), 2.0)]);
        assert_eq!(accepted.expect("the fixture's time"), expected);
        assert!(matches!(missing, Err(Error::NoResult { case, .. }) if case == "add/128/biguint"));
        assert!(matches!(stale, Err(Error::Stale { case, .. }) if case == "add/64/biguint"));
    }
}
