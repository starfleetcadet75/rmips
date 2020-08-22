use crate::memory::range::Range;

/// Default physical address for the halt device.
pub const HALTDEV_BASE_ADDRESS: u32 = 0x01010024;

/// Default (KSEG0) address for the halt device
// const HALT_ADDR: u32 = 0xa1010024;

/// Halt Control register
// const HALT_CONTROL: u8 = 0x00;

pub struct HaltDevice {
    data: Vec<u8>,
    base: u32,
}

impl HaltDevice {
    pub fn new() -> HaltDevice {
        HaltDevice {
            data: vec![0; 4],
            base: 0,
        }
    }
}

impl Range for HaltDevice {
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
