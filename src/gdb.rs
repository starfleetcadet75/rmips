use std::convert::TryInto;

use gdbstub::arch;
use gdbstub::arch::mips::reg::id::MipsRegId;
use gdbstub::arch::Arch;
use gdbstub::target::ext::base::singlethread::{ResumeAction, SingleThreadOps, StopReason};
use gdbstub::target::ext::base::BaseOps;
use gdbstub::target::ext::breakpoints::{
    HwWatchpoint, HwWatchpointOps, SwBreakpoint, SwBreakpointOps, WatchKind,
};
use gdbstub::target::{Target, TargetError, TargetResult};
use log::error;

use crate::control::cpzero;
use crate::emulator::{EmulationEvent, Emulator};
use crate::memory::Memory;
use crate::util::error::RmipsError;
use crate::Address;

impl Target for Emulator {
    type Arch = arch::mips::MipsWithDsp;
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
    ) -> Result<StopReason<Address>, Self::Error> {
        let event = match action {
            ResumeAction::Step => match self.step()? {
                EmulationEvent::Step => return Ok(StopReason::DoneStep),
                event => event,
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
        regs: &mut <Self::Arch as Arch>::Registers,
    ) -> TargetResult<(), Self> {
        regs.core.r = self.cpu.reg;
        regs.core.lo = self.cpu.low;
        regs.core.hi = self.cpu.high;
        regs.core.pc = self.cpu.pc;
        regs.core.cp0.status = self.cpu.cpzero.reg[cpzero::STATUS];
        regs.core.cp0.badvaddr = self.cpu.cpzero.reg[cpzero::BADVADDR];
        regs.core.cp0.cause = self.cpu.cpzero.reg[cpzero::CAUSE];
        Ok(())
    }

    fn write_registers(
        &mut self,
        regs: &<Self::Arch as Arch>::Registers,
    ) -> TargetResult<(), Self> {
        self.cpu.reg = regs.core.r;
        self.cpu.low = regs.core.lo;
        self.cpu.high = regs.core.hi;
        self.cpu.pc = regs.core.pc;
        self.cpu.cpzero.reg[cpzero::STATUS] = regs.core.cp0.status;
        self.cpu.cpzero.reg[cpzero::BADVADDR] = regs.core.cp0.badvaddr;
        self.cpu.cpzero.reg[cpzero::CAUSE] = regs.core.cp0.cause;
        Ok(())
    }

    fn read_register(&mut self, reg_id: MipsRegId<u32>, dst: &mut [u8]) -> TargetResult<(), Self> {
        let w = match reg_id {
            MipsRegId::Gpr(i) => self.cpu.reg[i as usize],
            MipsRegId::Status => self.cpu.cpzero.reg[cpzero::STATUS],
            MipsRegId::Lo => self.cpu.low,
            MipsRegId::Hi => self.cpu.high,
            MipsRegId::Badvaddr => self.cpu.cpzero.reg[cpzero::BADVADDR],
            MipsRegId::Cause => self.cpu.cpzero.reg[cpzero::CAUSE],
            MipsRegId::Pc => self.cpu.pc,
            // MipsRegId::Fpr(i) => todo!(),
            // MipsRegId::Fcsr => todo!(),
            // MipsRegId::Fir => todo!(),
            _ => return Err(().into()),
        };

        dst.copy_from_slice(&w.to_le_bytes());
        Ok(())
    }

    fn write_register(&mut self, reg_id: MipsRegId<u32>, value: &[u8]) -> TargetResult<(), Self> {
        let w = u32::from_le_bytes(value.try_into().expect("invalid write register data"));

        match reg_id {
            MipsRegId::Gpr(i) => self.cpu.reg[i as usize] = w,
            MipsRegId::Status => self.cpu.cpzero.reg[cpzero::STATUS] = w,
            MipsRegId::Lo => self.cpu.low = w,
            MipsRegId::Hi => self.cpu.high = w,
            MipsRegId::Badvaddr => self.cpu.cpzero.reg[cpzero::BADVADDR] = w,
            MipsRegId::Cause => self.cpu.cpzero.reg[cpzero::CAUSE] = w,
            MipsRegId::Pc => self.cpu.pc = w,
            // MipsRegId::Fpr(i) => todo!() = w,
            // MipsRegId::Fcsr => todo!() = w,
            // MipsRegId::Fir => todo!() = w,
            _ => return Err(().into()),
        };
        Ok(())
    }

    fn read_addrs(&mut self, start_address: u32, data: &mut [u8]) -> TargetResult<(), Self> {
        for (address, value) in (start_address..).zip(data.iter_mut()) {
            let address = self.cpu.cpzero.translate(address);
            *value = match self.bus.fetch_byte(address) {
                Ok(v) => v,
                Err(err) => {
                    error!("GDB failed to access memory: {}", err);
                    return Err(TargetError::NonFatal);
                }
            };
        }
        Ok(())
    }

    fn write_addrs(&mut self, start_address: u32, data: &[u8]) -> TargetResult<(), Self> {
        for (address, value) in (start_address..).zip(data.iter().copied()) {
            let address = self.cpu.cpzero.translate(address);
            if let Err(err) = self.bus.store_byte(address, value) {
                error!("GDB failed to access memory: {}", err);
                return Err(TargetError::NonFatal);
            };
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
        Ok(match self.breakpoints.iter().position(|x| *x == address) {
            Some(pos) => {
                self.breakpoints.remove(pos);
                true
            }
            None => false,
        })
    }
}

impl HwWatchpoint for Emulator {
    fn add_hw_watchpoint(
        &mut self,
        address: <Self::Arch as Arch>::Usize,
        kind: WatchKind,
    ) -> TargetResult<bool, Self> {
        match kind {
            WatchKind::Write => self.watchpoints.push(address),
            WatchKind::Read => self.watchpoints.push(address),
            WatchKind::ReadWrite => self.watchpoints.push(address),
        };

        Ok(true)
    }

    fn remove_hw_watchpoint(
        &mut self,
        address: <Self::Arch as Arch>::Usize,
        kind: WatchKind,
    ) -> TargetResult<bool, Self> {
        let pos = match self.watchpoints.iter().position(|x| *x == address) {
            Some(pos) => pos,
            None => return Ok(false),
        };

        match kind {
            WatchKind::Write => self.watchpoints.remove(pos),
            WatchKind::Read => self.watchpoints.remove(pos),
            WatchKind::ReadWrite => self.watchpoints.remove(pos),
        };

        Ok(true)
    }
}
