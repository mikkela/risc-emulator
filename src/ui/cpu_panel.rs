use eframe::egui;

use super::app::{EmuApp, RightTab};

pub fn show(ctx: &egui::Context, app: &mut EmuApp) {
    egui::SidePanel::right("right")
        .resizable(true)
        .default_width(320.0)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut app.ui.right_tab, RightTab::Cpu, "CPU");
                ui.selectable_value(&mut app.ui.right_tab, RightTab::Breakpoints, "BPs");
            });
            ui.separator();

            match app.ui.right_tab {
                RightTab::Cpu => cpu(ui, app),
                RightTab::Breakpoints => breakpoints(ui, app),
            }
        });
}

fn cpu(ui: &mut egui::Ui, app: &mut EmuApp) {
    ui.heading("CPU");
    let v = app.emu.machine.cpu.view();

    ui.monospace(format!("PC : 0x{:08X}", v.pc));
    ui.monospace(format!("H  : 0x{:08X}", v.h));
    ui.monospace(format!(
        "Flags: N={} Z={} C={} V={}",
        v.n as u8, v.z as u8, v.c as u8, v.v as u8
    ));

    ui.separator();
    ui.heading("Registers");
    egui::ScrollArea::vertical().show(ui, |ui| {
        for i in 0..16 {
            ui.monospace(format!("R{:02}: 0x{:08X}", i, v.r[i]));
        }
    });
}

fn breakpoints(ui: &mut egui::Ui, app: &mut EmuApp) {
    ui.heading("Breakpoints");

    let mut bps: Vec<u32> = app.emu.breakpoints.iter().copied().collect();
    bps.sort_unstable();

    egui::ScrollArea::vertical().show(ui, |ui| {
        if bps.is_empty() {
            ui.label("No breakpoints.");
            return;
        }

        for addr in bps {
            ui.horizontal(|ui| {
                ui.monospace(format!("0x{addr:08X}"));

                if ui.small_button("Go").clicked() {
                    app.ui.cursor_pc = Some(addr);
                    app.ui.disasm_scroll_to_pc = true;
                    app.ui.right_tab = RightTab::Cpu;
                }

                if ui.small_button("Run to").clicked() {
                    app.emu.run_to_target = Some(addr & !3);
                    app.emu.running = true;
                }

                if ui.small_button("Remove").clicked() {
                    app.emu.breakpoints.remove(&addr);
                }
            });
        }
    });

    ui.add_space(6.0);

    if ui.button("Clear all").clicked() {
        app.emu.breakpoints.clear();
    }
}
