use crate::control::instruction::Instruction;
use crate::control::registers::{Cp0Register, IndexMask, RandomMask, StatusMask};
use crate::control::tlbentry::TlbEntry;
use crate::control::{
    ExceptionCode, KERNEL_SPACE_MASK, KSEG0, KSEG1, KSEG2, KSEG2_TOP, KSEG_SELECT_MASK, KUSEG,
};
use crate::Address;

const TLB_ENTRIES: usize = 64;
const RANDOM_UPPER_BOUND: u32 = 63;

/// CP0 is the sytem control coprocessor that handles address translation and exception handling.
#[derive(Copy, Clone, Debug)]
pub struct CPZero {
    pub reg: [u32; 32],
    tlb: [TlbEntry; TLB_ENTRIES],
}

impl Default for CPZero {
    fn default() -> Self {
        CPZero {
            reg: [0; 32],
            tlb: [TlbEntry::default(); TLB_ENTRIES],
        }
    }
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

        // Random register is initialized to its maximum value (63) on reset
        self.reg[Cp0Register::Random] = RANDOM_UPPER_BOUND << 8;

        // Enable bootstrap exception vectors on reset
        self.reg[Cp0Register::Status] |= StatusMask::BEV.bits();

        // Start the processor in kernel-mode
        self.reg[Cp0Register::Status] &= !StatusMask::KUC.bits();

        // Disable interrupts
        self.reg[Cp0Register::Status] &= !StatusMask::IEC.bits();

        // TS is cleared to zero if MMU is present
        self.reg[Cp0Register::Status] &= !StatusMask::TS.bits();

        // Caches are not switched
        self.reg[Cp0Register::Status] &= !StatusMask::SWC.bits();

        // Set processor revision identifier to indicate a MIPS R3000A
        self.reg[Cp0Register::Prid] = 0x230;
    }

    /// Translates a virtual address to a physical address.
    ///
    /// Addresses in kuseg and kseg2 use the TLB for translation.
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
        self.reg[Cp0Register::Epc] = pc;

        // Switch to kernel-mode and disable interrupts
        self.reg[Cp0Register::Status] &= !(StatusMask::KUC.bits() | StatusMask::IEC.bits());

        // Cause is setup so that software can see the reason for the exception.
        // On address exceptions BadVaddr is also set.
        // Memory management system exceptions set up some of the MMU registers too; see the chapter on memory management for more
        // detail.
        self.reg[Cp0Register::Cause] = 1;

        // Transfer control to the exception entry point
    }

    /// Read Indexed TLB Entry
    pub fn tlbr_emulate(&self) {
        todo!()
    }

    /// Write Indexed TLB Entry
    pub fn tlbwi_emulate(&mut self) {
        let index = ((self.reg[Cp0Register::Index] & IndexMask::IDX.bits()) >> 8) as usize;
        self.tlb[index].entryhi = self.reg[Cp0Register::EntryHi];
        self.tlb[index].entrylo = self.reg[Cp0Register::EntryLo];
    }

    /// Write Random TLB Entry
    pub fn tlbwr_emulate(&mut self) {
        let index = ((self.reg[Cp0Register::Random] & RandomMask::VAL.bits()) >> 8) as usize;
        self.tlb[index].entryhi = self.reg[Cp0Register::EntryHi];
        self.tlb[index].entrylo = self.reg[Cp0Register::EntryLo];
    }

    /// Probe TLB For Matching Entry
    pub fn tlbp_emulate(&self) {
        todo!()
    }

    /// Restore from Exception
    pub fn rfe_emulate(&mut self) {
        self.reg[Cp0Register::Status] = (self.reg[Cp0Register::Status] & 0xfffffff0)
            | ((self.reg[Cp0Register::Status] >> 2) & 0x0f);
    }

    pub fn bc0x_emulate(&self, _instr: Instruction, _pc: Address) {
        todo!()
    }

    /// Checks if the given coprocessor number is enabled.
    pub fn coprocessor_usable(&self, coprocno: u32) -> bool {
        match coprocno {
            0 => (self.reg[Cp0Register::Status] & StatusMask::CU0.bits()) != 0,
            1 => (self.reg[Cp0Register::Status] & StatusMask::CU1.bits()) != 0,
            2 => (self.reg[Cp0Register::Status] & StatusMask::CU2.bits()) != 0,
            3 => (self.reg[Cp0Register::Status] & StatusMask::CU3.bits()) != 0,
            _ => unreachable!("Invalid coprocessor number"),
        }
    }

    /// Returns true if the processor is in kernel-mode.
    pub fn kernel_mode(&self) -> bool {
        (self.reg[Cp0Register::Status] & StatusMask::KUC.bits()) == 0
    }

    /// Returns true if interrupts are currently enabled.
    pub fn interrupts_enabled(&self) -> bool {
        (self.reg[Cp0Register::Status] & StatusMask::IEC.bits()) == 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn cpzero_reset() {
        let mut cp0 = CPZero::new();
        cp0.reset();

        assert!(cp0.kernel_mode());
        assert!(!cp0.interrupts_enabled());
        assert_eq!(
            (cp0.reg[Cp0Register::Random] & RandomMask::VAL.bits()) >> 8,
            RANDOM_UPPER_BOUND
        );
    }

    #[test]
    fn cpzero_rfe_emulate() {
        let mut cp0 = CPZero::new();
        cp0.reset();

        cp0.reg[Cp0Register::Status] |= StatusMask::KUO.bits();
        cp0.reg[Cp0Register::Status] &= !(StatusMask::IEO.bits());
        cp0.reg[Cp0Register::Status] &= !(StatusMask::KUP.bits());
        cp0.reg[Cp0Register::Status] |= StatusMask::IEP.bits();
        cp0.rfe_emulate();

        assert!(cp0.reg[Cp0Register::Status] & StatusMask::KUO.bits() != 0);
        assert!(cp0.reg[Cp0Register::Status] & StatusMask::IEO.bits() == 0);
        assert!(cp0.reg[Cp0Register::Status] & StatusMask::KUP.bits() != 0);
        assert!(cp0.reg[Cp0Register::Status] & StatusMask::IEP.bits() == 0);
        assert!(cp0.reg[Cp0Register::Status] & StatusMask::KUC.bits() == 0);
        assert!(cp0.reg[Cp0Register::Status] & StatusMask::IEC.bits() != 0);
    }
}
