use crate::memory::range::Range;
use std::fs::File;
use std::io::Read;

pub struct ROM {
    data: Vec<u8>,
    base: u32,
}

impl ROM {
    pub fn new(rom_path: &str, base: u32) -> Self {
        let mut f = File::open(rom_path).expect("Failed to open ROM file for reading");
        let mut data = Vec::new();
        f.read_to_end(&mut data)
            .expect("Failed to read from ROM file");

        ROM { data, base }
    }
}

impl Range for ROM {
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
