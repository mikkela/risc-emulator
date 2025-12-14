use crate::{
    bus::{Bus, BusError, BusResult, CpuBus},
    memory::{framebuffer::Damage, ram::Ram, rom::Rom},
    bus::io_bus::IoBus,
};

#[derive(Debug)]
pub struct SystemBus {
    pub mem_size: u32,
    pub display_start: u32,

    pub fb_width_words: i32,
    pub fb_height: i32,
    pub damage: Damage,

    pub ram: Ram,
    pub rom: Rom,
    pub io: IoBus,
}

impl SystemBus {
    pub fn new(
        mem_size: u32,
        display_start: u32,
        fb_width_words: i32,
        fb_height: i32,
        damage: Damage,
        ram: Ram,
        rom: Rom,
        io: IoBus,
    ) -> Self {
        Self {
            mem_size,
            display_start,
            fb_width_words,
            fb_height,
            damage,
            ram,
            rom,
            io,
        }
    }

    pub fn reset_damage(&mut self) -> Damage {
        let dmg = self.damage;
        self.damage = Damage::cleared(self.fb_width_words, self.fb_height);
        dmg
    }

    #[inline]
    fn is_in_ram(&self, addr: u32) -> bool {
        addr < self.mem_size
    }

    #[inline]
    fn is_in_fb(&self, addr: u32) -> bool {
        addr >= self.display_start && addr < self.mem_size
    }
}

impl CpuBus for SystemBus {
    fn read_word_for_cpu(&mut self, addr: u32, progress: &mut u32) -> BusResult<u32> {
        let a = addr & !3;
        if self.is_in_ram(a) {
            return self.ram.read_word_le(a);
        }
        if self.rom.contains(a) {
            return self.rom.read_word(a);
        }
        self.io.read_word_with_progress(a, progress)
    }

    fn write_word(&mut self, addr: u32, value: u32) -> BusResult<()> {
        <Self as Bus>::write_word(self, addr, value)
    }
}

impl Bus for SystemBus {
    fn read_word(&mut self, addr: u32) -> BusResult<u32> {
        let a = addr & !3;
        if self.is_in_ram(a) {
            return self.ram.read_word_le(a);
        }
        if self.rom.contains(a) {
            return self.rom.read_word(a);
        }
        Err(BusError::Unmapped(a))
    }

    fn write_word(&mut self, addr: u32, value: u32) -> BusResult<()> {
        let a = addr & !3;

        if self.is_in_ram(a) {
            self.ram.write_word_le(a, value)?;
            if self.is_in_fb(a) {
                let fb_word_index = ((a - self.display_start) as i32) / 4;
                self.damage
                    .update_word_index(self.fb_width_words, self.fb_height, fb_word_index);
            }
            return Ok(());
        }

        if self.rom.contains(a) {
            return Err(BusError::Device("write to ROM".into()));
        }

        self.io.write_word(a, value)
    }
}

impl SystemBus {
    pub fn framebuffer_words_copy(&self) -> Vec<u32> {
        let start = self.display_start;
        let words = (self.fb_width_words * self.fb_height) as usize;
        let mut out = Vec::with_capacity(words);

        // Læs word-for-word via RAM (antag little-endian word layout som din emu)
        for i in 0..words {
            let addr = start + (i as u32) * 4;
            // hvis du har en intern ram.read_word_le(...) så brug den direkte
            let w = self.ram.read_word_le(addr).unwrap_or(0);
            out.push(w);
        }
        out
    }

    pub fn fb_width_px(&self) -> usize {
        (self.fb_width_words as usize) * 32
    }

    pub fn fb_height_px(&self) -> usize {
        self.fb_height as usize
    }

    /// Read-only “peek” til debugger/UI (ingen progress/side-effekter)
    pub fn peek_word_le(&self, addr: u32) -> crate::bus::BusResult<u32> {
        // RAM / framebuffer
        if addr < self.mem_size {
            return self.ram.read_word_le(addr);
        }
        // ROM
        if self.rom.contains(addr) {
            return self.rom.read_word(addr);
        }
        // IO: i UI vil vi typisk ikke trigge side-effekter,
        // så returnér 0 eller Unmapped. Vælg én:
        Err(crate::bus::BusError::Unmapped(addr))
    }
}