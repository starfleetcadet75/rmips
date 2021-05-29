//! MIPS CP0 Context register.
//!
//! See Figure 6.5 in IDT R30xx Manual on page 6-4.
use bit_field::BitField;

/// Context Register.
#[derive(Clone, Copy, Debug)]
pub struct ContextRegister {
    pub bits: u32,
}

impl Default for ContextRegister {
    fn default() -> Self {
        Self::new()
    }
}

impl ContextRegister {
    /// Returns a new Context register.
    pub fn new() -> Self {
        ContextRegister { bits: 0 }
    }

    register_field!(get_ptebase, set_ptebase, 21, 31);
    register_field!(get_badvpn, set_badvpn, 2, 20);
}

impl From<u32> for ContextRegister {
    fn from(val: u32) -> Self {
        ContextRegister { bits: val }
    }
}

impl From<ContextRegister> for u32 {
    fn from(val: ContextRegister) -> Self {
        val.bits
    }
}
