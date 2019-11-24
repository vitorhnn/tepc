use std::fs::File;
use std::io::BufReader;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

use petgraph::{csr::Csr, Undirected};

use scoped_threadpool::Pool;

use tepc::common;
use tepc::rayon::is_chordal as rayon;
use tepc::serial::is_chordal as serial;
use tepc::threads::is_chordal as threaded;

pub fn load_graph(file_name: &str) -> Csr<(), (), Undirected> {
    let file = File::open(file_name).unwrap();

    common::graph_from_reader(BufReader::new(file)).unwrap()
}

pub fn low_samples() -> Criterion {
    Criterion::default().sample_size(10)
}

pub fn lexbfs(c: &mut Criterion) {
    let cpucount = num_cpus::get();
    let mut pool = Pool::new(cpucount as u32);

    let sizes = [10, 100, 500];
    let mut group = c.benchmark_group("lexbfs");

    for size in sizes.iter() {
        let graph = load_graph(&format!("k{}.txt", size));

        group.bench_with_input(BenchmarkId::new("rayon", size), &graph, |b, graph| {
            b.iter(|| rayon(graph));
        });

        group.bench_with_input(BenchmarkId::new("threads", size), &graph, |b, graph| {
            b.iter(|| threaded(&mut pool, graph));
        });

        group.bench_with_input(BenchmarkId::new("serial", size), &graph, |b, graph| {
            b.iter(|| serial(graph));
        });
    }

    group.finish();
}

criterion_group! {name = benches; config = low_samples(); targets = lexbfs}
criterion_main!(benches);
