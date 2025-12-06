use crate::cpu::Cpu;
use crate::system_bus::SystemBus;

pub struct Emulator {
    pub cpu: Cpu,
    pub bus: SystemBus,
}

impl Emulator {
    pub fn new() -> Self {
        /*let ram = Ram::new(/*1024 * 1024, 0x0000_0000*/);
        let display = Display::new(/*320, 200, 0x8000_0000*/);

        let bus = SystemBus::new(ram, display /*, ps2, disk osv. */);
        let cpu = Cpu::new(0);
*/
        Self { cpu: Cpu {}, bus: SystemBus {} }
    }

    /// Kør N CPU-steps
    pub fn step_many(&mut self, steps: usize) {
        for _ in 0..steps {
            /*if self.cpu.halted {
                break;
            }
            self.cpu.step(&mut self.bus);*/
        }
    }

    /*/// Nem adgang til framebuffer til UI
    pub fn framebuffer(&self) -> &[u8] {
        self.bus.display_buffer()
    }*/
}