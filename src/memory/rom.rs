use crate::memory::range::Range;
use crate::memory::Endian;
use crate::util::error::RmipsError;
use std::fs::File;
use std::io::Read;

pub struct ROM {
    rom_path: String,
    endian: Endian,
    data: Vec<u8>,
    base: u32,
}

impl ROM {
    pub fn new(endian: Endian, rom_path: &str, base: u32) -> Result<Self, RmipsError> {
        let mut f = File::open(rom_path)?;
        let mut data = Vec::new();
        f.read_to_end(&mut data)?;

        Ok(ROM {
            rom_path: rom_path.to_string(),
            endian,
            data,
            base,
        })
    }
}

impl Range for ROM {
    fn get_name(&self) -> &str {
        &self.rom_path
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
