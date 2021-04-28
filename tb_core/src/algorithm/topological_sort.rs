use std::collections::{HashMap, HashSet, LinkedList};
use std::hash::Hash;

use crate::error::*;

error_chain! {
    errors {
        CircularDependency
    }
}

struct Node<T> {
    item: T,
    dependencies: HashSet<T>,
}

pub struct TopologicalGraph<T: Eq + Hash + Clone> {
    nodes: HashMap<T, Node<T>>,
}

impl<T: Eq + Hash + Clone> TopologicalGraph<T> {
    pub fn clear(&mut self) {
        self.nodes.clear()
    }

    pub fn add_item(&mut self, item: T) {
        self.nodes.entry(item).or_insert_with_key(|item| Node {
            item: item.clone(),
            dependencies: Default::default(),
        });
    }

    /// # Description
    /// `a` depend on `b`
    pub fn add_dependency(&mut self, a: T, b: T) {
        if a == b {
            return;
        }
        self.nodes.entry(b.clone()).or_insert_with_key(|b| Node {
            item: b.clone(),
            dependencies: Default::default(),
        });
        self.nodes
            .entry(a)
            .or_insert_with_key(|a| Node {
                item: a.clone(),
                dependencies: Default::default(),
            })
            .dependencies
            .insert(b);
    }

    /// # Description
    /// `a` depend on `b`
    pub fn add_dependency_if_non_inverse(&mut self, a: T, b: T) {
        if a == b {
            return;
        }
        if self.is_dependent(&b, &a) {
            return;
        }
        self.add_dependency(a, b)
    }

    pub fn visit(&self) -> Visitor<T> {
        Visitor::new(self)
    }

    fn is_dependent(&self, a: &T, b: &T) -> bool {
        let a = match self.nodes.get(a) {
            None => {
                return false;
            }
            Some(a) => a,
        };

        if a.dependencies.contains(b) {
            return true;
        }

        for a in &a.dependencies {
            if self.is_dependent(a, b) {
                return true;
            }
        }

        false
    }
}

impl<T: Eq + Hash + Clone> Default for TopologicalGraph<T> {
    fn default() -> Self {
        Self {
            nodes: Default::default(),
        }
    }
}

pub struct Visitor<'d, T: Eq + Hash + Clone> {
    graph: &'d TopologicalGraph<T>,
    visited: HashSet<T>,
    node_iter: std::collections::hash_map::Iter<'d, T, Node<T>>,
    visiting_stack: LinkedList<(T, std::collections::hash_set::Iter<'d, T>)>,
    visiting_items: HashSet<T>,
    has_error: bool,
}

impl<'d, T: Eq + Hash + Clone> Visitor<'d, T> {
    pub fn new(graph: &'d TopologicalGraph<T>) -> Self {
        Self {
            graph,
            visited: Default::default(),
            node_iter: graph.nodes.iter(),
            visiting_stack: Default::default(),
            visiting_items: Default::default(),
            has_error: false,
        }
    }
}

impl<'d, T: Eq + Hash + Clone> Iterator for Visitor<'d, T> {
    type Item = Result<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.has_error {
            return None;
        }

        let visited = &self.visited;
        if self.visiting_stack.is_empty() {
            match self.node_iter.find(|node| !visited.contains(node.0)) {
                None => {
                    return None;
                }
                Some((item, node)) => {
                    self.visiting_items.insert(item.clone());
                    self.visiting_stack
                        .push_back((node.item.clone(), node.dependencies.iter()));
                }
            }
        };

        loop {
            let current = self.visiting_stack.back_mut().unwrap();
            if let Some(child) = current.1.next() {
                if visited.contains(child) {
                } else if self.visiting_items.insert(child.clone()) {
                    let child_node = self.graph.nodes.get(child).unwrap();
                    self.visiting_stack
                        .push_back((child_node.item.clone(), child_node.dependencies.iter()));
                } else {
                    self.has_error = true;
                    return Some(Err(ErrorKind::CircularDependency.into()));
                }
            } else {
                break;
            }
        }

        let current = self.visiting_stack.pop_back().unwrap().0;
        self.visited.insert(current.clone());
        self.visiting_items.remove(&current);
        Some(Ok(current))
    }
}

#[cfg(test)]
mod tests {
    use crate::algorithm::topological_sort::TopologicalGraph;

    #[test]
    fn it_works() {
        let mut t = TopologicalGraph::default();
        t.add_dependency(2, 3);
        t.add_dependency(1, 3);
        t.add_dependency(1, 2);

        let assert_result = vec![3, 2, 1];
        let mut result = vec![];
        for e in t.visit() {
            let item = e.unwrap();
            result.push(item);
        }
        assert_eq!(result, assert_result);
    }

    #[test]
    #[should_panic(expected = "CircularDependency")]
    fn circular_dependency() {
        let mut t = TopologicalGraph::default();
        t.add_dependency(1, 2);
        t.add_dependency(1, 3);
        t.add_dependency(2, 3);
        t.add_dependency(3, 1);
        for e in t.visit() {
            let _item = e.unwrap();
        }
    }
}
