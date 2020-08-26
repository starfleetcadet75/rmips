use crate::memory::range::Range;
use crate::memory::Memory;
use crate::util::error::RmipsError;
use std::fmt;

pub struct Mapper {
    ranges: Vec<Box<dyn Range>>,
}

impl Mapper {
    pub fn new() -> Self {
        Mapper { ranges: vec![] }
    }

    /// Add the given range to the mappings provided it does not overlap with any existing ranges.
    fn add_range(&mut self, range: Box<dyn Range>) -> Result<(), RmipsError> {
        for r in &self.ranges {
            if range.overlaps(&r) {
                return Err(RmipsError::MemoryMapping(
                    range.get_base(),
                    range.get_extent(),
                    r.get_base(),
                    r.get_extent(),
                ));
            }
        }
        self.ranges.push(range);
        Ok(())
    }

    /// Map a Range object at the given physical address in memory.
    pub fn map_at_physical_address(
        &mut self,
        mut range: Box<dyn Range>,
        paddress: u32,
    ) -> Result<(), RmipsError> {
        range.rebase(paddress);
        self.add_range(range)
    }

    /// Returns the first mapping in the list of memory ranges that contains the given address.
    fn find_mapping_range(&self, address: u32) -> Option<&Box<dyn Range>> {
        for range in &self.ranges {
            if range.contains(address) {
                return Some(range);
            }
        }
        None
    }

    /// Returns the first mapping in the list of memory ranges that contains the given address as a mutable reference.
    fn find_mapping_range_mut(&mut self, address: u32) -> Option<&mut Box<dyn Range>> {
        for range in &mut self.ranges {
            if range.contains(address) {
                return Some(range);
            }
        }
        None
    }
}

impl Memory for Mapper {
    fn fetch_word(&mut self, address: u32) -> Result<u32, RmipsError> {
        if let Some(range) = self.find_mapping_range(address) {
            range.fetch_word(address)
        } else {
            Err(RmipsError::UnmappedMemoryAccess(address))
        }
    }

    fn fetch_halfword(&mut self, address: u32) -> Result<u16, RmipsError> {
        if let Some(range) = self.find_mapping_range(address) {
            range.fetch_halfword(address)
        } else {
            Err(RmipsError::UnmappedMemoryAccess(address))
        }
    }

    fn fetch_byte(&mut self, address: u32) -> Result<u8, RmipsError> {
        if let Some(range) = self.find_mapping_range(address) {
            range.fetch_byte(address)
        } else {
            Err(RmipsError::UnmappedMemoryAccess(address))
        }
    }

    fn store_word(&mut self, address: u32, data: u32) -> Result<(), RmipsError> {
        if let Some(range) = self.find_mapping_range_mut(address) {
            range.store_word(address, data)
        } else {
            Err(RmipsError::UnmappedMemoryAccess(address))
        }
    }

    fn store_halfword(&mut self, address: u32, data: u16) -> Result<(), RmipsError> {
        if let Some(range) = self.find_mapping_range_mut(address) {
            range.store_halfword(address, data)
        } else {
            Err(RmipsError::UnmappedMemoryAccess(address))
        }
    }

    fn store_byte(&mut self, address: u32, data: u8) -> Result<(), RmipsError> {
        if let Some(range) = self.find_mapping_range_mut(address) {
            range.store_byte(address, data)
        } else {
            Err(RmipsError::UnmappedMemoryAccess(address))
        }
    }
}

impl fmt::Display for Mapper {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Mapped Memory Ranges:")?;
        for range in &self.ranges {
            writeln!(f, "{}", range)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestRange {
        data: Vec<u8>,
        base: u32,
    }

    impl TestRange {
        fn new(base: u32) -> Self {
            TestRange {
                data: vec![
                    0xef, 0xbe, 0xad, 0xde, 0xbe, 0xba, 0xfe, 0xca, 0x78, 0x56, 0x34, 0x12,
                ],
                base,
            }
        }
    }

    impl Range for TestRange {
        fn get_data_mut(&mut self) -> &mut Vec<u8> {
            &mut self.data
        }

        fn get_data(&self) -> &Vec<u8> {
            &self.data
        }

        fn get_base(&self) -> u32 {
            self.base
        }

        fn rebase(&mut self, paddress: u32) {
            self.base = paddress;
        }
    }

    fn setup() -> Mapper {
        let mut mem = Mapper::new();
        let paddress = 0x1fc00000;
        let range = TestRange::new(paddress);
        mem.map_at_physical_address(Box::new(range), paddress)
            .expect("Failed to map memory");
        mem
    }

    #[test]
    fn test_fetch_word() -> Result<(), RmipsError> {
        let mut mem = setup();
        assert_eq!(mem.fetch_word(0x1fc00000)?, 0xdeadbeef);
        assert_eq!(mem.fetch_word(0x1fc00004)?, 0xcafebabe);
        assert_eq!(mem.fetch_word(0x1fc00008)?, 0x12345678);
        Ok(())
    }

    #[test]
    fn test_fetch_halfword() -> Result<(), RmipsError> {
        let mut mem = setup();
        assert_eq!(mem.fetch_halfword(0x1fc00000)?, 0xbeef);
        assert_eq!(mem.fetch_halfword(0x1fc00002)?, 0xdead);
        assert_eq!(mem.fetch_halfword(0x1fc00004)?, 0xbabe);
        assert_eq!(mem.fetch_halfword(0x1fc00006)?, 0xcafe);
        Ok(())
    }

    #[test]
    fn test_fetch_byte() -> Result<(), RmipsError> {
        let mut mem = setup();
        assert_eq!(mem.fetch_byte(0x1fc00000)?, 0xef);
        assert_eq!(mem.fetch_byte(0x1fc00001)?, 0xbe);
        assert_eq!(mem.fetch_byte(0x1fc00002)?, 0xad);
        assert_eq!(mem.fetch_byte(0x1fc00003)?, 0xde);
        Ok(())
    }

    #[test]
    fn test_store_word() -> Result<(), RmipsError> {
        let mut mem = setup();
        mem.store_word(0x1fc00008, 0x1abcdef0)?;
        assert_eq!(mem.fetch_word(0x1fc00008)?, 0x1abcdef0);
        Ok(())
    }

    #[test]
    fn test_store_halfword() -> Result<(), RmipsError> {
        let mut mem = setup();
        mem.store_halfword(0x1fc00008, 0xabcd)?;
        assert_eq!(mem.fetch_halfword(0x1fc00008)?, 0xabcd);
        Ok(())
    }

    #[test]
    fn test_store_byte() -> Result<(), RmipsError> {
        let mut mem = setup();
        mem.store_byte(0x1fc00008, 0x42)?;
        assert_eq!(mem.fetch_byte(0x1fc00008)?, 0x42);
        Ok(())
    }
}
