use crate::bus::{BusError, BusResult};

#[derive(Debug)]
pub struct Ram {
    bytes: Vec<u8>,
}

impl Ram {
    pub fn new(size_bytes: u32) -> Self {
        Self { bytes: vec![0; size_bytes as usize] }
    }

    pub fn len(&self) -> u32 {
        self.bytes.len() as u32
    }

    pub fn read_word_le(&self, addr: u32) -> BusResult<u32> {
        let a = addr as usize;
        if a + 4 > self.bytes.len() {
            return Err(BusError::AddressOutOfBounds(addr));
        }
        Ok(u32::from_le_bytes(self.bytes[a..a + 4].try_into().unwrap()))
    }

    pub fn write_word_le(&mut self, addr: u32, value: u32) -> BusResult<()> {
        let a = addr as usize;
        if a + 4 > self.bytes.len() {
            return Err(BusError::AddressOutOfBounds(addr));
        }
        self.bytes[a..a + 4].copy_from_slice(&value.to_le_bytes());
        Ok(())
    }
}
