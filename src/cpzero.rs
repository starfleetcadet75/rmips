use crate::util::constants::ExceptionCode;
use crate::util::constants::NUM_GPR;
use crate::util::constants::{
    KSEG0, KSEG0_CONST_TRANSLATION, KSEG1, KSEG1_CONST_TRANSLATION, KSEG2, KSEG2_TOP,
    KSEG_SELECT_MASK,
};

// CP0 register names and numbers
const INDEX: usize = 0;
const RANDOM: usize = 1;
const ENTRYLO0: usize = 2;
const ENTRYLO1: usize = 3;
/// The Status register contains the operating mode, interrupt enable flag, and diagnostic states
const STATUS: usize = 12;
/// Contains the cause of the last exception
const CAUSE: usize = 13;
/// Contains the address to return to after handling an exception
const EPC: usize = 14;

bitflags! {
    /// Bitmask for extracting fields from the Status register
    struct StatusMask: u32 {
        /// Coprocessor 3 Usable
        const CU3 = 0b10000000_00000000_00000000_00000000;
        /// Coprocessor 2 Usable
        const CU2 = 0b01000000_00000000_00000000_00000000;
        /// Coprocessor 1 Usable
        const CU1 = 0b00100000_00000000_00000000_00000000;
        /// Coprocessor 0 Usable
        const CU0 = 0b00010000_00000000_00000000_00000000;
        /// Coprocessors Usable
        const CU = Self::CU3.bits | Self::CU2.bits | Self::CU1.bits | Self::CU0.bits;
        /// Reverse Endianness in User Mode
        const RE = 0b00000010_00000000_00000000_00000000;
        /// Bootstrap Exception Vector
        const BEV = 0b00000000_01000000_00000000_00000000;
        /// TLB Shutdown
        const TS = 0b00000000_00100000_00000000_00000000;
        /// Cache Parity Bit
        const PE = 0b00000000_00010000_00000000_00000000;
        /// Cache Miss
        const CM = 0b00000000_00001000_00000000_00000000;
        /// Cache parity forced to zero
        const PZ = 0b00000000_00000100_00000000_00000000;
        /// Swap Caches
        const SWC = 0b00000000_00000010_00000000_00000000;
        /// Isolate Cache
        const ISC = 0b00000000_00000001_00000000_00000000;
        /// Interrupt Mask
        const IM = 0b00000000_00000000_11111111_00000000;
        /// Old Kernel / User Status
        const KUO = 0b00000000_00000000_00000000_00100000;
        /// Old Interrupt Enable Status
        const IEO = 0b00000000_00000000_00000000_00010000;
        /// Previous Kernel / User Status
        const KUP = 0b00000000_00000000_00000000_00001000;
        /// Previous Interrupt Enable Status
        const IEP = 0b00000000_00000000_00000000_00000100;
        /// Current Kernel / User Status
        const KUC = 0b00000000_00000000_00000000_00000010;
        /// Current Interrupt Enable Status
        const IEC = 0b00000000_00000000_00000000_00000001;
    }
}

/// CP0 is the sytem control coprocessor that handles address translation and exception handling
pub struct CPZero {
    pub reg: [u32; NUM_GPR],
}

impl CPZero {
    pub fn new() -> CPZero {
        CPZero { reg: [0; NUM_GPR] }
    }

    pub fn reset(&mut self) {
        for r in self.reg.iter_mut() {
            *r = 0;
        }
    }

    /// Translates a virtual address to a physical address
    pub fn translate(&self, vaddress: u32) -> u32 {
        match vaddress & KSEG_SELECT_MASK {
            KSEG0 => vaddress - KSEG0_CONST_TRANSLATION,
            KSEG1 => vaddress - KSEG1_CONST_TRANSLATION,
            KSEG2 | KSEG2_TOP => todo!(),
            _ => todo!(),
        }
    }

    pub fn enter_exception(&mut self, pc: u32, exception_code: ExceptionCode) {
        self.reg[EPC] = pc; // Save the PC in the EPC register

        // Disable interrupts
        self.reg[STATUS] = self.reg[STATUS] & StatusMask::IEC.bits();

        // self.reg[STATUS] =
        //     (self.reg[STATUS] & !STATUS_KU_IE_MASK) | (self.reg[STATUS] & STATUS_KU_IE_MASK) << 2;

        // Enter kernel mode
    }

    /// Read Indexed TLB Entry
    pub fn tlbr_emulate(&self, _instr: u32) {
        todo!()
    }

    /// Write Indexed TLB Entry
    pub fn tlbwi_emulate(&self, _instr: u32) {
        todo!()
    }

    /// Write Random TLB Entry
    pub fn tlbwr_emulate(&self, _instr: u32) {
        todo!()
    }

    /// Probe TLB For Matching Entry
    pub fn tlbp_emulate(&self, _instr: u32) {
        todo!()
    }

    /// Return from Exception
    pub fn rfe_emulate(&self, _instr: u32) {
        todo!()
    }

    pub fn bc0x_emulate(&self, _instr: u32, _pc: u32) {
        todo!()
    }

    /// Checks if the given coprocessor number is enabled
    pub fn coprocessor_usable(&self, coprocno: u32) -> bool {
        match coprocno {
            0 => (self.reg[STATUS] & StatusMask::CU0.bits()) != 0,
            1 => (self.reg[STATUS] & StatusMask::CU1.bits()) != 0,
            2 => (self.reg[STATUS] & StatusMask::CU2.bits()) != 0,
            3 => (self.reg[STATUS] & StatusMask::CU3.bits()) != 0,
            _ => unreachable!("Invalid coprocessor number"),
        }
    }

    /// Returns true if the processor is in kernel mode
    pub fn kernel_mode(&self) -> bool {
        self.reg[STATUS] & StatusMask::KUC.bits() == 1
    }

    /// Returns true if interrupts are currently enabled
    pub fn interrupts_enabled(&self) -> bool {
        self.reg[STATUS] & StatusMask::IEC.bits() == 1
    }

    /// Prints the contents of the CP0 registers
    pub fn dump_regs(&self) {
        println!("CP0 Registers: [");
        for i in 0..16 {
            print!(" R{:02}={:08x} ", i, self.reg[i]);
            if i % 4 == 1 {
                println!("");
            }
        }
        println!("]");
    }
}
