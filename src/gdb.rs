use crate::emulator::{EmulationEvent, Emulator};
use crate::memory::Memory;
use crate::util::error::RmipsError;
use gdbstub::arch::mips;
use gdbstub::{
    BreakOp, OptResult, ResumeAction, StopReason, Target, Tid, TidSelector, WatchKind,
    SINGLE_THREAD_TID,
};
use log::info;
use std::ops::Range;

impl Target for Emulator {
    type Arch = mips::Mips;
    type Error = RmipsError;

    fn resume(
        &mut self,
        actions: &mut dyn Iterator<Item = (TidSelector, ResumeAction)>,
        check_gdb_interrupt: &mut dyn FnMut() -> bool,
    ) -> Result<(Tid, StopReason<u32>), Self::Error> {
        let (_, action) = actions.next().unwrap();

        let event = match action {
            ResumeAction::Step => match self.step()? {
                EmulationEvent::Step => return Ok((SINGLE_THREAD_TID, StopReason::DoneStep)),
                e => e,
            },
            ResumeAction::Continue => {
                let mut cycles = 0;
                loop {
                    let event = self.step()?;
                    if event != EmulationEvent::Step {
                        break event;
                    }

                    // Check for GDB interrupt every 1024 instructions
                    cycles += 1;
                    if cycles % 1024 == 0 && check_gdb_interrupt() {
                        return Ok((SINGLE_THREAD_TID, StopReason::GdbInterrupt));
                    }
                }
            }
        };

        Ok((
            SINGLE_THREAD_TID,
            match event {
                EmulationEvent::Halted => StopReason::Halted,
                EmulationEvent::Breakpoint => StopReason::HwBreak,
                EmulationEvent::Step => StopReason::DoneStep,
                EmulationEvent::WatchWrite(address) => StopReason::Watch {
                    kind: WatchKind::Write,
                    addr: address,
                },
                EmulationEvent::WatchRead(address) => StopReason::Watch {
                    kind: WatchKind::Read,
                    addr: address,
                },
            },
        ))
    }

    fn read_registers(
        &mut self,
        regs: &mut mips::reg::MipsCoreRegs<u32>,
    ) -> Result<(), Self::Error> {
        regs.r = self.cpu.reg;
        regs.lo = self.cpu.low;
        regs.hi = self.cpu.high;
        regs.pc = self.cpu.pc;
        Ok(())
    }

    fn write_registers(&mut self, regs: &mips::reg::MipsCoreRegs<u32>) -> Result<(), Self::Error> {
        self.cpu.reg = regs.r;
        self.cpu.low = regs.lo;
        self.cpu.high = regs.hi;
        self.cpu.pc = regs.pc;
        Ok(())
    }

    fn read_addrs(
        &mut self,
        addrs: Range<u32>,
        push_byte: &mut dyn FnMut(u8),
    ) -> Result<(), Self::Error> {
        for address in addrs {
            let address = self.cpu.cpzero.translate(address);
            info!("GDB reading from address: {:08x}", address);
            push_byte(self.memory.fetch_byte(address)?);
        }
        Ok(())
    }

    fn write_addrs(&mut self, start_address: u32, data: &[u8]) -> Result<(), Self::Error> {
        for (address, value) in (start_address..).zip(data.iter().copied()) {
            let address = self.cpu.cpzero.translate(address);
            info!("GDB writing {:x} to address: {:08x}", value, address);
            self.memory.store_byte(address, value)?;
        }
        Ok(())
    }

    fn update_sw_breakpoint(&mut self, address: u32, op: BreakOp) -> Result<bool, Self::Error> {
        match op {
            BreakOp::Add => self.breakpoints.push(address),
            BreakOp::Remove => {
                let pos = match self.breakpoints.iter().position(|x| *x == address) {
                    None => return Ok(false),
                    Some(pos) => pos,
                };
                self.breakpoints.remove(pos);
            }
        }

        Ok(true)
    }

    fn update_hw_watchpoint(
        &mut self,
        address: u32,
        op: BreakOp,
        kind: WatchKind,
    ) -> OptResult<bool, Self::Error> {
        match op {
            BreakOp::Add => {
                match kind {
                    WatchKind::Write => self.watchpoints.push(address),
                    WatchKind::Read => self.watchpoints.push(address),
                    WatchKind::ReadWrite => self.watchpoints.push(address),
                };
            }
            BreakOp::Remove => {
                let pos = match self.watchpoints.iter().position(|x| *x == address) {
                    None => return Ok(false),
                    Some(pos) => pos,
                };

                match kind {
                    WatchKind::Write => self.watchpoints.remove(pos),
                    WatchKind::Read => self.watchpoints.remove(pos),
                    WatchKind::ReadWrite => self.watchpoints.remove(pos),
                };
            }
        }

        Ok(true)
    }
}
