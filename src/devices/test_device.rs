use crate::memory::range::Range;

pub const TESTDEV_BASE_ADDRESS: u32 = 0x0201_0000;

pub struct TestDevice {
    data: Vec<u8>,
    base: u32,
}

impl TestDevice {
    pub fn new() -> TestDevice {
        let memsize = 0x100;
        TestDevice {
            data: vec![0; memsize],
            base: 0,
        }
    }
}

impl Range for TestDevice {
    fn get_data(&self) -> &Vec<u8> {
        &self.data
    }

    fn get_data_mut(&mut self) -> &mut Vec<u8> {
        &mut self.data
    }

    fn get_base(&self) -> u32 {
        self.base
    }

    fn rebase(&mut self, paddress: u32) {
        self.base = paddress;
    }
}
