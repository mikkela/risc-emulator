pub mod timer;
pub mod switches;
pub mod input;
pub mod spi;
mod disk;

use crate::bus::BusResult;

pub trait IoDevice: std::fmt::Debug {
    fn read(&mut self, offset: u32) -> BusResult<u32>;
    fn write(&mut self, offset: u32, value: u32) -> BusResult<()>;
}