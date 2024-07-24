use std::{
    collections::{BTreeSet, VecDeque},
    ops::{Deref, DerefMut},
};

#[derive(Clone, Default)]
pub struct Graph<T, E = ()> {
    nodes: Vec<T>,
    edges: Vec<(usize, usize, E)>,
}

pub struct Node<'a, T, E> {
    graph: &'a Graph<T, E>,
    idx: usize,
}

impl<'a, T, E> Clone for Node<'a, T, E> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, T, E> Copy for Node<'a, T, E> {}

impl<'a, T, E> Node<'a, T, E> {
    pub fn neighbors(&self) -> Neighbors<'a, T, E> {
        Neighbors {
            graph: self.graph,
            center_idx: self.idx,
            edge_idx: 0,
        }
    }

    pub fn breadth_first(&self) -> BreadthFirst<'a, T, E> {
        BreadthFirst {
            queue: vec![*self].into(),
            visited: BTreeSet::new(),
        }
    }

    pub fn depth_first(&self) -> DepthFirst<'a, T, E> {
        DepthFirst {
            graph: self.graph,
            stack: vec![self.idx],
            visited: BTreeSet::new(),
        }
    }

    pub fn clone_inner(&self) -> T
    where
        T: Clone,
    {
        self.graph.nodes[self.idx].clone()
    }
}

impl<'a, T, E> Deref for Node<'a, T, E> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.graph.nodes[self.idx]
    }
}

pub struct Neighbors<'a, T, E> {
    graph: &'a Graph<T, E>,
    center_idx: usize,
    edge_idx: usize,
}

impl<'a, T, E> Iterator for Neighbors<'a, T, E> {
    type Item = Node<'a, T, E>;
    fn next(&mut self) -> Option<Self::Item> {
        for (start, end, _content) in self.graph.edges.iter().skip(self.edge_idx) {
            self.edge_idx += 1;
            if *start == self.center_idx {
                return Some(Node {
                    graph: self.graph,
                    idx: *end,
                });
            }
        }
        None
    }
}

pub struct NodeMut<'a, T, E = ()> {
    graph: &'a mut Graph<T, E>,
    idx: usize,
}

impl<'a, T, E> NodeMut<'a, T, E> {
    pub fn neighbors(&'a self) -> Neighbors<'a, T, E> {
        Neighbors {
            graph: self.graph,
            center_idx: self.idx,
            edge_idx: 0,
        }
    }
}

impl<'a, T, E> Deref for NodeMut<'a, T, E> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.graph.nodes[self.idx]
    }
}

impl<'a, T, E> DerefMut for NodeMut<'a, T, E> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.graph.nodes[self.idx]
    }
}

pub struct DepthFirst<'a, T, E> {
    graph: &'a Graph<T, E>,
    stack: Vec<usize>,
    visited: BTreeSet<usize>,
}

impl<'a, T, E> Iterator for DepthFirst<'a, T, E> {
    type Item = Node<'a, T, E>;

    fn next(&mut self) -> Option<Self::Item> {
        let idx = self.stack.pop()?;
        for (start, end, _) in &self.graph.edges {
            let start = *start;
            let end = *end;
            if start != idx {
                continue;
            }
            if self.visited.contains(&end) {
                continue;
            }
            self.stack.push(end);
            self.visited.insert(end);
        }
        Some(Node {
            graph: self.graph,
            idx,
        })
    }
}

pub struct BreadthFirst<'a, T, E> {
    queue: VecDeque<Node<'a, T, E>>,
    visited: BTreeSet<usize>,
}

impl<'a, T, E> Iterator for BreadthFirst<'a, T, E> {
    type Item = Node<'a, T, E>;
    fn next(&mut self) -> Option<Self::Item> {
        let next = self.queue.pop_front()?;
        for n in next.neighbors() {
            if self.visited.contains(&n.idx) {
                continue;
            }
            self.visited.insert(n.idx);
            self.queue.push_back(n);
        }
        Some(next)
    }
}
