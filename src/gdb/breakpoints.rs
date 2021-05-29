use gdbstub::target;
use gdbstub::target::ext::breakpoints::WatchKind;
use gdbstub::target::TargetResult;

use crate::emulator::Emulator;
use crate::Address;

impl target::ext::breakpoints::Breakpoints for Emulator {
    #[inline(always)]
    fn sw_breakpoint(&mut self) -> Option<target::ext::breakpoints::SwBreakpointOps<Self>> {
        Some(self)
    }

    #[inline(always)]
    fn hw_watchpoint(&mut self) -> Option<target::ext::breakpoints::HwWatchpointOps<Self>> {
        Some(self)
    }
}

impl target::ext::breakpoints::SwBreakpoint for Emulator {
    fn add_sw_breakpoint(
        &mut self,
        address: Address,
        _kind: gdbstub_arch::mips::MipsBreakpointKind,
    ) -> TargetResult<bool, Self> {
        self.breakpoints.push(address);
        Ok(true)
    }

    fn remove_sw_breakpoint(
        &mut self,
        address: Address,
        _kind: gdbstub_arch::mips::MipsBreakpointKind,
    ) -> TargetResult<bool, Self> {
        match self.breakpoints.iter().position(|x| *x == address) {
            None => return Ok(false),
            Some(pos) => self.breakpoints.remove(pos),
        };

        Ok(true)
    }
}

impl target::ext::breakpoints::HwWatchpoint for Emulator {
    fn add_hw_watchpoint(&mut self, address: Address, kind: WatchKind) -> TargetResult<bool, Self> {
        match kind {
            WatchKind::Write => self.watchpoints.push(address),
            WatchKind::Read => self.watchpoints.push(address),
            WatchKind::ReadWrite => self.watchpoints.push(address),
        };

        Ok(true)
    }

    fn remove_hw_watchpoint(
        &mut self,
        address: Address,
        kind: WatchKind,
    ) -> TargetResult<bool, Self> {
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
