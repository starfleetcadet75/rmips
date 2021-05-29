use std::convert::TryInto;

use gdbstub::arch::Arch;
use gdbstub::target;
use gdbstub::target::ext::base::singlethread::{
    GdbInterrupt, ResumeAction, SingleThreadOps, StopReason,
};
use gdbstub::target::ext::breakpoints::WatchKind;
use gdbstub::target::{Target, TargetError, TargetResult};
use gdbstub_arch::mips::reg::id::MipsRegId;
use log::error;

use crate::emulator::Emulator;
use crate::memory::Memory;
use crate::util::error::RmipsError;
use crate::{Address, EmulationEvent};

mod breakpoints;

impl Target for Emulator {
    type Arch = gdbstub_arch::mips::MipsWithDsp;
    type Error = RmipsError;

    #[inline(always)]
    fn base_ops(&mut self) -> target::ext::base::BaseOps<Self::Arch, Self::Error> {
        target::ext::base::BaseOps::SingleThread(self)
    }

    #[inline(always)]
    fn breakpoints(&mut self) -> Option<target::ext::breakpoints::BreakpointsOps<Self>> {
        Some(self)
    }
}

impl Emulator {
    fn inner_resume(
        &mut self,
        action: ResumeAction,
        mut check_gdb_interrupt: impl FnMut() -> bool,
    ) -> Result<StopReason<Address>, <Emulator as Target>::Error> {
        let event = match action {
            ResumeAction::Step | ResumeAction::StepWithSignal(_) => match self.step()? {
                EmulationEvent::Step => return Ok(StopReason::DoneStep),
                event => event,
            },
            ResumeAction::Continue | ResumeAction::ContinueWithSignal(_) => {
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
            EmulationEvent::Halted => StopReason::Terminated(19), // SIGSTOP
            EmulationEvent::Breakpoint => StopReason::SwBreak,
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
}

impl SingleThreadOps for Emulator {
    fn resume(
        &mut self,
        action: ResumeAction,
        gdb_interrupt: GdbInterrupt<'_>,
    ) -> Result<StopReason<Address>, Self::Error> {
        let mut gdb_interrupt = gdb_interrupt.no_async();
        self.inner_resume(action, || gdb_interrupt.pending())
    }

    #[inline(always)]
    fn single_register_access(
        &mut self,
    ) -> Option<target::ext::base::SingleRegisterAccessOps<(), Self>> {
        Some(self)
    }

    fn read_registers(
        &mut self,
        regs: &mut <Self::Arch as Arch>::Registers,
    ) -> TargetResult<(), Self> {
        regs.core.r = self.cpu.reg;
        regs.core.lo = self.cpu.low;
        regs.core.hi = self.cpu.high;
        regs.core.pc = self.cpu.pc;
        regs.core.cp0.status = self.cpu.cpzero.status.into();
        regs.core.cp0.badvaddr = self.cpu.cpzero.badvaddr.into();
        regs.core.cp0.cause = self.cpu.cpzero.cause.into();
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
        self.cpu.cpzero.status = regs.core.cp0.status.into();
        self.cpu.cpzero.badvaddr = regs.core.cp0.badvaddr.into();
        self.cpu.cpzero.cause = regs.core.cp0.cause.into();
        Ok(())
    }

    fn read_addrs(&mut self, start_address: Address, data: &mut [u8]) -> TargetResult<(), Self> {
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

    fn write_addrs(&mut self, start_address: Address, data: &[u8]) -> TargetResult<(), Self> {
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

impl target::ext::base::SingleRegisterAccess<()> for Emulator {
    fn read_register(
        &mut self,
        _tid: (),
        reg_id: gdbstub_arch::mips::reg::id::MipsRegId<Address>,
        dst: &mut [u8],
    ) -> TargetResult<(), Self> {
        let w = match reg_id {
            MipsRegId::Gpr(i) => self.cpu.reg[i as usize],
            MipsRegId::Status => self.cpu.cpzero.status.into(),
            MipsRegId::Lo => self.cpu.low,
            MipsRegId::Hi => self.cpu.high,
            MipsRegId::Badvaddr => self.cpu.cpzero.badvaddr.into(),
            MipsRegId::Cause => self.cpu.cpzero.cause.into(),
            MipsRegId::Pc => self.cpu.pc,
            // MipsRegId::Fpr(i) => todo!(),
            // MipsRegId::Fcsr => todo!(),
            // MipsRegId::Fir => todo!(),
            _ => return Err(().into()),
        };

        dst.copy_from_slice(&w.to_le_bytes());
        Ok(())
    }

    fn write_register(
        &mut self,
        _tid: (),
        reg_id: gdbstub_arch::mips::reg::id::MipsRegId<Address>,
        value: &[u8],
    ) -> TargetResult<(), Self> {
        let w = Address::from_le_bytes(value.try_into().expect("invalid write register data"));

        match reg_id {
            MipsRegId::Gpr(i) => self.cpu.reg[i as usize] = w,
            MipsRegId::Status => self.cpu.cpzero.status = w.into(),
            MipsRegId::Lo => self.cpu.low = w,
            MipsRegId::Hi => self.cpu.high = w,
            MipsRegId::Badvaddr => self.cpu.cpzero.badvaddr = w.into(),
            MipsRegId::Cause => self.cpu.cpzero.cause = w.into(),
            MipsRegId::Pc => self.cpu.pc = w,
            // MipsRegId::Fpr(i) => todo!() = w,
            // MipsRegId::Fcsr => todo!() = w,
            // MipsRegId::Fir => todo!() = w,
            _ => return Err(().into()),
        };

        Ok(())
    }
}
