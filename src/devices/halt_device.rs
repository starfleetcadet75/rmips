use crate::memory::range::Range;
use crate::memory::Endian;

// Located at 0xa1010024 in vaddr
/// Default physical address for the halt device.
pub const HALTDEV_BASE_ADDRESS: u32 = 0x01010024;

pub struct HaltDevice {
    endian: Endian,
    data: Vec<u8>,
    base: u32,
}

impl HaltDevice {
    pub fn new(endian: Endian) -> Self {
        HaltDevice {
            endian,
            data: vec![0; 4],
            base: HALTDEV_BASE_ADDRESS,
        }
    }
}

impl Range for HaltDevice {
    fn get_endian(&self) -> Endian {
        self.endian
    }

    fn get_data(&self) -> &Vec<u8> {
        &self.data
    }

    fn get_data_mut(&mut self) -> &mut Vec<u8> {
        &mut self.data
    }

    fn get_base(&self) -> u32 {
        self.base
    }
}
