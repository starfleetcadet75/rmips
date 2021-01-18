use std::collections::BTreeMap;
use std::fmt;

use crate::devices::Device;
use crate::memory::range::Range;
use crate::memory::Memory;
use crate::util::error::{Result, RmipsError};
use crate::Address;

/// A container for routing reads and writes to the correct address space.
pub struct Bus {
    devices: BTreeMap<Range, Box<dyn Device>>,
}

impl Bus {
    pub fn new() -> Self {
        Self {
            devices: BTreeMap::new(),
        }
    }

    pub fn register(&mut self, device: Box<dyn Device>, base: Address, size: usize) -> Result<()> {
        if size == 0 {
            return Err(RmipsError::MemoryRangeOverlap);
        }

        // Validate that the addresses for the new `Device` do not overlap with an existing one.
        if self
            .devices
            .iter()
            .any(|(range, _)| range.overlaps(base, size))
        {
            return Err(RmipsError::MemoryRangeOverlap);
        }

        match self.devices.insert(Range::new(base, size), device) {
            Some(_) => Err(RmipsError::MemoryRangeOverlap),
            None => Ok(()),
        }
    }

    pub fn get_device_mut(&mut self, address: Address) -> Option<(&Range, &mut Box<dyn Device>)> {
        self.devices
            .range_mut(..=Range::new(address, 1))
            .nth_back(0)
            .filter(|pair| address <= pair.0.last())
    }

    fn read(&mut self, address: Address, data: &mut [u8]) -> Result<()> {
        if let Some((range, dev)) = self.get_device_mut(address) {
            let offset = address - range.base();
            dev.read(offset, data)
        } else {
            Err(RmipsError::UnmappedAddress(address))
        }
    }

    fn write(&mut self, address: Address, data: &[u8]) -> Result<()> {
        if let Some((range, dev)) = self.get_device_mut(address) {
            let offset = address - range.base();
            dev.write(offset, data)
        } else {
            Err(RmipsError::UnmappedAddress(address))
        }
    }
}

impl Memory for Bus {
    fn fetch_word(&mut self, address: Address) -> Result<u32> {
        let mut data = [0; 4];
        self.read(address, &mut data)?;
        Ok(u32::from_le_bytes(data))
    }

    fn fetch_halfword(&mut self, address: Address) -> Result<u16> {
        let mut data = [0; 2];
        self.read(address, &mut data)?;
        Ok(u16::from_le_bytes(data))
    }

    fn fetch_byte(&mut self, address: Address) -> Result<u8> {
        let mut data = [0; 1];
        self.read(address, &mut data)?;
        Ok(u8::from_le_bytes(data))
    }

    fn store_word(&mut self, address: Address, data: u32) -> Result<()> {
        let data = u32::to_le_bytes(data);
        self.write(address, &data)
    }

    fn store_halfword(&mut self, address: Address, data: u16) -> Result<()> {
        let data = u16::to_le_bytes(data);
        self.write(address, &data)
    }

    fn store_byte(&mut self, address: Address, data: u8) -> Result<()> {
        let data = u8::to_le_bytes(data);
        self.write(address, &data)
    }
}

impl fmt::Display for Bus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (range, device) in &self.devices {
            writeln!(f, "  {}  {}", range, device.debug_label())?;
        }
        write!(f, "")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Copy, Clone, Debug)]
    struct TestDevice {
        data: [u8; 8],
    }

    impl Device for TestDevice {
        fn debug_label(&self) -> String {
            "test-device".to_owned()
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

    #[test]
    fn bus_insert() {
        let mut bus = Bus::new();
        let device = Box::new(TestDevice { data: [0; 8] });
        assert!(bus.register(device.clone(), 0x100, 0x10).is_ok());
        assert!(bus.register(device.clone(), 0x105, 0x10).is_err());
        assert!(bus.register(device.clone(), 0x100, 0x10).is_err());
        assert!(bus.register(device.clone(), 0x0, 0x20).is_ok());
        assert!(bus.register(device, 0x0, 0x10).is_err());
    }

    #[test]
    fn bus_read() {
        let mut bus = Bus::new();
        let device = Box::new(TestDevice {
            data: [0xde, 0xad, 0xbe, 0xef, 0xca, 0xfe, 0xba, 0xbe],
        });
        assert!(bus.register(device, 0x100, 0x8).is_ok());

        let mut data = [0; 4];
        assert!(bus.read(0x100, &mut data).is_ok());
        assert_eq!(data, [0xde, 0xad, 0xbe, 0xef]);

        let mut data = [0; 2];
        assert!(bus.read(0x104, &mut data).is_ok());
        assert_eq!(data, [0xca, 0xfe]);

        assert!(bus.read(0x110, &mut data).is_err());
        assert!(bus.read(0xff, &mut data).is_err());
    }

    #[test]
    fn bus_write() {
        let mut bus = Bus::new();
        let device = Box::new(TestDevice { data: [0; 8] });
        assert!(bus.register(device, 0x100, 0x8).is_ok());

        let data = [0xde, 0xad, 0xbe, 0xef, 0xca, 0xfe, 0xba, 0xbe];
        assert!(bus.write(0x100, &data).is_ok());

        let mut data = [0; 8];
        assert!(bus.read(0x100, &mut data).is_ok());
        assert_eq!(data, [0xde, 0xad, 0xbe, 0xef, 0xca, 0xfe, 0xba, 0xbe]);
    }

    #[test]
    fn bus_fetch_word() -> Result<()> {
        let mut bus = Bus::new();
        let device = Box::new(TestDevice {
            data: [0xef, 0xbe, 0xad, 0xde, 0xbe, 0xba, 0xfe, 0xca],
        });
        assert!(bus.register(device, 0x100, 0x8).is_ok());

        assert_eq!(bus.fetch_word(0x100)?, 0xdeadbeef);
        assert_eq!(bus.fetch_word(0x104)?, 0xcafebabe);
        assert!(bus.fetch_word(0x108).is_err());

        Ok(())
    }

    #[test]
    fn bus_fetch_halfword() -> Result<()> {
        let mut bus = Bus::new();
        let device = Box::new(TestDevice {
            data: [0xef, 0xbe, 0xad, 0xde, 0xbe, 0xba, 0xfe, 0xca],
        });
        assert!(bus.register(device, 0x100, 0x8).is_ok());

        assert_eq!(bus.fetch_halfword(0x100)?, 0xbeef);
        assert_eq!(bus.fetch_halfword(0x102)?, 0xdead);
        assert_eq!(bus.fetch_halfword(0x104)?, 0xbabe);
        assert_eq!(bus.fetch_halfword(0x106)?, 0xcafe);
        assert!(bus.fetch_halfword(0x108).is_err());

        Ok(())
    }

    #[test]
    fn bus_fetch_byte() -> Result<()> {
        let mut bus = Bus::new();
        let device = Box::new(TestDevice {
            data: [0xef, 0xbe, 0xad, 0xde, 0xbe, 0xba, 0xfe, 0xca],
        });
        assert!(bus.register(device, 0x100, 0x8).is_ok());

        assert_eq!(bus.fetch_byte(0x100)?, 0xef);
        assert_eq!(bus.fetch_byte(0x101)?, 0xbe);
        assert_eq!(bus.fetch_byte(0x102)?, 0xad);
        assert!(bus.fetch_byte(0x109).is_err());

        Ok(())
    }

    #[test]
    fn bus_store_word() -> Result<()> {
        let mut bus = Bus::new();
        let device = Box::new(TestDevice {
            data: [0xef, 0xbe, 0xad, 0xde, 0xbe, 0xba, 0xfe, 0xca],
        });
        assert!(bus.register(device, 0x100, 0x8).is_ok());

        assert!(bus.store_word(0x100, 0x1abcdef0).is_ok());
        assert_eq!(bus.fetch_word(0x100)?, 0x1abcdef0);

        assert!(bus.store_word(0x108, 0x1abcdef0).is_err());
        Ok(())
    }

    #[test]
    fn bus_store_halfword() -> Result<()> {
        let mut bus = Bus::new();
        let device = Box::new(TestDevice {
            data: [0xef, 0xbe, 0xad, 0xde, 0xbe, 0xba, 0xfe, 0xca],
        });
        assert!(bus.register(device, 0x100, 0x8).is_ok());

        assert!(bus.store_halfword(0x104, 0xabcd).is_ok());
        assert_eq!(bus.fetch_halfword(0x104)?, 0xabcd);

        assert!(bus.store_halfword(0x108, 0xabcd).is_err());
        Ok(())
    }

    #[test]
    fn bus_store_byte() -> Result<()> {
        let mut bus = Bus::new();
        let device = Box::new(TestDevice {
            data: [0xef, 0xbe, 0xad, 0xde, 0xbe, 0xba, 0xfe, 0xca],
        });
        assert!(bus.register(device, 0x100, 0x8).is_ok());

        assert!(bus.store_byte(0x102, 0x13).is_ok());
        assert!(bus.store_byte(0x104, 0x37).is_ok());
        assert_eq!(bus.fetch_byte(0x102)?, 0x13);
        assert_eq!(bus.fetch_byte(0x104)?, 0x37);

        assert!(bus.store_byte(0x108, 0xff).is_err());
        Ok(())
    }
}
