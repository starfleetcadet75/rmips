use std::fmt;

use capstone::prelude::*;
use log::{error, warn};

use crate::control::cpzero::CPZero;
use crate::control::exception::Exception;
use crate::control::instruction::Instruction;
use crate::control::registers::Register;
use crate::memory::Memory;
use crate::util::error::{Result, RmipsError};
use crate::Address;

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub enum DelayState {
    /// No delay slot handling needs to occur
    Normal,
    /// The last instruction caused a branch to be taken
    Delaying,
    /// The last instruction was executed in a delay slot
    Delayslot,
}

impl Default for DelayState {
    fn default() -> Self {
        DelayState::Normal
    }
}

#[derive(Debug, Default)]
pub struct Cpu {
    /// The program counter.
    pub pc: Address,
    /// General-purpose registers.
    pub reg: [u32; 32],
    /// The current instruction.
    pub instruction: Instruction,
    /// High division result register.
    pub high: u32,
    /// Low division result register.
    pub low: u32,
    /// Indicates whether the current instruction is in a delay slot.
    pub delay_state: DelayState,
    /// Saved target address for delayed branches.
    pub delay_pc: Address,
    /// Indicates whether an exception is waiting to be handled.
    pub exception_pending: bool,
    /// The System Control Coprocessor (CP0).
    pub cpzero: CPZero,
    /// Capstone instance for disassembly.
    disassembler: Option<Capstone>,
}

impl Cpu {
    pub fn new(enable_disassembler: bool) -> Self {
        // Create an instance of Capstone to use as a disassembler if requested
        Cpu {
            disassembler: match enable_disassembler {
                true => Some(
                    Capstone::new()
                        .mips()
                        .mode(arch::mips::ArchMode::Mips32R6)
                        .detail(true)
                        .build()
                        .expect("Capstone failed to initialize"),
                ),
                false => None,
            },
            ..Default::default()
        }
    }

    /// Resets the `Cpu` state to initial startup values
    pub fn reset(&mut self) {
        self.reg[Register::Zero] = 0;
        self.pc = 0xbfc00000;
        self.cpzero.reset();
    }

    /// Decodes and executes the next instruction according to the value in the program counter
    pub fn step(&mut self, memory: &mut impl Memory) -> Result<()> {
        self.exception_pending = false;

        // Get the physical address of the next instruction
        let phys_pc = self.cpzero.translate(self.pc);

        // Fetch the next instruction from memory
        self.instruction = Instruction(memory.fetch_word(phys_pc)?);

        // Disassemble the instruction if enabled by the user
        if let Some(disassembler) = &self.disassembler {
            let code = self.instruction.0.to_le_bytes();
            if let Ok(instr) = disassembler.disasm_count(&code, self.pc.into(), 1) {
                // Should always be one instruction
                // There are a few valid instructions that Capstone seems to fail on
                if let Some(i) = instr.iter().next() {
                    println!(
                        "PC=0x{:08x} [{:08x}]\t{:08x}  {} {}",
                        self.pc,
                        phys_pc,
                        self.instruction.0,
                        i.mnemonic().expect("capstone errored"),
                        i.op_str().expect("capstone errored")
                    );
                } else {
                    println!(
                        "PC=0x{:08x} [{:08x}]\tDisassembly Failed: {:?}",
                        self.pc, phys_pc, self.instruction
                    );
                }
            } else {
                println!(
                    "PC=0x{:08x} [{:08x}]\tDisassembly Failed: {:?}",
                    self.pc, phys_pc, self.instruction
                );
            }
        }

        // Decode and emulate the instruction
        let instr = self.instruction;
        match instr.opcode() {
            0x00 => match instr.funct() {
                0x00 => self.sll_emulate(instr),
                0x02 => self.srl_emulate(instr),
                0x03 => self.sra_emulate(instr),
                0x04 => self.sllv_emulate(instr),
                0x06 => self.srlv_emulate(instr),
                0x07 => self.srav_emulate(instr),
                0x08 => self.jr_emulate(instr),
                0x09 => self.jalr_emulate(instr),
                0x0c => self.syscall_emulate()?,
                0x0d => self.break_emulate()?,
                0x10 => self.mfhi_emulate(instr),
                0x11 => self.mthi_emulate(instr),
                0x12 => self.mflo_emulate(instr),
                0x13 => self.mtlo_emulate(instr),
                0x18 => self.mult_emulate(instr),
                0x19 => self.multu_emulate(instr),
                0x1a => self.div_emulate(instr),
                0x1b => self.divu_emulate(instr),
                0x20 => self.add_emulate(instr)?,
                0x21 => self.addu_emulate(instr),
                0x22 => self.sub_emulate(instr)?,
                0x23 => self.subu_emulate(instr),
                0x24 => self.and_emulate(instr),
                0x25 => self.or_emulate(instr),
                0x26 => self.xor_emulate(instr),
                0x27 => self.nor_emulate(instr),
                0x2a => self.slt_emulate(instr),
                0x2b => self.sltu_emulate(instr),
                _ => self.ri_emulate()?,
            },
            0x01 => match instr.rt() {
                0 => self.bltz_emulate(instr),
                1 => self.bgez_emulate(instr),
                16 => self.bltzal_emulate(instr),
                17 => self.bgezal_emulate(instr),
                _ => self.ri_emulate()?,
            },
            0x02 => self.j_emulate(instr),
            0x03 => self.jal_emulate(instr),
            0x04 => self.beq_emulate(instr),
            0x05 => self.bne_emulate(instr),
            0x06 => self.blez_emulate(instr),
            0x07 => self.bgtz_emulate(instr),
            0x08 => self.addi_emulate(instr)?,
            0x09 => self.addiu_emulate(instr),
            0x0a => self.slti_emulate(instr),
            0x0b => self.sltiu_emulate(instr),
            0x0c => self.andi_emulate(instr),
            0x0d => self.ori_emulate(instr),
            0x0e => self.xori_emulate(instr),
            0x0f => self.lui_emulate(instr),
            0x10 => {
                // Handle CP0 instructions
                let rs = instr.rs();

                if 15 < rs {
                    match instr.funct() {
                        1 => self.cpzero.tlbr_emulate(),
                        2 => self.cpzero.tlbwi_emulate(),
                        6 => self.cpzero.tlbwr_emulate(),
                        8 => self.cpzero.tlbp_emulate(),
                        16 => self.cpzero.rfe_emulate(),
                        _ => self.exception(Exception::ReservedInstruction)?,
                    }
                } else {
                    match rs {
                        0 => self.mfc0_emulate(instr),
                        4 => self.mtc0_emulate(instr),
                        8 => self.cpzero.bc0x_emulate(instr, self.pc),
                        _ => self.exception(Exception::ReservedInstruction)?,
                    }
                }
            }
            0x11 => self.coprocessor_unimpl(1, instr)?,
            0x12 => self.coprocessor_unimpl(2, instr)?,
            0x13 => self.coprocessor_unimpl(3, instr)?,
            0x20 => self.lb_emulate(memory, instr)?,
            0x21 => self.lh_emulate(memory, instr)?,
            0x22 => self.lwl_emulate(instr),
            0x23 => self.lw_emulate(memory, instr)?,
            0x24 => self.lbu_emulate(memory, instr)?,
            0x25 => self.lhu_emulate(memory, instr)?,
            0x26 => self.lwr_emulate(instr),
            0x28 => self.sb_emulate(memory, instr)?,
            0x29 => self.sh_emulate(memory, instr)?,
            0x2a => self.swl_emulate(instr),
            0x2b => self.sw_emulate(memory, instr)?,
            0x2e => self.swr_emulate(instr),
            0x31 => self.lwc1_emulate(instr)?,
            0x32 => self.lwc2_emulate(instr)?,
            0x33 => self.lwc3_emulate(instr)?,
            0x38 => self.swc1_emulate(instr)?,
            0x39 => self.swc2_emulate(instr)?,
            0x3a => self.swc3_emulate(instr)?,
            _ => self.ri_emulate()?,
        }

        // Register $r0 is hardwired to a value of zero
        // It can be written to by instructions however the result is always discarded
        self.reg[Register::Zero] = 0;

        // The program counter is already updated to contain the address
        // of the exception handler in the `exception` function
        if self.exception_pending {
            // The first instruction in the exception handler will never be in a delay slot
            self.delay_state = DelayState::Normal;
            return Ok(());
        }

        // Update the program counter
        // `DelayState` tracks whether the current instruction should be executed from the delay slot
        match self.delay_state {
            // Increment the program counter by 4 for normal instructions
            DelayState::Normal => self.pc = self.pc.wrapping_add(4),
            // The next instruction to be executed is in the delay slot
            DelayState::Delaying => {
                self.pc = self.pc.wrapping_add(4);
                self.delay_state = DelayState::Delayslot;
            }
            // Current instruction was executed from the delay slot and the branch should now occur
            DelayState::Delayslot => {
                self.pc = self.delay_pc;
                self.delay_state = DelayState::Normal;
            }
        }

        Ok(())
    }

    pub fn coprocessor_unimpl(&mut self, coprocno: u32, instr: Instruction) -> Result<()> {
        if self.cpzero.coprocessor_usable(coprocno) {
            error!(
                "CP{} instruction {:x} is not implemented at PC={:08x}",
                coprocno,
                instr.opcode(),
                self.pc
            )
        }

        self.exception(Exception::CoprocessorUnusable)
    }

    pub fn exception(&mut self, exception: Exception) -> Result<()> {
        match exception {
            Exception::InstructionBusError => {
                warn!("Instruction bus error occurred");
                return Err(RmipsError::Halt);
            }
            Exception::Breakpoint => {
                warn!("BREAK instruction reached");
                return Err(RmipsError::Halt);
            }
            Exception::ReservedInstruction => warn!(
                "Encountered a reserved instruction:\n{:?}",
                self.instruction
            ),
            Exception::Overflow => warn!("Arithmetic overflow occurred"),
            _ => {}
        }

        // Prioritize the exception
        // TODO

        // Update the CP0 state to enter the exception
        self.cpzero.exception(
            self.pc,
            exception,
            self.delay_state == DelayState::Delayslot,
        );

        // The base of the exception handler address is determined by the BEV bit in the CP0 Status register.
        // The CPU initially uses the ROM (kseg1) space exception entry point at boot but will typically
        // be switched to use user supplied exception service routines.
        let base: Address = if self.cpzero.boot_exception_vector_enabled() {
            0xbfc00100
        } else {
            0x80000000
        };

        // If the exception was a TLB miss jump to the User TLB Miss exception vector.
        // Otherwise jump to the common exception vector.
        let vector = if (exception == Exception::TLBLoadMiss
            || exception == Exception::TLBStoreMiss)
            && self.cpzero.tlb_miss_user
        {
            0x000
        } else {
            0x080
        };

        // Transfer control to the exception entry point where emulation will continue
        self.pc = base + vector;
        self.exception_pending = true;

        Ok(())
    }
}

#[rustfmt::skip]
impl fmt::Display for Cpu {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut output = String::from("");
        let abi = [
            "zero", " at ", " v0 ", " v1 ", " a0 ", " a1 ", " a2 ", " a3 ", " t0 ", " t1 ", " t2 ",
            " t3 ", " t4 ", " t5 ", " t6 ", " t7 ", " s0 ", " s1 ", " s2 ", " s3 ", " s4 ", " s5 ",
            " s6 ", " s7 ", " t8 ", " t9 ", " k0 ", " k1 ", " gp ", " sp ", " fp ", " ra ",
        ];

        for i in (0..32).step_by(4) {
            output = format!(
                "{}\n{}",
                output,
                format!(
                    "{} = {:>#10x} {} = {:>#10x} {} = {:>#10x} {} = {:>#10x}",
                    abi[i],
                    self.reg[i],
                    abi[i + 1],
                    self.reg[i + 1],
                    abi[i + 2],
                    self.reg[i + 2],
                    abi[i + 3],
                    self.reg[i + 3],
                )
            );
        }

        write!(f, "{}", output)
    }
}
