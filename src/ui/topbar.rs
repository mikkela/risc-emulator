use eframe::egui;

use super::app::EmuApp;

pub fn show(ctx: &egui::Context, app: &mut EmuApp) {
    egui::TopBottomPanel::top("top").show(ctx, |ui| {
        ui.horizontal(|ui| {
            if ui.button(if app.emu.running { "Pause" } else { "Run" }).clicked() {
                app.emu.running = !app.emu.running;
            }

            if ui.button("Step").clicked() {
                app.step_instructions(1);
                if app.ui.follow_pc {
                    app.ui.cursor_pc = Some(app.pc_aligned());
                    app.ui.disasm_scroll_to_pc = true;
                }
            }

            ui.add(egui::DragValue::new(&mut app.ui.step_n).speed(1.0).range(1..=1_000_000));
            if ui.button("Step N").clicked() {
                app.step_instructions(app.ui.step_n);
                if app.ui.follow_pc {
                    app.ui.cursor_pc = Some(app.pc_aligned());
                    app.ui.disasm_scroll_to_pc = true;
                }
            }

            if ui.button("Run to cursor").clicked() {
                if let Some(target) = app.ui.cursor_pc {
                    app.emu.run_to_target = Some(target & !3);
                    app.emu.running = true;
                }
            }

            if ui.button("Clear BPs").clicked() {
                app.emu.breakpoints.clear();
            }

            ui.separator();

            ui.checkbox(&mut app.ui.follow_pc, "Follow PC");
            if ui.button("Center PC").clicked() {
                app.ui.disasm_scroll_to_pc = true;
            }

            ui.separator();

            ui.label("Cycles/frame:");
            ui.add(
                egui::DragValue::new(&mut app.emu.cycles_per_frame)
                    .speed(10.0)
                    .range(1..=50_000_000),
            );
        });
    });
}
