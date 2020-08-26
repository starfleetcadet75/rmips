//! This module contains the emulation helper functions that are used by the `CPU` for executing instructions.
use crate::cpu::cpu::{DelayState, CPU};
use crate::cpu::{ExceptionCode, REG_RA};
use crate::memory::Memory;
use crate::util::error::RmipsError;

#[allow(unused_variables)]
impl CPU {
    /// Shift left logical
    pub fn sll_emulate(&mut self, instr: u32) {
        self.reg[rd!(instr)] = self.reg[rt!(instr)] << shamt!(instr);
    }

    /// Logical shift right (zero-extended)
    pub fn srl_emulate(&mut self, instr: u32) {
        self.reg[rd!(instr)] = self.srl(self.reg[rt!(instr)], shamt!(instr));
    }

    /// Shift right logical variable
    pub fn srlv_emulate(&mut self, instr: u32) {
        self.reg[rd!(instr)] = self.srl(self.reg[rt!(instr)], self.reg[rs!(instr)] & 0x01f);
    }

    /// Arithmetic shift right (sign-extended)
    pub fn sra_emulate(&mut self, instr: u32) {
        self.reg[rd!(instr)] = self.sra(self.reg[rt!(instr)], shamt!(instr));
    }

    /// Shift left logical variable
    pub fn sllv_emulate(&mut self, instr: u32) {
        self.reg[rd!(instr)] = self.reg[rt!(instr)] << (self.reg[rs!(instr)] & 0x01f);
    }

    /// Signed right shift
    pub fn srav_emulate(&mut self, instr: u32) {
        self.reg[rd!(instr)] = self.sra(self.reg[rt!(instr)], self.reg[rs!(instr)] & 0x01f);
    }

    /// Jump register
    pub fn jr_emulate(&mut self, instr: u32) {
        if self.reg[rd!(instr)] != 0 {
            self.exception(ExceptionCode::ReservedInstruction);
        } else {
            self.control_transfer(self.reg[rs!(instr)]);
        }
    }

    /// Jump and link register
    pub fn jalr_emulate(&mut self, instr: u32, pc: u32) {
        let target_address = self.reg[rs!(instr)];
        self.control_transfer(target_address);

        self.reg[rd![instr]] = pc + 8;
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
    pub fn mfhi_emulate(&mut self, instr: u32) {
        self.reg[rd!(instr)] = self.high;
    }

    /// Move to HI register
    pub fn mthi_emulate(&mut self, instr: u32) {
        if rd!(instr) != 0 {
            self.exception(ExceptionCode::ReservedInstruction);
        } else {
            self.high = self.reg[rs!(instr)];
        }
    }

    /// Move from LO register
    pub fn mflo_emulate(&mut self, instr: u32) {
        self.reg[rd!(instr)] = self.low;
    }

    /// Move to LO register
    pub fn mtlo_emulate(&mut self, instr: u32) {
        if rd!(instr) != 0 {
            self.exception(ExceptionCode::ReservedInstruction);
        } else {
            self.low = self.reg[rs!(instr)];
        }
    }

    /// Multiply
    pub fn mult_emulate(&mut self, instr: u32) {
        todo!()
    }

    /// Multiply unsigned
    pub fn multu_emulate(&mut self, instr: u32) {
        todo!()
    }

    /// Divide
    pub fn div_emulate(&mut self, instr: u32) {
        todo!()
    }

    /// Divide unsigned
    pub fn divu_emulate(&mut self, instr: u32) {
        todo!()
    }

    /// Addition with overflow
    pub fn add_emulate(&mut self, instr: u32) {
        let rs = self.reg[rs!(instr)];
        let rt = self.reg[rt!(instr)];
        let (result, carry) = rs.overflowing_add(rt);

        if carry {
            self.exception(ExceptionCode::Overflow);
        } else {
            self.reg[rd!(instr)] = result;
        }
    }

    /// Add unsigned without overflow
    pub fn addu_emulate(&mut self, instr: u32) {
        let rs = self.reg[rs!(instr)];
        let rt = self.reg[rt!(instr)];
        self.reg[rd!(instr)] = rs.overflowing_add(rt).0;
    }

    /// Subtract with overflow
    pub fn sub_emulate(&mut self, instr: u32) {
        let rs = self.reg[rs!(instr)];
        let rt = self.reg[rt!(instr)];
        let (result, carry) = rs.overflowing_sub(rt);

        if carry {
            self.exception(ExceptionCode::Overflow);
        } else {
            self.reg[rd!(instr)] = result;
        }
    }

    /// Subtract unsigned
    pub fn subu_emulate(&mut self, instr: u32) {
        let rs = self.reg[rs!(instr)];
        let rt = self.reg[rt!(instr)];
        self.reg[rd!(instr)] = rs.overflowing_sub(rt).0;
    }

    /// Bitwise AND
    pub fn and_emulate(&mut self, instr: u32) {
        self.reg[rd!(instr)] = self.reg[rs!(instr)] & self.reg[rt!(instr)];
    }

    /// Bitwise OR
    pub fn or_emulate(&mut self, instr: u32) {
        self.reg[rd!(instr)] = self.reg[rs!(instr)] | self.reg[rt!(instr)];
    }

    /// Bitwise XOR
    pub fn xor_emulate(&mut self, instr: u32) {
        self.reg[rd!(instr)] = self.reg[rs!(instr)] ^ self.reg[rt!(instr)];
    }

    /// Bitwise NOR
    pub fn nor_emulate(&mut self, instr: u32) {
        self.reg[rd!(instr)] = !(self.reg[rs!(instr)] | self.reg[rt!(instr)]);
    }

    /// Set on less than (signed)
    pub fn slt_emulate(&mut self, instr: u32) {
        todo!()
    }

    /// Set on less than immediate (signed)
    pub fn slti_emulate(&mut self, instr: u32) {
        todo!()
    }

    /// Set on less than immediate unsigned
    pub fn sltiu_emulate(&mut self, instr: u32) {
        todo!()
    }

    /// Set on less than unsigned
    pub fn sltu_emulate(&mut self, instr: u32) {
        todo!()
    }

    /// Branch on less than zero
    pub fn bltz_emulate(&mut self, instr: u32) {
        todo!()
    }

    /// Branch on greater than or equal to zero
    pub fn bgez_emulate(&mut self, instr: u32) {
        todo!()
    }

    /// Branch on less than zero and link
    pub fn bltzal_emulate(&mut self, instr: u32) {
        todo!()
    }

    /// Branch on greater than or equal to zero and link
    pub fn bgezal_emulate(&mut self, instr: u32) {
        todo!()
    }

    /// Branch on equal
    pub fn beq_emulate(&mut self, instr: u32, pc: u32) {
        if self.reg[rs!(instr)] == self.reg[rt!(instr)] {
            self.branch(instr, pc);
        }
    }

    /// Branch on not equal
    pub fn bne_emulate(&mut self, instr: u32, pc: u32) {
        if self.reg[rs!(instr)] != self.reg[rt!(instr)] {
            self.branch(instr, pc);
        }
    }

    /// Branch on less than or equal to zero
    pub fn blez_emulate(&mut self, instr: u32, pc: u32) {
        todo!()
    }

    /// Branch on greater than zero
    pub fn bgtz_emulate(&mut self, instr: u32, pc: u32) {
        todo!()
    }

    /// Add immediate (with overflow)
    pub fn addi_emulate(&mut self, instr: u32) {
        let rs = self.reg[rs!(instr)];
        let imm = simmed!(instr);
        let (result, carry) = rs.overflowing_add(imm);

        if carry {
            self.exception(ExceptionCode::Overflow);
        } else {
            self.reg[rt!(instr)] = result;
        }
    }

    /// Add immediate unsigned (no overflow)
    pub fn addiu_emulate(&mut self, instr: u32) {
        let rs = self.reg[rs!(instr)];
        let imm = simmed!(instr);
        self.reg[rt!(instr)] = rs.overflowing_add(imm).0;
    }

    /// Bitwise AND immediate
    pub fn andi_emulate(&mut self, instr: u32) {
        self.reg[rt!(instr)] = self.reg[rs!(instr)] & immed!(instr);
    }

    /// Bitwise OR immediate
    pub fn ori_emulate(&mut self, instr: u32) {
        self.reg[rt!(instr)] = self.reg[rs!(instr)] | immed!(instr);
    }

    /// Bitwise XOR immediate
    pub fn xori_emulate(&mut self, instr: u32) {
        self.reg[rt!(instr)] = self.reg[rs!(instr)] ^ immed!(instr);
    }

    /// Load upper immediate
    pub fn lui_emulate(&mut self, instr: u32) {
        self.reg[rt!(instr)] = immed!(instr) << 16;
    }

    /// Load byte
    pub fn lb_emulate(&mut self, memory: &mut impl Memory, instr: u32) -> Result<(), RmipsError> {
        let base = self.reg[rs!(instr)];
        let offset = simmed!(instr);
        let vaddress = base + offset;

        let paddress = self.cpzero.translate(vaddress);
        let data = memory.fetch_byte(paddress)?;
        self.reg[rt!(instr)] = data as i32 as u32; // Sign-extend the byte
        Ok(())
    }

    /// Load halfword
    pub fn lh_emulate(&mut self, instr: u32) {
        todo!()
    }

    /// Load word left
    pub fn lwl_emulate(&mut self, instr: u32) {
        todo!()
    }

    /// Load word
    pub fn lw_emulate(&mut self, memory: &mut impl Memory, instr: u32) -> Result<(), RmipsError> {
        // Calculate the virtual address
        let base = self.reg[rs!(instr)];
        let offset = simmed!(instr);
        let vaddress = base + offset;

        // If either of the two least-significant bits of the virtual address
        // are non-zero, a load address exception occurs
        if vaddress % 4 != 0 {
            self.exception(ExceptionCode::LoadAddressError);
        } else {
            let paddress = self.cpzero.translate(vaddress);
            let data = memory.fetch_word(paddress)?;
            self.reg[rt!(instr)] = data;
        }
        Ok(())
    }

    /// Load byte unsigned
    pub fn lbu_emulate(&mut self, memory: &mut impl Memory, instr: u32) -> Result<(), RmipsError> {
        let base = self.reg[rs!(instr)];
        let offset = simmed!(instr);
        let vaddress = base + offset;

        let paddress = self.cpzero.translate(vaddress);
        let data = memory.fetch_byte(paddress)? & 0xff;
        self.reg[rt!(instr)] = data.into(); // Zero-extend the byte
        Ok(())
    }

    /// Load halfword unsigned
    pub fn lhu_emulate(&mut self, instr: u32) {
        todo!()
    }

    /// Load word right
    pub fn lwr_emulate(&mut self, instr: u32) {
        todo!()
    }

    /// Store byte
    pub fn sb_emulate(&mut self, memory: &mut impl Memory, instr: u32) -> Result<(), RmipsError> {
        let data = self.reg[rt!(instr)] as u8;
        let base = self.reg[rs!(instr)];
        let offset = simmed!(instr);
        let vaddress = base + offset;
        let paddress = self.cpzero.translate(vaddress);
        memory.store_byte(paddress, data)
    }

    /// Store halfword
    pub fn sh_emulate(&mut self, memory: &mut impl Memory, instr: u32) -> Result<(), RmipsError> {
        let data = self.reg[rt!(instr)] as u16;
        let base = self.reg[rs!(instr)];
        let offset = simmed!(instr);
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
    pub fn swl_emulate(&mut self, instr: u32) {
        todo!()
    }

    /// Store word
    pub fn sw_emulate(&mut self, memory: &mut impl Memory, instr: u32) -> Result<(), RmipsError> {
        let data = self.reg[rt!(instr)];
        let base = self.reg[rs!(instr)];
        let offset = simmed!(instr);
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
    pub fn swr_emulate(&mut self, instr: u32) {
        todo!()
    }

    /// Load word from CP1
    pub fn lwc1_emulate(&mut self, instr: u32) {
        self.coprocessor_unimpl(1, instr, self.pc);
    }

    /// Load word from CP2
    pub fn lwc2_emulate(&mut self, instr: u32) {
        self.coprocessor_unimpl(1, instr, self.pc);
    }

    /// Load word from CP3
    pub fn lwc3_emulate(&mut self, instr: u32) {
        self.coprocessor_unimpl(1, instr, self.pc);
    }

    /// Store word from CP1
    pub fn swc1_emulate(&mut self, instr: u32) {
        self.coprocessor_unimpl(1, instr, self.pc);
    }

    /// Store word from CP2
    pub fn swc2_emulate(&mut self, instr: u32) {
        self.coprocessor_unimpl(2, instr, self.pc);
    }

    /// Store word from CP3
    pub fn swc3_emulate(&mut self, instr: u32) {
        self.coprocessor_unimpl(3, instr, self.pc);
    }

    /// Jump
    pub fn j_emulate(&mut self, instr: u32, pc: u32) {
        self.jump(instr, pc);
    }

    /// Jump and link
    pub fn jal_emulate(&mut self, instr: u32, pc: u32) {
        self.jump(instr, pc);

        // Store the address of the instruction after the delay slot in the return address register
        self.reg[REG_RA] = pc + 8;
    }

    /// Move From System Control Coprocessor
    pub fn mfc0_emulate(&mut self, instr: u32) {
        self.reg[rt!(instr)] = self.cpzero.reg[rd!(instr)];
    }

    /// Move To System Control Coprocessor
    pub fn mtc0_emulate(&mut self, instr: u32) {
        self.cpzero.reg[rd!(instr)] = self.reg[rt!(instr)];
    }

    /// Reserved instruction
    pub fn ri_emulate(&mut self) {
        self.exception(ExceptionCode::ReservedInstruction);
    }

    fn srl(&self, a: u32, b: u32) -> u32 {
        if b == 0 {
            return a;
        } else if b == 32 {
            return 0;
        } else {
            return (a >> b) & ((1 << (32 - b)) - 1);
        }
    }

    fn sra(&self, a: u32, b: u32) -> u32 {
        if b == 0 {
            return a;
        } else {
            return (a >> b) | (((a >> 31) & 0x01) * (((1 << b) - 1) << (32 - b)));
        }
    }

    fn control_transfer(&mut self, dest: u32) {
        self.delay_state = DelayState::Delaying;
        self.delay_pc = dest;
    }

    fn branch(&mut self, instr: u32, pc: u32) {
        // Calculate the target address for the PC-relative branch
        let offset = simmed!(instr) << 2;
        let target_address = (pc + 4).wrapping_add(offset);
        self.control_transfer(target_address);
    }

    fn jump(&mut self, instr: u32, pc: u32) {
        // Calculate the address to jump to as the result of a J-format instruction
        let target_address = ((pc + 4) & 0xf000_0000) | (jumptarget!(instr) << 2);
        self.control_transfer(target_address);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu::REG_A0;

    #[test]
    fn test_sll_emulate() {
        let mut cpu = CPU::new(false);
        let instr = 0x00052140;
        cpu.reg[rt!(instr)] = 42;
        cpu.sll_emulate(instr);
        assert_eq!(cpu.reg[REG_A0], 0x540);
    }

    #[test]
    fn test_srl_emulate() {
        let mut cpu = CPU::new(false);
        let instr = 0x00052142;
        cpu.reg[rt!(instr)] = 42;
        cpu.srl_emulate(instr);
        assert_eq!(cpu.reg[REG_A0], 1);
    }

    #[test]
    fn test_srlv_emulate() {
        let mut cpu = CPU::new(false);
        let instr = 0x00a42006;
        cpu.reg[rt!(instr)] = 0xffff;
        cpu.reg[rs!(instr)] = 1;
        cpu.srlv_emulate(instr);
        assert_eq!(cpu.reg[REG_A0], 0x7fff);
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
        let mut cpu = CPU::new(false);
        let instr = 0x0040f809;
        let pc = 0xbfc019b0;

        cpu.reg[rs!(instr)] = 0xbfc019b8;
        cpu.jalr_emulate(instr, pc);

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
    fn test_add_emulate() {
        let mut cpu = CPU::new(false);
        let instr = 0x00a62020;
        cpu.reg[rt!(instr)] = 0xffff_0fff;
        cpu.reg[rs!(instr)] = 0x0001_0000;
        cpu.add_emulate(instr);
        assert_eq!(cpu.reg[REG_A0], 0x0000_0fff);
    }

    #[test]
    fn test_add_emulate_exception() {
        // TODO: Ensure an overflow exception is triggered by add
        todo!()
    }

    #[test]
    fn test_addu_emulate() {
        let mut cpu = CPU::new(false);
        let instr = 0x00a62021;
        cpu.reg[rt!(instr)] = 0xffff_0fff;
        cpu.reg[rs!(instr)] = 0x0001_0000;
        cpu.addu_emulate(instr);
        assert_eq!(cpu.reg[REG_A0], 0x0000_0fff);
    }

    #[test]
    fn test_sub_emulate() {
        let mut cpu = CPU::new(false);
        let instr = 0x00a62022;
        cpu.reg[rt!(instr)] = 40;
        cpu.reg[rs!(instr)] = 42;
        cpu.sub_emulate(instr);
        assert_eq!(cpu.reg[REG_A0], 2);
    }

    #[test]
    fn test_sub_emulate_exception() {
        // TODO: Ensure an overflow exception is triggered by sub
        todo!()
    }

    #[test]
    fn test_subu_emulate() {
        let mut cpu = CPU::new(false);
        let instr = 0x00a62023;
        cpu.reg[rt!(instr)] = 1;
        cpu.reg[rs!(instr)] = 0;
        cpu.subu_emulate(instr);
        assert_eq!(cpu.reg[REG_A0], 0xffff_ffff);
    }

    #[test]
    fn test_and_emulate() {
        let mut cpu = CPU::new(false);
        let instr = 0x00852024;
        cpu.reg[rt!(instr)] = 42;
        cpu.reg[rs!(instr)] = 13;
        cpu.and_emulate(instr);
        assert_eq!(cpu.reg[REG_A0], 8);
    }

    #[test]
    fn test_or_emulate() {
        let mut cpu = CPU::new(false);
        let instr = 0x00852024;
        cpu.reg[rt!(instr)] = 42;
        cpu.reg[rs!(instr)] = 13;
        cpu.or_emulate(instr);
        assert_eq!(cpu.reg[REG_A0], 0x2f);
    }

    #[test]
    fn test_xor_emulate() {
        let mut cpu = CPU::new(false);
        let instr = 0x00a62026;
        cpu.reg[rt!(instr)] = 4242;
        cpu.reg[rs!(instr)] = 88;
        cpu.xor_emulate(instr);
        assert_eq!(cpu.reg[REG_A0], 0x10ca);
    }

    #[test]
    fn test_nor_emulate() {
        let mut cpu = CPU::new(false);
        let instr = 0x00852027;
        cpu.reg[rt!(instr)] = 42;
        cpu.reg[rs!(instr)] = 13;
        cpu.nor_emulate(instr);
        assert_eq!(cpu.reg[REG_A0], 0xffff_ffd0);
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
        let mut cpu = CPU::new(false);
        let instr = 0x10530005;
        let pc = 0xbfc006ec;

        cpu.reg[rt!(instr)] = 42;
        cpu.reg[rs!(instr)] = 42;
        cpu.beq_emulate(instr, pc);

        assert_eq!(cpu.delay_pc, 0xbfc00704);
        assert_eq!(cpu.delay_state, DelayState::Delaying)
    }

    #[test]
    fn test_beq_emulate_not_branches() {
        let mut cpu = CPU::new(false);
        let instr = 0x10530005;
        let pc = 0xbfc006ec;

        cpu.reg[rt!(instr)] = 24;
        cpu.reg[rs!(instr)] = 42;
        cpu.beq_emulate(instr, pc);

        assert_eq!(cpu.delay_pc, 0);
        assert_eq!(cpu.delay_state, DelayState::Normal)
    }

    #[test]
    fn test_bne_emulate_branches() {
        let mut cpu = CPU::new(false);
        let instr = 0x10530005;
        let pc = 0xbfc006ec;

        cpu.reg[rt!(instr)] = 24;
        cpu.reg[rs!(instr)] = 42;
        cpu.bne_emulate(instr, pc);

        assert_eq!(cpu.delay_pc, 0xbfc00704);
        assert_eq!(cpu.delay_state, DelayState::Delaying)
    }

    #[test]
    fn test_bne_emulate_not_branches() {
        let mut cpu = CPU::new(false);
        let instr = 0x10530005;
        let pc = 0xbfc006ec;

        cpu.reg[rt!(instr)] = 42;
        cpu.reg[rs!(instr)] = 42;
        cpu.bne_emulate(instr, pc);

        assert_eq!(cpu.delay_pc, 0);
        assert_eq!(cpu.delay_state, DelayState::Normal)
    }

    #[test]
    fn test_blez_emulate() {}

    #[test]
    fn test_bgtz_emulate() {}

    #[test]
    fn test_addi_emulate() {
        let mut cpu = CPU::new(false);
        let instr = 0x20840080;
        cpu.reg[rs!(instr)] = 42;
        cpu.addi_emulate(instr);
        assert_eq!(cpu.reg[REG_A0], 0xaa);
    }

    #[test]
    fn test_addi_emulate_exception() {
        // TODO: Ensure an overflow exception is triggered by add
        todo!()
    }

    #[test]
    fn test_addiu_emulate() {
        let mut cpu = CPU::new(false);
        let instr = 0x248400ff;
        cpu.reg[rs!(instr)] = 42;
        cpu.addiu_emulate(instr);
        assert_eq!(cpu.reg[REG_A0], 0x129);
    }

    #[test]
    fn test_andi_emulate() {
        let mut cpu = CPU::new(false);
        let instr = 0x30a40fff;
        cpu.reg[rs!(instr)] = 0x0110;
        cpu.andi_emulate(instr);
        assert_eq!(cpu.reg[REG_A0], 0x0110);
    }

    #[test]
    fn test_ori_emulate() {
        let mut cpu = CPU::new(false);
        let instr = 0x34a41001;
        cpu.reg[rs!(instr)] = 0x0110;
        cpu.ori_emulate(instr);
        assert_eq!(cpu.reg[REG_A0], 0x1111);
    }

    #[test]
    fn test_xori_emulate() {
        let mut cpu = CPU::new(false);
        let instr = 0x38a44321;
        cpu.reg[rs!(instr)] = 0x1234;
        cpu.xori_emulate(instr);
        assert_eq!(cpu.reg[REG_A0], 0x5115);
    }

    #[test]
    fn test_lui_emulate() {
        let mut cpu = CPU::new(false);
        let instr = 0x3c040064;
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
        let mut cpu = CPU::new(false);
        cpu.j_emulate(0xbf00100, 0xbfc00000);
        assert_eq!(cpu.delay_pc, 0xbfc00400);
        assert_eq!(cpu.delay_state, DelayState::Delaying)
    }

    #[test]
    fn test_jal_emulate() {}

    #[test]
    fn test_ri_emulate() {}
}
