use crate::Address;

pub(crate) mod cpu;
pub(crate) mod cpzero;
mod exception;
mod instruction;
mod instructions;
pub mod registers;
mod tlbentry;

/// Address mask for determining which segment the address belongs to
pub const KSEG_SELECT_MASK: Address = 0xe0000000;
/// Start address of kernel-space
pub const KERNEL_SPACE_MASK: Address = KSEG0;
/// Start address of TLB-mapped cacheable user or kernel space
pub const KUSEG: Address = 0;
/// Start of direct-mapped cacheable kernel space
pub const KSEG0: Address = 0x80000000;
/// Start of direct-mapped non-cacheable kernel space
pub const KSEG1: Address = 0xa0000000;
/// Start of TLB-mapped cacheable kernel space
pub const KSEG2: Address = 0xc0000000;
/// Second half of mapped and cached kernel segment
pub const KSEG2_TOP: Address = 0xe0000000;
