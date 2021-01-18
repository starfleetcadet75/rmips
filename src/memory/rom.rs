use std::fs::File;
use std::io::Read;

use crate::devices::Device;
use crate::util::error::{Result, RmipsError};
use crate::Address;

#[derive(Clone, Debug)]
pub struct Rom {
    rom_path: String,
    data: Vec<u8>,
}

impl Rom {
    pub fn new(rom_path: String) -> Result<Rom> {
        let mut f =
            File::open(&rom_path).map_err(|_| RmipsError::RomLoading(rom_path.to_owned()))?;
        let mut data = Vec::new();
        f.read_to_end(&mut data)
            .map_err(|_| RmipsError::RomLoading(rom_path.to_owned()))?;

        // TODO: The current setup.s code tries to load one extra word at end of ROM
        // which causes a memory error. Need to either fix setup.s or align here.
        data.push(0);
        data.push(0);
        data.push(0);
        data.push(0);

        Ok(Self { rom_path, data })
    }

    pub fn size(&self) -> usize {
        self.data.len()
    }
}

impl Device for Rom {
    fn debug_label(&self) -> String {
        self.rom_path.to_owned()
    }

    fn read(&mut self, address: Address, data: &mut [u8]) -> Result<()> {
        for (i, v) in data.iter_mut().enumerate() {
            *v = *self
                .data
                .get((address as usize) + i)
                .ok_or(RmipsError::MemoryRead(address + (i as u32)))?;
        }

        Ok(())
    }

    fn write(&mut self, address: Address, data: &[u8]) -> Result<()> {
        for (i, v) in data.iter().enumerate() {
            if let Some(elem) = self.data.get_mut((address as usize) + i) {
                *elem = *v;
            } else {
                return Err(RmipsError::MemoryWrite(address + (i as u32)));
            }
        }

        Ok(())
    }
}
