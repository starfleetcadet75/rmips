//! MIPS CP0 Random register.
//!
//! Most systems never have to read or write the Random register, but it may
//! be useful for diagnostics. The hardware initializes the Random field to its
//! maximum value (63) on reset, and it decrements every clock period until it
//! reaches 8, when it wraps back to 63 and starts again.
//!
//! See Figure 6.4 in IDT R30xx Manual on page 6-4.
use bit_field::BitField;

/// Random Register.
#[derive(Clone, Copy, Debug)]
pub struct RandomRegister {
    pub bits: u32,
}

impl RandomRegister {
    /// Returns a new Random register.
    pub fn new() -> Self {
        RandomRegister { bits: 0 }
    }

    register_field!(get_value, set_value, 8, 13);
}

impl From<u32> for RandomRegister {
    fn from(val: u32) -> Self {
        RandomRegister { bits: val }
    }
}

impl From<RandomRegister> for u32 {
    fn from(val: RandomRegister) -> Self {
        val.bits
    }
}
