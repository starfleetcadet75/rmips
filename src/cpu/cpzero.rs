use crate::cpu::{
    ExceptionCode, CAUSE, EPC, KERNEL_SPACE_MASK, KSEG0, KSEG1, KSEG2, KSEG2_TOP, KSEG_SELECT_MASK,
    KUSEG, NUM_GPR, PRID, STATUS,
};
use std::fmt;

bitflags! {
    /// Bitmasks for extracting fields from the Status Register (SR)
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
        /// Current Kernel / User Mode Status (KUc = 0 indicates kernel-mode)
        const KUC = 0b00000000_00000000_00000000_00000010;
        /// Current Interrupt Enable Status
        const IEC = 0b00000000_00000000_00000000_00000001;
    }
}

/// CP0 is the sytem control coprocessor that handles address translation and exception handling.
#[derive(Default)]
pub struct CPZero {
    pub reg: [u32; NUM_GPR],
}

impl CPZero {
    pub fn new() -> Self {
        Default::default()
    }

    /// Resets the CP0 control registers to their initial state.
    /// See the IDT R30xx Family Software Reference Manual
    /// Chapter 7-1 Reset Initialization for details.
    pub fn reset(&mut self) {
        // Clear registers
        for r in self.reg.iter_mut() {
            *r = 0;
        }

        // Enable bootstrap exception vectors on reset
        self.reg[STATUS] = self.reg[STATUS] | StatusMask::BEV.bits();

        // Start the processor in kernel-mode
        self.reg[STATUS] = self.reg[STATUS] & !StatusMask::KUC.bits();

        // Disable interrupts
        self.reg[STATUS] = self.reg[STATUS] & !StatusMask::IEC.bits();

        // TS is cleared to zero if MMU is present
        self.reg[STATUS] = self.reg[STATUS] & !StatusMask::TS.bits();

        // Caches are not switched
        self.reg[STATUS] = self.reg[STATUS] & !StatusMask::SWC.bits();

        // Set processor revision identifier to indicate a MIPS R3000A
        self.reg[PRID] = 0x230;
    }

    /// Translates a virtual address to a physical address.
    pub fn translate(&self, vaddress: u32) -> u32 {
        if self.kernel_mode() {
            // Determine which kernel segment the address is located in
            match vaddress & KSEG_SELECT_MASK {
                KSEG0 => vaddress - KSEG0,
                KSEG1 => vaddress - KSEG1,
                KSEG2 | KSEG2_TOP => self.tlb_translate(KSEG2, vaddress),
                _ => self.tlb_translate(KUSEG, vaddress),
            }
        } else {
            if vaddress & KERNEL_SPACE_MASK == 1 {
                // Attempted to access kernel-space while not in kernel mode
                // Trigger an exception
                0xffff_ffff
            } else {
                self.tlb_translate(KUSEG, vaddress)
            }
        }
    }

    fn tlb_translate(&self, _segment: u32, _vaddress: u32) -> u32 {
        todo!();
    }

    /// Handles processor exceptions by setting the state of CP0.
    /// See the IDT R30xx Family Software Reference Manual
    /// Chapter 4-3 Exception Management for details.
    pub fn exception(&mut self, pc: u32, _exception_code: ExceptionCode) {
        // Save PC in the EPC register to point to the restart location
        self.reg[EPC] = pc;

        // Switch to kernel-mode and disable interrupts
        self.reg[STATUS] = self.reg[STATUS] & !(StatusMask::KUC.bits() | StatusMask::IEC.bits());

        // Cause is setup so that software can see the reason for the exception.
        // On address exceptions BadVaddr is also set.
        // Memory management system exceptions set up some of the MMU registers too; see the chapter on memory management for more
        // detail.
        self.reg[CAUSE] = 1;

        // Transfer control to the exception entry point
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

impl fmt::Display for CPZero {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let res = (1..=31)
            .map(|r| {
                format!(
                    "${:02} = 0x{:08x} {:13}",
                    r,
                    self.reg[r],
                    format!("({})", self.reg[r] as i32)
                )
            })
            .collect::<Vec<_>>()
            .chunks(4)
            .map(|chunk| chunk.join(" "))
            .collect::<Vec<_>>()
            .join("\n");
        write!(f, "{}", res)
    }
}
