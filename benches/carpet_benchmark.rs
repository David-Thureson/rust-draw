#![allow(unused_imports)]

use std::iter;

use criterion::{criterion_group, criterion_main, black_box, Criterion, BenchmarkId, BatchSize, Throughput, PlotConfiguration, AxisScale};

use draw::carpet::carpet::*;
use util::*;

/*
pub fn simple_carpet_vary_min_length(c: &mut Criterion) {
    // let mut group= c.benchmark_group("simple_carpet_vary_min_length");

    let size = 100;
    let mult = 0.68;

    for min_length in 4..12 {
        // group.throughput(Throughput::Elements(min_length as u64));
        // group.bench_with_input(BenchmarkId::new("simple_carpet",min_length), &min_length, |b, &min_length| {
        //     b.iter_batched_ref(|| , |v| { merge_sort::merge_sort(v); }, BatchSize::LargeInput)
        // });
        c.bench_function(&format!("simple_carpet_vary_min_length({})", min_length), |b|
            b.iter(|| create_one(size, min_length, mult))
        );
    }
    // group.finish();
}
*/

pub fn simple_carpet_vary_min_length(c: &mut Criterion) {
    let mut group = c.benchmark_group("simple_carpet_vary_min_length");

    let size = 100;
    let mult = 0.68;

    for min_length in 5..13 {
        group.throughput(Throughput::Elements(min_length as u64));
        group.bench_with_input(BenchmarkId::new("simple_carpet_vary_min_length", min_length), &min_length, |b, &min_length| {
            b.iter_batched_ref(|| min_length, |v| { create_one(size, min_length, mult); }, BatchSize::LargeInput)
            // b.iter(|| create_one(size, min_length, mult))
        });
    }
    group.finish();
}

pub fn simple_carpet_vary_size(c: &mut Criterion) {
    let mut group = c.benchmark_group("simple_carpet_vary_size");

    let min_length = 8;
    let mult = 0.68;

    for size in (10..100).step_by(10) {
        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(BenchmarkId::new("simple_carpet_vary_size", size), &size, |b, &size| {
            // b.iter_batched_ref(|| size, |v| { create_one(size, min_length, mult); }, BatchSize::LargeInput)
            b.iter(|| create_one(size, min_length, mult))
        });
    }
    group.finish();
}

pub fn simple_carpet_vary_mult(c: &mut Criterion) {
    let plot_config = PlotConfiguration::default()
        .summary_scale(AxisScale::Logarithmic);

    let mut group = c.benchmark_group("simple_carpet_vary_mult");
    group.plot_config(plot_config);

    let size = 100;
    let min_length = 8;

    //for mult_pct in (50..80).step_by(5) {
    for mult_pct in (55..80).step_by(1) {
        group.throughput(Throughput::Elements(mult_pct as u64));
        group.bench_with_input(BenchmarkId::new("simple_carpet_vary_mult", mult_pct), &mult_pct, |b, &mult_pct| {
            b.iter(|| create_one(size, min_length, mult_pct as f32 / 100.0))
        });
    }
    group.finish();
}

criterion_group!(benches,
    // simple_carpet_vary_size,
    // simple_carpet_vary_min_length,
    simple_carpet_vary_mult,
);
criterion_main!(benches);

// From the main project folder run:
//   cargo +nighly bench
// for all benchmarks or:
//   cargo +nightly bench --bench sort_benchmark
// for just the above group.
