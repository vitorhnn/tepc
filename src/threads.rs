use std::cmp::{Eq, Reverse};
use std::collections::{BTreeSet, HashSet};
use std::hash::Hash;
use std::marker::Send;
use std::ptr::NonNull;
use std::slice;

use petgraph::csr::Csr;
use petgraph::visit::{EdgeRef, IntoEdgeReferences, NodeIndexable};
use petgraph::Undirected;

use scoped_threadpool::Pool;

use crossbeam::channel::unbounded;

use crate::common::{complete_graph_edge_count, rose_cmp};

type Graph = Csr<(), (), Undirected>;

struct Sendable<T>(*const T);
struct MutSendable<T>(NonNull<T>);

unsafe impl<T> Send for Sendable<T> {}
unsafe impl<T> Send for MutSendable<T> {}

fn select2(pool: &mut Pool, sets: &[BTreeSet<Reverse<usize>>], numbered: &Vec<bool>) -> usize {
    let enumerated: Vec<(usize, &BTreeSet<Reverse<usize>>)> = sets.iter().enumerate().collect();
    let mut element_count = sets.len();

    // filter
    let (sender, receiver) = unbounded();
    let chunk_size = (element_count / pool.thread_count() as usize) + 1;
    pool.scoped(|scope| {
        for chunk in enumerated.chunks(chunk_size) {
            let sender = sender.clone();
            scope.execute(move || {
                chunk
                    .iter()
                    .filter(|(idx, _)| !numbered[*idx])
                    .for_each(|&pair| sender.send(pair).unwrap());
            });
        }
    });

    drop(sender);

    let mut res: Vec<(usize, &BTreeSet<Reverse<usize>>)> = receiver.iter().collect();

    // reduce
    while element_count != 1 {
        let (sender, receiver) = unbounded();
        let chunk_size = (element_count / pool.thread_count() as usize).max(2);
        pool.scoped(|scope| {
            for chunk in res.chunks(chunk_size) {
                let sender = sender.clone();
                scope.execute(move || {
                    let local_max = chunk
                        .iter()
                        .max_by(|(_, a), (_, b)| rose_cmp(a, b))
                        .unwrap();

                    sender.send(*local_max).unwrap();
                });
            }
        });

        drop(sender);
        res = receiver.iter().collect();

        element_count = res.len();
    }

    res[0].0
}

pub fn naive_lex_bfs(pool: &mut Pool, graph: &Graph) -> Vec<i32> {
    let n = graph.node_count();

    let mut sets: Vec<_> = vec![BTreeSet::new(); n];
    let mut output = vec![0; n];
    let mut numbered = vec![false; n];

    for i in (0..n).rev() {
        let max_set = select2(pool, &sets, &numbered);

        output[i] = max_set as i32;
        numbered[max_set] = true;

        let neighbors = graph.neighbors_slice(graph.from_index(max_set));
        let chunk_size = (neighbors.len() / pool.thread_count() as usize) + 1;

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

fn filter_reduce_edge_count<T, I>(pool: &mut Pool, edges: &[T], maybe_clique: &HashSet<I>) -> usize
where
    I: Eq + Hash + Send + Sync,
    T: EdgeRef<NodeId = I> + Send + Sync,
{
    // filter
    let (sender, receiver) = unbounded();
    let chunk_size = (edges.len() / pool.thread_count() as usize) + 1;
    pool.scoped(|scope| {
        for chunk in edges.chunks(chunk_size) {
            let sender = sender.clone();
            scope.execute(move || {
                chunk
                    .iter()
                    .filter(|x| {
                        maybe_clique.contains(&(x.source())) && maybe_clique.contains(&(x.target()))
                    })
                    .for_each(|_| sender.send(1).unwrap());
            });
        }
    });

    drop(sender);
    let mut res: Vec<usize> = receiver.iter().collect();

    if res.len() == 0 {
        return 0;
    }

    while res.len() != 1 {
        let (sender, receiver) = unbounded();
        let chunk_size = (res.len() / pool.thread_count() as usize).max(2);

        pool.scoped(|scope| {
            for chunk in res.chunks(chunk_size) {
                let sender = sender.clone();
                scope.execute(move || {
                    let local_count = chunk.iter().sum();

                    sender.send(local_count).unwrap();
                });
            }
        });

        drop(sender);
        res = receiver.iter().collect();
    }

    res[0] / 2
}

// Now, we're gonna catch the output from our naive lex-bfs
// and test if it is a PES (EEP)
pub fn is_pes(pool: &mut Pool, scheme: &[i32], graph: &Graph) -> bool {
    let mut maybe_clique: HashSet<u32> = HashSet::with_capacity(scheme.len());
    let mut eliminated_vertices: HashSet<u32> = HashSet::with_capacity(scheme.len());
    let mut neighborhood = HashSet::with_capacity(scheme.len());
    let mut edges = Vec::with_capacity(graph.edge_count());
    for eliminated_vertex in scheme {
        let eliminated_vertex = *eliminated_vertex as u32;
        eliminated_vertices.insert(eliminated_vertex);

        neighborhood.extend(graph.neighbors_slice(eliminated_vertex).iter().cloned());

        maybe_clique.clear();
        maybe_clique.extend(neighborhood.difference(&eliminated_vertices));

        neighborhood.clear();

        edges.clear();
        edges.extend(graph.edge_references());

        let subgraph_edge_count = filter_reduce_edge_count(pool, &edges, &maybe_clique);

        if subgraph_edge_count != complete_graph_edge_count(maybe_clique.len()) {
            return false;
        }
    }

    true
}

pub fn is_chordal(pool: &mut Pool, graph: &Graph) -> bool {
    let scheme = naive_lex_bfs(pool, &graph);

    is_pes(pool, &scheme, graph)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diamond_graph_threads() {
        let cpucount = num_cpus::get();
        let mut pool = Pool::new(cpucount as u32);

        let mut graph = Graph::new();

        let a = graph.add_node(());
        let b = graph.add_node(());
        let c = graph.add_node(());
        let d = graph.add_node(());

        graph.add_edge(a, b, ());
        graph.add_edge(a, c, ());
        graph.add_edge(a, d, ());
        graph.add_edge(b, c, ());
        graph.add_edge(b, d, ());
        graph.add_edge(c, d, ());

        println!("{:?}", graph);

        let res = naive_lex_bfs(&mut pool, &graph);

        println!("{:?}", res);

        assert_eq!(is_pes(&mut pool, &res, &graph), true);
    }

    #[test]
    fn gem_graph() {
        let cpucount = num_cpus::get();
        let mut pool = Pool::new(cpucount as u32);

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

        let res = naive_lex_bfs(&mut pool, &graph);

        println!("{:?}", res);

        assert_eq!(is_pes(&mut pool, &res, &graph), true);
    }

    #[test]
    fn long_chordal() {
        let cpucount = num_cpus::get();
        let mut pool = Pool::new(cpucount as u32);

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

        let res = naive_lex_bfs(&mut pool, &graph);

        println!("{:?}", res);

        assert_eq!(is_pes(&mut pool, &res, &graph), true);
    }
}
