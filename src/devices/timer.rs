use crate::{bus::BusResult, devices::IoDevice};

#[derive(Debug, Default)]
pub struct Timer {
    pub current_tick: u32,
}

impl IoDevice for Timer {
    fn read(&mut self, offset: u32) -> BusResult<u32> {
        if offset == 0 { Ok(self.current_tick) } else { Ok(0) }
    }
    fn write(&mut self, _offset: u32, _value: u32) -> BusResult<()> {
        Ok(())
    }
}
