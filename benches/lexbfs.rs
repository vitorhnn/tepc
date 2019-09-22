use std::fs::File;
use std::io::BufReader;

use criterion::{black_box, criterion_group, criterion_main, Criterion};

use tepc::common;
use tepc::serial::naive_lex_bfs as serial;
use tepc::threads::naive_lex_bfs as threaded;
use tepc::rayon::naive_lex_bfs as rayon;

#[inline(always)]
pub fn lexbfs_threaded_common(file_name: &str, id: &str, c: &mut Criterion) {
    let file = File::open(file_name).unwrap();

    let graph = common::graph_from_reader(BufReader::new(file)).unwrap();

    c.bench_function(id, move |b| b.iter(|| threaded(black_box(&graph))));
}

#[inline(always)]
pub fn lexbfs_serial_common(file_name: &str, id: &str, c: &mut Criterion) {
    let file = File::open(file_name).unwrap();

    let graph = common::graph_from_reader(BufReader::new(file)).unwrap();

    c.bench_function(id, move |b| b.iter(|| serial(black_box(&graph))));
}

#[inline(always)]
pub fn lexbfs_rayon_common(file_name: &str, id: &str, c: &mut Criterion) {
    let file = File::open(file_name).unwrap();

    let graph = common::graph_from_reader(BufReader::new(file)).unwrap();

    c.bench_function(id, move |b| b.iter(|| rayon(black_box(&graph))));
}

pub fn lexbfs_benchmark_k10_threads(c: &mut Criterion) {
    lexbfs_threaded_common("k10.txt", "threaded lexbfs, k10", c);
}

pub fn lexbfs_benchmark_k10_serial(c: &mut Criterion) {
    lexbfs_serial_common("k10.txt", "serial lexbfs, k10", c);
}

pub fn lexbfs_benchmark_k100_threads(c: &mut Criterion) {
    lexbfs_threaded_common("k100.txt", "threaded lexbfs, k100", c);
}

pub fn lexbfs_benchmark_k100_serial(c: &mut Criterion) {
    lexbfs_serial_common("k100.txt", "serial lexbfs, k100", c);
}

pub fn lexbfs_benchmark_k500_threads(c: &mut Criterion) {
    lexbfs_threaded_common("k500.txt", "threaded lexbfs, k500", c);
}

pub fn lexbfs_benchmark_k500_serial(c: &mut Criterion) {
    lexbfs_serial_common("k500.txt", "serial lexbfs, k500", c);
}

pub fn lexbfs_benchmark_k500_rayon(c: &mut Criterion) {
    lexbfs_rayon_common("k500.txt", "rayon lexbfs, k500", c);
}

criterion_group!(
    benches,
    /*
    lexbfs_benchmark_k10_threads,
    lexbfs_benchmark_k10_serial,
    lexbfs_benchmark_k100_threads,
    lexbfs_benchmark_k100_serial,
    lexbfs_benchmark_k500_threads,
    */
    lexbfs_benchmark_k500_rayon,
    lexbfs_benchmark_k500_serial,
);
criterion_main!(benches);
