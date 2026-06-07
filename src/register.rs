//! Register definitions and register files.

/// A register class (e.g., integer, float).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum RegisterClass {
    Integer,
    Float,
    Vector,
}

/// A physical register.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Register {
    /// Register number.
    pub number: u8,
    /// Register class.
    pub class: RegisterClass,
}

impl Register {
    /// Create a new register.
    pub fn new(number: u8, class: RegisterClass) -> Self {
        Self { number, class }
    }

    /// Create an integer register.
    pub fn int(number: u8) -> Self {
        Self::new(number, RegisterClass::Integer)
    }

    /// Create a float register.
    pub fn float(number: u8) -> Self {
        Self::new(number, RegisterClass::Float)
    }

    /// Returns true if this is an integer register.
    pub fn is_int(&self) -> bool {
        self.class == RegisterClass::Integer
    }

    /// Returns true if this is a float register.
    pub fn is_float(&self) -> bool {
        self.class == RegisterClass::Float
    }
}

impl std::fmt::Display for Register {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let prefix = match self.class {
            RegisterClass::Integer => "r",
            RegisterClass::Float => "f",
            RegisterClass::Vector => "v",
        };
        write!(f, "{}{}", prefix, self.number)
    }
}

/// A collection of physical registers.
pub struct RegisterFile {
    registers: Vec<Register>,
    /// Which registers are currently allocated.
    allocated: Vec<bool>,
}

impl RegisterFile {
    /// Create a register file with N integer registers.
    pub fn new_int(count: u8) -> Self {
        let registers: Vec<Register> = (0..count).map(Register::int).collect();
        Self {
            registers,
            allocated: vec![false; count as usize],
        }
    }

    /// Create a register file with N integer and N float registers.
    pub fn new_with_floats(int_count: u8, float_count: u8) -> Self {
        let mut registers: Vec<Register> = (0..int_count).map(Register::int).collect();
        registers.extend((0..float_count).map(Register::float));
        let len = registers.len();
        Self {
            registers,
            allocated: vec![false; len],
        }
    }

    /// Allocate a free register of the given class. Returns None if all allocated.
    pub fn allocate(&mut self, class: RegisterClass) -> Option<Register> {
        for (i, reg) in self.registers.iter().enumerate() {
            if reg.class == class && !self.allocated[i] {
                self.allocated[i] = true;
                return Some(*reg);
            }
        }
        None
    }

    /// Free a previously allocated register.
    pub fn free(&mut self, reg: Register) {
        for (i, r) in self.registers.iter().enumerate() {
            if *r == reg {
                self.allocated[i] = false;
                return;
            }
        }
    }

    /// Check if a register is allocated.
    pub fn is_allocated(&self, reg: Register) -> bool {
        self.registers
            .iter()
            .position(|r| *r == reg)
            .is_some_and(|i| self.allocated[i])
    }

    /// Number of available (free) registers of the given class.
    pub fn available(&self, class: RegisterClass) -> usize {
        self.registers
            .iter()
            .enumerate()
            .filter(|(i, r)| r.class == class && !self.allocated[*i])
            .count()
    }

    /// Total registers of the given class.
    pub fn total(&self, class: RegisterClass) -> usize {
        self.registers.iter().filter(|r| r.class == class).count()
    }

    /// Returns all registers.
    pub fn registers(&self) -> &[Register] {
        &self.registers
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_creation() {
        let r = Register::int(0);
        assert!(r.is_int());
        assert_eq!(format!("{}", r), "r0");
    }

    #[test]
    fn test_register_file_allocate() {
        let mut rf = RegisterFile::new_int(4);
        let r0 = rf.allocate(RegisterClass::Integer);
        assert!(r0.is_some());
        assert_eq!(rf.available(RegisterClass::Integer), 3);
    }

    #[test]
    fn test_register_file_exhaust() {
        let mut rf = RegisterFile::new_int(2);
        rf.allocate(RegisterClass::Integer);
        rf.allocate(RegisterClass::Integer);
        let r2 = rf.allocate(RegisterClass::Integer);
        assert!(r2.is_none());
    }

    #[test]
    fn test_register_file_free() {
        let mut rf = RegisterFile::new_int(2);
        let r = rf.allocate(RegisterClass::Integer).unwrap();
        rf.free(r);
        assert_eq!(rf.available(RegisterClass::Integer), 2);
    }

    #[test]
    fn test_register_display() {
        let f = Register::float(3);
        assert_eq!(format!("{}", f), "f3");
    }

    #[test]
    fn test_mixed_classes() {
        let mut rf = RegisterFile::new_with_floats(2, 2);
        assert_eq!(rf.available(RegisterClass::Integer), 2);
        assert_eq!(rf.available(RegisterClass::Float), 2);
        let ri = rf.allocate(RegisterClass::Integer);
        assert!(ri.is_some());
        assert_eq!(rf.available(RegisterClass::Float), 2);
    }
}
