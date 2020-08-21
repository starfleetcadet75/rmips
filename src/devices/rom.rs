use crate::util::error::{RmipsResult, RmipsResultExt};
use crate::util::range::Range;
use std::fs::File;
use std::io::Read;

pub struct ROM {
    data: Vec<u8>,
    base: u32,
}

impl ROM {
    pub fn new(rom_path: &str) -> RmipsResult<ROM> {
        let mut f = File::open(rom_path).chain_err(|| "Failed to open the ROM file")?;
        let mut data = Vec::new();
        f.read_to_end(&mut data)
            .chain_err(|| "Failed to read from ROM file")?;

        Ok(ROM { data, base: 0 })
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

    fn rebase(&mut self, paddress: u32) {
        self.base = paddress;
    }
}
