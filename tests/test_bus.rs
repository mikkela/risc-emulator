use risc_emulator::bus::{BusError, BusResult, CpuBus};

pub const IO_START: u32 = 0xFFFF_FFC0;
pub const ROM_START: u32 = 0xFFFF_F800;

#[derive(Default)]
pub struct TestBus {
    pub ram: Vec<u32>, // word addressed
    pub rom: Vec<u32>, // word addressed
}

impl TestBus {
    pub fn new(ram_words: usize, rom_words: usize) -> Self {
        Self { ram: vec![0; ram_words], rom: vec![0; rom_words] }
    }

    fn read_word_raw(&self, addr: u32) -> BusResult<u32> {
        if addr >= ROM_START {
            let i = ((addr - ROM_START) / 4) as usize;
            return self.rom.get(i).copied().ok_or(BusError::Unmapped(addr));
        }
        if addr >= IO_START {
            return Ok(0);
        }
        let i = (addr / 4) as usize;
        self.ram.get(i).copied().ok_or(BusError::AddressOutOfBounds(addr))
    }

    fn write_word_raw(&mut self, addr: u32, v: u32) -> BusResult<()> {
        if addr >= ROM_START {
            return Err(BusError::Device("write to ROM".into()));
        }
        if addr >= IO_START {
            return Ok(());
        }
        let i = (addr / 4) as usize;
        let slot = self.ram.get_mut(i).ok_or(BusError::AddressOutOfBounds(addr))?;
        *slot = v;
        Ok(())
    }
}

impl CpuBus for TestBus {
    fn read_word_for_cpu(&mut self, addr: u32, _progress: &mut u32) -> BusResult<u32> {
        self.read_word_raw(addr & !3)
    }

    fn write_word(&mut self, addr: u32, value: u32) -> BusResult<()> {
        self.write_word_raw(addr & !3, value)
    }
}
