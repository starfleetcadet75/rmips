use crate::control::exception::Exception;
use crate::control::instruction::Instruction;
use crate::control::registers::{
    BadVaddrRegister, CauseRegister, ContextRegister, EpcRegister, IndexRegister, PridRegister,
    RandomRegister, StatusRegister,
};
use crate::control::tlbentry::TlbEntry;
use crate::control::{KERNEL_SPACE_MASK, KSEG0, KSEG1, KSEG2, KSEG2_TOP, KSEG_SELECT_MASK, KUSEG};
use crate::Address;

const TLB_ENTRIES: usize = 64;
const RANDOM_UPPER_BOUND: u32 = 63;

/// CP0 is the sytem control coprocessor that handles address translation and exception handling.
#[derive(Copy, Clone, Debug)]
pub struct CPZero {
    pub index: IndexRegister,
    pub random: RandomRegister,
    pub entrylo: u32,
    pub context: ContextRegister,
    pub badvaddr: BadVaddrRegister,
    pub entryhi: u32,
    pub status: StatusRegister,
    pub cause: CauseRegister,
    pub epc: EpcRegister,
    pub prid: PridRegister,
    pub tlb_miss_user: bool,
    tlb: [TlbEntry; TLB_ENTRIES],
}

impl Default for CPZero {
    fn default() -> Self {
        CPZero {
            index: IndexRegister::new(),
            random: RandomRegister::new(),
            entrylo: 0,
            context: ContextRegister::new(),
            badvaddr: BadVaddrRegister::new(),
            entryhi: 0,
            status: StatusRegister::new(),
            cause: CauseRegister::new(),
            epc: EpcRegister::new(),
            prid: PridRegister::new(),
            tlb_miss_user: false,
            tlb: [TlbEntry::default(); TLB_ENTRIES],
        }
    }
}

impl CPZero {
    pub fn new() -> Self {
        Default::default()
    }

    /// Resets the CP0 control registers to their initial states.
    /// Refer to Chapter 7 of the IDT R30xx Manual for details.
    pub fn reset(&mut self) {
        // Random register is initialized to its maximum value (63) on reset
        self.random.set_value(RANDOM_UPPER_BOUND);

        // Enable bootstrap exception vector (BEV) on reset
        self.status.enter_bootstrap();

        // Start the processor in kernel-mode
        self.status.enter_kernel_mode();

        // Disable interrupts
        self.status.disable_interrupts();

        // TS is cleared to zero if MMU is present
        self.status.clear_ts();

        // Caches are not switched
        self.status.clear_swc();

        // Set processor revision identifier to indicate a MIPS R3000A
        self.prid.bits = 0x230;
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
    pub fn exception(&mut self, pc: Address, exception: Exception, delayslot: bool) {
        // Save current PC in the EPC register to point to the restart location
        self.epc.address = pc;

        // Switch to kernel-mode
        self.status.enter_kernel_mode();

        // Disable interrupts
        self.status.disable_interrupts();

        // Clear the Cause register
        self.cause.bits = 0;

        // Set Cause register CE field if this is a Coprocessor Unusable exception
        if exception == Exception::CoprocessorUnusable {
            // TODO: This should be the coprocessor number that caused the error
            self.cause.set_coprocessor_error(2);
        }

        // Save the ExcCode in the Cause register
        self.cause.set_exception_code(exception);

        // If the exception occurred from a delay slot, EPC does not point to the actual exception
        // instruction but rather to the branch instruction which immediately precedes it.
        // This is indicated by setting the BD bit.
        if delayslot {
            self.cause.set_branch_delay();
        }

        // Set the interrupt pending field of the Cause register
        // TODO
    }

    /// Read Indexed TLB Entry
    pub fn tlbr_emulate(&self) {
        todo!()
    }

    /// Write Indexed TLB Entry
    pub fn tlbwi_emulate(&mut self) {
        let index = self.index.get_index() as usize;
        self.tlb[index].entryhi = self.entryhi;
        self.tlb[index].entrylo = self.entrylo;
    }

    /// Write Random TLB Entry
    pub fn tlbwr_emulate(&mut self) {
        let index = self.random.get_value() as usize;
        self.tlb[index].entryhi = self.entryhi;
        self.tlb[index].entrylo = self.entrylo;
    }

    /// Probe TLB For Matching Entry
    pub fn tlbp_emulate(&self) {
        todo!()
    }

    /// Restore from Exception
    /// Restores the proper Interrupt Enable and Kernel/User mode
    /// bits of the status register on return from exception.
    pub fn rfe_emulate(&mut self) {
        self.status.bits = (self.status.bits & 0xfffffff0) | ((self.status.bits >> 2) & 0x0f);
    }

    pub fn bc0x_emulate(&self, _instr: Instruction, _pc: Address) {
        todo!()
    }

    /// Checks if the given coprocessor number is enabled.
    pub fn coprocessor_usable(&self, coprocno: u32) -> bool {
        match coprocno {
            0 => self.status.cu0(),
            1 => self.status.cu1(),
            2 => self.status.cu2(),
            3 => self.status.cu3(),
            _ => unreachable!("Invalid coprocessor number"),
        }
    }

    /// Returns true if the processor is in kernel-mode.
    pub fn kernel_mode(&self) -> bool {
        self.status.is_kernel_mode()
    }

    /// Returns true if interrupts are currently enabled.
    pub fn interrupts_enabled(&self) -> bool {
        self.status.are_interrupts_enabled()
    }

    /// Returns true if the Bootstrap Exception Vector (BEV) is enabled.
    pub fn boot_exception_vector_enabled(&self) -> bool {
        self.status.is_bootstrap_mode()
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

        assert_eq!(cp0.random.get_value(), RANDOM_UPPER_BOUND);
        assert_eq!(cp0.boot_exception_vector_enabled(), true);
        assert_eq!(cp0.kernel_mode(), true);
        assert_eq!(cp0.interrupts_enabled(), false);
        assert_eq!(cp0.coprocessor_usable(0), false);
        assert_eq!(cp0.coprocessor_usable(1), false);
        assert_eq!(cp0.coprocessor_usable(2), false);
        assert_eq!(cp0.coprocessor_usable(3), false);
    }

    #[test]
    fn cpzero_exception_coprocessor_unusable() {
        let mut cp0 = CPZero::new();
        cp0.reset();

        cp0.exception(0xbfc00400, Exception::CoprocessorUnusable, false);
        assert_eq!(cp0.kernel_mode(), true);
        assert_eq!(cp0.interrupts_enabled(), false);

        assert_ne!(cp0.cause.get_coprocessor_error(), 0);
        assert_eq!(
            cp0.cause.get_exception_code(),
            Exception::CoprocessorUnusable
        );
        assert_eq!(cp0.cause.is_branch_delay(), false);
    }

    #[test]
    fn cpzero_exception_interrupt() {
        let mut cp0 = CPZero::new();
        cp0.reset();

        cp0.exception(0xbfc00400, Exception::Interrupt, true);
        assert_eq!(cp0.kernel_mode(), true);
        assert_eq!(cp0.interrupts_enabled(), false);

        assert_eq!(cp0.cause.get_coprocessor_error(), 0);
        assert_eq!(cp0.cause.get_exception_code(), Exception::Interrupt);
        assert_eq!(cp0.cause.is_branch_delay(), true);
    }

    #[test]
    fn cpzero_rfe_emulate() {
        let mut cp0 = CPZero::new();
        cp0.reset();

        cp0.status.set_kuo();
        cp0.status.clear_ieo();
        cp0.status.clear_kup();
        cp0.status.set_iep();
        cp0.rfe_emulate();

        assert_eq!(cp0.status.kuo(), true);
        assert_eq!(cp0.status.ieo(), false);
        assert_eq!(cp0.status.kup(), true);
        assert_eq!(cp0.status.iep(), false);
        assert_eq!(cp0.status.is_kernel_mode(), false);
        assert_eq!(cp0.status.are_interrupts_enabled(), true);
    }
}
