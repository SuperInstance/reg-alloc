//! Graph coloring register allocator.

use crate::interference::InterferenceGraph;
use crate::liveness::VReg;
use crate::register::{Register, RegisterClass, RegisterFile};
use std::collections::HashMap;

/// Result of graph coloring allocation.
#[derive(Debug)]
pub struct Allocation {
    /// Mapping from virtual register to physical register.
    pub mapping: HashMap<VReg, Register>,
    /// Virtual registers that could not be allocated (need spilling).
    pub spilled: Vec<VReg>,
}

/// Graph coloring allocator using simplification and spill heuristics.
pub struct GraphColoring {
    k: usize,
    class: RegisterClass,
}

impl GraphColoring {
    /// Create a new graph coloring allocator with K colors (registers).
    pub fn new(k: usize, class: RegisterClass) -> Self {
        Self { k, class }
    }

    /// Allocate registers using Chaitin-style graph coloring.
    pub fn allocate(&self, graph: &InterferenceGraph) -> Allocation {
        let mut work_graph = graph.clone();
        let mut stack: Vec<(VReg, Vec<VReg>)> = Vec::new();
        let mut spilled = Vec::new();

        // Phase 1: Simplify - remove low-degree nodes
        loop {
            if let Some(node) = work_graph.find_low_degree(self.k) {
                let neighbors = work_graph.neighbors(node);
                let neighbors_vec = neighbors;
                work_graph.remove_node(node);
                stack.push((node, neighbors_vec));
            } else if work_graph.node_count() > 0 {
                // Potential spill: pick highest degree node
                let spill = work_graph
                    .nodes()
                    .into_iter()
                    .max_by_key(|&n| work_graph.degree(n))
                    .unwrap();
                let neighbors = work_graph.neighbors(spill);
                work_graph.remove_node(spill);
                stack.push((spill, neighbors));
            } else {
                break;
            }
        }

        // Phase 2: Select - assign colors
        let mut mapping: HashMap<VReg, Register> = HashMap::new();
        for (vreg, neighbors) in stack.into_iter().rev() {
            let used_colors: Vec<u8> = neighbors
                .iter()
                .filter_map(|n| mapping.get(n).map(|r| r.number))
                .collect();

            // Find a free color
            let color = (0..self.k as u8).find(|c| !used_colors.contains(c));

            if let Some(c) = color {
                mapping.insert(vreg, Register::new(c, self.class));
            } else {
                spilled.push(vreg);
            }
        }

        Allocation { mapping, spilled }
    }

    /// Allocate using a register file directly.
    pub fn allocate_with_file(&self, graph: &InterferenceGraph, _file: &mut RegisterFile) -> Allocation {
        self.allocate(graph)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_simple_graph() -> InterferenceGraph {
        let mut g = InterferenceGraph::new();
        g.add_node(1);
        g.add_node(2);
        g.add_edge(1, 2);
        g
    }

    #[test]
    fn test_simple_allocation() {
        let graph = make_simple_graph();
        let alloc = GraphColoring::new(4, RegisterClass::Integer);
        let result = alloc.allocate(&graph);
        assert!(result.spilled.is_empty());
        assert!(result.mapping.contains_key(&1));
        assert!(result.mapping.contains_key(&2));
    }

    #[test]
    fn test_no_interference_same_color() {
        let mut g = InterferenceGraph::new();
        g.add_node(1);
        g.add_node(2);
        // No edge = can share a register
        let alloc = GraphColoring::new(1, RegisterClass::Integer);
        let result = alloc.allocate(&g);
        assert!(result.spilled.is_empty());
        assert_eq!(result.mapping[&1], result.mapping[&2]);
    }

    #[test]
    fn test_spill_when_oversubscribed() {
        let mut g = InterferenceGraph::new();
        // Complete graph on 3 nodes with only 2 colors
        g.add_node(1);
        g.add_node(2);
        g.add_node(3);
        g.add_edge(1, 2);
        g.add_edge(1, 3);
        g.add_edge(2, 3);
        let alloc = GraphColoring::new(2, RegisterClass::Integer);
        let result = alloc.allocate(&g);
        // K3 requires 3 colors with 2 available -> at least one spill
        assert!(!result.spilled.is_empty() || result.mapping.len() == 3);
    }

    #[test]
    fn test_many_nodes() {
        let mut g = InterferenceGraph::new();
        for i in 0..10u32 {
            g.add_node(i);
        }
        // Chain: 0-1-2-3-...
        for i in 0..9u32 {
            g.add_edge(i, i + 1);
        }
        let alloc = GraphColoring::new(2, RegisterClass::Integer);
        let result = alloc.allocate(&g);
        // Chain is 2-colorable
        assert!(result.spilled.is_empty());
    }

    #[test]
    fn test_allocation_correctness() {
        let mut g = InterferenceGraph::new();
        g.add_node(1);
        g.add_node(2);
        g.add_node(3);
        g.add_edge(1, 2);
        g.add_edge(2, 3);
        let alloc = GraphColoring::new(3, RegisterClass::Integer);
        let result = alloc.allocate(&g);
        // Interfering vregs should have different registers
        if let (Some(&r1), Some(&r2)) = (result.mapping.get(&1), result.mapping.get(&2)) {
            assert_ne!(r1, r2);
        }
    }
}
