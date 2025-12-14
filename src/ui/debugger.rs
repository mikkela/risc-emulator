use eframe::egui;
use crate::disasm::disassemble_at;
use super::app::EmuApp;

pub fn show(ctx: &egui::Context, app: &mut EmuApp) {
    egui::SidePanel::left("left_debugger")
        .resizable(true)
        .default_width(520.0)
        .show(ctx, |ui| {
            ui.heading("Debugger");
            ui.separator();

            ui.horizontal(|ui| {
                ui.label("Before:");
                ui.add(egui::DragValue::new(&mut app.ui.disasm_before).speed(1.0).range(0..=500));
                ui.label("After:");
                ui.add(egui::DragValue::new(&mut app.ui.disasm_after).speed(1.0).range(1..=2000));
            });

            ui.add_space(6.0);
            disasm(ui, app);
        });
}

fn disasm(ui: &mut egui::Ui, app: &mut EmuApp) {
    let pc = app.pc_aligned();

    let start = pc.wrapping_sub((app.ui.disasm_before.max(0) as u32) * 4);
    let end = pc.wrapping_add((app.ui.disasm_after.max(0) as u32) * 4);

    egui::ScrollArea::vertical()
        .id_source("disasm_scroll")
        .auto_shrink([false; 2])
        .show(ui, |ui| {
            let mut addr = start;

            while addr <= end {
                let is_pc = addr == pc;
                let word = app.read_word_at(addr);
                let d = word.map(|w| disassemble_at(addr, w));

                let line = match (&d, word) {
                    (Some(dd), Some(w)) => format!("0x{addr:08X}:  0x{w:08X}  {}", dd.text),
                    _ => format!("0x{addr:08X}:  ----"),
                };

                ui.horizontal(|ui| {
                    // breakpoint gutter
                    let has_bp = app.emu.breakpoints.contains(&addr);
                    let bp_txt = if has_bp { "●" } else { " " };

                    if ui.small_button(bp_txt).clicked() {
                        if has_bp {
                            app.emu.breakpoints.remove(&addr);
                        } else {
                            app.emu.breakpoints.insert(addr);
                        }
                    }

                    // line selectable
                    let selected = app.ui.cursor_pc == Some(addr);
                    let label = if is_pc { format!("▶ {line}") } else { line };

                    let resp = ui.selectable_label(selected, label);

                    if resp.clicked() {
                        app.ui.cursor_pc = Some(addr);
                    }

                    // scroll-to-PC (center)
                    if app.ui.disasm_scroll_to_pc && is_pc {
                        resp.scroll_to_me(Some(egui::Align::Center));
                        app.ui.disasm_scroll_to_pc = false;
                    }

                    // branch target quick jump
                    if let Some(dd) = &d {
                        if let Some(tgt) = dd.branch_target {
                            if ui.small_button(format!("→ 0x{tgt:08X}")).clicked() {
                                app.ui.cursor_pc = Some(tgt & !3);
                                app.emu.run_to_target = None;
                                app.ui.disasm_scroll_to_pc = true;
                            }
                        }
                    }
                });

                addr = addr.wrapping_add(4);
            }
        });
}
