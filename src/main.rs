use std::collections::VecDeque;
use std::iter;

use indexmap::IndexSet;

use petgraph::graph::{NodeIndex, Neighbors};
use petgraph::{Graph, Undirected};

type U64Graph = Graph<u64, u64, Undirected>;

fn lex_bfs(graph: U64Graph) -> Vec<NodeIndex<u32>> {
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

        for neighbor in graph.neighbors(v) {

        }
    }

    output
}

fn main() {
    let mut graph = Graph::new_undirected();

    let a = graph.add_node(0);
    let b = graph.add_node(1);
    let c = graph.add_node(2);
    let d = graph.add_node(3);
    graph.add_edge(a, b, 0);
    graph.add_edge(a, c, 1);
    graph.add_edge(c, d, 2);

    lex_bfs(graph);

    println!("Hello, world!");
}
