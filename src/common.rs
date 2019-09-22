use std::cmp::{Ordering, Reverse};
use std::collections::{BTreeSet, HashSet};
use std::error::Error;
use std::io::BufRead;

use petgraph::csr::Csr;
use petgraph::Undirected;

// Everything that was written here was wrong. It's just standard lexicographical comparsion
// However, as we've wrapped the indices in Reverse to build a set, we have to undo that
pub fn rose_cmp(a: &BTreeSet<Reverse<usize>>, b: &BTreeSet<Reverse<usize>>) -> Ordering {
    a.iter()
        .map(|x| x.0)
        .cmp(b.iter().map(|y| y.0))
        .then(a.len().cmp(&b.len()))
}

pub fn graph_from_reader(reader: impl BufRead) -> Result<Csr<(), (), Undirected>, Box<dyn Error>> {
    let mut edges = HashSet::new();
    let mut node_count = 0;

    for (row, line_str) in reader.lines().enumerate() {
        let line_str = line_str?;
        let elements: Vec<_> = line_str.split_ascii_whitespace().collect();
        node_count = elements.len();
        elements
            .iter()
            .enumerate()
            .filter(|&(_, value)| value == &"1")
            .for_each(|(column, _)| {
                let max = column.max(row);
                let min = column.min(row);

                edges.insert((max as u32, min as u32));
            });
    }

    let mut graph = Csr::with_nodes(node_count);

    for (v, w) in edges.into_iter() {
        graph.add_edge(v, w, ());
    }

    Ok(graph)
}
