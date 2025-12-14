use crate::{bus::BusResult, devices::IoDevice};

#[derive(Debug, Default)]
pub struct Switches {
    pub value: u32,
}

impl IoDevice for Switches {
    fn read(&mut self, offset: u32) -> BusResult<u32> {
        if offset == 4 { Ok(self.value) } else { Ok(0) }
    }
    fn write(&mut self, _offset: u32, _value: u32) -> BusResult<()> {
        Ok(())
    }
}
