//! This module contains the helper functions that are used by the `Cpu` for executing instructions.

use log::warn;

use crate::control::cpu::{Cpu, DelayState};
use crate::control::instruction::Instruction;
use crate::control::{ExceptionCode, REG_RA};
use crate::memory::Memory;
use crate::util::error::Result;
use crate::Address;

#[allow(unused_variables)]
impl Cpu {
    /// Shift left logical
    pub fn sll_emulate(&mut self, instr: Instruction) {
        self.reg[instr.rd() as usize] = self.reg[instr.rt() as usize] << instr.shamt();
    }

    /// Logical shift right (zero-extended)
    pub fn srl_emulate(&mut self, instr: Instruction) {
        self.reg[instr.rd() as usize] = self.srl(self.reg[instr.rt() as usize], instr.shamt());
    }

    /// Shift right logical variable
    pub fn srlv_emulate(&mut self, instr: Instruction) {
        self.reg[instr.rd() as usize] = self.srl(
            self.reg[instr.rt() as usize],
            self.reg[instr.rs() as usize] & 0x01f,
        );
    }

    /// Arithmetic shift right (sign-extended)
    pub fn sra_emulate(&mut self, instr: Instruction) {
        self.reg[instr.rd() as usize] = self.sra(self.reg[instr.rt() as usize], instr.shamt());
    }

    /// Shift left logical variable
    pub fn sllv_emulate(&mut self, instr: Instruction) {
        self.reg[instr.rd() as usize] =
            self.reg[instr.rt() as usize] << (self.reg[instr.rs() as usize] & 0x01f);
    }

    /// Signed right shift
    pub fn srav_emulate(&mut self, instr: Instruction) {
        self.reg[instr.rd() as usize] = self.sra(
            self.reg[instr.rt() as usize],
            self.reg[instr.rs() as usize] & 0x01f,
        );
    }

    /// Jump register
    pub fn jr_emulate(&mut self, instr: Instruction) {
        if self.reg[instr.rd() as usize] != 0 {
            self.exception(ExceptionCode::ReservedInstruction);
        } else {
            self.control_transfer(self.reg[instr.rs() as usize]);
        }
    }

    /// Jump and link register
    pub fn jalr_emulate(&mut self, instr: Instruction) {
        let target_address = self.reg[instr.rs() as usize];
        self.control_transfer(target_address);

        self.reg[instr.rd() as usize] = self.pc + 8;
    }

    /// System call
    pub fn syscall_emulate(&mut self) {
        self.exception(ExceptionCode::Syscall)
    }

    /// Break
    pub fn break_emulate(&mut self) {
        self.exception(ExceptionCode::Break)
    }

    /// Move from HI register
    pub fn mfhi_emulate(&mut self, instr: Instruction) {
        self.reg[instr.rd() as usize] = self.high;
    }

    /// Move to HI register
    pub fn mthi_emulate(&mut self, instr: Instruction) {
        if instr.rd() != 0 {
            self.exception(ExceptionCode::ReservedInstruction);
        } else {
            self.high = self.reg[instr.rs() as usize];
        }
    }

    /// Move from LO register
    pub fn mflo_emulate(&mut self, instr: Instruction) {
        self.reg[instr.rd() as usize] = self.low;
    }

    /// Move to LO register
    pub fn mtlo_emulate(&mut self, instr: Instruction) {
        if instr.rd() != 0 {
            self.exception(ExceptionCode::ReservedInstruction);
        } else {
            self.low = self.reg[instr.rs() as usize];
        }
    }

    /// Multiply
    pub fn mult_emulate(&mut self, instr: Instruction) {
        todo!()
    }

    /// Multiply unsigned
    pub fn multu_emulate(&mut self, instr: Instruction) {
        todo!()
    }

    /// Divide
    pub fn div_emulate(&mut self, instr: Instruction) {
        todo!()
    }

    /// Divide unsigned
    pub fn divu_emulate(&mut self, instr: Instruction) {
        todo!()
    }

    /// Addition with overflow
    pub fn add_emulate(&mut self, instr: Instruction) {
        let rs = self.reg[instr.rs() as usize];
        let rt = self.reg[instr.rt() as usize];
        let (result, carry) = rs.overflowing_add(rt);

        if carry {
            self.exception(ExceptionCode::Overflow);
        } else {
            self.reg[instr.rd() as usize] = result;
        }
    }

    /// Add unsigned without overflow
    pub fn addu_emulate(&mut self, instr: Instruction) {
        let rs = self.reg[instr.rs() as usize];
        let rt = self.reg[instr.rt() as usize];
        self.reg[instr.rd() as usize] = rs.overflowing_add(rt).0;
    }

    /// Subtract with overflow
    pub fn sub_emulate(&mut self, instr: Instruction) {
        let rs = self.reg[instr.rs() as usize];
        let rt = self.reg[instr.rt() as usize];
        let (result, carry) = rs.overflowing_sub(rt);

        if carry {
            self.exception(ExceptionCode::Overflow);
        } else {
            self.reg[instr.rd() as usize] = result;
        }
    }

    /// Subtract unsigned
    pub fn subu_emulate(&mut self, instr: Instruction) {
        let rs = self.reg[instr.rs() as usize];
        let rt = self.reg[instr.rt() as usize];
        self.reg[instr.rd() as usize] = rs.overflowing_sub(rt).0;
    }

    /// Bitwise AND
    pub fn and_emulate(&mut self, instr: Instruction) {
        self.reg[instr.rd() as usize] =
            self.reg[instr.rs() as usize] & self.reg[instr.rt() as usize];
    }

    /// Bitwise OR
    pub fn or_emulate(&mut self, instr: Instruction) {
        self.reg[instr.rd() as usize] =
            self.reg[instr.rs() as usize] | self.reg[instr.rt() as usize];
    }

    /// Bitwise XOR
    pub fn xor_emulate(&mut self, instr: Instruction) {
        self.reg[instr.rd() as usize] =
            self.reg[instr.rs() as usize] ^ self.reg[instr.rt() as usize];
    }

    /// Bitwise NOR
    pub fn nor_emulate(&mut self, instr: Instruction) {
        self.reg[instr.rd() as usize] =
            !(self.reg[instr.rs() as usize] | self.reg[instr.rt() as usize]);
    }

    /// Set on less than (signed)
    pub fn slt_emulate(&mut self, instr: Instruction) {
        todo!()
    }

    /// Set on less than immediate (signed)
    pub fn slti_emulate(&mut self, instr: Instruction) {
        todo!()
    }

    /// Set on less than immediate unsigned
    pub fn sltiu_emulate(&mut self, instr: Instruction) {
        todo!()
    }

    /// Set on less than unsigned
    pub fn sltu_emulate(&mut self, instr: Instruction) {
        todo!()
    }

    /// Branch on less than zero
    pub fn bltz_emulate(&mut self, instr: Instruction) {
        todo!()
    }

    /// Branch on greater than or equal to zero
    pub fn bgez_emulate(&mut self, instr: Instruction) {
        todo!()
    }

    /// Branch on less than zero and link
    pub fn bltzal_emulate(&mut self, instr: Instruction) {
        todo!()
    }

    /// Branch on greater than or equal to zero and link
    pub fn bgezal_emulate(&mut self, instr: Instruction) {
        todo!()
    }

    /// Branch on equal
    pub fn beq_emulate(&mut self, instr: Instruction) {
        if self.reg[instr.rs() as usize] == self.reg[instr.rt() as usize] {
            self.branch(instr);
        }
    }

    /// Branch on not equal
    pub fn bne_emulate(&mut self, instr: Instruction) {
        if self.reg[instr.rs() as usize] != self.reg[instr.rt() as usize] {
            self.branch(instr);
        }
    }

    /// Branch on less than or equal to zero
    pub fn blez_emulate(&mut self, instr: Instruction) {
        todo!()
    }

    /// Branch on greater than zero
    pub fn bgtz_emulate(&mut self, instr: Instruction) {
        todo!()
    }

    /// Add immediate (with overflow)
    pub fn addi_emulate(&mut self, instr: Instruction) {
        let rs = self.reg[instr.rs() as usize];
        let imm = instr.simmed();
        let (result, carry) = rs.overflowing_add(imm);

        if carry {
            self.exception(ExceptionCode::Overflow);
        } else {
            self.reg[instr.rt() as usize] = result;
        }
    }

    /// Add immediate unsigned (no overflow)
    pub fn addiu_emulate(&mut self, instr: Instruction) {
        let rs = self.reg[instr.rs() as usize];
        let imm = instr.simmed();
        self.reg[instr.rt() as usize] = rs.overflowing_add(imm).0;
    }

    /// Bitwise AND immediate
    pub fn andi_emulate(&mut self, instr: Instruction) {
        self.reg[instr.rt() as usize] = self.reg[instr.rs() as usize] & instr.immed();
    }

    /// Bitwise OR immediate
    pub fn ori_emulate(&mut self, instr: Instruction) {
        self.reg[instr.rt() as usize] = self.reg[instr.rs() as usize] | instr.immed();
    }

    /// Bitwise XOR immediate
    pub fn xori_emulate(&mut self, instr: Instruction) {
        self.reg[instr.rt() as usize] = self.reg[instr.rs() as usize] ^ instr.immed();
    }

    /// Load upper immediate
    pub fn lui_emulate(&mut self, instr: Instruction) {
        self.reg[instr.rt() as usize] = instr.immed() << 16;
    }

    /// Load byte
    pub fn lb_emulate(&mut self, memory: &mut impl Memory, instr: Instruction) -> Result<()> {
        let base = self.reg[instr.rs() as usize];
        let offset = instr.simmed();
        let vaddress = base + offset;

        let paddress = self.cpzero.translate(vaddress);
        let data = memory.fetch_byte(paddress)?;
        self.reg[instr.rt() as usize] = data as i32 as u32; // Sign-extend the byte first

        Ok(())
    }

    /// Load halfword
    pub fn lh_emulate(&mut self, instr: Instruction) {
        todo!()
    }

    /// Load word left
    pub fn lwl_emulate(&mut self, instr: Instruction) {
        todo!()
    }

    /// Load word
    pub fn lw_emulate(&mut self, memory: &mut impl Memory, instr: Instruction) -> Result<()> {
        // Calculate the virtual address
        let base = self.reg[instr.rs() as usize];
        let offset = instr.simmed();
        let vaddress = base + offset;

        // If either of the two least-significant bits of the virtual address
        // are non-zero, a load address exception occurs
        if vaddress % 4 != 0 {
            self.exception(ExceptionCode::LoadAddressError);
        } else {
            let paddress = self.cpzero.translate(vaddress);
            let data = memory.fetch_word(paddress)?;
            self.reg[instr.rt() as usize] = data;
        }
        Ok(())
    }

    /// Load byte unsigned
    pub fn lbu_emulate(&mut self, memory: &mut impl Memory, instr: Instruction) -> Result<()> {
        let base = self.reg[instr.rs() as usize];
        let offset = instr.simmed();
        let vaddress = base + offset;

        let paddress = self.cpzero.translate(vaddress);
        let data = memory.fetch_byte(paddress)?;
        self.reg[instr.rt() as usize] = data.into(); // Zero-extend the byte
        Ok(())
    }

    /// Load halfword unsigned
    pub fn lhu_emulate(&mut self, instr: Instruction) {
        todo!()
    }

    /// Load word right
    pub fn lwr_emulate(&mut self, instr: Instruction) {
        todo!()
    }

    /// Store byte
    pub fn sb_emulate(&mut self, memory: &mut impl Memory, instr: Instruction) -> Result<()> {
        let data = self.reg[instr.rt() as usize] as u8;
        let base = self.reg[instr.rs() as usize];
        let offset = instr.simmed();
        let vaddress = base + offset;
        let paddress = self.cpzero.translate(vaddress);
        memory.store_byte(paddress, data)
    }

    /// Store halfword
    pub fn sh_emulate(&mut self, memory: &mut impl Memory, instr: Instruction) -> Result<()> {
        let data = self.reg[instr.rt() as usize] as u16;
        let base = self.reg[instr.rs() as usize];
        let offset = instr.simmed();
        let vaddress = base + offset;

        // If the least-significant bit of the virtual address
        // is non-zero, a store address exception occurs
        if vaddress % 2 != 0 {
            self.exception(ExceptionCode::StoreAddressError);
        } else {
            let paddress = self.cpzero.translate(vaddress);
            memory.store_halfword(paddress, data)?;
        }
        Ok(())
    }

    /// Store word left
    pub fn swl_emulate(&mut self, instr: Instruction) {
        todo!()
    }

    /// Store word
    pub fn sw_emulate(&mut self, memory: &mut impl Memory, instr: Instruction) -> Result<()> {
        let data = self.reg[instr.rt() as usize];
        let base = self.reg[instr.rs() as usize];
        let offset = instr.simmed();
        let vaddress = base + offset;

        // If either of the two least-significant bits of the virtual address
        // are non-zero, a store address exception occurs
        if vaddress % 4 != 0 {
            self.exception(ExceptionCode::StoreAddressError);
        } else {
            let paddress = self.cpzero.translate(vaddress);
            memory.store_word(paddress, data)?;
        }
        Ok(())
    }

    /// Store word right
    pub fn swr_emulate(&mut self, instr: Instruction) {
        todo!()
    }

    /// Load word from CP1
    pub fn lwc1_emulate(&mut self, instr: Instruction) {
        self.coprocessor_unimpl(1, instr);
    }

    /// Load word from CP2
    pub fn lwc2_emulate(&mut self, instr: Instruction) {
        self.coprocessor_unimpl(1, instr);
    }

    /// Load word from CP3
    pub fn lwc3_emulate(&mut self, instr: Instruction) {
        self.coprocessor_unimpl(1, instr);
    }

    /// Store word from CP1
    pub fn swc1_emulate(&mut self, instr: Instruction) {
        self.coprocessor_unimpl(1, instr);
    }

    /// Store word from CP2
    pub fn swc2_emulate(&mut self, instr: Instruction) {
        self.coprocessor_unimpl(2, instr);
    }

    /// Store word from CP3
    pub fn swc3_emulate(&mut self, instr: Instruction) {
        self.coprocessor_unimpl(3, instr);
    }

    /// Jump
    pub fn j_emulate(&mut self, instr: Instruction) {
        self.jump(instr);
    }

    /// Jump and link
    pub fn jal_emulate(&mut self, instr: Instruction) {
        self.jump(instr);

        // Store the address of the instruction after the delay slot in the return address register
        self.reg[REG_RA] = self.pc + 8;
    }

    /// Move From System Control Coprocessor
    pub fn mfc0_emulate(&mut self, instr: Instruction) {
        self.reg[instr.rt() as usize] = self.cpzero.reg[instr.rd() as usize];
    }

    /// Move To System Control Coprocessor
    pub fn mtc0_emulate(&mut self, instr: Instruction) {
        self.cpzero.reg[instr.rd() as usize] = self.reg[instr.rt() as usize];
    }

    /// Reserved instruction
    pub fn ri_emulate(&mut self, instr: Instruction) {
        warn!("Encountered a reserved instruction:\n{:?}", instr);
        self.exception(ExceptionCode::ReservedInstruction);
    }

    fn srl(&self, a: u32, b: u32) -> u32 {
        if b == 0 {
            a
        } else if b == 32 {
            0
        } else {
            (a >> b) & ((1 << (32 - b)) - 1)
        }
    }

    fn sra(&self, a: u32, b: u32) -> u32 {
        if b == 0 {
            a
        } else {
            (a >> b) | (((a >> 31) & 0x01) * (((1 << b) - 1) << (32 - b)))
        }
    }

    fn branch(&mut self, instr: Instruction) {
        // Calculate the target address for the PC-relative branch
        let offset = instr.simmed() << 2;
        let target_address = (self.pc + 4).wrapping_add(offset);
        self.control_transfer(target_address);
    }

    fn jump(&mut self, instr: Instruction) {
        // Calculate the address to jump to as the result of a J-format instruction
        let target_address = ((self.pc + 4) & 0xf000_0000) | (instr.jumptarget() << 2);
        self.control_transfer(target_address);
    }

    fn control_transfer(&mut self, dest: Address) {
        self.delay_state = DelayState::Delaying;
        self.delay_pc = dest;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::control::REG_A0;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_sll_emulate() {
        let mut cpu = Cpu::new(false);
        let instr = Instruction(0x00052140);
        cpu.reg[instr.rt() as usize] = 42;
        cpu.sll_emulate(instr);
        assert_eq!(cpu.reg[instr.rd() as usize], 0x540);
    }

    #[test]
    fn test_srl_emulate() {
        let mut cpu = Cpu::new(false);
        let instr = Instruction(0x00052142);
        cpu.reg[instr.rt() as usize] = 42;
        cpu.srl_emulate(instr);
        assert_eq!(cpu.reg[instr.rd() as usize], 1);
    }

    #[test]
    fn test_srlv_emulate() {
        let mut cpu = Cpu::new(false);
        let instr = Instruction(0x00a42006);
        cpu.reg[instr.rt() as usize] = 0xffff;
        cpu.reg[instr.rs() as usize] = 1;
        cpu.srlv_emulate(instr);
        assert_eq!(cpu.reg[instr.rd() as usize], 0x7fff);
    }

    #[test]
    fn test_sra_emulate() {}

    #[test]
    fn test_sllv_emulate() {}

    #[test]
    fn test_srav_emulate() {}

    #[test]
    fn test_jr_emulate() {}

    #[test]
    fn test_jalr_emulate() {
        let mut cpu = Cpu::new(false);
        cpu.pc = 0xbfc019b0;

        let instr = Instruction(0x0040f809);
        cpu.reg[instr.rs() as usize] = 0xbfc019b8;
        cpu.jalr_emulate(instr);

        assert_eq!(cpu.delay_pc, 0xbfc019b8);
        assert_eq!(cpu.delay_state, DelayState::Delaying)
    }

    #[test]
    fn test_syscall_emulate() {}

    #[test]
    fn test_break_emulate() {}

    #[test]
    fn test_mfhi_emulate() {}

    #[test]
    fn test_mthi_emulate() {}

    #[test]
    fn test_mflo_emulate() {}

    #[test]
    fn test_mtlo_emulate() {}

    #[test]
    fn test_mult_emulate() {}

    #[test]
    fn test_multu_emulate() {}

    #[test]
    fn test_div_emulate() {}

    #[test]
    fn test_divu_emulate() {}

    #[test]
    fn test_add_emulate() {}

    #[test]
    fn test_add_emulate_exception() {
        // TODO: Ensure an overflow exception is triggered by add on a carry
        // let mut cpu = Cpu::new(false);
        // let instr = Instruction(0x00a62020);
        // cpu.reg[instr.rt() as usize] = 0xffff_0fff;
        // cpu.reg[instr.rs() as usize] = 0x0001_0000;
        // cpu.add_emulate(instr);
        // assert_eq!(cpu.reg[instr.rd() as usize], 0x0000_0fff);
    }

    #[test]
    fn test_addu_emulate() {
        let mut cpu = Cpu::new(false);
        let instr = Instruction(0x00a62021);
        cpu.reg[instr.rt() as usize] = 0xffff_0fff;
        cpu.reg[instr.rs() as usize] = 0x0001_0000;
        cpu.addu_emulate(instr);
        assert_eq!(cpu.reg[instr.rd() as usize], 0x0000_0fff);
    }

    #[test]
    fn test_sub_emulate() {
        let mut cpu = Cpu::new(false);
        let instr = Instruction(0x00a62022);
        cpu.reg[instr.rt() as usize] = 40;
        cpu.reg[instr.rs() as usize] = 42;
        cpu.sub_emulate(instr);
        assert_eq!(cpu.reg[instr.rd() as usize], 2);
    }

    #[test]
    fn test_sub_emulate_exception() {
        // TODO: Ensure an overflow exception is triggered by sub
    }

    #[test]
    fn test_subu_emulate() {
        let mut cpu = Cpu::new(false);
        let instr = Instruction(0x00a62023);
        cpu.reg[instr.rt() as usize] = 1;
        cpu.reg[instr.rs() as usize] = 0;
        cpu.subu_emulate(instr);
        assert_eq!(cpu.reg[instr.rd() as usize], 0xffff_ffff);
    }

    #[test]
    fn test_and_emulate() {
        let mut cpu = Cpu::new(false);
        let instr = Instruction(0x00852024);
        cpu.reg[instr.rt() as usize] = 42;
        cpu.reg[instr.rs() as usize] = 13;
        cpu.and_emulate(instr);
        assert_eq!(cpu.reg[instr.rd() as usize], 8);
    }

    #[test]
    fn test_or_emulate() {
        let mut cpu = Cpu::new(false);
        let instr = Instruction(0x00852024);
        cpu.reg[instr.rt() as usize] = 42;
        cpu.reg[instr.rs() as usize] = 13;
        cpu.or_emulate(instr);
        assert_eq!(cpu.reg[instr.rd() as usize], 0x2f);
    }

    #[test]
    fn test_xor_emulate() {
        let mut cpu = Cpu::new(false);
        let instr = Instruction(0x00a62026);
        cpu.reg[instr.rt() as usize] = 4242;
        cpu.reg[instr.rs() as usize] = 88;
        cpu.xor_emulate(instr);
        assert_eq!(cpu.reg[instr.rd() as usize], 0x10ca);
    }

    #[test]
    fn test_nor_emulate() {
        let mut cpu = Cpu::new(false);
        let instr = Instruction(0x00852027);
        cpu.reg[instr.rt() as usize] = 42;
        cpu.reg[instr.rs() as usize] = 13;
        cpu.nor_emulate(instr);
        assert_eq!(cpu.reg[instr.rd() as usize], 0xffff_ffd0);
    }

    #[test]
    fn test_slt_emulate() {}

    #[test]
    fn test_slti_emulate() {}

    #[test]
    fn test_sltiu_emulate() {}

    #[test]
    fn test_sltu_emulate() {}

    #[test]
    fn test_bltz_emulate() {}

    #[test]
    fn test_bgez_emulate() {}

    #[test]
    fn test_bltzal_emulate() {}

    #[test]
    fn test_bgezal_emulate() {}

    #[test]
    fn test_beq_emulate_branches() {
        let mut cpu = Cpu::new(false);
        cpu.pc = 0xbfc006ec;

        let instr = Instruction(0x10530005);
        cpu.reg[instr.rt() as usize] = 42;
        cpu.reg[instr.rs() as usize] = 42;
        cpu.beq_emulate(instr);

        assert_eq!(cpu.delay_pc, 0xbfc00704);
        assert_eq!(cpu.delay_state, DelayState::Delaying)
    }

    #[test]
    fn test_beq_emulate_not_branches() {
        let mut cpu = Cpu::new(false);
        cpu.pc = 0xbfc006ec;

        let instr = Instruction(0x10530005);
        cpu.reg[instr.rt() as usize] = 24;
        cpu.reg[instr.rs() as usize] = 42;
        cpu.beq_emulate(instr);

        assert_eq!(cpu.delay_pc, 0);
        assert_eq!(cpu.delay_state, DelayState::Normal)
    }

    #[test]
    fn test_bne_emulate_branches() {
        let mut cpu = Cpu::new(false);
        cpu.pc = 0xbfc006ec;

        let instr = Instruction(0x10530005);
        cpu.reg[instr.rt() as usize] = 24;
        cpu.reg[instr.rs() as usize] = 42;
        cpu.bne_emulate(instr);

        assert_eq!(cpu.delay_pc, 0xbfc00704);
        assert_eq!(cpu.delay_state, DelayState::Delaying)
    }

    #[test]
    fn test_bne_emulate_not_branches() {
        let mut cpu = Cpu::new(false);
        cpu.pc = 0xbfc006ec;

        let instr = Instruction(0x10530005);
        cpu.reg[instr.rt() as usize] = 42;
        cpu.reg[instr.rs() as usize] = 42;
        cpu.bne_emulate(instr);

        assert_eq!(cpu.delay_pc, 0);
        assert_eq!(cpu.delay_state, DelayState::Normal)
    }

    #[test]
    fn test_blez_emulate() {}

    #[test]
    fn test_bgtz_emulate() {}

    #[test]
    fn test_addi_emulate() {
        let mut cpu = Cpu::new(false);
        let instr = Instruction(0x20840080);
        cpu.reg[instr.rs() as usize] = 42;
        cpu.addi_emulate(instr);
        assert_eq!(cpu.reg[REG_A0], 0xaa);
    }

    #[test]
    fn test_addi_emulate_exception() {
        // TODO: Ensure an overflow exception is triggered by add
    }

    #[test]
    fn test_addiu_emulate() {
        let mut cpu = Cpu::new(false);
        let instr = Instruction(0x248400ff);
        cpu.reg[instr.rs() as usize] = 42;
        cpu.addiu_emulate(instr);
        assert_eq!(cpu.reg[REG_A0], 0x129);
    }

    #[test]
    fn test_andi_emulate() {
        let mut cpu = Cpu::new(false);
        let instr = Instruction(0x30a40fff);
        cpu.reg[instr.rs() as usize] = 0x0110;
        cpu.andi_emulate(instr);
        assert_eq!(cpu.reg[REG_A0], 0x0110);
    }

    #[test]
    fn test_ori_emulate() {
        let mut cpu = Cpu::new(false);
        let instr = Instruction(0x34a41001);
        cpu.reg[instr.rs() as usize] = 0x0110;
        cpu.ori_emulate(instr);
        assert_eq!(cpu.reg[REG_A0], 0x1111);
    }

    #[test]
    fn test_xori_emulate() {
        let mut cpu = Cpu::new(false);
        let instr = Instruction(0x38a44321);
        cpu.reg[instr.rs() as usize] = 0x1234;
        cpu.xori_emulate(instr);
        assert_eq!(cpu.reg[REG_A0], 0x5115);
    }

    #[test]
    fn test_lui_emulate() {
        let mut cpu = Cpu::new(false);
        let instr = Instruction(0x3c040064);
        cpu.lui_emulate(instr);
        assert_eq!(cpu.reg[REG_A0], 0x0064_0000);
    }

    #[test]
    fn test_lb_emulate() {}

    #[test]
    fn test_lh_emulate() {}

    #[test]
    fn test_lwl_emulate() {}

    #[test]
    fn test_lw_emulate() {}

    #[test]
    fn test_lbu_emulate() {}

    #[test]
    fn test_lhu_emulate() {}

    #[test]
    fn test_lwr_emulate() {}

    #[test]
    fn test_sb_emulate() {}

    #[test]
    fn test_sh_emulate() {}

    #[test]
    fn test_swl_emulate() {}

    #[test]
    fn test_sw_emulate() {}

    #[test]
    fn test_swr_emulate() {}

    #[test]
    fn test_lwc1_emulate() {}

    #[test]
    fn test_lwc2_emulate() {}

    #[test]
    fn test_lwc3_emulate() {}

    #[test]
    fn test_swc1_emulate() {}

    #[test]
    fn test_swc2_emulate() {}

    #[test]
    fn test_swc3_emulate() {}

    #[test]
    fn test_j_emulate() {
        let mut cpu = Cpu::new(false);
        let instr = Instruction(0xbf00100);

        cpu.reset();
        cpu.j_emulate(instr);

        assert_eq!(cpu.delay_pc, 0xbfc00400);
        assert_eq!(cpu.delay_state, DelayState::Delaying)
    }

    #[test]
    fn test_jal_emulate() {}

    #[test]
    fn test_ri_emulate() {}
}
