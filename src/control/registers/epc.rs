//! MIPS CP0 EPC register.
//!
//! This is a 32-bit register containing the 32-bit address of the return point
//! for this exception. The instruction causing (or suffering) the exception is
//! at EPC, unless BD is set in Cause, in which case EPC points to the previous
//! (branch) instruction.

/// EPC Register.
#[derive(Clone, Copy, Debug)]
pub struct EpcRegister {
    pub address: u32,
}

impl EpcRegister {
    /// Returns a new EPC register.
    pub fn new() -> Self {
        EpcRegister { address: 0 }
    }
}

impl From<u32> for EpcRegister {
    fn from(val: u32) -> Self {
        EpcRegister { address: val }
    }
}

impl From<EpcRegister> for u32 {
    fn from(val: EpcRegister) -> Self {
        val.address
    }
}
