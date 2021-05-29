use numeric_enum_macro::numeric_enum;

#[macro_use]
mod macros;
mod badvaddr;
mod cause;
mod context;
mod epc;
mod index;
mod prid;
mod random;
mod status;

pub use badvaddr::BadVaddrRegister;
pub use cause::CauseRegister;
pub use context::ContextRegister;
pub use epc::EpcRegister;
pub use index::IndexRegister;
pub use prid::PridRegister;
pub use random::RandomRegister;
pub use status::StatusRegister;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Register {
    /// Zero register.
    Zero = 0,
    /// Assembler temporary register.
    At = 1,
    V0 = 2,
    V1 = 3,
    A0 = 4,
    A1 = 5,
    A2 = 6,
    A3 = 7,
    T0 = 8,
    T1 = 9,
    T2 = 10,
    T3 = 11,
    T4 = 12,
    T5 = 13,
    T6 = 14,
    T7 = 15,
    S0 = 16,
    S1 = 17,
    S2 = 18,
    S3 = 19,
    S4 = 20,
    S5 = 21,
    S6 = 22,
    S7 = 23,
    T8 = 24,
    T9 = 25,
    K0 = 26,
    K1 = 27,
    /// Global pointer.
    Gp = 28,
    /// Stack pointer.
    Sp = 29,
    /// Frame pointer.
    Fp = 30,
    /// Return address register.
    Ra = 31,
}

impl std::ops::Index<Register> for [u32] {
    type Output = u32;

    fn index(&self, idx: Register) -> &Self::Output {
        &self[idx as usize]
    }
}

impl std::ops::IndexMut<Register> for [u32] {
    fn index_mut(&mut self, idx: Register) -> &mut Self::Output {
        &mut self[idx as usize]
    }
}

numeric_enum! {
    #[repr(u32)]
    #[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Ord, Eq, Hash)]
    pub enum Cp0Register {
        /// TLB entry index register.
        Index = 0,
        /// TLB randomized access register.
        Random = 1,
        /// Low-order word of "current" TLB entry.
        EntryLo = 2,
        /// Page-table lookup address.
        Context = 4,
        /// Contains the last invalid program address which caused a trap.
        BadVaddr = 8,
        /// High-order word of "current" TLB entry.
        EntryHi = 10,
        /// The Status register contains the operating mode, interrupt enable flag, and diagnostic states.
        Status = 12,
        /// Contains the cause of the last exception.
        Cause = 13,
        /// Contains the address to return to after handling an exception.
        Epc = 14,
        /// Processor Revision Identifier.
        Prid = 15,
    }
}
