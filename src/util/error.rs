/// RmipsError enumerates all possible errors returned by this library.
#[derive(Debug)]
pub enum RmipsError {
    /// Represents an invalid instruction.
    InvalidInstruction(u32),
    /// Represents an attempt to access invalid memory.
    InvalidMemoryAccess(u32),
    /// Represents an attempt to access an unmapped memory range.
    UnmappedMemoryAccess(u32),
    /// Represents a failure to map memory at the given addresses.
    MemoryMapping(u32, usize, u32, usize),
    /// Represents cases of `std::io::Error`.
    IOError(std::io::Error),
}

impl std::error::Error for RmipsError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            RmipsError::InvalidInstruction(..) => None,
            RmipsError::InvalidMemoryAccess(..) => None,
            RmipsError::UnmappedMemoryAccess(..) => None,
            RmipsError::MemoryMapping(..) => None,
            RmipsError::IOError(_) => None,
        }
    }
}

impl std::fmt::Display for RmipsError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            RmipsError::InvalidInstruction(instruction) => write!(f, "Attempted to execute an invalid instruction: 0x{:08x}", instruction),
            RmipsError::InvalidMemoryAccess(address) => write!(f, "Attempted to access an invalid memory address: 0x{:08x}", address),
            RmipsError::UnmappedMemoryAccess(address) => write!(f, "Attempted to access an unmapped range of memory: 0x{:08x}", address),
            RmipsError::MemoryMapping(base, size, other_base, other_size) => write!(f, "Unable to map memory range: (base 0x{:08x} size 0x{:08x}) and (base 0x{:08x} size 0x{:08x})", base, size, other_base, other_size),
            RmipsError::IOError(ref err) => err.fmt(f),
        }
    }
}

impl From<std::io::Error> for RmipsError {
    fn from(err: std::io::Error) -> RmipsError {
        RmipsError::IOError(err)
    }
}
