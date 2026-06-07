//! Liveness analysis.

use std::collections::{HashMap, HashSet};

/// A virtual register (SSA value or variable).
pub type VReg = u32;

/// A program point (instruction index).
pub type ProgramPoint = usize;

/// A live range from start to end (inclusive).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct LiveRange {
    pub start: ProgramPoint,
    pub end: ProgramPoint,
}

impl LiveRange {
    /// Create a new live range.
    pub fn new(start: ProgramPoint, end: ProgramPoint) -> Self {
        Self { start, end }
    }

    /// Length of the range.
    pub fn len(&self) -> usize {
        self.end.saturating_sub(self.start) + 1
    }

    /// Returns true if the range is a single point (len == 1).
    pub fn is_empty(&self) -> bool {
        false // LiveRange always has at least one point
    }

    /// Returns true if the range covers a single point.
    pub fn is_point(&self) -> bool {
        self.start == self.end
    }

    /// Check if this range overlaps another.
    pub fn overlaps(&self, other: &LiveRange) -> bool {
        self.start <= other.end && other.start <= self.end
    }

    /// Check if a point is within this range.
    pub fn contains(&self, point: ProgramPoint) -> bool {
        self.start <= point && point <= self.end
    }

    /// Merge with another range.
    pub fn merge(&self, other: &LiveRange) -> LiveRange {
        LiveRange::new(self.start.min(other.start), self.end.max(other.end))
    }
}

/// A basic block for liveness analysis purposes.
#[derive(Clone, Debug)]
pub struct LivenessBlock {
    pub id: usize,
    /// Virtual registers defined (written) in this block.
    pub defs: HashSet<VReg>,
    /// Virtual registers used (read) in this block.
    pub uses: HashSet<VReg>,
    /// Successor block indices.
    pub successors: Vec<usize>,
}

/// Result of liveness analysis.
#[derive(Debug)]
pub struct LivenessAnalysis {
    /// Live-in sets per block.
    pub live_in: HashMap<usize, HashSet<VReg>>,
    /// Live-out sets per block.
    pub live_out: HashMap<usize, HashSet<VReg>>,
    /// Live ranges per virtual register.
    pub ranges: HashMap<VReg, LiveRange>,
}

impl LivenessAnalysis {
    /// Run liveness analysis on a sequence of blocks.
    pub fn analyze(blocks: &[LivenessBlock]) -> Self {
        let mut live_in: HashMap<usize, HashSet<VReg>> = HashMap::new();
        let mut live_out: HashMap<usize, HashSet<VReg>> = HashMap::new();

        for block in blocks {
            live_in.insert(block.id, HashSet::new());
            live_out.insert(block.id, HashSet::new());
        }

        // Iterate until fixed point (backward analysis)
        let mut changed = true;
        while changed {
            changed = false;
            for block in blocks.iter().rev() {
                // live_out = union of live_in of successors
                let mut new_out: HashSet<VReg> = HashSet::new();
                for &succ in &block.successors {
                    if let Some(succ_in) = live_in.get(&succ) {
                        new_out.extend(succ_in.iter().copied());
                    }
                }

                // live_in = uses ∪ (live_out - defs)
                let mut new_in: HashSet<VReg> = block.uses.clone();
                for &v in &new_out {
                    if !block.defs.contains(&v) {
                        new_in.insert(v);
                    }
                }

                if live_in.get(&block.id) != Some(&new_in) {
                    live_in.insert(block.id, new_in);
                    changed = true;
                }
                if live_out.get(&block.id) != Some(&new_out) {
                    live_out.insert(block.id, new_out);
                    changed = true;
                }
            }
        }

        // Compute live ranges per vreg
        let mut ranges: HashMap<VReg, LiveRange> = HashMap::new();
        for block in blocks {
            for &vreg in &block.uses {
                let range = ranges.entry(vreg).or_insert(LiveRange::new(block.id, block.id));
                *range = range.merge(&LiveRange::new(block.id, block.id));
            }
            for &vreg in &block.defs {
                let range = ranges.entry(vreg).or_insert(LiveRange::new(block.id, block.id));
                *range = range.merge(&LiveRange::new(block.id, block.id));
            }
        }

        Self { live_in, live_out, ranges }
    }

    /// Check if two virtual registers have overlapping live ranges.
    pub fn interferes(&self, a: VReg, b: VReg) -> bool {
        match (self.ranges.get(&a), self.ranges.get(&b)) {
            (Some(ra), Some(rb)) => ra.overlaps(rb),
            _ => false,
        }
    }

    /// Get the live range for a virtual register.
    pub fn range(&self, vreg: VReg) -> Option<&LiveRange> {
        self.ranges.get(&vreg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_live_range_overlap() {
        let a = LiveRange::new(0, 5);
        let b = LiveRange::new(3, 8);
        assert!(a.overlaps(&b));
    }

    #[test]
    fn test_live_range_no_overlap() {
        let a = LiveRange::new(0, 3);
        let b = LiveRange::new(4, 8);
        assert!(!a.overlaps(&b));
    }

    #[test]
    fn test_live_range_contains() {
        let r = LiveRange::new(2, 6);
        assert!(r.contains(4));
        assert!(!r.contains(7));
    }

    #[test]
    fn test_live_range_merge() {
        let a = LiveRange::new(0, 3);
        let b = LiveRange::new(5, 8);
        let merged = a.merge(&b);
        assert_eq!(merged.start, 0);
        assert_eq!(merged.end, 8);
    }

    #[test]
    fn test_liveness_simple() {
        let blocks = vec![
            LivenessBlock {
                id: 0,
                defs: [1u32].into_iter().collect(),
                uses: [].into_iter().collect(),
                successors: vec![1],
            },
            LivenessBlock {
                id: 1,
                defs: [].into_iter().collect(),
                uses: [1u32].into_iter().collect(),
                successors: vec![],
            },
        ];
        let analysis = LivenessAnalysis::analyze(&blocks);
        // vreg 1 should be live-in to block 1 (used there, defined in block 0)
        assert!(analysis.live_in[&1].contains(&1));
    }

    #[test]
    fn test_interference() {
        let blocks = vec![
            LivenessBlock {
                id: 0,
                defs: [1u32, 2u32].into_iter().collect(),
                uses: [].into_iter().collect(),
                successors: vec![1],
            },
            LivenessBlock {
                id: 1,
                defs: [].into_iter().collect(),
                uses: [1u32, 2u32].into_iter().collect(),
                successors: vec![],
            },
        ];
        let analysis = LivenessAnalysis::analyze(&blocks);
        assert!(analysis.interferes(1, 2));
    }

    #[test]
    fn test_no_interference() {
        let blocks = vec![
            LivenessBlock {
                id: 0,
                defs: [1u32].into_iter().collect(),
                uses: [].into_iter().collect(),
                successors: vec![1],
            },
            LivenessBlock {
                id: 1,
                defs: [2u32].into_iter().collect(),
                uses: [1u32].into_iter().collect(),
                successors: vec![2],
            },
            LivenessBlock {
                id: 2,
                defs: [].into_iter().collect(),
                uses: [2u32].into_iter().collect(),
                successors: vec![],
            },
        ];
        let analysis = LivenessAnalysis::analyze(&blocks);
        // vreg1 live in 0-1, vreg2 live in 1-2 - they overlap at block 1
        assert!(analysis.interferes(1, 2));
    }
}
