use std::fmt;
use std::io;

use crate::Address;

/// A type alias for `Result<T, RmipsError>`.
pub type Result<T> = std::result::Result<T, RmipsError>;

#[derive(Debug)]
pub enum RmipsError {
    Io(io::Error),
    MemoryRangeOverlap,
    MemoryRead(Address),
    MemoryWrite(Address),
    UnmappedAddress(Address),
    InvalidInstruction(u32),
    Halt,
}

impl std::error::Error for RmipsError {}

impl fmt::Display for RmipsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::RmipsError::*;

        match self {
            Io(err) => err.fmt(f),
            MemoryRangeOverlap => write!(f, "New memory range overlaps an existing one"),
            MemoryRead(address) => write!(f, "Failed to read memory from 0x{:08x}", address),
            MemoryWrite(address) => write!(f, "Failed to write memory to 0x{:08x}", address),
            UnmappedAddress(address) => write!(
                f,
                "Address 0x{:08x} is not in a valid address space",
                address
            ),
            InvalidInstruction(instr) => write!(
                f,
                "Attempted to execute an invalid instruction: 0x{:08x}",
                instr
            ),
            Halt => write!(f, "System halt triggered"),
        }
    }
}

impl From<io::Error> for RmipsError {
    fn from(err: io::Error) -> RmipsError {
        RmipsError::Io(err)
    }
}
