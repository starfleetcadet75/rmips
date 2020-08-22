use crate::memory::range::Range;

pub struct RAM {
    data: Vec<u8>,
    base: u32,
}

impl RAM {
    pub fn new(memsize: usize) -> RAM {
        RAM {
            data: vec![0; memsize],
            base: 0,
        }
    }
}

impl Range for RAM {
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
