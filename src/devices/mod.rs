pub mod timer;
pub mod switches;
pub mod input;
pub mod spi;
pub mod disk;
pub mod clipboard;

use crate::bus::BusResult;

pub trait IoDevice: std::fmt::Debug {
    fn read(&mut self, offset: u32) -> BusResult<u32>;
    fn write(&mut self, offset: u32, value: u32) -> BusResult<()>;
}