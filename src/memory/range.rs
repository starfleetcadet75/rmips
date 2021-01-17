use std::cmp::Ordering;

use crate::Address;

#[derive(Copy, Clone, Debug)]
pub struct Range {
    base: Address,
    size: usize,
}

impl Range {
    pub fn new(base: Address, size: usize) -> Self {
        Self { base, size }
    }

    /// Returns true if `address` is within the range.
    #[allow(dead_code)]
    pub fn contains(&self, address: Address) -> bool {
        self.base <= address && address < self.base + self.size as Address
    }

    /// Returns true if there is an overlap with this range.
    pub fn overlaps(&self, base: Address, len: usize) -> bool {
        self.base < (base + len as Address) && base < self.base + self.size as Address
    }

    /// Return the last address that is part of the range.
    pub fn last(&self) -> Address {
        self.base + (self.size as Address - 1)
    }

    pub fn base(&self) -> Address {
        self.base
    }

    #[allow(dead_code)]
    pub fn size(&self) -> usize {
        self.size
    }
}

impl Eq for Range {}

impl PartialEq for Range {
    fn eq(&self, other: &Range) -> bool {
        self.base == other.base
    }
}

impl Ord for Range {
    fn cmp(&self, other: &Range) -> Ordering {
        self.base.cmp(&other.base)
    }
}

impl PartialOrd for Range {
    fn partial_cmp(&self, other: &Range) -> Option<Ordering> {
        self.base.partial_cmp(&other.base)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn range_contains() {
        let range = Range::new(0x1000, 0x500);
        assert!(range.contains(0x1000));
        assert!(range.contains(0x1337));
        assert!(!range.contains(0xfff));
        assert!(!range.contains(0x1500));
    }

    #[test]
    fn range_overlaps() {
        let range = Range::new(0x1000, 0x500);
        assert!(range.overlaps(0x1000, 0x500));
        assert!(range.overlaps(0xf00, 0x500));
        assert!(range.overlaps(0x1000, 0x20));
        assert!(range.overlaps(0xfff, 0x2));
        assert!(range.overlaps(0x14ff, 0x100));
        assert!(!range.overlaps(0x1500, 0x100));
        assert!(!range.overlaps(0xf00, 0x100));
    }
}
