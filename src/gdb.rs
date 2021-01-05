use crate::emulator::{EmulationEvent, Emulator};
use crate::memory::Memory;
use crate::util::error::RmipsError;
use gdbstub::arch;
use gdbstub::arch::mips::reg::id::MipsRegId;
use gdbstub::target::ext::base::singlethread::{ResumeAction, SingleThreadOps, StopReason};
use gdbstub::target::ext::base::BaseOps;
use gdbstub::target::ext::breakpoints::{
    HwWatchpoint, HwWatchpointOps, SwBreakpoint, SwBreakpointOps, WatchKind,
};
use gdbstub::target::{Target, TargetError, TargetResult};
use log::info;
use std::convert::TryInto;

impl Target for Emulator {
    type Arch = arch::mips::Mips;
    type Error = RmipsError;

    fn base_ops(&mut self) -> BaseOps<Self::Arch, Self::Error> {
        BaseOps::SingleThread(self)
    }

    fn sw_breakpoint(&mut self) -> Option<SwBreakpointOps<Self>> {
        Some(self)
    }

    fn hw_watchpoint(&mut self) -> Option<HwWatchpointOps<Self>> {
        Some(self)
    }
}

impl SingleThreadOps for Emulator {
    fn resume(
        &mut self,
        action: ResumeAction,
        check_gdb_interrupt: &mut dyn FnMut() -> bool,
    ) -> Result<StopReason<u32>, Self::Error> {
        let event = match action {
            ResumeAction::Step => match self.step()? {
                EmulationEvent::Step => return Ok(StopReason::DoneStep),
                e => e,
            },
            ResumeAction::Continue => {
                let mut cycles = 0;
                loop {
                    let event = self.step()?;
                    if event != EmulationEvent::Step {
                        break event;
                    };

                    // Check for GDB interrupt every 1024 instructions
                    cycles += 1;
                    if cycles % 1024 == 0 && check_gdb_interrupt() {
                        return Ok(StopReason::GdbInterrupt);
                    }
                }
            }
        };

        Ok(match event {
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
        })
    }

    fn read_registers(
        &mut self,
        regs: &mut arch::mips::reg::MipsCoreRegs<u32>,
    ) -> TargetResult<(), Self> {
        regs.r = self.cpu.reg;
        regs.lo = self.cpu.low;
        regs.hi = self.cpu.high;
        regs.pc = self.cpu.pc;
        Ok(())
    }

    fn write_registers(
        &mut self,
        regs: &arch::mips::reg::MipsCoreRegs<u32>,
    ) -> TargetResult<(), Self> {
        self.cpu.reg = regs.r;
        self.cpu.low = regs.lo;
        self.cpu.high = regs.hi;
        self.cpu.pc = regs.pc;
        Ok(())
    }

    fn read_register(&mut self, reg_id: MipsRegId, dst: &mut [u8]) -> TargetResult<(), Self> {
        let w = match reg_id {
            MipsRegId::Gpr(i) => self.cpu.reg[i as usize],
            MipsRegId::Status => self.cpu.cpzero.status,
            MipsRegId::Lo => self.cpu.low,
            MipsRegId::Hi => self.cpu.high,
            MipsRegId::Badvaddr => self.cpu.cpzero.badvaddr,
            MipsRegId::Cause => self.cpu.cpzero.cause,
            MipsRegId::Pc => self.cpu.pc,
            // MipsRegId::Fpr(i) => todo!(),
            // MipsRegId::Fcsr => todo!(),
            // MipsRegId::Fir => todo!(),
            _ => return Err(().into()),
        };

        dst.copy_from_slice(&w.to_le_bytes());
        Ok(())
    }

    fn write_register(&mut self, reg_id: MipsRegId, value: &[u8]) -> TargetResult<(), Self> {
        let w = u32::from_le_bytes(
            value.try_into().unwrap(), // .map_err(|_| TargetError::Fatal("invalid data"))?,
        );

        match reg_id {
            MipsRegId::Gpr(i) => self.cpu.reg[i as usize] = w,
            MipsRegId::Status => self.cpu.cpzero.status = w,
            MipsRegId::Lo => self.cpu.low = w,
            MipsRegId::Hi => self.cpu.high = w,
            MipsRegId::Badvaddr => self.cpu.cpzero.badvaddr = w,
            MipsRegId::Cause => self.cpu.cpzero.cause = w,
            MipsRegId::Pc => self.cpu.pc = w,
            // MipsRegId::Fpr(i) => todo!() = w,
            // MipsRegId::Fcsr => todo!() = w,
            // MipsRegId::Fir => todo!() = w,
            _ => return Err(().into()),
        };
        Ok(())
    }

    fn read_addrs(&mut self, start_address: u32, data: &mut [u8]) -> TargetResult<(), Self> {
        info!(
            "[gdb::read_addrs] Read data from start_address: 0x{:08x}",
            start_address
        );

        for (address, value) in (start_address..).zip(data.iter_mut()) {
            let address = self.cpu.cpzero.translate(address);
            info!("GDB reading from address: {:08x}, {:x}", address, value);
            *value = self.memory.fetch_byte(address)?
        }
        Ok(())
    }

    fn write_addrs(&mut self, start_address: u32, data: &[u8]) -> TargetResult<(), Self> {
        for (address, value) in (start_address..).zip(data.iter().copied()) {
            let address = self.cpu.cpzero.translate(address);
            info!("GDB writing {:x} to address: {:08x}", value, address);
            self.memory.store_byte(address, value)?
        }
        Ok(())
    }
}

impl SwBreakpoint for Emulator {
    fn add_sw_breakpoint(&mut self, address: u32) -> TargetResult<bool, Self> {
        self.breakpoints.push(address);
        Ok(true)
    }

    fn remove_sw_breakpoint(&mut self, address: u32) -> TargetResult<bool, Self> {
        match self.breakpoints.iter().position(|x| *x == address) {
            None => return Ok(false),
            Some(pos) => self.breakpoints.remove(pos),
        };

        Ok(true)
    }
}

impl HwWatchpoint for Emulator {
    fn add_hw_watchpoint(&mut self, address: u32, kind: WatchKind) -> TargetResult<bool, Self> {
        match kind {
            WatchKind::Write => self.watchpoints.push(address),
            WatchKind::Read => self.watchpoints.push(address),
            WatchKind::ReadWrite => self.watchpoints.push(address),
        };

        Ok(true)
    }

    fn remove_hw_watchpoint(&mut self, address: u32, kind: WatchKind) -> TargetResult<bool, Self> {
        let pos = match self.watchpoints.iter().position(|x| *x == address) {
            None => return Ok(false),
            Some(pos) => pos,
        };

        match kind {
            WatchKind::Write => self.watchpoints.remove(pos),
            WatchKind::Read => self.watchpoints.remove(pos),
            WatchKind::ReadWrite => self.watchpoints.remove(pos),
        };

        Ok(true)
    }
}

impl From<RmipsError> for TargetError<RmipsError> {
    fn from(e: RmipsError) -> Self {
        TargetError::Fatal(e)
    }
}
