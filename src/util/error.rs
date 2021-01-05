use thiserror::Error;

/// RmipsError enumerates all possible errors returned by this library.
#[derive(Error, Debug)]
pub enum RmipsError {
    #[error("Attempted to execute an invalid instruction: 0x{:08x}", .0)]
    InvalidInstruction(u32),
    #[error("Attempted to access an invalid memory address: 0x{:08x}", .0)]
    InvalidMemoryAccess(u32),
    #[error("Attempted to access an unmapped range of memory: 0x{:08x}", .0)]
    UnmappedMemoryAccess(u32),
    #[error("Unable to map memory range: (base 0x{:08x} size 0x{:08x}) and (base 0x{:08x} size 0x{:08x})", .0, .1, .2, .3)]
    MemoryMapping(u32, usize, u32, usize),
    #[error(transparent)]
    IOError(#[from] std::io::Error),
}
