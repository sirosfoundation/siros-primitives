//! Compares `biguint` with other Rust big-integer crates: add, sub, mul, div, mod and mulmod at
//! 64, 128, 256, 384 and 512 bits.
//!
//! The bench has three modes:
//! - by default, criterion times every case;
//! - `--cases` prints every cell of the comparison as one JSON object per line;
//! - `--count <case>` runs one case once through [`run_batch`], whose instructions callgrind
//!   counts, and prints the number of operations it ran.
//!
//! `scripts/benchmarks.py` runs the modes on two builds and writes `BENCHMARKS.md`.

mod suite;

use std::process::ExitCode;
use std::time::Duration;

use criterion::Criterion;

use suite::{BATCH, Call, CaseId, Visitor, run_batch};

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        Some("--cases") => {
            for line in suite::visit(Cases::default()).lines {
                println!("{line}");
            }
            ExitCode::SUCCESS
        }
        Some("--count") => match args.next() {
            Some(case) => count(case),
            None => {
                eprintln!("usage: compare --count <op/bits/crate>");
                ExitCode::FAILURE
            }
        },
        _ => {
            let mut criterion = Criterion::default()
                .warm_up_time(Duration::from_secs(1))
                .measurement_time(Duration::from_secs(3))
                .configure_from_args();
            suite::visit(Timing {
                criterion: &mut criterion,
            })
            .criterion
            .final_summary();
            ExitCode::SUCCESS
        }
    }
}

/// Times every case with criterion. Each iteration runs the case's whole batch.
struct Timing<'a> {
    criterion: &'a mut Criterion,
}

impl Visitor for Timing<'_> {
    fn wants(&self, _id: &CaseId) -> bool {
        true
    }

    fn measure<I, O>(self, id: &CaseId, batch: &[I], run: impl Fn(&I) -> O + Copy) -> Self {
        Self {
            criterion: self.criterion.bench_function(&id.to_string(), |bencher| {
                bencher.iter(|| run_batch(batch, run));
            }),
        }
    }
}

/// Collects one JSON line per cell: its id, its parts, what it runs, and the batch size.
#[derive(Default)]
struct Cases {
    lines: Vec<String>,
}

impl Visitor for Cases {
    fn note(&mut self, id: &CaseId, call: &Call) {
        self.lines.push(format!(
            "{{\"case\":\"{id}\",\"op\":\"{}\",\"bits\":{},\"crate\":\"{}\",\"kind\":\"{}\",\
             \"detail\":{:?},\"batch\":{BATCH}}}",
            id.op,
            id.bits,
            id.lib,
            call.kind(),
            call.detail(),
        ));
    }

    fn wants(&self, _id: &CaseId) -> bool {
        false
    }

    fn measure<I, O>(self, _id: &CaseId, _batch: &[I], _run: impl Fn(&I) -> O + Copy) -> Self {
        self
    }
}

/// Runs the case `wanted` once through [`run_batch`] and keeps the number of operations it ran.
struct Count {
    wanted: String,
    operations: Option<usize>,
}

impl Visitor for Count {
    fn wants(&self, id: &CaseId) -> bool {
        id.to_string() == self.wanted
    }

    fn measure<I, O>(self, _id: &CaseId, batch: &[I], run: impl Fn(&I) -> O + Copy) -> Self {
        Self {
            operations: Some(run_batch(batch, run)),
            ..self
        }
    }
}

/// Runs the case `wanted` for callgrind and prints `{"case": ..., "operations": ...}`.
fn count(wanted: String) -> ExitCode {
    let count = suite::visit(Count {
        wanted,
        operations: None,
    });
    match count.operations {
        Some(operations) => {
            println!(
                "{{\"case\":\"{}\",\"operations\":{operations}}}",
                count.wanted
            );
            ExitCode::SUCCESS
        }
        None => {
            eprintln!("no case {}", count.wanted);
            ExitCode::FAILURE
        }
    }
}
