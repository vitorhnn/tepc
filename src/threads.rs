use std::cmp::Reverse;
use std::collections::BTreeSet;
use std::marker::Send;
use std::ptr::NonNull;
use std::slice;

use petgraph::csr::Csr;
use petgraph::visit::NodeIndexable;
use petgraph::Undirected;

use scoped_threadpool::Pool;

use num_cpus;

use crate::common::rose_cmp;

type Graph = Csr<(), (), Undirected>;

struct Sendable<T>(*const T);
struct MutSendable<T>(NonNull<T>);

unsafe impl<T> Send for Sendable<T> {}
unsafe impl<T> Send for MutSendable<T> {}

pub fn naive_lex_bfs(graph: &Graph) -> Vec<i32> {
    let cpucount = num_cpus::get();
    let mut pool = Pool::new(cpucount as u32);
    let n = graph.node_count();

    let mut sets: Vec<_> = vec![BTreeSet::new(); n];
    let mut output = vec![0; n];
    let mut numbered = vec![false; n];

    for i in (0..n).rev() {
        let max_set = sets
            .iter()
            .enumerate()
            .filter(|&(idx, _)| !numbered[idx])
            .max_by(|&(_, a), &(_, b)| rose_cmp(a, b))
            .expect("output vector was empty");

        output[i] = max_set.0 as i32;
        numbered[max_set.0] = true;

        let neighbors = graph.neighbors_slice(graph.from_index(max_set.0));
        let chunk_size = (neighbors.len() / cpucount) + 1;

        pool.scoped(|scope| {
            neighbors.chunks(chunk_size).for_each(|chunk| {
                let numbered_ptr = MutSendable(NonNull::new(numbered.as_mut_ptr()).unwrap());
                let sets_ptr = MutSendable(NonNull::new(sets.as_mut_ptr()).unwrap());
                let chunk_ptr = Sendable(chunk.as_ptr());
                let chunk_len = chunk.len();
                scope.execute(move || unsafe {
                    let chunk = slice::from_raw_parts(chunk_ptr.0, chunk_len);
                    chunk
                        .iter()
                        .filter(|&neighbor| {
                            let ptr = (numbered_ptr.0.as_ptr()).offset(*neighbor as isize);
                            !*ptr
                        })
                        .for_each(|w| {
                            let ptr = (sets_ptr.0.as_ptr()).offset(*w as isize);

                            (*ptr).insert(Reverse(i));
                        });
                });
            });
        });
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common;
    use std::fs::File;
    use std::io::BufReader;
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

        let res = naive_lex_bfs(&graph);

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

        let res = naive_lex_bfs(&graph);

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

        let res = naive_lex_bfs(&graph);

        println!("{:?}", res);
    }

    #[test]
    fn from_file() {
        let file = File::open("k500.txt").unwrap();

        let graph = common::graph_from_reader(BufReader::new(file)).unwrap();

        println!("{:?}", graph);

        let res = naive_lex_bfs(&graph);

        println!("{:?}", res);
    }
}
