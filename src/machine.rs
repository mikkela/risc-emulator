use crate::boot::BOOTLOADER;
use crate::bus::io_bus::IoBus;
use crate::bus::system_bus::SystemBus;
use crate::cpu::Cpu;
use crate::devices;
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
        let input = Box::new(devices::input::Input::default()); // lav den som i C (mouse+keybuf)
        let io = IoBus::new(IO_START, timer, switches, input);

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
            devices::{input::Input, switches::Switches, timer::Timer},
            memory::{framebuffer::Damage, ram::Ram, rom::Rom},
        };

        let ram = Ram::new(mem_size);
        let rom = Rom::new(crate::machine::ROM_START, boot_rom_words);

        let timer = Box::new(Timer::default());
        let switches = Box::new(Switches::default());
        let input = Box::new(Input::default());

        let io = IoBus::new(crate::machine::IO_START, timer, switches, input);

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