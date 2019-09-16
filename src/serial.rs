use std::cmp::Reverse;
use std::collections::{BTreeSet, VecDeque};
use std::iter;

use indexmap::IndexSet;

use petgraph::graph::NodeIndex;
use petgraph::{Graph, Undirected};

mod common;
use common::rose_cmp;

type U32Graph = Graph<u32, u32, Undirected>;

// unfinished. Habib's algorithm
pub fn lex_bfs(graph: U32Graph) -> Vec<NodeIndex<u32>> {
    let mut sets: VecDeque<_> = iter::once(graph.node_indices().collect::<IndexSet<_>>()).collect();
    let mut output = Vec::with_capacity(graph.node_count());

    while !sets.is_empty() {
        let set = &mut sets[0];
        let v = set.pop().expect("Failed to pop element from set");

        if set.is_empty() {
            sets.pop_front().unwrap(); // this should never fail
        }

        // yield v
        output.push(v);

        let neighborhood: IndexSet<_> = graph.neighbors(v).collect();

        for set in &sets {
            let _diff = set.difference(&neighborhood);
            let _intersection = set.intersection(&neighborhood);
        }
    }

    output
}

// A naive implementation of Rose's LexBFS algorithm
// Not optimal, as the search for an unnumbered vertex with the largest label is O(n) here
// Thus, this is O(n²), as opposed to O(n) in Rose's paper
pub fn naive_lex_bfs(graph: U32Graph) -> Vec<i32> {
    // assigning ∅ to all vertices
    let mut sets: Vec<_> = graph.node_indices().map(|_| BTreeSet::new()).collect();
    // initialize α to nothing
    let mut output = vec![0; graph.node_count()];
    // helper to check if a vertex is numbered
    // increases space complexity, but is very fast
    let mut numbered = vec![false; graph.node_count()];

    for i in (0..graph.node_count()).rev() {
        // "select" portion of the algorithm
        // this is the not optimal part, as we have to make a linear search on the sets
        let max_set = sets
            .iter()
            .enumerate()
            .filter(|&(idx, _)| !numbered[idx])
            .max_by(|&(_, a), &(_, b)| rose_cmp(&a, &b))
            .expect("output vector was empty");

        // α(i) = v
        output[i] = max_set.0 as i32;
        numbered[max_set.0] = true;

        // "update2"
        // afaik optimal
        graph
            .neighbors(NodeIndex::new(max_set.0))
            .filter(|&neighbor| !numbered[neighbor.index()])
            .for_each(|w| {
                sets[w.index()].insert(Reverse(i));
            });
    }

    output
}

/*
//Now, we're gonna catch the output from our naive lex-bfs
//and test if it is a PES (EEP)
pub fn is_pes(scheme: Vec<i32>, graph: U32Graph) -> bool {
    let answer: bool = false;
    let iterator = scheme.iter();

    for val in iterator {
        graph.neighbors(NodeIndex::new(*val as usize)).filter(|x|);
        //ok, i gave up by now, i'm gonna save it and retry later...
    }

    answer
}
*/

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn diamond_graph() {
        let mut graph = Graph::new_undirected();

        let a = graph.add_node(0);
        let b = graph.add_node(1);
        let c = graph.add_node(2);
        let d = graph.add_node(3);

        graph.add_edge(a, b, 0);
        graph.add_edge(a, c, 0);
        graph.add_edge(b, c, 0);
        graph.add_edge(b, d, 0);
        graph.add_edge(c, d, 0);

        println!("{:?}", graph);

        let res = naive_lex_bfs(graph);

        println!("{:?}", res);
    }

    #[test]
    fn gem_graph() {
        let mut graph = Graph::new_undirected();

        let a = graph.add_node(0);
        let b = graph.add_node(1);
        let c = graph.add_node(2);
        let d = graph.add_node(3);
        let e = graph.add_node(4);

        graph.add_edge(a, b, 0);
        graph.add_edge(b, c, 0);
        graph.add_edge(b, d, 0);
        graph.add_edge(b, e, 0);
        graph.add_edge(c, d, 0);
        graph.add_edge(d, e, 0);
        graph.add_edge(e, a, 0);

        println!("{:?}", graph);

        let res = naive_lex_bfs(graph);

        println!("{:?}", res);
    }

    #[test]
    fn long_chordal() {
        let mut graph = Graph::new_undirected();

        let a = graph.add_node(0);
        let b = graph.add_node(1);
        let c = graph.add_node(2);
        let d = graph.add_node(3);
        let e = graph.add_node(4);
        let f = graph.add_node(5);
        let g = graph.add_node(6);

        graph.add_edge(a, b, 0);
        graph.add_edge(b, c, 0);
        graph.add_edge(c, d, 0);

        graph.add_edge(g, f, 0);
        graph.add_edge(f, e, 0);

        graph.add_edge(g, b, 0);
        graph.add_edge(b, f, 0);
        graph.add_edge(f, c, 0);
        graph.add_edge(c, e, 0);

        graph.add_edge(a, g, 0);
        graph.add_edge(e, d, 0);

        println!("{:?}", graph);

        let res = naive_lex_bfs(graph);

        println!("{:?}", res);
    }
}

fn main() {}
