use crate::util::error::Result;
use crate::Address;

pub(crate) mod bus;
pub(crate) mod monitor;
pub(crate) mod ram;
pub(crate) mod range;
pub(crate) mod rom;

pub trait Memory {
    fn fetch_word(&mut self, address: Address) -> Result<u32>;
    fn fetch_halfword(&mut self, address: Address) -> Result<u16>;
    fn fetch_byte(&mut self, address: Address) -> Result<u8>;
    fn store_word(&mut self, address: Address, data: u32) -> Result<()>;
    fn store_halfword(&mut self, address: Address, data: u16) -> Result<()>;
    fn store_byte(&mut self, address: Address, data: u8) -> Result<()>;
}
