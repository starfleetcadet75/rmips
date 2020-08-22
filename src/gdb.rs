use crate::emulator::{EmulationEvent, Emulator};
use gdbstub::arch::mips;
use gdbstub::{BreakOp, ResumeAction, StopReason, Target, Tid, TidSelector, SINGLE_THREAD_TID};
use std::ops::Range;

impl Target for Emulator {
    type Arch = mips::Mips;
    type Error = &'static str;

    fn resume(
        &mut self,
        actions: &mut dyn Iterator<Item = (TidSelector, ResumeAction)>,
        check_gdb_interrupt: &mut dyn FnMut() -> bool,
    ) -> Result<(Tid, StopReason<u32>), Self::Error> {
        let (_, action) = actions.next().unwrap();

        let event = match action {
            ResumeAction::Step => match self.step().unwrap() {
                EmulationEvent::Step => return Ok((SINGLE_THREAD_TID, StopReason::DoneStep)),
                e => e,
            },
            ResumeAction::Continue => {
                let mut cycles = 0;
                loop {
                    let event = self.step().unwrap();
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
                // Event::WatchWrite(addr) => StopReason::Watch {
                //     kind: WatchKind::Write,
                //     addr,
                // },
                // Event::WatchRead(addr) => StopReason::Watch {
                //     kind: WatchKind::Read,
                //     addr,
                // },
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
            // push_byte(self.mem.r8(addr))
        }
        Ok(())
    }

    fn write_addrs(&mut self, start_address: u32, data: &[u8]) -> Result<(), Self::Error> {
        todo!()
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
}
