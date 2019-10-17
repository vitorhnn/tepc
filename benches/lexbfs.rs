use std::fs::File;
use std::io::BufReader;

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

use petgraph::{csr::Csr, Undirected};

use tepc::common;
use tepc::rayon::naive_lex_bfs as rayon;
use tepc::serial::naive_lex_bfs as serial;
use tepc::threads::naive_lex_bfs as threaded;

pub fn load_graph(file_name: &str) -> Csr<(), (), Undirected> {
    let file = File::open(file_name).unwrap();

    common::graph_from_reader(BufReader::new(file)).unwrap()
}

pub fn lexbfs(c: &mut Criterion) {
    let sizes = [10, 100, 500];
    let mut group = c.benchmark_group("lexbfs");

    for size in sizes.iter() {
        let graph = load_graph(&format!("k{}.txt", size));

        group.bench_with_input(BenchmarkId::new("rayon", size), &graph, |b, graph| {
            b.iter(|| rayon(graph));
        });

        group.bench_with_input(BenchmarkId::new("serial", size), &graph, |b, graph| {
            b.iter(|| serial(graph));
        });

        group.bench_with_input(BenchmarkId::new("threads", size), &graph, |b, graph| {
            b.iter(|| threaded(graph));
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    lexbfs
);
criterion_main!(benches);
