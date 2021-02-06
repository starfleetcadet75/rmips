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

#[derive(Debug, Clone, Copy, PartialEq)]
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

impl std::ops::Index<Cp0Register> for [u32] {
    type Output = u32;

    fn index(&self, idx: Cp0Register) -> &Self::Output {
        &self[idx as usize]
    }
}

impl std::ops::IndexMut<Cp0Register> for [u32] {
    fn index_mut(&mut self, idx: Cp0Register) -> &mut Self::Output {
        &mut self[idx as usize]
    }
}

bitflags! {
    /// Bitmasks for extracting fields from the Index register.
    /// See Figure 6.3 in IDT R30xx Manual on page 6-4.
    pub struct IndexMask: u32 {
        /// Set when a tlbp instruction failed to find a valid translation.
        const P = 0b1000_0000_0000_0000_0000_0000_0000_0000;
        /// The index into the TLB.
        const IDX = 0b0000_0000_0000_0000_0011_1111_0000_0000;
    }
}

bitflags! {
    /// Bitmasks for extracting fields from the Random register.
    /// See Figure 6.4 in IDT R30xx Manual on page 6-4.
    pub struct RandomMask: u32 {
        /// The actual random value.
        const VAL = 0x00003f00;
    }
}

bitflags! {
    /// Bitmasks for extracting fields from the Status register (SR).
    /// See Figure 3.2 in IDT R30xx Manual on page 3-4.
    pub struct StatusMask: u32 {
        /// Coprocessor 3 Usable
        const CU3 = 0b1000_0000_0000_0000_0000_0000_0000_0000;
        /// Coprocessor 2 Usable
        const CU2 = 0b0100_0000_0000_0000_0000_0000_0000_0000;
        /// Coprocessor 1 Usable
        const CU1 = 0b0010_0000_0000_0000_0000_0000_0000_0000;
        /// Coprocessor 0 Usable
        const CU0 = 0b0001_0000_0000_0000_0000_0000_0000_0000;
        /// Coprocessors Usable
        const CU = Self::CU3.bits | Self::CU2.bits | Self::CU1.bits | Self::CU0.bits;
        /// Reverse Endianness in User Mode
        const RE = 0b0000_0010_0000_0000_0000_0000_0000_0000;
        /// Bootstrap Exception Vector
        const BEV = 0b0000_0000_0100_0000_0000_0000_0000_0000;
        /// TLB Shutdown
        const TS = 0b0000_0000_0010_0000_0000_0000_0000_0000;
        /// Cache Parity Bit
        const PE = 0b0000_0000_0001_0000_0000_0000_0000_0000;
        /// Cache Miss
        const CM = 0b0000_0000_0000_1000_0000_0000_0000_0000;
        /// Cache parity forced to zero
        const PZ = 0b0000_0000_0000_0100_0000_0000_0000_0000;
        /// Swap Caches
        const SWC = 0b0000_0000_0000_0010_0000_0000_0000_0000;
        /// Isolate Cache
        const ISC = 0b0000_0000_0000_0001_0000_0000_0000_0000;
        /// Interrupt Mask
        const IM = 0b0000_0000_0000_0000_1111_1111_0000_0000;
        /// Old Kernel / User Status
        const KUO = 0b0000_0000_0000_0000_0000_0000_0010_0000;
        /// Old Interrupt Enable Status
        const IEO = 0b0000_0000_0000_0000_0000_0000_0001_0000;
        /// Previous Kernel / User Status
        const KUP = 0b0000_0000_0000_0000_0000_0000_0000_1000;
        /// Previous Interrupt Enable Status
        const IEP = 0b0000_0000_0000_0000_0000_0000_0000_0100;
        /// Current Kernel / User Mode Status (KUc = 0 indicates kernel-mode)
        const KUC = 0b0000_0000_0000_0000_0000_0000_0000_0010;
        /// Current Interrupt Enable Status
        const IEC = 0b0000_0000_0000_0000_0000_0000_0000_0001;
    }
}
