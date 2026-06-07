//! Spill code generation and management.

use crate::liveness::VReg;
use crate::register::Register;
use std::collections::HashMap;

/// A spill slot in the stack frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SpillSlot {
    /// Byte offset from the frame pointer.
    pub offset: usize,
}

impl SpillSlot {
    /// Create a new spill slot at the given offset.
    pub fn new(offset: usize) -> Self {
        Self { offset }
    }
}

/// Represents a spill instruction.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SpillInstruction {
    /// Store a register to a spill slot.
    Store { reg: Register, slot: SpillSlot },
    /// Load from a spill slot into a register.
    Load { slot: SpillSlot, reg: Register },
    /// Store a virtual register's value (pre-allocation).
    VStore { vreg: VReg, slot: SpillSlot },
    /// Load a virtual register's value (pre-allocation).
    VLoad { slot: SpillSlot, vreg: VReg },
}

/// Manages spill slots and spill code generation.
pub struct SpillManager {
    /// Total spill slots allocated.
    slot_count: usize,
    /// Size of each spill slot in bytes.
    slot_size: usize,
    /// Mapping from virtual register to spill slot.
    spills: HashMap<VReg, SpillSlot>,
    /// Generated spill instructions.
    instructions: Vec<SpillInstruction>,
}

impl SpillManager {
    /// Create a new spill manager with the given slot size.
    pub fn new(slot_size: usize) -> Self {
        Self {
            slot_count: 0,
            slot_size,
            spills: HashMap::new(),
            instructions: Vec::new(),
        }
    }

    /// Allocate a spill slot for a virtual register.
    pub fn allocate_slot(&mut self, vreg: VReg) -> SpillSlot {
        if let Some(&slot) = self.spills.get(&vreg) {
            return slot;
        }
        let slot = SpillSlot::new(self.slot_count * self.slot_size);
        self.slot_count += 1;
        self.spills.insert(vreg, slot);
        slot
    }

    /// Generate a store instruction (spill a register to memory).
    pub fn emit_store(&mut self, reg: Register, slot: SpillSlot) {
        self.instructions.push(SpillInstruction::Store { reg, slot });
    }

    /// Generate a load instruction (fill from memory to a register).
    pub fn emit_load(&mut self, slot: SpillSlot, reg: Register) {
        self.instructions.push(SpillInstruction::Load { slot, reg });
    }

    /// Generate a virtual register store.
    pub fn emit_vstore(&mut self, vreg: VReg, slot: SpillSlot) {
        self.instructions.push(SpillInstruction::VStore { vreg, slot });
    }

    /// Generate a virtual register load.
    pub fn emit_vload(&mut self, slot: SpillSlot, vreg: VReg) {
        self.instructions.push(SpillInstruction::VLoad { slot, vreg });
    }

    /// Total spill slots used.
    pub fn slot_count(&self) -> usize {
        self.slot_count
    }

    /// Total stack frame bytes needed for spills.
    pub fn frame_size(&self) -> usize {
        self.slot_count * self.slot_size
    }

    /// Get the spill slot for a virtual register.
    pub fn get_slot(&self, vreg: VReg) -> Option<SpillSlot> {
        self.spills.get(&vreg).copied()
    }

    /// Get all generated spill instructions.
    pub fn instructions(&self) -> &[SpillInstruction] {
        &self.instructions
    }

    /// Number of spill instructions.
    pub fn instruction_count(&self) -> usize {
        self.instructions.len()
    }

    /// Clear all generated instructions (keeps slot allocations).
    pub fn clear_instructions(&mut self) {
        self.instructions.clear();
    }

    /// Spill a set of virtual registers, generating load/store pairs.
    pub fn spill_vregs(&mut self, vregs: &[VReg]) {
        for &vreg in vregs {
            let slot = self.allocate_slot(vreg);
            self.emit_vstore(vreg, slot);
            self.emit_vload(slot, vreg);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::register::RegisterClass;

    #[test]
    fn test_allocate_slot() {
        let mut sm = SpillManager::new(8);
        let slot = sm.allocate_slot(1);
        assert_eq!(slot.offset, 0);
        let slot2 = sm.allocate_slot(2);
        assert_eq!(slot2.offset, 8);
    }

    #[test]
    fn test_reuse_slot() {
        let mut sm = SpillManager::new(8);
        let s1 = sm.allocate_slot(1);
        let s2 = sm.allocate_slot(1);
        assert_eq!(s1, s2);
    }

    #[test]
    fn test_frame_size() {
        let mut sm = SpillManager::new(8);
        sm.allocate_slot(1);
        sm.allocate_slot(2);
        assert_eq!(sm.frame_size(), 16);
    }

    #[test]
    fn test_emit_store_load() {
        let mut sm = SpillManager::new(8);
        let reg = Register::new(0, RegisterClass::Integer);
        let slot = SpillSlot::new(0);
        sm.emit_store(reg, slot);
        sm.emit_load(slot, reg);
        assert_eq!(sm.instruction_count(), 2);
    }

    #[test]
    fn test_spill_vregs() {
        let mut sm = SpillManager::new(4);
        sm.spill_vregs(&[1, 2, 3]);
        assert_eq!(sm.slot_count(), 3);
        assert_eq!(sm.instruction_count(), 6); // store + load for each
    }

    #[test]
    fn test_clear_instructions() {
        let mut sm = SpillManager::new(8);
        sm.emit_store(Register::new(0, RegisterClass::Integer), SpillSlot::new(0));
        sm.clear_instructions();
        assert_eq!(sm.instruction_count(), 0);
        // Slots preserved
        sm.allocate_slot(1);
        assert_eq!(sm.slot_count(), 1);
    }

    #[test]
    fn test_get_slot() {
        let mut sm = SpillManager::new(8);
        sm.allocate_slot(5);
        assert!(sm.get_slot(5).is_some());
        assert!(sm.get_slot(99).is_none());
    }
}
