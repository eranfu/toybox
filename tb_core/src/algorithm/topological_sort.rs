use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet, LinkedList};
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::hash::Hash;

struct Node<T> {
    item: T,
    dependencies: HashSet<T>,
}

pub struct TopologicalGraph<T: Eq + Hash + Clone> {
    nodes: HashMap<T, Node<T>>,
}

impl<T: Eq + Hash + Clone> TopologicalGraph<T> {
    pub fn add_item(&mut self, item: T) {
        self.nodes.entry(item).or_insert_with_key(|item| Node {
            item: item.clone(),
            dependencies: Default::default(),
        });
    }

    /// # Description
    /// `a` dependency `b`
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
    /// `a` dependency `b`
    pub fn add_dependency_if_non_inverse(&mut self, a: T, b: T) {
        let node_b = match self.nodes.get(&b) {
            None => return,
            Some(node_b) => node_b,
        };
        if node_b.dependencies.contains(&a) {
            return;
        }
        self.add_dependency(a, b)
    }

    pub fn visit_with_flag<F: Clone + Ord>(&self) -> VisitorWithFlag<T, F> {
        VisitorWithFlag::new(self)
    }
}

impl<T: Eq + Hash + Clone> Default for TopologicalGraph<T> {
    fn default() -> Self {
        Self {
            nodes: Default::default(),
        }
    }
}

pub struct VisitorWithFlag<'d, T: Eq + Hash + Clone, F: Clone + Ord> {
    graph: &'d TopologicalGraph<T>,
    flags: HashMap<T, F>,
    visited: HashSet<T>,
    node_iter: std::collections::hash_map::Iter<'d, T, Node<T>>,
    visiting_stack: LinkedList<(T, std::collections::hash_set::Iter<'d, T>)>,
    visiting_items: HashSet<T>,
    has_error: bool,
}

impl<'d, T: Eq + Hash + Clone, F: Clone + Ord> VisitorWithFlag<'d, T, F> {
    pub fn new(graph: &'d TopologicalGraph<T>) -> Self {
        Self {
            graph,
            flags: Default::default(),
            visited: Default::default(),
            node_iter: graph.nodes.iter(),
            visiting_stack: Default::default(),
            visiting_items: Default::default(),
            has_error: false,
        }
    }
}

impl<'d, T: Eq + Hash + Clone, F: 'd + Clone + Ord> Iterator for VisitorWithFlag<'d, T, F> {
    type Item = Result<(T, Flag<'d, T, F>), VisitingError>;

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
                    if let Some(child_flag) = self.flags.get(child) {
                        let child_flag = child_flag.clone();
                        set_flag(&mut self.flags, current.0.clone(), child_flag);
                    }
                } else if self.visiting_items.insert(child.clone()) {
                    let child_node = self.graph.nodes.get(child).unwrap();
                    self.visiting_stack
                        .push_back((child_node.item.clone(), child_node.dependencies.iter()));
                } else {
                    self.has_error = true;
                    return Some(Err(VisitingError::CircularDependency));
                }
            } else {
                break;
            }
        }

        let current = self.visiting_stack.pop_back().unwrap().0;
        self.visited.insert(current.clone());
        self.visiting_items.remove(&current);
        if let Some(parent) = self.visiting_stack.back() {
            if let Some(current_flag) = self.flags.get(&current) {
                let current_flag = current_flag.clone();
                set_flag(&mut self.flags, parent.0.clone(), current_flag);
            }
        }
        Some(Ok((
            current.clone(),
            Flag::<'d, T, F> {
                item: current,
                flags: unsafe { std::mem::transmute(&mut self.flags) },
                visiting_stack: unsafe { std::mem::transmute(&self.visiting_stack) },
            },
        )))
    }
}

#[derive(Debug)]
pub enum VisitingError {
    CircularDependency,
}

impl Display for VisitingError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            VisitingError::CircularDependency => {
                Display::fmt("VisitingError::CircularDependency", f)
            }
        }
    }
}

impl Error for VisitingError {}

pub struct Flag<'d, T: Eq + Hash + Clone, F: Clone + Ord> {
    item: T,
    flags: &'d mut HashMap<T, F>,
    visiting_stack: &'d LinkedList<(T, std::collections::hash_set::Iter<'d, T>)>,
}

impl<'d, T: Eq + Hash + Clone, F: Clone + Ord> Flag<'d, T, F> {
    pub fn get(&self) -> Option<&F> {
        self.flags.get(&self.item)
    }

    pub fn set_flag(&mut self, flag: F) {
        if set_flag(self.flags, self.item.clone(), flag.clone()) {
            if let Some(parent) = self.visiting_stack.back() {
                set_flag(self.flags, parent.0.clone(), flag);
            }
        }
    }
}

fn set_flag<T: Eq + Hash, F: Ord>(flags: &mut HashMap<T, F>, item: T, flag: F) -> bool {
    match flags.entry(item) {
        Entry::Occupied(mut entry) => {
            let old = entry.get_mut();
            if flag > *old {
                *old = flag;
                true
            } else {
                false
            }
        }
        Entry::Vacant(entry) => {
            entry.insert(flag);
            true
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::algorithm::topological_sort::{TopologicalGraph, VisitingError};

    #[test]
    fn it_works() {
        let mut t = TopologicalGraph::default();
        t.add_dependency(2, 3);
        t.add_dependency(1, 3);
        t.add_dependency(1, 2);

        let assert_result = vec![3, 2, 1];
        let assert_flag_result = vec![0, 1, 2];
        let mut result = vec![];
        let mut flag_result = vec![];
        for e in t.visit_with_flag() {
            let (item, mut flag) = e.unwrap();
            result.push(item);
            flag.set_flag(flag.get().map_or(0, |current_flag| current_flag + 1));
            flag_result.push(*flag.get().unwrap());
        }
        assert_eq!(result, assert_result);
        assert_eq!(flag_result, assert_flag_result);
    }

    #[test]
    #[should_panic(expected = "CircularDependency")]
    fn circular_dependency() {
        let mut t = TopologicalGraph::default();
        t.add_dependency(1, 2);
        t.add_dependency(1, 3);
        t.add_dependency(2, 3);
        t.add_dependency(3, 1);
        for e in t.visit_with_flag::<usize>() {
            let (_item, mut _flag) = e.unwrap();
        }
    }

    #[test]
    fn set_flag_failed() {
        let mut t = TopologicalGraph::default();
        t.add_dependency(2, 3);
        t.add_dependency(1, 3);
        t.add_dependency(1, 2);

        let assert_result = vec![3, 2, 1];
        let assert_flag_result = vec![10, 10, 10];
        let mut result = vec![];
        let mut flag_result = vec![];
        for e in t.visit_with_flag() {
            let (item, mut flag) = e.unwrap();
            result.push(item);
            flag.set_flag(flag.get().map_or(10, |current_flag| current_flag - 1));
            flag_result.push(*flag.get().unwrap());
        }
        assert_eq!(result, assert_result);
        assert_eq!(flag_result, assert_flag_result);
    }

    #[test]
    fn display_visiting_error() {
        let e = VisitingError::CircularDependency;
        println!("{}", e)
    }
}
