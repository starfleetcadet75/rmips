use byteorder::{ByteOrder, NativeEndian};

/// Trait for an object that manages a range of mapped memory.
/// Memory-mapped devices implement this trait.
pub trait Range {
    /// Returns a reference to the underlying vector of bytes.
    fn get_data(&self) -> &Vec<u8>;

    /// Returns a mutable reference to the underlying vector of bytes.
    fn get_data_mut(&mut self) -> &mut Vec<u8>;

    /// The first physical address represented.
    fn get_base(&self) -> u32;

    /// Move the base address of this Range of memory.
    fn rebase(&mut self, paddress: u32);

    /// The number of bytes of memory provided.
    fn get_extent(&self) -> usize {
        self.get_data().len()
    }

    /// Fetch the word at the given offset into the Range.
    fn fetch_word(&self, offset: u32) -> u32 {
        NativeEndian::read_u32(&self.get_data()[offset as usize..])
    }

    /// Fetch the halfword at the given offset into the Range.
    fn fetch_halfword(&self, offset: u32) -> u16 {
        NativeEndian::read_u16(&self.get_data()[offset as usize..])
    }

    /// Fetch the byte at the given offset into the Range.
    fn fetch_byte(&self, offset: u32) -> u8 {
        self.get_data()[offset as usize]
    }

    /// Store a word at the given offset into the Range.
    fn store_word(&mut self, offset: u32, data: u32) {
        NativeEndian::write_u32(&mut self.get_data_mut()[offset as usize..], data);
    }

    /// Store a halfword at the given offset into the Range.
    fn store_halfword(&mut self, offset: u32, data: u16) {
        NativeEndian::write_u16(&mut self.get_data_mut()[offset as usize..], data);
    }

    /// Store a byte at the given offset into the Range.
    fn store_byte(&mut self, offset: u32, data: u8) {
        self.get_data_mut()[offset as usize] = data;
    }

    /// Returns true if the given address is mapped by this Range.
    fn contains(&self, address: u32) -> bool {
        (self.get_base() <= address) && (address < (self.get_base() + self.get_extent() as u32))
    }

    /// Returns true if the given Range overlaps with this Range.
    fn overlaps(&self, range: &Box<dyn Range>) -> bool {
        let end = self.get_base() + self.get_extent() as u32;
        let other_end = range.get_base() + range.get_extent() as u32;

        if self.get_base() <= range.get_base() && range.get_base() < end {
            true
        } else if range.get_base() <= self.get_base() && self.get_base() < other_end {
            true
        } else if range.get_base() <= self.get_base() && end <= other_end {
            true
        } else if self.get_base() <= range.get_base() && other_end <= end {
            true
        } else {
            false
        }
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
        fn new() -> Self {
            TestRange {
                data: vec![
                    0xef, 0xbe, 0xad, 0xde, 0xbe, 0xba, 0xfe, 0xca, 0x78, 0x56, 0x34, 0x12,
                ],
                base: 0,
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

    #[test]
    fn test_fetch_word() {
        let range = TestRange::new();
        assert_eq!(range.fetch_word(0), 0xdeadbeef);
        assert_eq!(range.fetch_word(4), 0xcafebabe);
        assert_eq!(range.fetch_word(8), 0x12345678);
    }

    #[test]
    fn test_fetch_halfword() {
        let range = TestRange::new();
        assert_eq!(range.fetch_halfword(0), 0xbeef);
        assert_eq!(range.fetch_halfword(2), 0xdead);
        assert_eq!(range.fetch_halfword(4), 0xbabe);
        assert_eq!(range.fetch_halfword(6), 0xcafe);
    }

    #[test]
    fn test_fetch_byte() {
        let range = TestRange::new();
        assert_eq!(range.fetch_byte(0), 0xef);
        assert_eq!(range.fetch_byte(1), 0xbe);
        assert_eq!(range.fetch_byte(2), 0xad);
        assert_eq!(range.fetch_byte(3), 0xde);
    }

    #[test]
    fn test_store_word() {
        let mut range = TestRange::new();
        range.store_word(8, 0x1abcdef0);
        assert_eq!(range.fetch_word(8), 0x1abcdef0);
    }

    #[test]
    fn test_store_halfword() {
        let mut range = TestRange::new();
        range.store_word(8, 0xabcd);
        assert_eq!(range.fetch_word(8), 0xabcd);
    }

    #[test]
    fn test_store_byte() {
        let mut range = TestRange::new();
        range.store_byte(4, 0x42);
        assert_eq!(range.fetch_byte(4), 0x42);
    }
}
