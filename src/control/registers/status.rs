//! MIPS CP0 Status register.
//!
//! See Figure 3.2 in IDT R30xx Manual on page 3-4.
use bit_field::BitField;

/// Status Register
#[derive(Clone, Copy, Debug)]
pub struct StatusRegister {
    pub bits: u32,
}

impl Default for StatusRegister {
    fn default() -> Self {
        Self::new()
    }
}

impl StatusRegister {
    /// Returns a new Status register
    pub fn new() -> Self {
        StatusRegister { bits: 0 }
    }

    // Current Interrupt Enable Status
    register_rw!(
        are_interrupts_enabled,
        enable_interrupts,
        disable_interrupts,
        0
    );
    // Current Kernel / User Mode Status
    register_rw!(is_kernel_mode, enter_kernel_mode, enter_user_mode, 1);
    // Previous Interrupt Enable Status
    register_rw!(iep, set_iep, clear_iep, 2);
    // Previous Kernel / User Status
    register_rw!(kup, set_kup, clear_kup, 3);
    // Old Interrupt Enable Status
    register_rw!(ieo, set_ieo, clear_ieo, 4);
    // Old Kernel / User Status
    register_rw!(kuo, set_kuo, clear_kuo, 5);
    // Isolate Cache
    register_rw!(isc, set_isc, clear_isc, 16);
    // Swap Caches
    register_rw!(swc, set_swc, clear_swc, 17);
    // Cache parity forced to zero
    register_r!(pz, 18);
    // Cache Miss
    register_r!(cm, 19);
    // Cache Parity Bit
    register_r!(pe, 20);
    // TLB Shutdown
    register_rw!(ts, set_ts, clear_ts, 21);
    // Bootstrap Exception Vector
    register_rw!(is_bootstrap_mode, enter_bootstrap, leave_bootstrap, 22);
    // Reverse Endianness in User Mode
    register_r!(re, 25);
    // Coprocessor 0 Usable
    register_r!(cu0, 28);
    // Coprocessor 1 Usable
    register_r!(cu1, 29);
    // Coprocessor 2 Usable
    register_r!(cu2, 30);
    // Coprocessor 3 Usable
    register_r!(cu3, 31);
    // Interrupt Mask
    register_field!(get_interrupt_mask, set_interrupt_mask, 8, 15);
}

impl From<u32> for StatusRegister {
    fn from(val: u32) -> Self {
        StatusRegister { bits: val }
    }
}

impl From<StatusRegister> for u32 {
    fn from(val: StatusRegister) -> Self {
        val.bits
    }
}
