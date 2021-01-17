use log::debug;

use crate::devices::Device;
use crate::util::error::Result;
use crate::Address;

/// The address for the test device.
pub const BASE_ADDRESS: Address = 0x0201_0000;
/// Size of the test device in memory.
pub const DATA_LEN: usize = 0x100;

pub struct TestDevice {
    data: [u8; DATA_LEN],
}

impl TestDevice {
    pub fn new() -> Self {
        // Set initial device state
        // data[0x0b] = 0x02;  // Status register in 24 hour mode

        Self {
            data: [0; DATA_LEN],
        }
    }
}

impl Device for TestDevice {
    fn debug_label(&self) -> String {
        "test-device".to_owned()
    }

    fn read(&mut self, address: Address, data: &mut [u8]) -> Result<()> {
        debug!("read from test device @ 0x{:08x}", address);

        data[0] = self.data[0];

        Ok(())
    }

    fn write(&mut self, address: Address, _data: &[u8]) -> Result<()> {
        debug!("write to test device @ 0x{:08x}", address);

        // match address {
        //     DATA_OFFSET => self.data[self.index as usize] = data[0],
        //     addr => panic!("bad write to halt device: 0x{:08x}", addr)
        // }

        Ok(())
    }
}
