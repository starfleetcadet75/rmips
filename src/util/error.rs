use error_chain::error_chain;

error_chain! {
  types {
    RmipsError, RmipsErrorKind, RmipsResultExt, RmipsResult;
  }
  foreign_links {
    Io(std::io::Error);
    ParseInt(std::num::ParseIntError);
  }
  errors {
    DuplicateMemoryRange(base: u32, extent: usize, other_base: u32, other_extent: usize) {
      description("Attempted to map multiple components to the same memory range")
      display("Attempted to map multiple components to the same memory range: (base 0x{:08x} extent 0x{:08x}) and (base 0x{:08x} extent 0x{:08x})", base, extent, other_base, other_extent)
    }
    InvalidMemoryAccess(address: u32) {
      description("Attempted to access an invalid memory address")
      display("Attempted to access an invalid memory address: 0x{:08x}", address)
    }
    InvalidInstruction(opcode: u8) {
      description("Invalid instruction")
      display("Encountered an invalid instruction: 0x{:02x}", opcode)
    }
  }
}
