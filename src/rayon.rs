use std::cmp::Reverse;
use std::collections::{BTreeSet, HashSet};
use std::ptr::NonNull;

use petgraph::csr::Csr;
use petgraph::visit::{EdgeRef, IntoEdgeReferences, NodeIndexable};
use petgraph::Undirected;

use rayon::prelude::*;

use crate::common::{complete_graph_edge_count, rose_cmp};

type Graph = Csr<(), (), Undirected>;

struct UnsafeMutPtr<T>(NonNull<T>);

unsafe impl<T> Sync for UnsafeMutPtr<T> {}
unsafe impl<T> Send for UnsafeMutPtr<T> {}

// A naive implementation of Rose's LexBFS algorithm
// Not optimal, as the search for an unnumbered vertex with the largest label is O(n) here
// Thus, this is O(n²), as opposed to O(n) in Rose's paper
pub fn naive_lex_bfs(graph: &Graph) -> Vec<i32> {
    let n = graph.node_count();

    // assigning ∅ to all vertices
    let mut sets: Vec<_> = vec![BTreeSet::new(); n];
    // initialize α to nothing
    let mut output = vec![0; n];
    // helper to check if a vertex is numbered
    // increases space complexity, but is very fast
    let mut numbered = vec![false; n];

    for i in (0..n).rev() {
        // "select" portion of the algorithm
        // this is the not optimal part, as we have to make a linear search on the sets
        let max_set = sets
            .par_iter()
            .enumerate()
            .filter(|&(idx, _)| !numbered[idx])
            .max_by(|&(_, a), &(_, b)| rose_cmp(&a, &b))
            .expect("output vector was empty");

        let v = max_set.0;

        // α(i) = v
        output[i] = v as i32;
        numbered[max_set.0] = true;

        // "update2"
        // afaik optimal
        let sets_ptr = UnsafeMutPtr(NonNull::new(sets.as_mut_ptr()).unwrap());

        graph
            .neighbors_slice(graph.from_index(v))
            .par_iter()
            .filter(|&neighbor| !numbered[*neighbor as usize])
            .for_each(move |w| unsafe {
                let ptr = sets_ptr.0.as_ptr();
                (*ptr.add(*w as usize)).insert(Reverse(i));
            });
    }

    output
}

// Now, we're gonna catch the output from our naive lex-bfs
// and test if it is a PES (EEP)
pub fn is_pes(scheme: &[i32], graph: &Graph) -> bool {
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

        // Dividing by two here because petgraph counts inbound and outbound vertices
        edges.clear();
        edges.extend(graph.edge_references());
        let subgraph_edge_count = edges
            .par_iter()
            .filter(|x| {
                maybe_clique.contains(&(x.source())) && maybe_clique.contains(&(x.target()))
            })
            .count()
            / 2;

        if subgraph_edge_count != complete_graph_edge_count(maybe_clique.len()) {
            return false;
        }
    }

    true
}

pub fn is_chordal(graph: &Graph) -> bool {
    let scheme = naive_lex_bfs(&graph);

    is_pes(&scheme, graph)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::graph_from_reader;
    use std::fs::File;
    use std::io::BufReader;
    #[test]
    fn diamond_graph_rayon() {
        let mut graph = Csr::new();

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

        let res = naive_lex_bfs(&graph);

        println!("{:?}", res);

        assert_eq!(is_pes(&res, &graph), true);
    }

    #[test]
    fn gem_graph_rayon() {
        let mut graph = Csr::new();

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

        assert_eq!(is_pes(&res, &graph), true);
    }

    #[test]
    fn long_chordal() {
        let mut graph = Csr::new();

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

        assert_eq!(is_pes(&res, &graph), true);
    }

    #[test]
    fn not_chordal() {
        let mut graph = Csr::new();

        let a = graph.add_node(());
        let b = graph.add_node(());
        let c = graph.add_node(());
        let d = graph.add_node(());

        graph.add_edge(a, b, ());
        graph.add_edge(b, c, ());
        graph.add_edge(c, d, ());
        graph.add_edge(d, a, ());

        let res = naive_lex_bfs(&graph);

        assert_eq!(is_pes(&res, &graph), false);
    }

    #[test]
    fn from_file_rayon() {
        let file = File::open("k100.txt").unwrap();

        let graph = graph_from_reader(BufReader::new(file)).unwrap();

        let res = naive_lex_bfs(&graph);

        assert_eq!(is_pes(&res, &graph), true);
    }
}
