//! # Reg Alloc
//!
//! Register allocation with liveness analysis and graph coloring.

pub mod coloring;
pub mod interference;
pub mod liveness;
pub mod register;
pub mod spill;

pub use coloring::GraphColoring;
pub use interference::InterferenceGraph;
pub use liveness::{LivenessAnalysis, LiveRange};
pub use register::{Register, RegisterClass, RegisterFile};
pub use spill::SpillManager;
