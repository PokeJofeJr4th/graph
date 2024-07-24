#![warn(clippy::pedantic, clippy::nursery)]
use std::{
    cmp::Ordering,
    collections::{btree_map, BTreeMap, BTreeSet, VecDeque},
    marker::PhantomData,
    ops::{Add, AddAssign, Deref, DerefMut},
    slice,
};

#[derive(Clone, Default)]
pub struct Graph<T, E = ()> {
    nodes: Vec<Adjacency<T, E>>,
}

impl<T, E> Graph<T, E> {
    /// Connect two nodes with a weight
    ///
    /// # Panics
    ///
    /// Panics if either the start or end node refers outside of the pool kept by the graph.
    pub fn connect_weighted(&mut self, start: WeakNode<T, E>, end: WeakNode<T, E>, weight: E) {
        assert!(
            start.0 < self.nodes.len(),
            "Attempt to create connection with a node that is not part of this graph."
        );
        assert!(
            end.0 < self.nodes.len(),
            "Attempt to create connection with a node that is not part of this graph."
        );
        self.nodes[start.0].edges.insert(end.0, weight);
    }

    /// Connect two nodes with a weight, using a bidirectional connection
    ///
    /// # Panics
    ///
    /// Panics if either the start or end node refers outside of the pool kept by the graph.
    pub fn connect_undirected_weighted(
        &mut self,
        start: WeakNode<T, E>,
        end: WeakNode<T, E>,
        weight: E,
    ) where
        E: Clone,
    {
        self.connect_weighted(start, end, weight.clone());
        self.connect_weighted(end, start, weight);
    }

    #[must_use]
    pub const fn arbitrary_node(&self) -> Node<'_, T, E> {
        Node {
            graph: self,
            idx: 0,
        }
    }

    /// Find one node in the graph whose content equals the provided value.
    pub fn find(&self, item: &T) -> Option<Node<'_, T, E>>
    where
        T: PartialEq,
    {
        Some(Node {
            graph: self,
            idx: self.nodes.iter().position(|p| &p.value == item)?,
        })
    }

    /// Convert a weak reference to a strong reference. See `WeakNode` and `Node` for more information.
    ///
    /// This will cause unexpected behavior if the provided `WeakNode` is not from this graph
    /// or if the graph has changed since the `WeakNode` reference was created.
    ///
    /// # Panics
    ///
    /// Panics if the node would return a reference outside of the pool kept by the graph.
    #[must_use]
    pub fn weak_ref(&self, node: WeakNode<T, E>) -> Node<'_, T, E> {
        assert!(
            node.0 < self.nodes.len(),
            "Attempt to use a weak ref past end of graph"
        );
        Node {
            graph: self,
            idx: node.0,
        }
    }

    /// Convert a weak reference to a strong mutable reference. See `WeakNode` and `NodeMut` for more information.
    ///
    /// This will cause unexpected behavior if the provided `WeakNode` is not from this graph
    /// or if the graph has changed since the `WeakNode` reference was created.
    ///
    /// # Panics
    ///
    /// Panics if the node would return a reference outside of the pool kept by the graph.
    #[must_use]
    pub fn weak_mut(&mut self, node: WeakNode<T, E>) -> NodeMut<'_, T, E> {
        assert!(
            node.0 < self.nodes.len(),
            "Attempt to use a weak ref past end of graph"
        );
        NodeMut {
            graph: self,
            idx: node.0,
        }
    }

    /// Returns the shortest path between two nodes, if a path exists and the edges can be manipulated and
    /// compared appropriately.
    ///
    /// # Panics
    ///
    /// Panics if either the start or end node is not part of this graph.
    #[must_use]
    pub fn dijkstras(&self, start: Node<'_, T, E>, end: Node<'_, T, E>) -> Option<Path<'_, T, E>>
    where
        E: Default + Clone + Ord + Add<E, Output = E>,
    {
        assert!(
            std::ptr::eq(self, start.graph),
            "Attempt to generate path for node outside of graph"
        );
        assert!(
            std::ptr::eq(self, end.graph),
            "Attempt to generate path for node outside of graph"
        );
        let mut remaining: Vec<_> = (0..self.nodes.len()).collect();
        let mut distance: Vec<_> = vec![None; self.nodes.len()];
        distance[start.idx] = Some(E::default());
        let mut predecessors: Vec<_> = vec![None; self.nodes.len()];

        'outer: while let Some((next_rem, &next)) =
            remaining.iter().enumerate().min_by(|(_, &a), (_, &b)| {
                match (&distance[a], &distance[b]) {
                    (Some(_), None) => Ordering::Less,
                    (None, None) => Ordering::Equal,
                    (None, Some(_)) => Ordering::Greater,
                    (Some(a), Some(b)) => a.cmp(b),
                }
            })
        {
            remaining.remove(next_rem);
            for (step, weight) in &self.nodes[next].edges {
                if next == end.idx {
                    break 'outer;
                }
                let new_weight = distance[next].clone()? + weight.clone();
                // if the old distance is less than the old one, do nothing.
                if distance[*step]
                    .as_ref()
                    .is_some_and(|dst| dst <= &new_weight)
                {
                    continue;
                }
                distance[*step] = Some(new_weight);
                predecessors[*step] = Some(next);
            }
        }
        let mut prev = end.idx;
        let mut path = Vec::new();
        while prev != start.idx {
            path.push(prev);
            prev = predecessors[prev]?;
        }
        path.reverse();
        Some(Path { graph: self, path })
    }
}

#[derive(Clone)]
struct Adjacency<T, E = ()> {
    value: T,
    edges: BTreeMap<usize, E>,
}

impl<T> Graph<T> {
    /// Create a directed connection between the two input vertices
    ///
    /// # Panics
    ///
    /// Panics if either the start or end node refers outside of the pool kept by the graph.
    pub fn connect(&mut self, start: WeakNode<T>, end: WeakNode<T>) {
        self.connect_weighted(start, end, ());
    }

    /// Create an undirected connection between the two input vertices
    ///
    /// # Panics
    ///
    /// Panics if either the start or end node refers outside of the pool kept by the graph.
    pub fn connect_undirected(&mut self, start: WeakNode<T>, end: WeakNode<T>) {
        self.connect_weighted(start, end, ());
        self.connect_weighted(end, start, ());
    }
}

/// A reference to a single node within a graph
pub struct Node<'a, T, E = ()> {
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
    /// Returns the neighbors of this `Node`.
    #[must_use]
    pub fn neighbors(&self) -> Neighbors<'a, T, E> {
        Neighbors {
            graph: self.graph,
            neighbors: self.graph.nodes[self.idx].edges.iter(),
        }
    }

    /// Returns the breadth-first iterator through the graph; starting from this `Node`.
    #[must_use]
    pub fn breadth_first(&self) -> BreadthFirst<'a, T, E> {
        BreadthFirst {
            queue: vec![*self].into(),
            visited: BTreeSet::new(),
        }
    }

    /// Returns the dept-first iterator through the graph; starting from this `Node`.
    #[must_use]
    pub fn depth_first(&self) -> DepthFirst<'a, T, E> {
        DepthFirst {
            graph: self.graph,
            stack: vec![self.idx],
            visited: BTreeSet::new(),
        }
    }

    /// Clones the inner value of this `Node`.
    #[must_use]
    pub fn clone_inner(&self) -> T
    where
        T: Clone,
    {
        self.graph.nodes[self.idx].value.clone()
    }

    /// Returns the weak reference of this `Node`.
    #[must_use]
    pub const fn weak(&self) -> WeakNode<T, E> {
        WeakNode(self.idx, PhantomData)
    }
}

impl<'a, T, E> Deref for Node<'a, T, E> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.graph.nodes[self.idx].value
    }
}

/// A weak reference to a node within a graph
///
/// # Safety
///
/// If you use a weak node from a different graph, unexpected behavior may occur.
pub struct WeakNode<T, E = ()>(usize, PhantomData<(T, E)>);

impl<T, E> Copy for WeakNode<T, E> {}
impl<T, E> Clone for WeakNode<T, E> {
    fn clone(&self) -> Self {
        *self
    }
}

/// An iterator over the direct neighbors of a node within a graph
pub struct Neighbors<'a, T, E> {
    graph: &'a Graph<T, E>,
    neighbors: btree_map::Iter<'a, usize, E>,
}

impl<'a, T, E> Iterator for Neighbors<'a, T, E> {
    type Item = Node<'a, T, E>;

    fn next(&mut self) -> Option<Self::Item> {
        Some(Node {
            graph: self.graph,
            idx: *self.neighbors.next()?.0,
        })
    }
}

/// A mutable reference to a node within a graph
pub struct NodeMut<'a, T, E = ()> {
    graph: &'a mut Graph<T, E>,
    idx: usize,
}

impl<'a, T, E> NodeMut<'a, T, E> {
    /// Returns the neighbors of this `NodeMut`.
    #[must_use]
    pub fn neighbors(&'a self) -> Neighbors<'a, T, E> {
        Neighbors {
            graph: self.graph,
            neighbors: self.graph.nodes[self.idx].edges.iter(),
        }
    }

    /// Converts this `NodeMut` to a weak reference, allowing the corresponding `Graph` to be used elsewhere.
    #[must_use]
    pub const fn weak(&self) -> WeakNode<T, E> {
        WeakNode(self.idx, PhantomData)
    }
}

impl<'a, T, E> Deref for NodeMut<'a, T, E> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.graph.nodes[self.idx].value
    }
}

impl<'a, T, E> DerefMut for NodeMut<'a, T, E> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.graph.nodes[self.idx].value
    }
}

/// An iterator over a `Graph`, returning `Node`s in depth-first order
pub struct DepthFirst<'a, T, E> {
    graph: &'a Graph<T, E>,
    stack: Vec<usize>,
    visited: BTreeSet<usize>,
}

impl<'a, T, E> Iterator for DepthFirst<'a, T, E> {
    type Item = Node<'a, T, E>;

    fn next(&mut self) -> Option<Self::Item> {
        let idx = self.stack.pop()?;
        for end in self.graph.nodes[idx].edges.keys() {
            if self.visited.contains(end) {
                continue;
            }
            self.stack.push(*end);
            self.visited.insert(*end);
        }
        Some(Node {
            graph: self.graph,
            idx,
        })
    }
}

/// An iterator over a `Graph`, returning `Node`s in breadth-first order
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

/// A path through a `Graph`
pub struct Path<'a, T, E> {
    graph: &'a Graph<T, E>,
    path: Vec<usize>,
}

impl<'a, T, E> Path<'a, T, E> {
    /// Returns an iterator over the `Node`s that make up this `Path`
    #[must_use]
    pub fn iter(&'a self) -> PathIterator<'a, T, E> {
        PathIterator {
            graph: self.graph,
            iter: self.path.iter(),
        }
    }

    /// Add the provided `WeakNode` to the end of this `Path`
    ///
    /// # Panics
    ///
    /// Panics if the provided `WeakNode` would index outside of the pool used by the `Graph`
    pub fn push(&mut self, node: WeakNode<T, E>) {
        assert!(
            node.0 < self.graph.nodes.len(),
            "Attempt to access Node outside of the Graph"
        );
        self.path.push(node.0);
    }
}

impl<'a, T, E: Default + Clone + AddAssign<E>> Path<'a, T, E> {
    #[must_use]
    /// Calculate the length of the path
    pub fn len(&self) -> E {
        let mut len = E::default();
        for i in 0..(self.path.len() - 1) {
            len += self.graph.nodes[self.path[i]].edges[&self.path[i + 1]].clone();
        }
        len
    }
}

impl<'a, T, E> IntoIterator for &'a Path<'a, T, E> {
    type IntoIter = PathIterator<'a, T, E>;
    type Item = Node<'a, T, E>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// An iterator over `Node`s in a `Path`
pub struct PathIterator<'a, T, E> {
    graph: &'a Graph<T, E>,
    iter: slice::Iter<'a, usize>,
}

impl<'a, T, E> Iterator for PathIterator<'a, T, E> {
    type Item = Node<'a, T, E>;

    fn next(&mut self) -> Option<Self::Item> {
        Some(Node {
            graph: self.graph,
            idx: *self.iter.next()?,
        })
    }
}
