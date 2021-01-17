//! This module provides emulated hardware and virtual devices.

use crate::util::error::Result;
use crate::Address;

pub(crate) mod halt_device;
pub(crate) mod test_device;

pub trait Device {
    /// Returns a device name for debug output.
    fn debug_label(&self) -> String;
    /// Reads at `offset` from this device.
    fn read(&mut self, offset: Address, data: &mut [u8]) -> Result<()>;
    /// Writes at `offset` into this device.
    fn write(&mut self, offset: Address, data: &[u8]) -> Result<()>;
}
