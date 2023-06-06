// Copyright (c) 2023 Joining7943 <joining@posteo.de>
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

//! Flamegraphs for the standard module and the DurationParser

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use fundu::TimeUnit::*;
use fundu_systemd::TimeSpanParser;
use pprof::criterion::{Output, PProfProfiler};
use pprof::flamegraph::Options as FlamegraphOptions;

fn flamegraph_parsing(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("time span parser");
    for &input in &["1", "1s", "1ns", "1minutes"] {
        group.bench_with_input(
            BenchmarkId::new("parsing", input),
            &(TimeSpanParser::new(), input),
            |b, (parser, input)| b.iter(|| black_box(parser).parse(input)),
        );
    }
    group.finish();
}

criterion_group!(
    name = parsing;
    config = Criterion::default().with_profiler(PProfProfiler::new(1_000_000, Output::Flamegraph({
        let mut options = FlamegraphOptions::default();
        options.title = "Flame graph for standard duration parser".to_string();
        options.subtitle = Some("Parsing".to_string());
        Some(options)
    })));
    targets = flamegraph_parsing
);
criterion_main!(parsing);
