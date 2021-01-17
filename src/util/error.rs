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
}

impl std::error::Error for RmipsError {}

impl fmt::Display for RmipsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RmipsError::Io(err) => err.fmt(f),
            RmipsError::MemoryRangeOverlap => {
                write!(f, "New memory range overlaps an existing one")
            }
            RmipsError::MemoryRead(address) => {
                write!(f, "Failed to read memory from 0x{:08x}", address)
            }
            RmipsError::MemoryWrite(address) => {
                write!(f, "Failed to write memory to 0x{:08x}", address)
            }
            RmipsError::UnmappedAddress(address) => write!(
                f,
                "Address 0x{:08x} is not in a valid address space",
                address
            ),
            RmipsError::InvalidInstruction(instr) => write!(
                f,
                "Attempted to execute an invalid instruction: 0x{:08x}",
                instr
            ),
        }
    }
}

impl From<io::Error> for RmipsError {
    fn from(err: io::Error) -> RmipsError {
        RmipsError::Io(err)
    }
}
