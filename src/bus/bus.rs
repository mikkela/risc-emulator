pub type BusResult<T> = Result<T, BusError>;

#[derive(Debug, thiserror::Error)]
pub enum BusError {
    #[error("address out of bounds: 0x{0:08X}")]
    AddressOutOfBounds(u32),

    #[error("unmapped address: 0x{0:08X}")]
    Unmapped(u32),

    #[error("device error: {0}")]
    Device(String),
}

/// “Generel” bus. CPU bruger i praksis CpuBus (progress-aware reads).
pub trait Bus {
    fn read_word(&mut self, addr: u32) -> BusResult<u32>;
    fn write_word(&mut self, addr: u32, value: u32) -> BusResult<()>;

    fn read_byte(&mut self, addr: u32) -> BusResult<u8> {
        let w = self.read_word(addr & !3)?;
        let shift = (addr & 3) * 8;
        Ok(((w >> shift) & 0xFF) as u8)
    }

    fn write_byte(&mut self, addr: u32, value: u8) -> BusResult<()> {
        let base = addr & !3;
        let mut w = self.read_word(base)?;
        let shift = (addr & 3) * 8;
        w &= !(0xFFu32 << shift);
        w |= (value as u32) << shift;
        self.write_word(base, w)
    }
}
