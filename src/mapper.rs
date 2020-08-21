use crate::util::error::RmipsErrorKind::DuplicateMemoryRange;
use crate::util::error::RmipsErrorKind::InvalidMemoryAccess;
use crate::util::error::{RmipsError, RmipsResult};
use crate::util::range::Range;

pub struct Mapper {
    ranges: Vec<Box<dyn Range>>,
}

impl Mapper {
    pub fn new() -> Mapper {
        Mapper { ranges: vec![] }
    }

    /// Add the given range to the mappings provided it does not overlap with any existing ranges.
    fn add_range(&mut self, range: Box<dyn Range>) -> RmipsResult<()> {
        for r in &self.ranges {
            if range.overlaps(&r) {
                return Err(RmipsError::from(DuplicateMemoryRange(
                    range.get_base(),
                    range.get_extent(),
                    r.get_base(),
                    r.get_extent(),
                )));
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
    ) -> RmipsResult<()> {
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

    pub fn fetch_word(&self, address: u32) -> RmipsResult<u32> {
        if let Some(range) = self.find_mapping_range(address) {
            let offset = address - range.get_base();
            Ok(range.fetch_word(offset))
        } else {
            Err(RmipsError::from(InvalidMemoryAccess(address)))
        }
    }

    pub fn fetch_halfword(&self, address: u32) -> RmipsResult<u16> {
        if let Some(range) = self.find_mapping_range(address) {
            let offset = address - range.get_base();
            Ok(range.fetch_halfword(offset))
        } else {
            Err(RmipsError::from(InvalidMemoryAccess(address)))
        }
    }

    pub fn fetch_byte(&self, address: u32) -> RmipsResult<u8> {
        if let Some(range) = self.find_mapping_range(address) {
            Ok(range.fetch_byte(address))
        } else {
            Err(RmipsError::from(InvalidMemoryAccess(address)))
        }
    }

    pub fn store_word(&mut self, address: u32, data: u32) {
        if let Some(range) = self.find_mapping_range_mut(address) {
            range.store_word(address, data);
        }
    }

    pub fn store_halfword(&mut self, address: u32, data: u16) {
        let range = self.find_mapping_range_mut(address);
        if let Some(range) = range {
            range.store_halfword(address, data);
        }
    }

    pub fn store_byte(&mut self, address: u32, data: u8) {
        if let Some(range) = self.find_mapping_range_mut(address) {
            range.store_byte(address, data);
        }
    }
}
