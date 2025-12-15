use std::collections::HashSet;

use eframe::egui;

use crate::Machine;

use super::{cpu_panel, debugger, framebuffer, topbar};

const CPU_HZ: u32 = 25_000_000;
const FPS: u32 = 60;

pub struct EmuApp {
    pub(crate) emu: EmuState,
    pub(crate) ui: UiState,
}

pub(crate) struct EmuState {
    pub(crate) machine: Machine,
    pub(crate) running: bool,
    pub(crate) cycles_per_frame: u32,
    pub(crate) run_to_target: Option<u32>,
    pub(crate) breakpoints: HashSet<u32>,
}

pub(crate) struct UiState {
    // framebuffer texture
    pub(crate) tex: Option<egui::TextureHandle>,
    pub(crate) fb_rgba: Vec<egui::Color32>,
    pub(crate) fb_w: usize,
    pub(crate) fb_h: usize,

    // palette
    pub(crate) white: egui::Color32,
    pub(crate) black: egui::Color32,

    // debugger UI
    pub(crate) step_n: u32,
    pub(crate) disasm_before: i32,
    pub(crate) disasm_after: i32,
    pub(crate) follow_pc: bool,
    pub(crate) disasm_scroll_to_pc: bool,
    pub(crate) cursor_pc: Option<u32>,

    // right panel tabs
    pub(crate) right_tab: RightTab,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RightTab {
    Cpu,
    Breakpoints,
}

impl EmuApp {
    pub fn new() -> Self {
        let fb_w = 1024;
        let fb_h = 768;

        let mut machine = Machine::new(fb_w as i32, fb_h as i32);
        let _ = machine.attach_disk("oberon.dsk");

        Self {
            emu: EmuState {
                machine,
                running: true,
                cycles_per_frame: CPU_HZ / FPS,
                run_to_target: None,
                breakpoints: HashSet::new(),
            },
            ui: UiState {
                tex: None,
                fb_rgba: vec![egui::Color32::BLACK; fb_w * fb_h],
                fb_w,
                fb_h,

                // Solarized-ish (0xRRGGBB)
                black: egui::Color32::from_rgb(0x65, 0x7b, 0x83),
                white: egui::Color32::from_rgb(0xfd, 0xf6, 0xe3),

                step_n: 1,
                disasm_before: 20,
                disasm_after: 40,
                follow_pc: true,
                disasm_scroll_to_pc: true,
                cursor_pc: None,

                right_tab: RightTab::Cpu,
            },
        }
    }

    pub(crate) fn pc_aligned(&self) -> u32 {
        self.emu.machine.cpu.view().pc & !3
    }

    pub(crate) fn step_instructions(&mut self, n: u32) {
        let _ = self.emu.machine.cpu.run(&mut self.emu.machine.bus, n);
    }

    pub(crate) fn read_word_at(&mut self, addr: u32) -> Option<u32> {
        match self.emu.machine.bus.peek_word_le(addr) {
            Ok(w) => Some(w),
            Err(_) => None,
        }
    }

    pub(crate) fn tick(&mut self, ctx: &egui::Context) {
        if !self.emu.running {
            return;
        }

        let mut remaining = self.emu.cycles_per_frame;

        while remaining > 0 {
            let pc = self.pc_aligned();

            if self.emu.run_to_target == Some(pc) {
                self.emu.running = false;
                self.emu.run_to_target = None;
                break;
            }

            if self.emu.breakpoints.contains(&pc) {
                self.emu.running = false;
                self.emu.run_to_target = None;
                break;
            }

            self.step_instructions(1);
            remaining -= 1;
        }

        if self.ui.follow_pc {
            self.ui.cursor_pc = Some(self.pc_aligned());
        }

        ctx.request_repaint();
    }
}

impl eframe::App for EmuApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 1) emulator tick
        self.tick(ctx);

        // 2) UI layout
        topbar::show(ctx, self);
        debugger::show(ctx, self);
        cpu_panel::show(ctx, self);
        framebuffer::show(ctx, self);
    }
}
