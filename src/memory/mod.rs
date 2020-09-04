use crate::util::error::RmipsError;

pub mod mapper;
pub mod monitor;
pub mod ram;
pub mod range;
pub mod rom;

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Eq, Ord)]
pub enum Endian {
    Big,
    Little,
}

pub trait Memory {
    fn fetch_word(&mut self, offset: u32) -> Result<u32, RmipsError>;
    fn fetch_halfword(&mut self, offset: u32) -> Result<u16, RmipsError>;
    fn fetch_byte(&mut self, offset: u32) -> Result<u8, RmipsError>;
    fn store_word(&mut self, offset: u32, data: u32) -> Result<(), RmipsError>;
    fn store_halfword(&mut self, offset: u32, data: u16) -> Result<(), RmipsError>;
    fn store_byte(&mut self, offset: u32, data: u8) -> Result<(), RmipsError>;
}
