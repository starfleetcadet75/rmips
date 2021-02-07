//! This module contains the helper functions that are used by the `Cpu` for executing instructions.

use crate::control::cpu::{Cpu, DelayState};
use crate::control::instruction::Instruction;
use crate::control::registers::Register;
use crate::control::ExceptionCode;
use crate::memory::Memory;
use crate::util::error::Result;
use crate::Address;

impl Cpu {
    /// Shift left logical
    pub fn sll_emulate(&mut self, instr: Instruction) {
        self.reg[instr.rd()] = self.reg[instr.rt()] << instr.shamt();
    }

    /// Logical shift right (zero-extended)
    pub fn srl_emulate(&mut self, instr: Instruction) {
        self.reg[instr.rd()] = self.srl(self.reg[instr.rt()], instr.shamt());
    }

    /// Shift right logical variable
    pub fn srlv_emulate(&mut self, instr: Instruction) {
        self.reg[instr.rd()] = self.srl(self.reg[instr.rt()], self.reg[instr.rs()] & 0x01f);
    }

    /// Arithmetic shift right (sign-extended)
    pub fn sra_emulate(&mut self, instr: Instruction) {
        self.reg[instr.rd()] = self.sra(self.reg[instr.rt()], instr.shamt());
    }

    /// Shift left logical variable
    pub fn sllv_emulate(&mut self, instr: Instruction) {
        self.reg[instr.rd()] = self.reg[instr.rt()] << (self.reg[instr.rs()] & 0x01f);
    }

    /// Signed right shift
    pub fn srav_emulate(&mut self, instr: Instruction) {
        self.reg[instr.rd()] = self.sra(self.reg[instr.rt()], self.reg[instr.rs()] & 0x01f);
    }

    /// Jump register
    pub fn jr_emulate(&mut self, instr: Instruction) {
        self.control_transfer(self.reg[instr.rs()]);
    }

    /// Jump and link register
    pub fn jalr_emulate(&mut self, instr: Instruction) {
        let target_address = self.reg[instr.rs()];
        self.control_transfer(target_address);

        self.reg[instr.rd()] = self.pc + 8;
    }

    /// System call
    pub fn syscall_emulate(&mut self) -> Result<()> {
        self.exception(ExceptionCode::Syscall)
    }

    /// Break
    pub fn break_emulate(&mut self) -> Result<()> {
        self.exception(ExceptionCode::Break)
    }

    /// Move from HI register
    pub fn mfhi_emulate(&mut self, instr: Instruction) {
        self.reg[instr.rd()] = self.high;
    }

    /// Move to HI register
    pub fn mthi_emulate(&mut self, instr: Instruction) {
        self.high = self.reg[instr.rs()];
    }

    /// Move from LO register
    pub fn mflo_emulate(&mut self, instr: Instruction) {
        self.reg[instr.rd()] = self.low;
    }

    /// Move to LO register
    pub fn mtlo_emulate(&mut self, instr: Instruction) {
        self.low = self.reg[instr.rs()];
    }

    /// Multiply word
    pub fn mult_emulate(&mut self, instr: Instruction) {
        let t = (self.reg[instr.rs()] as i64).wrapping_mul(self.reg[instr.rt()] as i64);
        self.low = t as u32;
        self.high = (t >> 32) as u32;
    }

    /// Multiply unsigned
    pub fn multu_emulate(&mut self, instr: Instruction) {
        let t = (self.reg[instr.rs()] as u64).wrapping_mul(self.reg[instr.rt()] as u64);
        self.low = t as u32;
        self.high = (t >> 32) as u32;
    }

    /// Divide word
    pub fn div_emulate(&mut self, instr: Instruction) {
        let rs = self.reg[instr.rs()] as i32;
        let rt = self.reg[instr.rt()] as i32;

        // According to the documentation the arithmetic result value is
        // unpredictable if the divisor in register rt is zero. For now
        // we follow MARS behavior and explicitly set rt/rs to zero.
        self.low = rs.checked_div(rt).unwrap_or(0) as u32;
        self.high = rs.checked_rem(rt).unwrap_or(0) as u32;
    }

    /// Divide unsigned word
    pub fn divu_emulate(&mut self, instr: Instruction) {
        let rs = self.reg[instr.rs()];
        let rt = self.reg[instr.rt()];
        self.low = rs.checked_div(rt).unwrap_or(0);
        self.high = rs.checked_rem(rt).unwrap_or(0);
    }

    /// Addition with overflow
    pub fn add_emulate(&mut self, instr: Instruction) -> Result<()> {
        let rs = self.reg[instr.rs()];
        let rt = self.reg[instr.rt()];
        let (result, carry) = rs.overflowing_add(rt);

        if carry {
            self.exception(ExceptionCode::Overflow)
        } else {
            self.reg[instr.rd()] = result;
            Ok(())
        }
    }

    /// Add unsigned without overflow
    pub fn addu_emulate(&mut self, instr: Instruction) {
        let rs = self.reg[instr.rs()];
        let rt = self.reg[instr.rt()];
        self.reg[instr.rd()] = rs.overflowing_add(rt).0;
    }

    /// Subtract with overflow
    pub fn sub_emulate(&mut self, instr: Instruction) -> Result<()> {
        let rs = self.reg[instr.rs()];
        let rt = self.reg[instr.rt()];
        let (result, carry) = rs.overflowing_sub(rt);

        if carry {
            self.exception(ExceptionCode::Overflow)
        } else {
            self.reg[instr.rd()] = result;
            Ok(())
        }
    }

    /// Subtract unsigned
    pub fn subu_emulate(&mut self, instr: Instruction) {
        let rs = self.reg[instr.rs()];
        let rt = self.reg[instr.rt()];
        self.reg[instr.rd()] = rs.overflowing_sub(rt).0;
    }

    /// Bitwise AND
    pub fn and_emulate(&mut self, instr: Instruction) {
        self.reg[instr.rd()] = self.reg[instr.rs()] & self.reg[instr.rt()];
    }

    /// Bitwise OR
    pub fn or_emulate(&mut self, instr: Instruction) {
        self.reg[instr.rd()] = self.reg[instr.rs()] | self.reg[instr.rt()];
    }

    /// Bitwise XOR
    pub fn xor_emulate(&mut self, instr: Instruction) {
        self.reg[instr.rd()] = self.reg[instr.rs()] ^ self.reg[instr.rt()];
    }

    /// Bitwise NOR
    pub fn nor_emulate(&mut self, instr: Instruction) {
        self.reg[instr.rd()] = !(self.reg[instr.rs()] | self.reg[instr.rt()]);
    }

    /// Set on less than (signed)
    pub fn slt_emulate(&mut self, instr: Instruction) {
        if (self.reg[instr.rs()] as i32) < self.reg[instr.rt()] as i32 {
            self.reg[instr.rd()] = 1;
        } else {
            self.reg[instr.rd()] = 0;
        }
    }

    /// Set on less than immediate (signed)
    pub fn slti_emulate(&mut self, instr: Instruction) {
        if (self.reg[instr.rs()] as i32) < instr.simmed() as i32 {
            self.reg[instr.rt()] = 1;
        } else {
            self.reg[instr.rt()] = 0;
        }
    }

    /// Set on less than immediate unsigned
    pub fn sltiu_emulate(&mut self, instr: Instruction) {
        if self.reg[instr.rs()] < instr.simmed() {
            self.reg[instr.rt()] = 1;
        } else {
            self.reg[instr.rt()] = 0;
        }
    }

    /// Set on less than unsigned
    pub fn sltu_emulate(&mut self, instr: Instruction) {
        if self.reg[instr.rs()] < self.reg[instr.rt()] {
            self.reg[instr.rd()] = 1;
        } else {
            self.reg[instr.rd()] = 0;
        }
    }

    /// Branch on less than zero
    pub fn bltz_emulate(&mut self, instr: Instruction) {
        if (self.reg[instr.rs()] as i32) < 0 {
            self.branch(instr);
        }
    }

    /// Branch on greater than or equal to zero
    pub fn bgez_emulate(&mut self, instr: Instruction) {
        if 0 <= (self.reg[instr.rs()] as i32) {
            self.branch(instr);
        }
    }

    /// Branch on less than zero and link
    pub fn bltzal_emulate(&mut self, instr: Instruction) {
        self.reg[Register::Ra] = self.pc + 8;

        if (self.reg[instr.rs()] as i32) < 0 {
            self.branch(instr);
        }
    }

    /// Branch on greater than or equal to zero and link
    pub fn bgezal_emulate(&mut self, instr: Instruction) {
        self.reg[Register::Ra] = self.pc + 8;

        if 0 <= (self.reg[instr.rs()] as i32) {
            self.branch(instr);
        }
    }

    /// Branch on equal
    pub fn beq_emulate(&mut self, instr: Instruction) {
        if self.reg[instr.rs()] == self.reg[instr.rt()] {
            self.branch(instr);
        }
    }

    /// Branch on not equal
    pub fn bne_emulate(&mut self, instr: Instruction) {
        if self.reg[instr.rs()] != self.reg[instr.rt()] {
            self.branch(instr);
        }
    }

    /// Branch on less than or equal to zero
    pub fn blez_emulate(&mut self, instr: Instruction) {
        // Sign bit is 1 or value is zero
        if self.reg[instr.rs()] == 0 || (self.reg[instr.rs()] & 0x80000000) != 0 {
            self.branch(instr);
        }
    }

    /// Branch on greater than zero
    pub fn bgtz_emulate(&mut self, instr: Instruction) {
        // Sign bit is 0 but value not zero
        if self.reg[instr.rs()] != 0 && (self.reg[instr.rs()] & 0x80000000) == 0 {
            self.branch(instr);
        }
    }

    /// Add immediate (with overflow)
    pub fn addi_emulate(&mut self, instr: Instruction) -> Result<()> {
        let rs = self.reg[instr.rs()];
        let imm = instr.simmed();
        let (result, carry) = rs.overflowing_add(imm);

        if carry {
            self.exception(ExceptionCode::Overflow)
        } else {
            self.reg[instr.rt()] = result;
            Ok(())
        }
    }

    /// Add immediate unsigned (no overflow)
    pub fn addiu_emulate(&mut self, instr: Instruction) {
        let rs = self.reg[instr.rs()];
        let imm = instr.simmed();
        self.reg[instr.rt()] = rs.overflowing_add(imm).0;
    }

    /// Bitwise AND immediate
    pub fn andi_emulate(&mut self, instr: Instruction) {
        self.reg[instr.rt()] = self.reg[instr.rs()] & instr.immed();
    }

    /// Bitwise OR immediate
    pub fn ori_emulate(&mut self, instr: Instruction) {
        self.reg[instr.rt()] = self.reg[instr.rs()] | instr.immed();
    }

    /// Bitwise XOR immediate
    pub fn xori_emulate(&mut self, instr: Instruction) {
        self.reg[instr.rt()] = self.reg[instr.rs()] ^ instr.immed();
    }

    /// Load upper immediate
    pub fn lui_emulate(&mut self, instr: Instruction) {
        self.reg[instr.rt()] = instr.immed() << 16;
    }

    /// Load byte
    pub fn lb_emulate(&mut self, memory: &mut impl Memory, instr: Instruction) -> Result<()> {
        let base = self.reg[instr.rs()];
        let offset = instr.simmed();
        let vaddress = base + offset;

        let paddress = self.cpzero.translate(vaddress);
        let data = memory.fetch_byte(paddress)? as i8; // Sign-extend the byte first
        self.reg[instr.rt()] = data as u32;
        Ok(())
    }

    /// Load halfword
    pub fn lh_emulate(&mut self, memory: &mut impl Memory, instr: Instruction) -> Result<()> {
        // Calculate the virtual address
        let base = self.reg[instr.rs()];
        let offset = instr.simmed();
        let vaddress = base + offset;

        // Check for a halfword-aligned address
        if vaddress % 2 != 0 {
            self.exception(ExceptionCode::LoadAddressError)
        } else {
            let paddress = self.cpzero.translate(vaddress);
            let data = memory.fetch_halfword(paddress)? as i16; // Sign-extend the word first
            self.reg[instr.rt()] = data as u32;
            Ok(())
        }
    }

    /// Load word left
    pub fn lwl_emulate(&mut self, _instr: Instruction) {
        todo!()
    }

    /// Load word
    pub fn lw_emulate(&mut self, memory: &mut impl Memory, instr: Instruction) -> Result<()> {
        // Calculate the virtual address
        let base = self.reg[instr.rs()];
        let offset = instr.simmed();
        let vaddress = base + offset;

        // If either of the two least-significant bits of the virtual address
        // are non-zero a load address exception occurs
        if vaddress % 4 != 0 {
            self.exception(ExceptionCode::LoadAddressError)
        } else {
            let paddress = self.cpzero.translate(vaddress);
            let data = memory.fetch_word(paddress)?;
            self.reg[instr.rt()] = data;
            Ok(())
        }
    }

    /// Load byte unsigned
    pub fn lbu_emulate(&mut self, memory: &mut impl Memory, instr: Instruction) -> Result<()> {
        let base = self.reg[instr.rs()];
        let offset = instr.simmed();
        let vaddress = base + offset;

        let paddress = self.cpzero.translate(vaddress);
        let data = memory.fetch_byte(paddress)?;
        self.reg[instr.rt()] = data.into(); // Zero-extend the byte
        Ok(())
    }

    /// Load halfword unsigned
    pub fn lhu_emulate(&mut self, memory: &mut impl Memory, instr: Instruction) -> Result<()> {
        let base = self.reg[instr.rs()];
        let offset = instr.simmed();
        let vaddress = base + offset;

        // Check for a halfword-aligned address
        if vaddress % 2 != 0 {
            self.exception(ExceptionCode::LoadAddressError)
        } else {
            let paddress = self.cpzero.translate(vaddress);
            let data = memory.fetch_halfword(paddress)?;
            self.reg[instr.rt()] = data.into();
            Ok(())
        }
    }

    /// Load word right
    pub fn lwr_emulate(&mut self, _instr: Instruction) {
        todo!()
    }

    /// Store byte
    pub fn sb_emulate(&mut self, memory: &mut impl Memory, instr: Instruction) -> Result<()> {
        let data = self.reg[instr.rt()] as u8;
        let base = self.reg[instr.rs()];
        let offset = instr.simmed();
        let vaddress = base + offset;
        let paddress = self.cpzero.translate(vaddress);
        memory.store_byte(paddress, data)
    }

    /// Store halfword
    pub fn sh_emulate(&mut self, memory: &mut impl Memory, instr: Instruction) -> Result<()> {
        let data = self.reg[instr.rt()] as u16;
        let base = self.reg[instr.rs()];
        let offset = instr.simmed();
        let vaddress = base + offset;

        // If the least-significant bit of the virtual address
        // is non-zero, a store address exception occurs
        if vaddress % 2 != 0 {
            self.exception(ExceptionCode::StoreAddressError)?;
        } else {
            let paddress = self.cpzero.translate(vaddress);
            memory.store_halfword(paddress, data)?;
        }
        Ok(())
    }

    /// Store word left
    pub fn swl_emulate(&mut self, _instr: Instruction) {
        todo!()
    }

    /// Store word
    pub fn sw_emulate(&mut self, memory: &mut impl Memory, instr: Instruction) -> Result<()> {
        let data = self.reg[instr.rt()];
        let base = self.reg[instr.rs()];
        let offset = instr.simmed();
        let vaddress = base + offset;

        // If either of the two least-significant bits of the virtual address
        // are non-zero, a store address exception occurs
        if vaddress % 4 != 0 {
            self.exception(ExceptionCode::StoreAddressError)?;
        } else {
            let paddress = self.cpzero.translate(vaddress);
            memory.store_word(paddress, data)?;
        }
        Ok(())
    }

    /// Store word right
    pub fn swr_emulate(&mut self, _instr: Instruction) {
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
        self.reg[Register::Ra] = self.pc + 8;
    }

    /// Move From System Control Coprocessor
    pub fn mfc0_emulate(&mut self, instr: Instruction) {
        self.reg[instr.rt()] = self.cpzero.reg[instr.rd()];
    }

    /// Move To System Control Coprocessor
    pub fn mtc0_emulate(&mut self, instr: Instruction) {
        self.cpzero.reg[instr.rd()] = self.reg[instr.rt()];
    }

    /// Reserved instruction
    pub fn ri_emulate(&mut self) -> Result<()> {
        self.exception(ExceptionCode::ReservedInstruction)
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
    use pretty_assertions::assert_eq;

    #[test]
    fn sll_emulate() {
        let mut cpu = Cpu::new(false);
        let instr = Instruction(0x00052140);
        cpu.reg[instr.rt()] = 42;
        cpu.sll_emulate(instr);
        assert_eq!(cpu.reg[instr.rd()], 0x540);
    }

    #[test]
    fn srl_emulate() {
        let mut cpu = Cpu::new(false);
        let instr = Instruction(0x00052142);
        cpu.reg[instr.rt()] = 42;
        cpu.srl_emulate(instr);
        assert_eq!(cpu.reg[instr.rd()], 1);
    }

    #[test]
    fn srlv_emulate() {
        let mut cpu = Cpu::new(false);
        let instr = Instruction(0x00a42006);
        cpu.reg[instr.rt()] = 0xffff;
        cpu.reg[instr.rs()] = 1;
        cpu.srlv_emulate(instr);
        assert_eq!(cpu.reg[instr.rd()], 0x7fff);
    }

    #[test]
    fn sra_emulate() {}

    #[test]
    fn sllv_emulate() {}

    #[test]
    fn srav_emulate() {}

    #[test]
    fn jr_emulate() {}

    #[test]
    fn jalr_emulate() {
        let mut cpu = Cpu::new(false);
        cpu.pc = 0xbfc019b0;

        let instr = Instruction(0x0040f809);
        cpu.reg[instr.rs()] = 0xbfc019b8;
        cpu.jalr_emulate(instr);

        assert_eq!(cpu.delay_pc, 0xbfc019b8);
        assert_eq!(cpu.delay_state, DelayState::Delaying);
    }

    #[test]
    fn syscall_emulate() {}

    #[test]
    fn break_emulate() {}

    #[test]
    fn mfhi_emulate() {
        let mut cpu = Cpu::new(false);
        let instr = Instruction(0x00004010);
        cpu.high = 0x4200ff00;
        cpu.mfhi_emulate(instr);
        assert_eq!(cpu.reg[instr.rd()], cpu.high);
    }

    #[test]
    fn mthi_emulate() {
        let mut cpu = Cpu::new(false);
        let instr = Instruction(0x01000011);
        cpu.reg[instr.rs()] = 0x4200ff00;
        cpu.mthi_emulate(instr);
        assert_eq!(cpu.reg[instr.rs()], cpu.high);
    }

    #[test]
    fn mflo_emulate() {
        let mut cpu = Cpu::new(false);
        let instr = Instruction(0x00004012);
        cpu.low = 0x4200ff00;
        cpu.mflo_emulate(instr);
        assert_eq!(cpu.reg[instr.rd()], cpu.low);
    }

    #[test]
    fn mtlo_emulate() {
        let mut cpu = Cpu::new(false);
        let instr = Instruction(0x01000013);
        cpu.reg[instr.rs()] = 0x4200ff00;
        cpu.mtlo_emulate(instr);
        assert_eq!(cpu.reg[instr.rs()], cpu.low);
    }

    #[test]
    fn mult_emulate() {
        let mut cpu = Cpu::new(false);
        let instr = Instruction(0x00850018);
        cpu.reg[instr.rt()] = 0xffffffff;
        cpu.reg[instr.rs()] = 0x7fffffff;
        cpu.mult_emulate(instr);
        assert_eq!(cpu.low, 0x80000001);
        assert_eq!(cpu.high, 0x7ffffffe);
    }

    #[test]
    fn multu_emulate() {
        // TODO: Verify this result vs mult
        let mut cpu = Cpu::new(false);
        let instr = Instruction(0x00850018);
        cpu.reg[instr.rt()] = 0xffffffff;
        cpu.reg[instr.rs()] = 0x7fffffff;
        cpu.multu_emulate(instr);
        assert_eq!(cpu.low, 0x80000001);
        assert_eq!(cpu.high, 0x7ffffffe);
    }

    #[test]
    fn div_emulate_mod() {
        let mut cpu = Cpu::new(false);
        let instr = Instruction(0x0109001a);

        cpu.reg[instr.rs()] = 2;
        cpu.reg[instr.rt()] = 5;
        cpu.div_emulate(instr);

        assert_eq!(cpu.low, 0);
        assert_eq!(cpu.high, 2);
    }

    #[test]
    fn div_emulate_divide() {
        let mut cpu = Cpu::new(false);
        let instr = Instruction(0x0109001a);

        cpu.reg[instr.rs()] = 5;
        cpu.reg[instr.rt()] = 2;
        cpu.div_emulate(instr);

        assert_eq!(cpu.low, 2);
        assert_eq!(cpu.high, 1);
    }

    #[test]
    fn div_emulate_divide_negative() {
        let mut cpu = Cpu::new(false);
        let instr = Instruction(0x0109001a);

        cpu.reg[instr.rs()] = -16 as i32 as u32;
        cpu.reg[instr.rt()] = 4;
        cpu.div_emulate(instr);

        assert_eq!(cpu.low, 0xfffffffc);
        assert_eq!(cpu.high, 0);
    }

    #[test]
    fn div_emulate_divide_by_zero() {
        let mut cpu = Cpu::new(false);
        let instr = Instruction(0x0109001a);

        cpu.reg[instr.rs()] = 5;
        cpu.reg[instr.rt()] = 0;
        cpu.div_emulate(instr);

        assert_eq!(cpu.low, 0);
        assert_eq!(cpu.high, 0);
    }

    #[test]
    fn divu_emulate_mod() {
        let mut cpu = Cpu::new(false);
        let instr = Instruction(0x0109001b);

        cpu.reg[instr.rs()] = 2;
        cpu.reg[instr.rt()] = 5;
        cpu.divu_emulate(instr);

        assert_eq!(cpu.low, 0);
        assert_eq!(cpu.high, 2);
    }

    #[test]
    fn divu_emulate_divide() {
        let mut cpu = Cpu::new(false);
        let instr = Instruction(0x0109001b);

        cpu.reg[instr.rs()] = 5;
        cpu.reg[instr.rt()] = 2;
        cpu.divu_emulate(instr);

        assert_eq!(cpu.low, 2);
        assert_eq!(cpu.high, 1);
    }

    #[test]
    fn divu_emulate_divide_negative() {
        let mut cpu = Cpu::new(false);
        let instr = Instruction(0x0109001b);

        cpu.reg[instr.rs()] = -16 as i32 as u32;
        cpu.reg[instr.rt()] = 4;
        cpu.divu_emulate(instr);

        assert_eq!(cpu.low, 0x3ffffffc);
        assert_eq!(cpu.high, 0);
    }

    #[test]
    fn divu_emulate_divide_by_zero() {
        let mut cpu = Cpu::new(false);
        let instr = Instruction(0x0109001b);

        cpu.reg[instr.rs()] = 5;
        cpu.reg[instr.rt()] = 0;
        cpu.divu_emulate(instr);

        assert_eq!(cpu.low, 0);
        assert_eq!(cpu.high, 0);
    }

    #[test]
    fn add_emulate() {}

    #[test]
    fn add_emulate_exception() {
        // TODO: Ensure an overflow exception is triggered by add on a carry
        // let mut cpu = Cpu::new(false);
        // let instr = Instruction(0x00a62020);
        // cpu.reg[instr.rt()] = 0xffff_0fff;
        // cpu.reg[instr.rs()] = 0x0001_0000;
        // cpu.add_emulate(instr);
        // assert_eq!(cpu.reg[instr.rd()], 0x0000_0fff);
    }

    #[test]
    fn addu_emulate() {
        let mut cpu = Cpu::new(false);
        let instr = Instruction(0x00a62021);
        cpu.reg[instr.rt()] = 0xffff_0fff;
        cpu.reg[instr.rs()] = 0x0001_0000;
        cpu.addu_emulate(instr);
        assert_eq!(cpu.reg[instr.rd()], 0x0000_0fff);
    }

    #[test]
    fn sub_emulate() -> Result<()> {
        let mut cpu = Cpu::new(false);
        let instr = Instruction(0x00a62022);
        cpu.reg[instr.rt()] = 40;
        cpu.reg[instr.rs()] = 42;
        cpu.sub_emulate(instr)?;
        assert_eq!(cpu.reg[instr.rd()], 2);
        Ok(())
    }

    #[test]
    fn sub_emulate_exception() {
        // TODO: Ensure an overflow exception is triggered by sub
    }

    #[test]
    fn subu_emulate() {
        let mut cpu = Cpu::new(false);
        let instr = Instruction(0x00a62023);
        cpu.reg[instr.rt()] = 1;
        cpu.reg[instr.rs()] = 0;
        cpu.subu_emulate(instr);
        assert_eq!(cpu.reg[instr.rd()], 0xffff_ffff);
    }

    #[test]
    fn and_emulate() {
        let mut cpu = Cpu::new(false);
        let instr = Instruction(0x00852024);
        cpu.reg[instr.rt()] = 42;
        cpu.reg[instr.rs()] = 13;
        cpu.and_emulate(instr);
        assert_eq!(cpu.reg[instr.rd()], 8);
    }

    #[test]
    fn or_emulate() {
        let mut cpu = Cpu::new(false);
        let instr = Instruction(0x00852024);
        cpu.reg[instr.rt()] = 42;
        cpu.reg[instr.rs()] = 13;
        cpu.or_emulate(instr);
        assert_eq!(cpu.reg[instr.rd()], 0x2f);
    }

    #[test]
    fn xor_emulate() {
        let mut cpu = Cpu::new(false);
        let instr = Instruction(0x00a62026);
        cpu.reg[instr.rt()] = 4242;
        cpu.reg[instr.rs()] = 88;
        cpu.xor_emulate(instr);
        assert_eq!(cpu.reg[instr.rd()], 0x10ca);
    }

    #[test]
    fn nor_emulate() {
        let mut cpu = Cpu::new(false);
        let instr = Instruction(0x00852027);
        cpu.reg[instr.rt()] = 42;
        cpu.reg[instr.rs()] = 13;
        cpu.nor_emulate(instr);
        assert_eq!(cpu.reg[instr.rd()], 0xffff_ffd0);
    }

    #[test]
    fn slt_emulate_less_than() {
        let mut cpu = Cpu::new(false);
        let instr = Instruction(0x014b482a);
        cpu.reg[instr.rt()] = 0;
        cpu.reg[instr.rs()] = -1 as i32 as u32;
        cpu.slt_emulate(instr);
        assert_eq!(cpu.reg[instr.rd()], 1);
    }

    #[test]
    fn slt_emulate_greater_than() {
        let mut cpu = Cpu::new(false);
        let instr = Instruction(0x014b482a);
        cpu.reg[instr.rt()] = 40;
        cpu.reg[instr.rs()] = 42;
        cpu.slt_emulate(instr);
        assert_eq!(cpu.reg[instr.rd()], 0);
    }

    #[test]
    fn slti_emulate_less_than() {
        let mut cpu = Cpu::new(false);
        let instr = Instruction(0x2949ff9c);
        cpu.reg[instr.rs()] = -128 as i32 as u32;
        cpu.slti_emulate(instr);
        assert_eq!(cpu.reg[instr.rt()], 1);
    }

    #[test]
    fn slti_emulate_greater_than() {
        let mut cpu = Cpu::new(false);
        let instr = Instruction(0x2949ff80);
        cpu.reg[instr.rs()] = -100 as i32 as u32;
        cpu.slti_emulate(instr);
        assert_eq!(cpu.reg[instr.rt()], 0);
    }

    #[test]
    fn sltiu_emulate_less_than() {
        let mut cpu = Cpu::new(false);
        let instr = Instruction(0x2d49ffff);
        cpu.reg[instr.rs()] = 10;
        cpu.sltiu_emulate(instr);
        assert_eq!(cpu.reg[instr.rt()], 1);
    }

    #[test]
    fn sltiu_emulate_greater_than() {
        let mut cpu = Cpu::new(false);
        let instr = Instruction(0x2949000a);
        cpu.reg[instr.rs()] = -1 as i32 as u32;
        cpu.sltiu_emulate(instr);
        assert_eq!(cpu.reg[instr.rt()], 0);
    }

    #[test]
    fn sltu_emulate_less_than() {
        let mut cpu = Cpu::new(false);
        let instr = Instruction(0x00400004);
        cpu.reg[instr.rt()] = 13;
        cpu.reg[instr.rs()] = 12;
        cpu.sltu_emulate(instr);
        assert_eq!(cpu.reg[instr.rd()], 1);
    }

    #[test]
    fn sltu_emulate_greater_than() {
        let mut cpu = Cpu::new(false);
        let instr = Instruction(0x00400004);
        cpu.reg[instr.rt()] = 0;
        cpu.reg[instr.rs()] = -1 as i32 as u32;
        cpu.sltu_emulate(instr);
        assert_eq!(cpu.reg[instr.rd()], 0);
    }

    #[test]
    fn bltz_emulate_taken() {
        let mut cpu = Cpu::new(false);
        cpu.pc = 0xbfc00004;

        let instr = Instruction(0x05200004);
        cpu.reg[instr.rs()] = -101 as i32 as u32;
        cpu.bltz_emulate(instr);

        assert_eq!(cpu.delay_pc, 0xbfc00018);
        assert_eq!(cpu.delay_state, DelayState::Delaying)
    }

    #[test]
    fn bltz_emulate_not_taken() {
        let mut cpu = Cpu::new(false);
        cpu.pc = 0xbfc00004;

        let instr = Instruction(0x05200004);
        cpu.reg[instr.rs()] = 101;
        cpu.bltz_emulate(instr);

        assert_eq!(cpu.delay_pc, 0);
        assert_eq!(cpu.delay_state, DelayState::Normal)
    }

    #[test]
    fn bgez_emulate_taken_zero() {
        let mut cpu = Cpu::new(false);
        cpu.pc = 0xbfc00004;

        let instr = Instruction(0x05210004);
        cpu.reg[instr.rs()] = 0;
        cpu.bgez_emulate(instr);

        assert_eq!(cpu.delay_pc, 0xbfc00018);
        assert_eq!(cpu.delay_state, DelayState::Delaying)
    }

    #[test]

    fn bgez_emulate_taken_greater_than_zero() {
        let mut cpu = Cpu::new(false);
        cpu.pc = 0xbfc00004;

        let instr = Instruction(0x05210004);
        cpu.reg[instr.rs()] = 100;
        cpu.bgez_emulate(instr);

        assert_eq!(cpu.delay_pc, 0xbfc00018);
        assert_eq!(cpu.delay_state, DelayState::Delaying)
    }

    #[test]

    fn bgez_emulate_not_taken() {
        let mut cpu = Cpu::new(false);
        cpu.pc = 0xbfc00004;

        let instr = Instruction(0x05210004);
        cpu.reg[instr.rs()] = -101 as i32 as u32;
        cpu.bgez_emulate(instr);

        assert_eq!(cpu.delay_pc, 0);
        assert_eq!(cpu.delay_state, DelayState::Normal)
    }

    #[test]
    fn bltzal_emulate_taken() {
        let mut cpu = Cpu::new(false);
        cpu.pc = 0xbfc00004;

        let instr = Instruction(0x05300004);
        cpu.reg[instr.rs()] = -101 as i32 as u32;
        cpu.bltzal_emulate(instr);

        assert_eq!(cpu.reg[Register::Ra], 0xbfc0000c);
        assert_eq!(cpu.delay_pc, 0xbfc00018);
        assert_eq!(cpu.delay_state, DelayState::Delaying)
    }

    #[test]

    fn bltzal_emulate_not_taken() {
        let mut cpu = Cpu::new(false);
        cpu.pc = 0xbfc00004;

        let instr = Instruction(0x05300004);
        cpu.reg[instr.rs()] = 0;
        cpu.bltzal_emulate(instr);

        assert_eq!(cpu.delay_pc, 0);
        assert_eq!(cpu.delay_state, DelayState::Normal)
    }

    #[test]
    fn bgezal_emulate_taken_zero() {
        let mut cpu = Cpu::new(false);
        cpu.pc = 0xbfc00004;

        let instr = Instruction(0x05310004);
        cpu.reg[instr.rs()] = 0;
        cpu.bgezal_emulate(instr);

        assert_eq!(cpu.reg[Register::Ra], 0xbfc0000c);
        assert_eq!(cpu.delay_pc, 0xbfc00018);
        assert_eq!(cpu.delay_state, DelayState::Delaying)
    }

    #[test]

    fn bgezal_emulate_taken_greater_than_zero() {
        let mut cpu = Cpu::new(false);
        cpu.pc = 0xbfc00004;

        let instr = Instruction(0x05310004);
        cpu.reg[instr.rs()] = 100;
        cpu.bgezal_emulate(instr);

        assert_eq!(cpu.reg[Register::Ra], 0xbfc0000c);
        assert_eq!(cpu.delay_pc, 0xbfc00018);
        assert_eq!(cpu.delay_state, DelayState::Delaying)
    }

    #[test]

    fn bgezal_emulate_not_taken() {
        let mut cpu = Cpu::new(false);
        cpu.pc = 0xbfc00004;

        let instr = Instruction(0x05310004);
        cpu.reg[instr.rs()] = -101 as i32 as u32;
        cpu.bgezal_emulate(instr);

        assert_eq!(cpu.delay_pc, 0);
        assert_eq!(cpu.delay_state, DelayState::Normal)
    }

    #[test]
    fn beq_emulate_taken() {
        let mut cpu = Cpu::new(false);
        cpu.pc = 0xbfc006ec;

        let instr = Instruction(0x10530005);
        cpu.reg[instr.rt()] = 42;
        cpu.reg[instr.rs()] = 42;
        cpu.beq_emulate(instr);

        assert_eq!(cpu.delay_pc, 0xbfc00704);
        assert_eq!(cpu.delay_state, DelayState::Delaying)
    }

    #[test]
    fn beq_emulate_not_taken() {
        let mut cpu = Cpu::new(false);
        cpu.pc = 0xbfc006ec;

        let instr = Instruction(0x10530005);
        cpu.reg[instr.rt()] = 24;
        cpu.reg[instr.rs()] = 42;
        cpu.beq_emulate(instr);

        assert_eq!(cpu.delay_pc, 0);
        assert_eq!(cpu.delay_state, DelayState::Normal)
    }

    #[test]
    fn bne_emulate_taken() {
        let mut cpu = Cpu::new(false);
        cpu.pc = 0xbfc006ec;

        let instr = Instruction(0x10530005);
        cpu.reg[instr.rt()] = 24;
        cpu.reg[instr.rs()] = 42;
        cpu.bne_emulate(instr);

        assert_eq!(cpu.delay_pc, 0xbfc00704);
        assert_eq!(cpu.delay_state, DelayState::Delaying)
    }

    #[test]
    fn bne_emulate_not_taken() {
        let mut cpu = Cpu::new(false);
        cpu.pc = 0xbfc006ec;

        let instr = Instruction(0x10530005);
        cpu.reg[instr.rt()] = 42;
        cpu.reg[instr.rs()] = 42;
        cpu.bne_emulate(instr);

        assert_eq!(cpu.delay_pc, 0);
        assert_eq!(cpu.delay_state, DelayState::Normal)
    }

    #[test]
    fn blez_emulate_taken() {
        let mut cpu = Cpu::new(false);
        cpu.pc = 0xbfc00004;

        let instr = Instruction(0x19200004);
        cpu.reg[instr.rs()] = 0;
        cpu.blez_emulate(instr);

        assert_eq!(cpu.delay_pc, 0xbfc00018);
        assert_eq!(cpu.delay_state, DelayState::Delaying)
    }

    #[test]
    fn blez_emulate_taken_less_than_zero() {
        let mut cpu = Cpu::new(false);
        cpu.pc = 0xbfc00004;

        let instr = Instruction(0x19200004);
        cpu.reg[instr.rs()] = -101 as i32 as u32;
        cpu.blez_emulate(instr);

        assert_eq!(cpu.delay_pc, 0xbfc00018);
        assert_eq!(cpu.delay_state, DelayState::Delaying)
    }

    #[test]
    fn blez_emulate_not_taken() {
        let mut cpu = Cpu::new(false);
        cpu.pc = 0xbfc00004;

        let instr = Instruction(0x19200004);
        cpu.reg[instr.rs()] = 100;
        cpu.blez_emulate(instr);

        assert_eq!(cpu.delay_pc, 0);
        assert_eq!(cpu.delay_state, DelayState::Normal)
    }

    #[test]
    fn bgtz_emulate_taken() {
        let mut cpu = Cpu::new(false);
        cpu.pc = 0xbfc00004;

        let instr = Instruction(0x1d200004);
        cpu.reg[instr.rs()] = 42;
        cpu.bgtz_emulate(instr);

        assert_eq!(cpu.delay_pc, 0xbfc00018);
        assert_eq!(cpu.delay_state, DelayState::Delaying)
    }

    #[test]
    fn bgtz_emulate_not_taken() {
        let mut cpu = Cpu::new(false);
        cpu.pc = 0xbfc00004;

        let instr = Instruction(0x1d200004);
        cpu.reg[instr.rs()] = 0;
        cpu.bgtz_emulate(instr);

        assert_eq!(cpu.delay_pc, 0);
        assert_eq!(cpu.delay_state, DelayState::Normal)
    }

    #[test]
    fn addi_emulate() -> Result<()> {
        let mut cpu = Cpu::new(false);
        let instr = Instruction(0x20840080);
        cpu.reg[instr.rs()] = 42;
        cpu.addi_emulate(instr)?;
        assert_eq!(cpu.reg[Register::A0], 0xaa);
        Ok(())
    }

    #[test]
    fn addi_emulate_exception() {
        // TODO: Ensure an overflow exception is triggered by add
    }

    #[test]
    fn addiu_emulate() {
        let mut cpu = Cpu::new(false);
        let instr = Instruction(0x248400ff);
        cpu.reg[instr.rs()] = 42;
        cpu.addiu_emulate(instr);
        assert_eq!(cpu.reg[Register::A0], 0x129);
    }

    #[test]
    fn andi_emulate() {
        let mut cpu = Cpu::new(false);
        let instr = Instruction(0x30a40fff);
        cpu.reg[instr.rs()] = 0x0110;
        cpu.andi_emulate(instr);
        assert_eq!(cpu.reg[Register::A0], 0x0110);
    }

    #[test]
    fn ori_emulate() {
        let mut cpu = Cpu::new(false);
        let instr = Instruction(0x34a41001);
        cpu.reg[instr.rs()] = 0x0110;
        cpu.ori_emulate(instr);
        assert_eq!(cpu.reg[Register::A0], 0x1111);
    }

    #[test]
    fn xori_emulate() {
        let mut cpu = Cpu::new(false);
        let instr = Instruction(0x38a44321);
        cpu.reg[instr.rs()] = 0x1234;
        cpu.xori_emulate(instr);
        assert_eq!(cpu.reg[Register::A0], 0x5115);
    }

    #[test]
    fn lui_emulate() {
        let mut cpu = Cpu::new(false);
        let instr = Instruction(0x3c040064);
        cpu.lui_emulate(instr);
        assert_eq!(cpu.reg[Register::A0], 0x0064_0000);
    }

    #[test]
    fn lb_emulate() {}

    #[test]
    fn lh_emulate() {}

    #[test]
    fn lwl_emulate() {}

    #[test]
    fn lw_emulate() {}

    #[test]
    fn lbu_emulate() {}

    #[test]
    fn lhu_emulate() {}

    #[test]
    fn lwr_emulate() {}

    #[test]
    fn sb_emulate() {}

    #[test]
    fn sh_emulate() {}

    #[test]
    fn swl_emulate() {}

    #[test]
    fn sw_emulate() {}

    #[test]
    fn swr_emulate() {}

    #[test]
    fn lwc1_emulate() {}

    #[test]
    fn lwc2_emulate() {}

    #[test]
    fn lwc3_emulate() {}

    #[test]
    fn swc1_emulate() {}

    #[test]
    fn swc2_emulate() {}

    #[test]
    fn swc3_emulate() {}

    #[test]
    fn j_emulate() {
        let mut cpu = Cpu::new(false);
        let instr = Instruction(0xbf00100);

        cpu.reset();
        cpu.j_emulate(instr);

        assert_eq!(cpu.delay_pc, 0xbfc00400);
        assert_eq!(cpu.delay_state, DelayState::Delaying)
    }

    #[test]
    fn jal_emulate() {
        let mut cpu = Cpu::new(false);
        let instr = Instruction(0x0c100006);

        cpu.pc = 0x00400000;
        cpu.jal_emulate(instr);

        assert_eq!(cpu.reg[Register::Ra], 0x00400008);
        assert_eq!(cpu.delay_pc, 0x00400018);
        assert_eq!(cpu.delay_state, DelayState::Delaying)
    }

    #[test]
    fn ri_emulate() {}
}
