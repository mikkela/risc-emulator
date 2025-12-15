use crate::boot::BOOTLOADER;
use crate::bus::io_bus::IoBus;
use crate::bus::system_bus::SystemBus;
use crate::bus::BusResult;
use crate::cpu::Cpu;
use crate::devices;
use crate::devices::disk::Disk;
use crate::memory::framebuffer::Damage;
use crate::memory::ram::Ram;
use crate::memory::rom::Rom;

pub const DEFAULT_MEM_SIZE: u32 = 0x0010_0000;
pub const DEFAULT_DISPLAY_START: u32 = 0x000E_7F00;
pub const IO_START: u32 = 0xFFFF_FFC0;
pub const ROM_START: u32 = 0xFFFF_F800;

pub struct Machine {
    pub cpu: Cpu,
    pub bus: SystemBus,
}

impl Machine {
    pub fn new(fb_width_px: i32, fb_height: i32) -> Self {
        let fb_width_words = fb_width_px / 32;
        let ram = Ram::new(DEFAULT_MEM_SIZE);
        let rom = Rom::new(ROM_START, BOOTLOADER.to_vec());

        let timer = Box::new(devices::timer::Timer::default());
        let switches = Box::new(devices::switches::Switches::default());
        let io = IoBus::new(IO_START, timer, switches);

        let bus = SystemBus::new(
            DEFAULT_MEM_SIZE,
            DEFAULT_DISPLAY_START,
            fb_width_words,
            fb_height,
            Damage::full(fb_width_words, fb_height),
            ram,
            rom,
            io,
        );

        let mut cpu = Cpu::default();
        cpu.reset();

        Self { cpu, bus }
    }

    pub fn new_for_tests(
        boot_rom_words: Vec<u32>,
        mem_size: u32,
        display_start: u32,
        fb_width_words: i32,
        fb_height: i32,
    ) -> Self {
        use crate::{
            bus::{io_bus::IoBus, system_bus::SystemBus},
            devices::{switches::Switches, timer::Timer},
            memory::{framebuffer::Damage, ram::Ram, rom::Rom},
        };

        let ram = Ram::new(mem_size);
        let rom = Rom::new(crate::machine::ROM_START, boot_rom_words);

        let timer = Box::new(Timer::default());
        let switches = Box::new(Switches::default());

        let io = IoBus::new(crate::machine::IO_START, timer, switches);

        let bus = SystemBus::new(
            mem_size,
            display_start,
            fb_width_words,
            fb_height,
            Damage::full(fb_width_words, fb_height),
            ram,
            rom,
            io,
        );

        let mut cpu = crate::cpu::Cpu::default();
        cpu.reset();

        Self { cpu, bus }
    }
}

impl Machine {
    pub fn mouse_moved(&mut self, x: i32, y: i32) {
        self.bus.io.input.mouse_moved(x, y);
    }

    pub fn mouse_button(&mut self, button: u32, down: bool) {
        self.bus.io.input.mouse_button(button, down);
    }

    pub fn keyboard_ps2(&mut self, bytes: &[u8]) {
        let _ = self.bus.io.input.keyboard_input(bytes);
    }

    pub fn attach_disk(&mut self, path: &str) -> BusResult<()> {
        let disk = Disk::new(Some(path))?;
        let _ =self.bus.io.set_spi(1, Box::new(disk));
        Ok(())
    }
}