#[macro_use]
extern crate bitflags;

mod control;
mod devices;
pub mod emulator;
mod gdb;
mod memory;
pub mod util;

type Address = u32;

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Eq, Ord)]
pub enum Endian {
    Big,
    Little,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum EmulationEvent {
    Step,
    Halted,
    Breakpoint,
    WatchWrite(Address),
    WatchRead(Address),
}

pub use control::registers;
