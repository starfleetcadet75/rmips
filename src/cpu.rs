use crate::cpzero::CPZero;
use crate::mapper::Mapper;
use crate::util::constants::ExceptionCode;
use crate::util::constants::{NUM_GPR, REG_ZERO};
use crate::util::error::RmipsResult;
use capstone::Capstone;
use log::{error, warn};

macro_rules! opcode {
    ($instr:ident) => {
        (($instr >> 26) & 0b0011_1111)
    };
}

macro_rules! rs {
    ($instr:ident) => {
        (($instr >> 21) & 0b0001_1111) as usize
    };
}

macro_rules! rt {
    ($instr:ident) => {
        (($instr >> 16) & 0b0001_1111) as usize
    };
}

macro_rules! rd {
    ($instr:ident) => {
        (($instr >> 11) & 0b0001_1111) as usize
    };
}

macro_rules! immed {
    ($instr:ident) => {
        ($instr & 0xffff)
    };
}

// Must start as a signed value before sign-extending to a u32
macro_rules! simmed {
    ($instr:ident) => {
        ($instr & 0xffff) as i16 as u32
    };
}

macro_rules! shamt {
    ($instr:ident) => {
        (($instr >> 6) & 0b0001_1111)
    };
}

macro_rules! funct {
    ($instr:ident) => {
        ($instr & 0b0011_1111)
    };
}

macro_rules! jumptarget {
    ($instr:ident) => {
        $instr & 0x03ffffff
    };
}

#[derive(Debug, PartialEq)]
pub enum DelayState {
    /// No delay slot handling needs to occur
    NORMAL,
    /// The last instruction caused a branch to be taken
    DELAYING,
    /// The last instruction was executed in a delay slot
    DELAYSLOT,
}

pub struct CPU {
    /// Program counter
    pub pc: u32,
    /// General-purpose registers
    pub reg: [u32; NUM_GPR],
    /// The current instruction
    pub instruction: u32,
    /// High division result
    pub high: u32,
    /// Low division result
    pub low: u32,
    /// Indicates whether the current instruction is in a delay slot
    pub delay_state: DelayState,
    /// Stores the saved target address for a delayed branch
    pub delay_pc: u32,
    pub mapper: Mapper,
    pub cpzero: CPZero,
    /// Capstone instance for disassembly
    disassembler: Option<Capstone>,
}

impl CPU {
    pub fn new(mapper: Mapper, disassembler: Option<Capstone>) -> CPU {
        CPU {
            pc: 0,
            reg: [0; NUM_GPR],
            instruction: 0,
            high: 0,
            low: 0,
            delay_state: DelayState::NORMAL,
            delay_pc: 0,
            mapper,
            cpzero: CPZero::new(),
            disassembler,
        }
    }

    #[cfg(test)]
    pub fn new_test() -> CPU {
        CPU {
            pc: 0,
            reg: [0; NUM_GPR],
            instruction: 0,
            high: 0,
            low: 0,
            delay_state: DelayState::NORMAL,
            delay_pc: 0,
            mapper: Mapper::new(),
            cpzero: CPZero::new(),
            disassembler: None,
        }
    }

    /// Resets the CPU state to the initial values on startup
    pub fn reset(&mut self) {
        self.reg[REG_ZERO] = 0;
        self.pc = 0xbfc00000;
        self.cpzero.reset();
    }

    /// Decodes and executes the next instruction according to the value in the program counter
    pub fn step(&mut self) -> RmipsResult<()> {
        // Get the physical address of the next instruction
        let phys_pc = self.cpzero.translate(self.pc);

        // Fetch the next instruction from memory
        self.instruction = self.mapper.fetch_word(phys_pc)?;

        // Disassemble the instruction if enabled by the user
        if let Some(disassembler) = &self.disassembler {
            let code = self.instruction.to_le_bytes();
            if let Ok(instr) = disassembler.disasm_count(&code, self.pc.into(), 1) {
                // Will always be one instruction
                for i in instr.iter() {
                    println!(
                        "PC=0x{:08x} [{:08x}]\t{:08x}  {} {}",
                        self.pc,
                        phys_pc,
                        self.instruction,
                        i.mnemonic().unwrap(),
                        i.op_str().unwrap()
                    );
                }
            } else {
                println!(
                    "PC=0x{:08x} [{:08x}]\t{:08x}  Disassembly Failed",
                    self.pc, phys_pc, self.instruction,
                );
            }
        }

        // Decode and emulate the instruction
        let instr = self.instruction;
        match opcode!(instr) {
            0x00 => match funct!(instr) {
                0x00 => self.sll_emulate(instr),
                0x02 => self.srl_emulate(instr),
                0x03 => self.sra_emulate(instr),
                0x04 => self.sllv_emulate(instr),
                0x06 => self.srlv_emulate(instr),
                0x07 => self.srav_emulate(instr),
                0x08 => self.jr_emulate(instr),
                0x09 => self.jalr_emulate(instr, self.pc),
                0x0c => self.syscall_emulate(),
                0x0d => self.break_emulate(),
                0x10 => self.mfhi_emulate(instr),
                0x11 => self.mthi_emulate(instr),
                0x12 => self.mflo_emulate(instr),
                0x13 => self.mtlo_emulate(instr),
                0x18 => self.mult_emulate(instr),
                0x19 => self.multu_emulate(instr),
                0x1a => self.div_emulate(instr),
                0x1b => self.divu_emulate(instr),
                0x20 => self.add_emulate(instr),
                0x21 => self.addu_emulate(instr),
                0x22 => self.sub_emulate(instr),
                0x23 => self.subu_emulate(instr),
                0x24 => self.and_emulate(instr),
                0x25 => self.or_emulate(instr),
                0x26 => self.xor_emulate(instr),
                0x27 => self.nor_emulate(instr),
                0x2a => self.slt_emulate(instr),
                0x2b => self.sltu_emulate(instr),
                _ => self.ri_emulate(),
            },
            0x01 => match rt!(instr) {
                0 => self.bltz_emulate(instr),
                1 => self.bgez_emulate(instr),
                16 => self.bltzal_emulate(instr),
                17 => self.bgezal_emulate(instr),
                _ => self.ri_emulate(),
            },
            0x02 => self.j_emulate(instr, self.pc),
            0x03 => self.jal_emulate(instr, self.pc),
            0x04 => self.beq_emulate(instr, self.pc),
            0x05 => self.bne_emulate(instr, self.pc),
            0x06 => self.blez_emulate(instr, self.pc),
            0x07 => self.bgtz_emulate(instr, self.pc),
            0x08 => self.addi_emulate(instr),
            0x09 => self.addiu_emulate(instr),
            0x0a => self.slti_emulate(instr),
            0x0b => self.sltiu_emulate(instr),
            0x0c => self.andi_emulate(instr),
            0x0d => self.ori_emulate(instr),
            0x0e => self.xori_emulate(instr),
            0x0f => self.lui_emulate(instr),
            0x10 => {
                // Handle CP0 instructions
                let rs = rs!(instr);

                if 15 < rs {
                    match funct!(instr) {
                        1 => self.cpzero.tlbr_emulate(instr),
                        2 => self.cpzero.tlbwi_emulate(instr),
                        6 => self.cpzero.tlbwr_emulate(instr),
                        8 => self.cpzero.tlbp_emulate(instr),
                        16 => self.cpzero.rfe_emulate(instr),
                        _ => self.exception(ExceptionCode::ReservedInstruction),
                    }
                } else {
                    match rs {
                        0 => self.mfc0_emulate(instr),
                        4 => self.mtc0_emulate(instr),
                        8 => self.cpzero.bc0x_emulate(instr, self.pc),
                        _ => self.exception(ExceptionCode::ReservedInstruction),
                    }
                }
            }
            0x11 => self.coprocessor_unimpl(1, instr, self.pc),
            0x12 => self.coprocessor_unimpl(2, instr, self.pc),
            0x13 => self.coprocessor_unimpl(3, instr, self.pc),
            0x20 => self.lb_emulate(instr)?,
            0x21 => self.lh_emulate(instr),
            0x22 => self.lwl_emulate(instr),
            0x23 => self.lw_emulate(instr)?,
            0x24 => self.lbu_emulate(instr)?,
            0x25 => self.lhu_emulate(instr),
            0x26 => self.lwr_emulate(instr),
            0x28 => self.sb_emulate(instr),
            0x29 => self.sh_emulate(instr),
            0x2a => self.swl_emulate(instr),
            0x2b => self.sw_emulate(instr),
            0x2e => self.swr_emulate(instr),
            0x31 => self.lwc1_emulate(instr),
            0x32 => self.lwc2_emulate(instr),
            0x33 => self.lwc3_emulate(instr),
            0x38 => self.swc1_emulate(instr),
            0x39 => self.swc2_emulate(instr),
            0x3a => self.swc3_emulate(instr),
            _ => self.ri_emulate(),
        }

        // Register $r0 is hardwired to a value of zero
        // It can be written to by instructions however the result is always discarded
        self.reg[REG_ZERO] = 0;

        // Update the program counter
        // DelayState tracks whether the current instruction should be executed from the delay slot
        match self.delay_state {
            // Increment the program counter by 4 for normal instructions
            DelayState::NORMAL => self.pc += 4,
            // The next instruction to be executed is in the delay slot
            DelayState::DELAYING => {
                self.pc += 4;
                self.delay_state = DelayState::DELAYSLOT;
            }
            // Current instruction was executed from the delay slot and the branch should now occur
            DelayState::DELAYSLOT => {
                self.pc = self.delay_pc;
                self.delay_state = DelayState::NORMAL;
            }
        }
        Ok(())
    }

    pub fn coprocessor_unimpl(&mut self, coprocno: u32, instr: u32, pc: u32) {
        if self.cpzero.coprocessor_usable(coprocno) {
            error!(
                "CP{} instruction {:x} is not implemented at PC={:08x}",
                coprocno, instr, pc
            )
        }

        self.exception(ExceptionCode::CoprocessorUnusable);
    }

    pub fn exception(&mut self, exception_code: ExceptionCode) {
        match exception_code {
            ExceptionCode::ReservedInstruction => warn!("Encountered a reserved instruction"),
            ExceptionCode::Overflow => {
                warn!("Arithmetic overflow occurred");
            }
            _ => {}
        }

        self.cpzero.enter_exception(self.pc, exception_code);
    }

    /// Prints the contents of the registers
    pub fn dump_regs(&self) {
        let next_epc = 0;
        print!(
            "Registers: [\nPC={:08x}  LastInstr={:08x}  HI={:08x}  LO={:08x}\nDelayState={:?}  DelayPC={:08x}  NextEPC={:08x}\n",
            self.pc, self.instruction, self.high, self.low, self.delay_state, self.delay_pc, next_epc
        );

        for i in 0..32 {
            print!(" R{:02}={:08x} ", i, self.reg[i]);

            if i % 5 == 4 {
                print!("\n");
            } else if i == 31 {
                print!("]\n");
            }
        }
    }
}
