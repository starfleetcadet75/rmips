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
