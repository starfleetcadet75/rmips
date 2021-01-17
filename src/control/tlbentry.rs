//! See Chapter 6 Memory Management and the TLB in the IDT R30xx Manual.

bitflags! {
    struct EntryHiMask: u32 {
        /// Virtual page number
        const VPN = 0xffff_f000;
        /// Address Space Identifier
        const ASID = 0x0000_0fc0;
    }
}

bitflags! {
    struct EntryLoMask: u32 {
        /// Physical frame number
        const PFN = 0xffff_f000;
        /// Cache control bit
        const NONCACHE = 0x0000_0800;
        /// Write control bit
        const DIRTY = 0x0000_0400;
        /// Valid bit
        const VALID = 0x0000_0200;
        /// Global bit
        const GLOBAL = 0x0000_0100;
    }
}

/// Represents an entry in the TLB for `CPZero`.
///
/// A TLB entry is 64 bits wide but is represented here
/// as two separate fields: `entryhi` and `entrylo`.
#[derive(Copy, Clone, Debug, Default)]
pub struct TlbEntry {
    pub entryhi: u32,
    pub entrylo: u32,
}

impl TlbEntry {
    fn vpn(&self) -> u32 {
        self.entryhi & EntryHiMask::VPN.bits()
    }

    fn asid(&self) -> u16 {
        (self.entryhi & EntryHiMask::ASID.bits()) as u16
    }

    fn pfn(&self) -> u32 {
        self.entrylo & EntryLoMask::PFN.bits()
    }

    fn noncacheable(&self) -> bool {
        (self.entrylo & EntryLoMask::NONCACHE.bits()) != 0
    }

    fn dirty(&self) -> bool {
        (self.entrylo & EntryLoMask::DIRTY.bits()) != 0
    }

    fn valid(&self) -> bool {
        (self.entrylo & EntryLoMask::VALID.bits()) != 0
    }

    fn global(&self) -> bool {
        (self.entrylo & EntryLoMask::GLOBAL.bits()) != 0
    }
}
