//! Compares `biguint` with other Rust big-integer crates: add, sub, mul, div, mod and mulmod at
//! 64 to 4096 bits.
//!
//! By default, criterion times every case. With `--cases`, the bench prints every cell of the
//! comparison as one JSON object per line instead. `biguint-bench-report` runs both and writes
//! `BENCHMARKS.md`.

mod suite;

use std::time::Duration;

use criterion::Criterion;

use suite::{BATCH, Call, CaseId, Visitor, run_batch};

fn main() {
    if std::env::args().any(|arg| arg == "--cases") {
        for line in suite::visit(Cases::default()).lines {
            println!("{line}");
        }
    } else {
        let mut criterion = Criterion::default()
            .warm_up_time(Duration::from_secs(1))
            .measurement_time(Duration::from_secs(3))
            .configure_from_args();
        suite::visit(Timing {
            criterion: &mut criterion,
        })
        .criterion
        .final_summary();
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
