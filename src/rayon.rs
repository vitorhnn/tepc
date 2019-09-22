use std::cmp::Reverse;
use std::collections::BTreeSet;
use std::ptr::NonNull;

use petgraph::Undirected;
use petgraph::csr::Csr;
use petgraph::visit::NodeIndexable;

use rayon::prelude::*;

use crate::common::rose_cmp;

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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn diamond_graph() {
        let mut graph = Csr::new();

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
    }
}
