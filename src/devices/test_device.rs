use crate::memory::range::Range;
use crate::memory::Endian;

pub const TESTDEV_BASE_ADDRESS: u32 = 0x0201_0000;

pub struct TestDevice {
    endian: Endian,
    data: Vec<u8>,
    base: u32,
}

impl TestDevice {
    pub fn new(endian: Endian) -> Self {
        let memsize = 0x100;
        TestDevice {
            endian,
            data: vec![0; memsize],
            base: TESTDEV_BASE_ADDRESS,
        }
    }
}

impl Range for TestDevice {
    fn get_name(&self) -> &str {
        "test-device"
    }

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
