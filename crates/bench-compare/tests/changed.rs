//! The action cache over the figures of a record: what a changed cell re-measures, how a round's
//! figures replace the kept ones, and the rule that a verdict compares figures of one run.

use std::collections::HashSet;

use bench_compare::{
    Cell, CellId, Count, Events, Layout, Load, Measurement, Operation, Outcome, Record, Recorded,
    Run, Selection, SelectionError, Sweep, ToolError, check_records, compact, merge, overtaken,
    select,
};

const SUBJECT: &str = "subject";

const LAYOUT: Layout<'static, u32> = Layout {
    subject: SUBJECT,
    tool: "tool",
    operations: &[
        Operation::new("add", "add", "a + b"),
        Operation::constant_time("add_ct", "add", "a + b"),
        Operation::new("mul", "mul", "a * b"),
    ],
    row_header: "Bits",
    rows_phrase: |_| String::new(),
    bits: |bits| *bits,
    method: &[],
    wasm_native: "",
};

fn counted(op: &str, key: u32, library: &str, cycles: f64) -> Cell<u32> {
    Cell {
        op: op.to_owned(),
        key,
        library: library.to_owned(),
        outcome: Outcome::Counted(Count {
            function: "function".to_owned(),
            deviation: None,
            batch: Events {
                operations: 1,
                cycles,
                instructions: 1.0,
                branch_misses: 0.0,
            },
            chain: None,
        }),
    }
}

fn recorded(op: &str, key: u32, library: &str, cycles: f64, run: usize) -> Recorded<u32> {
    Recorded {
        cell: counted(op, key, library, cycles),
        run,
        code: Some("code".to_owned()),
    }
}

fn absent(op: &str, key: u32, library: &str) -> Recorded<u32> {
    Recorded {
        cell: Cell {
            op: op.to_owned(),
            key,
            library: library.to_owned(),
            outcome: Outcome::Absent("no function".to_owned()),
        },
        run: 0,
        code: None,
    }
}

fn id(op: &str, key: u32, library: &str) -> CellId<u32> {
    CellId {
        op: op.to_owned(),
        key,
        library: library.to_owned(),
    }
}

/// `add` on the row `key`, where `a` holds only the constant-time twin.
fn add_rows(key: u32) -> [Recorded<u32>; 6] {
    [
        recorded("add", key, SUBJECT, 10.0, 0),
        absent("add", key, "a"),
        recorded("add", key, "b", 7.0, 0),
        recorded("add_ct", key, SUBJECT, 12.0, 0),
        recorded("add_ct", key, "a", 5.0, 0),
        absent("add_ct", key, "b"),
    ]
}

/// `add` and `mul` on two rows. `a` leads `mul` on the row of 256 bits, and `b` is the best
/// competitor on the row of 512 bits.
fn sample() -> Vec<Recorded<u32>> {
    let mut cells: Vec<Recorded<u32>> = [256, 512].into_iter().flat_map(add_rows).collect();
    cells.extend([
        recorded("mul", 256, SUBJECT, 10.0, 0),
        recorded("mul", 256, "a", 5.0, 0),
        recorded("mul", 256, "b", 7.0, 0),
        recorded("mul", 512, SUBJECT, 20.0, 0),
        recorded("mul", 512, "a", 40.0, 0),
        recorded("mul", 512, "b", 30.0, 0),
    ]);
    cells
}

fn set(cells: &[(&str, u32, &str)]) -> HashSet<CellId<u32>> {
    cells
        .iter()
        .map(|(op, key, library)| id(op, *key, library))
        .collect()
}

/// A verdict needs the changed cell, the subject and the leaders of its row; the other cells keep
/// their figures.
#[test]
fn a_changed_cell_selects_what_its_row_compares() {
    let cells = sample();
    let selected = |seed| select(&LAYOUT, &cells, [seed]).expect("a table of the sample");
    assert_eq!(
        selected(id("mul", 256, "b")),
        set(&[("mul", 256, "b"), ("mul", 256, SUBJECT), ("mul", 256, "a"),])
    );
    // The leader of `add` is `a`'s constant-time twin, whose own row then counts.
    assert_eq!(
        selected(id("add", 512, SUBJECT)),
        set(&[
            ("add", 512, SUBJECT),
            ("add_ct", 512, "a"),
            ("add_ct", 512, SUBJECT),
        ])
    );
}

/// Figures of a round replace the kept ones, also upwards. A kept figure that a re-measured leader
/// fell behind starts another round, which ends with all the row's figures in one run.
#[test]
fn rounds_end_with_every_verdict_matched() {
    let mut cells = sample();
    let rows: Vec<(&str, f64)> = vec![(SUBJECT, 8.0), ("a", 9.0)];
    let round = |rows: &[(&str, f64)]| -> Vec<Cell<u32>> {
        rows.iter()
            .map(|(library, cycles)| counted("mul", 256, library, *cycles))
            .collect()
    };

    merge(&mut cells, round(&rows), 1).expect("cells of the sample");
    let kept = cells
        .iter()
        .find(|cell| CellId::of(&cell.cell) == id("mul", 256, "a"))
        .expect("a cell of the sample");
    let Outcome::Counted(count) = &kept.cell.outcome else {
        panic!("a counted cell");
    };
    assert!((count.batch.cycles - 9.0).abs() < f64::EPSILON);

    let seeds: HashSet<CellId<u32>> = overtaken(&LAYOUT, &cells)
        .expect("a table of the sample")
        .into_iter()
        .collect();
    assert_eq!(
        seeds,
        set(&[("mul", 256, "b"), ("mul", 256, SUBJECT), ("mul", 256, "a"),])
    );

    let rows: Vec<(&str, f64)> = vec![(SUBJECT, 8.0), ("a", 9.0), ("b", 7.5)];
    merge(&mut cells, round(&rows), 2).expect("cells of the sample");
    // The runs of the other rows are the sample's first run.
    assert!(overtaken(&LAYOUT, &cells).expect("a table").is_empty());

    assert!(merge(&mut cells, vec![counted("mul", 128, "a", 1.0)], 3).is_err());

    let runs = compact(vec![run(), run(), run()], &mut cells);
    assert_eq!(runs.len(), 2);
    let moved = cells
        .iter()
        .find(|cell| CellId::of(&cell.cell) == id("mul", 256, "b"))
        .expect("a cell of the sample");
    assert_eq!(moved.run, 1);
}

fn run() -> Run {
    Run {
        commit: "0123456789abcdef".to_owned(),
        dirty: false,
        date: "2026-11-07".to_owned(),
        sweep: Sweep {
            cpu: 8,
            load: Load {
                before: 1.0,
                after: 1.0,
            },
            siblings: Vec::new(),
        },
    }
}

fn record(cells: Vec<Recorded<u32>>) -> Record<u32> {
    Record {
        date: "2026-11-07".to_owned(),
        head: "0123456789abcdef".to_owned(),
        library_commit: "0123456789abcdef".to_owned(),
        dirty: false,
        rustc: "rustc".to_owned(),
        profile: Vec::new(),
        llvm: "llvm".to_owned(),
        host: "host".to_owned(),
        native_cpu: "cpu".to_owned(),
        cpu_model: "model".to_owned(),
        machine: None,
        package: "package".to_owned(),
        features: Vec::new(),
        versions: Vec::new(),
        builds: vec![Measurement {
            name: String::new(),
            rustflags: "flags".to_owned(),
            target_features: Vec::new(),
            browser: None,
            overheads: Vec::new(),
            harness: None,
            runs: vec![run(), run()],
            cells,
        }],
    }
}

/// The owner's rule: a fresh figure is never compared with one recorded under other conditions.
#[test]
fn report_refuses_a_verdict_across_runs() {
    let matched = record(sample());
    check_records(&LAYOUT, &matched, None).expect("a record of one run");

    // The subject comes from the second run, and the row's leader stays in the first.
    let mut unmatched = sample();
    merge(&mut unmatched, vec![counted("mul", 256, SUBJECT, 8.0)], 1)
        .expect("a cell of the sample");
    let error =
        check_records(&LAYOUT, &record(unmatched), None).expect_err("a verdict across runs");
    assert!(matches!(error, ToolError::Unmatched(_)), "{error}");

    // A figure of the second run ties the leader's kept figure: the verdict holds in that run.
    let mut tied = sample();
    tied.extend(add_rows(1024));
    tied.extend([
        recorded("mul", 1024, SUBJECT, 8.0, 1),
        recorded("mul", 1024, "a", 6.0, 0),
        recorded("mul", 1024, "b", 6.0, 1),
    ]);
    check_records(&LAYOUT, &record(tied), None).expect("a tie that holds a cell of the run");
}

/// A pattern that names no cell of the bench stops a run before it measures: a mistyped selection
/// would otherwise measure nothing and look like a result.
#[test]
fn a_selection_refuses_a_pattern_without_a_cell() {
    assert!(matches!(
        Selection::parse("mul/256"),
        Err(SelectionError::Malformed { .. })
    ));
    let mut selection = Selection::parse("mul/256/*,mul/512/a").expect("two patterns");
    selection.note("mul", "256", "b");
    assert!(matches!(
        selection.check_matched(),
        Err(SelectionError::Unmatched { pattern }) if pattern == "mul/512/a"
    ));
    selection.note("mul", "512", "a");
    assert!(selection.check_matched().is_ok());
}
