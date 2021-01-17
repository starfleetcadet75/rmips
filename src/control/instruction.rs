use std::fmt;

/// Represents a 32-bit MIPS instruction and its fields.
#[derive(Copy, Clone, Default, PartialEq, PartialOrd)]
pub struct Instruction(pub u32);

impl Instruction {
    /// Returns the opcode that represents the instruction mnemonic.
    pub fn opcode(&self) -> u32 {
        (self.0 >> 26) & 0b0011_1111
    }

    /// Returns the numeric representation of the source register.
    pub fn rs(&self) -> u32 {
        (self.0 >> 21) & 0b0001_1111
    }

    /// Returns the numeric representation of the target register.
    pub fn rt(&self) -> u32 {
        (self.0 >> 16) & 0b0001_1111
    }

    /// Returns the numeric representation of the destination register.
    pub fn rd(&self) -> u32 {
        (self.0 >> 11) & 0b0001_1111
    }

    /// Returns the shift amount used by shift and rotate instructions.
    pub fn shamt(&self) -> u32 {
        (self.0 >> 6) & 0b0001_1111
    }

    /// Returns the control code used by R-type instructions.
    pub fn funct(&self) -> u32 {
        self.0 & 0b0011_1111
    }

    /// Returns the immediate value used by I-type instructions.
    pub fn immed(&self) -> u32 {
        self.0 & 0xffff
    }

    /// Returns the signed immediate value.
    pub fn simmed(&self) -> u32 {
        // Must start as a signed value before sign-extending to a u32
        (self.0 & 0xffff) as i16 as u32
    }

    /// Returns the jump target for J-type instructions.
    pub fn jumptarget(&self) -> u32 {
        self.0 & 0x03ffffff
    }
}

impl fmt::Debug for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Instruction")
            .field("value", &format!("0x{:08X}", self.0))
            .field("opcode", &self.opcode())
            .field("rs", &self.rs())
            .field("rt", &self.rt())
            .field("rd", &self.rd())
            .field("shamt", &self.shamt())
            .field("funct", &self.funct())
            .field("immed", &format!("0x{:04X}", self.immed()))
            .field("simmed", &format!("0x{:04X}", self.simmed()))
            .field("jumptarget", &format!("0x{:08X}", self.jumptarget() << 2))
            .finish()
    }
}
