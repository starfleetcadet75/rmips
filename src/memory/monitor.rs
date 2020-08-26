use crate::memory::Memory;
use crate::util::error::RmipsError;

macro_rules! impl_memsniff_r {
    ($fn:ident, $ret:ty) => {
        fn $fn(&mut self, address: u32) -> Result<$ret, RmipsError> {
            let ret = self.memory.$fn(address)?;
            if self.addresses.contains(&address) {
                (self.on_access)(Access {
                    kind: AccessKind::Read,
                    address,
                    data: ret as u32,
                    len: ret.to_le_bytes().len(),
                });
            }
            Ok(ret)
        }
    };
}

macro_rules! impl_memsniff_w {
    ($fn:ident, $data:ty) => {
        fn $fn(&mut self, address: u32, data: $data) -> Result<(), RmipsError> {
            self.memory.$fn(address, data)?;
            if self.addresses.contains(&address) {
                (self.on_access)(Access {
                    kind: AccessKind::Write,
                    address,
                    data: data as u32,
                    len: data.to_le_bytes().len(),
                });
            }
            Ok(())
        }
    };
}

pub enum AccessKind {
    Read,
    Write,
}

pub struct Access {
    pub kind: AccessKind,
    pub address: u32,
    pub data: u32,
    pub len: usize,
}

pub struct Monitor<'a, M: Memory, F: FnMut(Access)> {
    memory: &'a mut M,
    addresses: &'a [u32],
    on_access: F,
}

impl<'a, M: Memory, F: FnMut(Access)> Monitor<'a, M, F> {
    pub fn new(memory: &'a mut M, addresses: &'a [u32], on_access: F) -> Monitor<'a, M, F> {
        Monitor {
            memory,
            addresses,
            on_access,
        }
    }
}

impl<'a, M: Memory, F: FnMut(Access)> Memory for Monitor<'a, M, F> {
    impl_memsniff_r!(fetch_word, u32);
    impl_memsniff_r!(fetch_halfword, u16);
    impl_memsniff_r!(fetch_byte, u8);
    impl_memsniff_w!(store_word, u32);
    impl_memsniff_w!(store_halfword, u16);
    impl_memsniff_w!(store_byte, u8);
}
