use log::debug;

use crate::devices::Device;
use crate::util::error::Result;
use crate::Address;

/// The physical address for the halt device.
pub const BASE_ADDRESS: Address = 0x01010024;
/// Size of the halt device in memory.
pub const DATA_LEN: usize = 4;

pub struct HaltDevice {
    data: [u8; DATA_LEN],
}

impl HaltDevice {
    pub fn new() -> Self {
        let data = [0u8; DATA_LEN];

        // Set initial device state
        // data[0x0b] = 0x02;  // Status register in 24 hour mode

        Self { data }
    }
}

impl Device for HaltDevice {
    fn debug_label(&self) -> String {
        "halt-device".to_owned()
    }

    fn read(&mut self, address: Address, data: &mut [u8]) -> Result<()> {
        debug!("read from halt device @ 0x{:08x}", address);

        data[0] = self.data[0];

        Ok(())
    }

    fn write(&mut self, address: Address, _data: &[u8]) -> Result<()> {
        debug!("write to halt device @ 0x{:08x}", address);

        // match address {
        //     DATA_OFFSET => self.data[self.index as usize] = data[0],
        //     addr => panic!("bad write to halt device: 0x{:08x}", addr)
        // }

        Ok(())
    }
}
