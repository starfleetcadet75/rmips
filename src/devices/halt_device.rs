use log::debug;

use crate::devices::Device;
use crate::util::error::{Result, RmipsError};
use crate::Address;

/// The physical address for the halt device.
pub const BASE_ADDRESS: Address = 0x01010024;

pub struct HaltDevice;

impl Device for HaltDevice {
    fn debug_label(&self) -> String {
        "halt-device".to_owned()
    }

    fn read(&mut self, address: Address, data: &mut [u8]) -> Result<()> {
        debug!("read from halt device @ 0x{:08x}", address);

        // All reads to the halt device should return 0
        for v in data {
            *v = 0;
        }

        Ok(())
    }

    fn write(&mut self, address: Address, data: &[u8]) -> Result<()> {
        debug!("write to halt device @ 0x{:08x}", address);

        // Any valid writes to the halt device trigger the system to halt
        for v in data {
            if *v != 0 {
                return Err(RmipsError::Halt);
            }
        }

        Ok(())
    }
}
