use eframe::egui;

use super::app::EmuApp;

pub fn show(ctx: &egui::Context, app: &mut EmuApp) {
    egui::TopBottomPanel::top("top").show(ctx, |ui| {
        ui.horizontal(|ui| {
            if ui.button(if app.emu.running { "Pause" } else { "Run" }).clicked() {
                app.emu.running = !app.emu.running;
            }

            ui.menu_button("File", |ui| {
                if ui.button("Attach Disk 1 (SPI1)…").clicked() {
                    ui.close_menu();
                    if let Some(path) = rfd::FileDialog::new().pick_file() {
                        if let Err(e) = app.emu.machine.attach_disk (1, &path) {
                            app.emu.last_error = Some(format!("Attach disk1 failed: {e:?}"));
                        } else {
                            app.emu.disk1_path = Some(path);
                        }
                    }
                }
                if ui.button("Eject Disk 1 (SPI1)…").clicked() {
                    ui.close_menu();
                    if let Err(e) = app.emu.machine.eject_disk(1) {
                        app.emu.last_error = Some(format!("Eject disk1 failed: {e:?}"));
                    } else {
                        app.emu.disk1_path = None;
                    }
                }

                ui.separator();

                if ui.button("Attach Disk 2 (SPI2)…").clicked() {
                    ui.close_menu();
                    if let Some(path) = rfd::FileDialog::new().pick_file() {
                        if let Err(e) = app.emu.machine.attach_disk (2, &path) {
                            app.emu.last_error = Some(format!("Attach disk2 failed: {e:?}"));
                        } else {
                            app.emu.disk2_path = Some(path);
                        }
                    }
                }
                if ui.button("Eject Disk 2 (SPI2)…").clicked() {
                    ui.close_menu();
                    if let Err(e) = app.emu.machine.eject_disk(2) {
                        app.emu.last_error = Some(format!("Eject disk2 failed: {e:?}"));
                    } else {
                        app.emu.disk2_path = None;
                    }
                }
            });

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

            ui.separator();
            ui.monospace(format!(
                "D1={}  D2={}",
                app.emu.disk1_path.as_ref().and_then(|p| p.file_name()).and_then(|s| s.to_str()).unwrap_or("-"),
                app.emu.disk2_path.as_ref().and_then(|p| p.file_name()).and_then(|s| s.to_str()).unwrap_or("-"),
            ));
            if let Some(err) = &app.emu.last_error {
                ui.colored_label(egui::Color32::LIGHT_RED, err);
            }
        });
    });
}
