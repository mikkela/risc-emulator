use crate::bus::BusResult;

pub trait SpiDevice: std::fmt::Debug {
    fn read_data(&mut self) -> BusResult<u32>;
    fn write_data(&mut self, value: u32) -> BusResult<()>;
}