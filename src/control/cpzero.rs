use crate::control::instruction::Instruction;
use crate::control::Register;
use crate::control::{
    ExceptionCode, KERNEL_SPACE_MASK, KSEG0, KSEG1, KSEG2, KSEG2_TOP, KSEG_SELECT_MASK, KUSEG,
};
use crate::Address;

/// Contains the last invalid program address which caused a trap.
pub const BADVADDR: Register = 8;
/// The Status register contains the operating mode, interrupt enable flag, and diagnostic states.
pub const STATUS: Register = 12;
/// Contains the cause of the last exception.
pub const CAUSE: Register = 13;
/// Contains the address to return to after handling an exception.
pub const EPC: Register = 14;
/// Processor Revision Identifier.
pub const PRID: Register = 15;

bitflags! {
    /// Bitmasks for extracting fields from the Status Register (SR).
    /// See Figure 3.2 in IDT R30xx Manual on page 3-4.
    struct StatusMask: u32 {
        /// Coprocessor 3 Usable
        const CU3 = 0b1000_0000_0000_0000_0000_0000_0000_0000;
        /// Coprocessor 2 Usable
        const CU2 = 0b0100_0000_0000_0000_0000_0000_0000_0000;
        /// Coprocessor 1 Usable
        const CU1 = 0b0010_0000_0000_0000_0000_0000_0000_0000;
        /// Coprocessor 0 Usable
        const CU0 = 0b0001_0000_0000_0000_0000_0000_0000_0000;
        /// Coprocessors Usable
        const CU = Self::CU3.bits | Self::CU2.bits | Self::CU1.bits | Self::CU0.bits;
        /// Reverse Endianness in User Mode
        const RE = 0b0000_0010_0000_0000_0000_0000_0000_0000;
        /// Bootstrap Exception Vector
        const BEV = 0b0000_0000_0100_0000_0000_0000_0000_0000;
        /// TLB Shutdown
        const TS = 0b0000_0000_0010_0000_0000_0000_0000_0000;
        /// Cache Parity Bit
        const PE = 0b0000_0000_0001_0000_0000_0000_0000_0000;
        /// Cache Miss
        const CM = 0b0000_0000_0000_1000_0000_0000_0000_0000;
        /// Cache parity forced to zero
        const PZ = 0b0000_0000_0000_0100_0000_0000_0000_0000;
        /// Swap Caches
        const SWC = 0b0000_0000_0000_0010_0000_0000_0000_0000;
        /// Isolate Cache
        const ISC = 0b0000_0000_0000_0001_0000_0000_0000_0000;
        /// Interrupt Mask
        const IM = 0b0000_0000_0000_0000_1111_1111_0000_0000;
        /// Old Kernel / User Status
        const KUO = 0b0000_0000_0000_0000_0000_0000_0010_0000;
        /// Old Interrupt Enable Status
        const IEO = 0b0000_0000_0000_0000_0000_0000_0001_0000;
        /// Previous Kernel / User Status
        const KUP = 0b0000_0000_0000_0000_0000_0000_0000_1000;
        /// Previous Interrupt Enable Status
        const IEP = 0b0000_0000_0000_0000_0000_0000_0000_0100;
        /// Current Kernel / User Mode Status (KUc = 0 indicates kernel-mode)
        const KUC = 0b0000_0000_0000_0000_0000_0000_0000_0010;
        /// Current Interrupt Enable Status
        const IEC = 0b0000_0000_0000_0000_0000_0000_0000_0001;
    }
}

/// CP0 is the sytem control coprocessor that handles address translation and exception handling.
#[derive(Copy, Clone, Debug, Default)]
pub struct CPZero {
    pub reg: [u32; 32],
}

impl CPZero {
    pub fn new() -> Self {
        Default::default()
    }

    /// Resets the CP0 control registers to their initial state.
    /// Refer to Chapter 7 of the IDT R30xx Manual for details.
    pub fn reset(&mut self) {
        // Clear registers
        for r in self.reg.iter_mut() {
            *r = 0;
        }

        // Enable bootstrap exception vectors on reset
        self.reg[STATUS] |= StatusMask::BEV.bits();

        // Start the processor in kernel-mode
        self.reg[STATUS] &= !StatusMask::KUC.bits();

        // Disable interrupts
        self.reg[STATUS] &= !StatusMask::IEC.bits();

        // TS is cleared to zero if MMU is present
        self.reg[STATUS] &= !StatusMask::TS.bits();

        // Caches are not switched
        self.reg[STATUS] &= !StatusMask::SWC.bits();

        // Set processor revision identifier to indicate a MIPS R3000A
        self.reg[PRID] = 0x230;
    }

    /// Translates a virtual address to a physical address.
    pub fn translate(&self, vaddress: Address) -> Address {
        // let mut cacheable = false;

        if self.kernel_mode() {
            // Determine which kernel segment the address is located in
            match vaddress & KSEG_SELECT_MASK {
                KSEG0 => {
                    // cacheable = true;
                    vaddress - KSEG0
                }
                KSEG1 => vaddress - KSEG1,
                KSEG2 | KSEG2_TOP => self.tlb_translate(KSEG2, vaddress),
                _ => self.tlb_translate(KUSEG, vaddress),
            }
        } else if vaddress & KERNEL_SPACE_MASK != 0 {
            // Attempted to access kernel-space while not in kernel mode
            // Trigger an exception
            0xffff_ffff
        } else {
            // Translate a user-space address
            self.tlb_translate(KUSEG, vaddress)
        }
    }

    fn tlb_translate(&self, _segment: Address, vaddress: Address) -> Address {
        // TODO: Implement TLB
        vaddress
    }

    /// Handles processor exceptions by updating the state of `CPZero`.
    /// See Chapter 4-3 Exception Management in IDT R30xx Manual.
    pub fn exception(&mut self, pc: Address, _exception_code: ExceptionCode) {
        // Save current PC in the EPC register to point to the restart location
        self.reg[EPC] = pc;

        // Switch to kernel-mode and disable interrupts
        self.reg[STATUS] &= !(StatusMask::KUC.bits() | StatusMask::IEC.bits());

        // Cause is setup so that software can see the reason for the exception.
        // On address exceptions BadVaddr is also set.
        // Memory management system exceptions set up some of the MMU registers too; see the chapter on memory management for more
        // detail.
        self.reg[CAUSE] = 1;

        // Transfer control to the exception entry point
    }

    /// Read Indexed TLB Entry
    pub fn tlbr_emulate(&self, _instr: Instruction) {
        todo!()
    }

    /// Write Indexed TLB Entry
    pub fn tlbwi_emulate(&self, _instr: Instruction) {
        todo!()
    }

    /// Write Random TLB Entry
    pub fn tlbwr_emulate(&self, _instr: Instruction) {
        todo!()
    }

    /// Probe TLB For Matching Entry
    pub fn tlbp_emulate(&self, _instr: Instruction) {
        todo!()
    }

    /// Restore from Exception
    /// This instruction restores the status register to go back to the state prior to the trap.
    pub fn rfe_emulate(&self, _instr: Instruction) {
        todo!()
    }

    pub fn bc0x_emulate(&self, _instr: Instruction, _pc: u32) {
        todo!()
    }

    /// Checks if the given coprocessor number is enabled.
    pub fn coprocessor_usable(&self, coprocno: u32) -> bool {
        match coprocno {
            0 => (self.reg[STATUS] & StatusMask::CU0.bits()) != 0,
            1 => (self.reg[STATUS] & StatusMask::CU1.bits()) != 0,
            2 => (self.reg[STATUS] & StatusMask::CU2.bits()) != 0,
            3 => (self.reg[STATUS] & StatusMask::CU3.bits()) != 0,
            _ => unreachable!("Invalid coprocessor number"),
        }
    }

    /// Returns true if the processor is in kernel-mode.
    pub fn kernel_mode(&self) -> bool {
        (self.reg[STATUS] & StatusMask::KUC.bits()) == 0
    }

    /// Returns true if interrupts are currently enabled.
    pub fn interrupts_enabled(&self) -> bool {
        (self.reg[STATUS] & StatusMask::IEC.bits()) == 1
    }
}
