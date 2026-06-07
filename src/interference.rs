//! Interference graph construction.

use crate::liveness::{LivenessAnalysis, VReg};
use std::collections::{HashMap, HashSet};

/// An interference graph where nodes are virtual registers.
#[derive(Debug, Clone)]
pub struct InterferenceGraph {
    /// Adjacency list: vreg -> set of interfering vregs.
    adj: HashMap<VReg, HashSet<VReg>>,
    /// Node degrees.
    degrees: HashMap<VReg, usize>,
}

impl InterferenceGraph {
    /// Create a new empty interference graph.
    pub fn new() -> Self {
        Self {
            adj: HashMap::new(),
            degrees: HashMap::new(),
        }
    }

    /// Build from liveness analysis.
    pub fn from_liveness(analysis: &LivenessAnalysis) -> Self {
        let mut graph = Self::new();

        // Add all vregs as nodes
        for &vreg in analysis.ranges.keys() {
            graph.add_node(vreg);
        }

        // Add edges for interfering vregs
        let vregs: Vec<VReg> = analysis.ranges.keys().copied().collect();
        for i in 0..vregs.len() {
            for j in (i + 1)..vregs.len() {
                let a = vregs[i];
                let b = vregs[j];
                if analysis.interferes(a, b) {
                    graph.add_edge(a, b);
                }
            }
        }

        graph
    }

    /// Add a node to the graph.
    pub fn add_node(&mut self, vreg: VReg) {
        self.adj.entry(vreg).or_default();
        self.degrees.entry(vreg).or_insert(0);
    }

    /// Add an undirected edge between two vregs.
    pub fn add_edge(&mut self, a: VReg, b: VReg) {
        if a == b {
            return;
        }
        let added_a = self.adj.entry(a).or_default().insert(b);
        let added_b = self.adj.entry(b).or_default().insert(a);
        if added_a {
            *self.degrees.entry(a).or_insert(0) += 1;
        }
        if added_b {
            *self.degrees.entry(b).or_insert(0) += 1;
        }
    }

    /// Check if two vregs interfere.
    pub fn interferes(&self, a: VReg, b: VReg) -> bool {
        self.adj.get(&a).is_some_and(|s| s.contains(&b))
    }

    /// Get the degree of a node.
    pub fn degree(&self, vreg: VReg) -> usize {
        self.degrees.get(&vreg).copied().unwrap_or(0)
    }

    /// Get neighbors of a node.
    pub fn neighbors(&self, vreg: VReg) -> Vec<VReg> {
        self.adj
            .get(&vreg)
            .map(|s| s.iter().copied().collect())
            .unwrap_or_default()
    }

    /// Number of nodes.
    pub fn node_count(&self) -> usize {
        self.adj.len()
    }

    /// Number of edges.
    pub fn edge_count(&self) -> usize {
        self.degrees.values().sum::<usize>() / 2
    }

    /// Remove a node from the graph.
    pub fn remove_node(&mut self, vreg: VReg) -> HashSet<VReg> {
        let neighbors = self.adj.remove(&vreg).unwrap_or_default();
        self.degrees.remove(&vreg);
        for &n in &neighbors {
            if let Some(adj) = self.adj.get_mut(&n) {
                adj.remove(&vreg);
            }
            if let Some(deg) = self.degrees.get_mut(&n) {
                *deg = deg.saturating_sub(1);
            }
        }
        neighbors
    }

    /// Find a node with degree < k (for simplification).
    pub fn find_low_degree(&self, k: usize) -> Option<VReg> {
        self.degrees
            .iter()
            .find(|(_, &d)| d < k)
            .map(|(&v, _)| v)
    }

    /// Returns all vregs in the graph.
    pub fn nodes(&self) -> Vec<VReg> {
        self.adj.keys().copied().collect()
    }
}

impl Default for InterferenceGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_graph() {
        let g = InterferenceGraph::new();
        assert_eq!(g.node_count(), 0);
    }

    #[test]
    fn test_add_nodes() {
        let mut g = InterferenceGraph::new();
        g.add_node(1);
        g.add_node(2);
        assert_eq!(g.node_count(), 2);
        assert_eq!(g.edge_count(), 0);
    }

    #[test]
    fn test_add_edge() {
        let mut g = InterferenceGraph::new();
        g.add_node(1);
        g.add_node(2);
        g.add_edge(1, 2);
        assert!(g.interferes(1, 2));
        assert!(g.interferes(2, 1));
        assert_eq!(g.degree(1), 1);
        assert_eq!(g.degree(2), 1);
    }

    #[test]
    fn test_remove_node() {
        let mut g = InterferenceGraph::new();
        g.add_node(1);
        g.add_node(2);
        g.add_edge(1, 2);
        g.remove_node(1);
        assert_eq!(g.node_count(), 1);
        assert_eq!(g.degree(2), 0);
    }

    #[test]
    fn test_find_low_degree() {
        let mut g = InterferenceGraph::new();
        g.add_node(1);
        g.add_node(2);
        g.add_node(3);
        g.add_edge(1, 2);
        g.add_edge(1, 3);
        // Node 2 has degree 1, node 3 has degree 1
        assert!(g.find_low_degree(2).is_some());
    }

    #[test]
    fn test_no_self_edge() {
        let mut g = InterferenceGraph::new();
        g.add_node(1);
        g.add_edge(1, 1);
        assert_eq!(g.degree(1), 0);
    }
}
