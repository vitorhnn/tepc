use std::cmp::Reverse;
use std::collections::BTreeSet;
use std::ops::Deref;
use std::sync::Arc;

use petgraph::csr::Csr;
use petgraph::visit::NodeIndexable;
use petgraph::Undirected;

use crossbeam_utils::thread::scope;

use num_cpus;

mod common;
use common::rose_cmp;

type Graph = Csr<(), (), Undirected>;

pub fn naive_lex_bfs(graph: Graph) -> Vec<i32> {
    let cpucount = num_cpus::get();
    let n = graph.node_count();

    // this is mutable, btw. we're just using unsafe
    let sets: Arc<Vec<_>> = Arc::new(vec![BTreeSet::new(); n]);
    let mut output = vec![0; n];
    let mut numbered = Arc::new(vec![false; n]);

    for i in (0..n).rev() {
        let max_set = sets
            .iter()
            .enumerate()
            .filter(|&(idx, _)| !numbered[idx])
            .max_by(|&(_, a), &(_, b)| rose_cmp(a.deref(), &*b))
            .expect("output vector was empty");

        output[i] = max_set.0 as i32;
        Arc::get_mut(&mut numbered).expect("this arc should never fail")[max_set.0] = true;

        let neighbors = graph.neighbors_slice(graph.from_index(max_set.0));
        let mut chunk_size = neighbors.len() / cpucount;

        if chunk_size == 0 {
            chunk_size = 1;
        }

        println!("chunk size {}, neighbors {}", chunk_size, neighbors.len());

        scope(|s| {
            neighbors.chunks(chunk_size).for_each(|chunk| {
                println!("thread will get {} elements", chunk.len());
                let sets = sets.clone();
                let numbered = numbered.clone();
                s.spawn(move |_| {
                    chunk
                        .iter()
                        .filter(|&neighbor| unsafe {
                            let ptr = (numbered.as_ptr() as *mut bool).offset(*neighbor as isize);
                            !*ptr
                        })
                        .for_each(|w| unsafe {
                            let ptr = (sets.as_ptr() as *mut BTreeSet<Reverse<usize>>)
                                .offset(*w as isize);

                            (*ptr).insert(Reverse(i));
                        })
                });
            });
        })
        .expect("worker thread panicked");
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn diamond_graph() {
        let mut graph = Graph::new();

        let a = graph.add_node(());
        let b = graph.add_node(());
        let c = graph.add_node(());
        let d = graph.add_node(());

        graph.add_edge(a, b, ());
        graph.add_edge(a, c, ());
        graph.add_edge(b, c, ());
        graph.add_edge(b, d, ());
        graph.add_edge(c, d, ());

        println!("{:?}", graph);

        let res = naive_lex_bfs(graph);

        println!("{:?}", res);
    }

    #[test]
    fn gem_graph() {
        let mut graph = Graph::new();

        let a = graph.add_node(());
        let b = graph.add_node(());
        let c = graph.add_node(());
        let d = graph.add_node(());
        let e = graph.add_node(());

        graph.add_edge(a, b, ());
        graph.add_edge(b, c, ());
        graph.add_edge(b, d, ());
        graph.add_edge(b, e, ());
        graph.add_edge(c, d, ());
        graph.add_edge(d, e, ());
        graph.add_edge(e, a, ());

        println!("{:?}", graph);

        let res = naive_lex_bfs(graph);

        println!("{:?}", res);
    }

    #[test]
    fn long_chordal() {
        let mut graph = Graph::new();

        let a = graph.add_node(());
        let b = graph.add_node(());
        let c = graph.add_node(());
        let d = graph.add_node(());
        let e = graph.add_node(());
        let f = graph.add_node(());
        let g = graph.add_node(());

        graph.add_edge(a, b, ());
        graph.add_edge(b, c, ());
        graph.add_edge(c, d, ());

        graph.add_edge(g, f, ());
        graph.add_edge(f, e, ());

        graph.add_edge(g, b, ());
        graph.add_edge(b, f, ());
        graph.add_edge(f, c, ());
        graph.add_edge(c, e, ());

        graph.add_edge(a, g, ());
        graph.add_edge(e, d, ());

        println!("{:?}", graph);

        let res = naive_lex_bfs(graph);

        println!("{:?}", res);
    }
}

fn main() {}
