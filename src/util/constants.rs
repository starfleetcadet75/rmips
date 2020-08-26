/// Address mask for determining which segment the address belongs to
pub const KSEG_SELECT_MASK: u32 = 0xe0000000;
/// Start address of user-space
pub const KUSEG: u32 = 0;
/// Start address of kernel-space
pub const KERNEL_SPACE_MASK: u32 = 0x80000000;
/// Start of unmapped and cached kernel segment
pub const KSEG0: u32 = 0x80000000;
/// Address difference for kseg0 when translating from virtual to physical memory
pub const KSEG0_CONST_TRANSLATION: u32 = 0x80000000;
/// Start of unmapped and uncached kernel segment
pub const KSEG1: u32 = 0xa0000000;
/// Address difference for kseg1 when translating from virtual to physical memory
pub const KSEG1_CONST_TRANSLATION: u32 = 0xa0000000;
/// Start of mapped and cached kernel segment
pub const KSEG2: u32 = 0xc0000000;
/// Second half of mapped and cached kernel segment
pub const KSEG2_TOP: u32 = 0xe0000000;

/// Zero register
pub const REG_ZERO: usize = 0;
pub const REG_AT: usize = 1;
pub const REG_V0: usize = 2;
pub const REG_V1: usize = 3;
pub const REG_A0: usize = 4;
pub const REG_A1: usize = 5;
pub const REG_A2: usize = 6;
pub const REG_A3: usize = 7;
pub const REG_T0: usize = 8;
pub const REG_T1: usize = 9;
pub const REG_T2: usize = 10;
pub const REG_T3: usize = 11;
pub const REG_T4: usize = 12;
pub const REG_T5: usize = 13;
pub const REG_T6: usize = 14;
pub const REG_T7: usize = 15;
pub const REG_S0: usize = 16;
pub const REG_S1: usize = 17;
pub const REG_S2: usize = 18;
pub const REG_S3: usize = 19;
pub const REG_S4: usize = 20;
pub const REG_S5: usize = 21;
pub const REG_S6: usize = 22;
pub const REG_S7: usize = 23;
pub const REG_T8: usize = 24;
pub const REG_T9: usize = 25;
pub const REG_K0: usize = 26;
pub const REG_K1: usize = 27;
/// Global pointer
pub const REG_GP: usize = 28;
/// Stack pointer
pub const REG_SP: usize = 29;
/// Frame pointer
pub const REG_FP: usize = 30;
/// Return address register
pub const REG_RA: usize = 31;
/// Number of general-purpose registers in the processor
pub const NUM_GPR: usize = 32;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExceptionCode {
    Interrupt = 0,
    TlbMod = 1,
    TlbLoad = 2,
    TlbStore = 3,
    /// Address error exception generated by a load instruction
    LoadAddressError = 4,
    /// Address error exception generated by a store instruction
    StoreAddressError = 5,
    InstructionBusError = 6,
    DataBusError = 7,
    Syscall = 8,
    Break = 9,
    ReservedInstruction = 10,
    CoprocessorUnusable = 11,
    Overflow = 12,
    Trap = 13,
}