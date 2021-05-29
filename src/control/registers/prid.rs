//! MIPS CP0 PRId register.
//!
//! See Figure 3.1 in IDT R30xx Manual on page 3-4.
use bit_field::BitField;

/// PRId Register.
#[derive(Clone, Copy, Debug)]
pub struct PridRegister {
    pub bits: u32,
}

impl PridRegister {
    /// Returns a new PRId register.
    pub fn new() -> Self {
        PridRegister { bits: 0 }
    }

    /// Get the implementation number from the PRId register.
    #[inline]
    pub fn imp(&self) -> u32 {
        self.bits.get_bits(8..=15)
    }

    /// Get the revision number from the PRId register.
    #[inline]
    pub fn rev(&self) -> u32 {
        self.bits.get_bits(0..=7)
    }
}

impl From<u32> for PridRegister {
    fn from(val: u32) -> Self {
        PridRegister { bits: val }
    }
}

impl From<PridRegister> for u32 {
    fn from(val: PridRegister) -> Self {
        val.bits
    }
}
