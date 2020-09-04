use crate::memory::range::Range;
use crate::memory::Endian;

pub struct RAM {
    endian: Endian,
    data: Vec<u8>,
    base: u32,
}

impl RAM {
    pub fn new(endian: Endian, memsize: usize, base: u32) -> Self {
        RAM {
            endian,
            data: vec![0; memsize],
            base,
        }
    }
}

impl Range for RAM {
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
