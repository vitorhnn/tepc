use std::cmp::{Ordering, Reverse};
use std::collections::BTreeSet;

// Everything that was written here was wrong. It's just standard lexicographical comparsion
// However, as we've wrapped the indices in Reverse to build a set, we have to undo that
pub fn rose_cmp(a: &BTreeSet<Reverse<usize>>, b: &BTreeSet<Reverse<usize>>) -> Ordering {
    a.iter()
        .map(|x| x.0)
        .cmp(b.iter().map(|y| y.0))
        .then(a.len().cmp(&b.len()))
}
