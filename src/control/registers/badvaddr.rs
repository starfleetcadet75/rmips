//! MIPS CP0 BadVaddr register.
//!
//! A 32-bit register containing the address whose reference led to an exception;
//! set on any MMU-related exception, on an attempt by a user program to access
//! addresses outside kuseg, or if an address is wrongly aligned for the datum
//! size referenced.
//! After any other exception this register is undefined. Note in particular
//! that it is not set after a bus error.

/// BadVaddr Register.
#[derive(Clone, Copy, Debug)]
pub struct BadVaddrRegister {
    pub address: u32,
}

impl Default for BadVaddrRegister {
    fn default() -> Self {
        Self::new()
    }
}

impl BadVaddrRegister {
    /// Returns a new BadVaddr register.
    pub fn new() -> Self {
        BadVaddrRegister { address: 0 }
    }
}

impl From<u32> for BadVaddrRegister {
    fn from(val: u32) -> Self {
        BadVaddrRegister { address: val }
    }
}

impl From<BadVaddrRegister> for u32 {
    fn from(val: BadVaddrRegister) -> Self {
        val.address
    }
}
