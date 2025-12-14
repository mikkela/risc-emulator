use crate::bus::{BusError, BusResult};

#[derive(Debug)]
pub struct Rom {
    words: Vec<u32>,
    start_addr: u32,
}

impl Rom {
    pub fn new(start_addr: u32, words: Vec<u32>) -> Self {
        Self { start_addr, words }
    }

    pub fn contains(&self, addr: u32) -> bool {
        addr >= self.start_addr && addr < self.start_addr + (self.words.len() as u32) * 4
    }

    pub fn read_word(&self, addr: u32) -> BusResult<u32> {
        if !self.contains(addr) {
            return Err(BusError::Unmapped(addr));
        }
        let idx = ((addr - self.start_addr) / 4) as usize;
        Ok(self.words[idx])
    }
}
