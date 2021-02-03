use std::collections::{HashMap, HashSet};
use std::hash::Hash;

#[derive(Default)]
pub struct Dependency<T> {
    nodes: HashMap<T, Node<T>>,
}

impl<T: Eq + Hash + Copy> Dependency<T> {
    /// # Description
    /// `a` dependency `b`
    fn add_dependency(&mut self, a: T, b: T) {
        self.nodes.entry(b).or_insert_with_key(|b| Node {
            item: *b,
            dependencies: Default::default(),
        });
        self.nodes
            .entry(a)
            .or_insert_with_key(|a| Node {
                item: *a,
                dependencies: Default::default(),
            })
            .dependencies
            .insert(b);
    }
}

struct Node<T> {
    item: T,
    dependencies: HashSet<T>,
}
