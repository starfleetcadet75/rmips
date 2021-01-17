use crate::devices::Device;
use crate::util::error::{Result, RmipsError};
use crate::Address;

#[derive(Clone, Debug)]
pub struct Ram {
    data: Vec<u8>,
}

impl Ram {
    pub fn new(size: usize) -> Self {
        Self {
            data: vec![0; size],
        }
    }
}

impl Device for Ram {
    fn debug_label(&self) -> String {
        "RAM".to_owned()
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
