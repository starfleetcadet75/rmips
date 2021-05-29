//! MIPS CP0 Cause register.
//!
//! See Figure 3.3 in IDT R30xx Manual on page 3-7.
use std::convert::TryFrom;

use bit_field::BitField;

use crate::control::exception::Exception;

/// Cause Register.
#[derive(Clone, Copy, Debug)]
pub struct CauseRegister {
    pub bits: u32,
}

impl Default for CauseRegister {
    fn default() -> Self {
        Self::new()
    }
}

impl CauseRegister {
    /// Returns a new Cause register.
    pub fn new() -> Self {
        CauseRegister { bits: 0 }
    }

    // Indicates whether the EPC points to a branch instruction that precedes the actual exception instruction.
    register_rw!(is_branch_delay, set_branch_delay, clear_branch_delay, 31);
    register_field!(get_coprocessor_error, set_coprocessor_error, 28, 29);
    register_field!(get_interrupt_pending, set_interrupt_pending, 8, 15);

    /// Get the exception code field from the Cause register.
    #[inline]
    pub fn get_exception_code(&self) -> Exception {
        Exception::try_from(self.bits.get_bits(2..=6)).unwrap_or(Exception::Unknown)
    }

    /// Set the exception code field from the Cause register.
    #[inline]
    pub fn set_exception_code(&mut self, code: Exception) {
        self.bits.set_bits(2..=6, code.into());
    }
}

impl From<u32> for CauseRegister {
    fn from(val: u32) -> Self {
        CauseRegister { bits: val }
    }
}

impl From<CauseRegister> for u32 {
    fn from(val: CauseRegister) -> Self {
        val.bits
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn cause_exception_code() {
        let mut cause = CauseRegister::new();
        cause.set_exception_code(Exception::Breakpoint);
        assert_eq!(cause.get_exception_code(), Exception::Breakpoint);
    }
}
