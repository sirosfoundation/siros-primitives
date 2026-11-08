//! The lines of several runs of one bench as the lines of one: each form takes the run with the
//! fewest cycles, and runs whose cells differ are refused.

use bench_compare::Error;
use bench_tools::{ToolError, fewest};
use serde_json::{Value, json};

fn events(cycles: f64, instructions: f64) -> Value {
    json!({"operations": 64, "cycles": cycles, "instructions": instructions, "branch_misses": 0.0})
}

/// The line of a counted cell, with the cycles and instructions of its batch and of its chain.
fn add(batch: (f64, f64), chain: (f64, f64)) -> Value {
    json!({"cell": {"op": "add", "bits": 64, "crate": "biguint", "counted": {
        "function": "add",
        "deviation": null,
        "batch": events(batch.0, batch.1),
        "chain": {"class": "gpr", "events": events(chain.0, chain.1)},
    }}})
}

/// The cell of the line of [`absent`], which names it.
fn absent_name() -> Value {
    json!({"op": "mul", "bits": 64, "crate": "ruint", "absent": "no function"})
}

fn absent() -> Value {
    json!({ "cell": absent_name() })
}

fn sweep(load: f64) -> Value {
    json!({"sweep": {"cpu": 9, "load": {"before": load, "after": load}, "siblings": []}})
}

fn lines(values: &[Value]) -> Vec<String> {
    values.iter().map(Value::to_string).collect()
}

#[test]
fn each_form_takes_the_run_with_the_fewest_cycles() {
    let runs = vec![
        lines(&[add((10.0, 30.0), (5.5, 12.0)), absent(), sweep(1.0)]),
        lines(&[absent(), add((9.0, 29.0), (5.0, 11.0)), sweep(2.0)]),
        lines(&[add((8.0, 28.0), (6.0, 13.0)), absent(), sweep(3.0)]),
    ];
    let merged: Vec<Value> = fewest(runs)
        .expect("runs of the same cells")
        .iter()
        .map(|line| serde_json::from_str(line).expect("a line of JSON"))
        .collect();
    assert_eq!(
        merged,
        [add((8.0, 28.0), (5.0, 11.0)), absent(), sweep(1.0)]
    );
}

#[test]
fn runs_of_other_cells_are_refused() {
    let counted = || add((8.0, 28.0), (5.0, 11.0));
    for runs in [
        [lines(&[counted(), absent()]), lines(&[counted()])],
        [lines(&[counted()]), lines(&[counted(), absent()])],
    ] {
        let error = fewest(runs.into()).expect_err("runs of other cells");
        assert!(
            matches!(
                &error,
                ToolError::Harness(Error::Missing { what, .. })
                    if serde_json::from_str::<Value>(what).ok() == Some(absent_name())
            ),
            "{error}"
        );
    }
}
