//! MIPS CP0 Index register.
//!
//! See Figure 6.3 in IDT R30xx Manual on page 6-4.
use bit_field::BitField;

/// Index Register.
#[derive(Clone, Copy, Debug)]
pub struct IndexRegister {
    pub bits: u32,
}

impl IndexRegister {
    /// Returns a new Index register.
    pub fn new() -> Self {
        IndexRegister { bits: 0 }
    }

    // Set when a tlbp instruction failed to find a valid translation.
    register_rw!(is_p, set_p, clear_p, 31);
    register_field!(get_index, set_index, 8, 13);
}

impl From<u32> for IndexRegister {
    fn from(val: u32) -> Self {
        IndexRegister { bits: val }
    }
}

impl From<IndexRegister> for u32 {
    fn from(val: IndexRegister) -> Self {
        val.bits
    }
}
