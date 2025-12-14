use eframe::egui;

use super::app::EmuApp;

pub fn show(ctx: &egui::Context, app: &mut EmuApp) {
    egui::CentralPanel::default().show(ctx, |ui| {
        refresh_framebuffer_full(app, ctx);

        if let Some(tex) = &app.ui.tex {
            let avail = ui.available_size();
            let scale = (avail.x / app.ui.fb_w as f32)
                .min(avail.y / app.ui.fb_h as f32)
                .max(0.1);

            let size = egui::vec2(app.ui.fb_w as f32 * scale, app.ui.fb_h as f32 * scale);
            ui.image((tex.id(), size));
        }
    });
}

fn ensure_texture(app: &mut EmuApp, ctx: &egui::Context) {
    if app.ui.tex.is_none() {
        let img = egui::ColorImage {
            size: [app.ui.fb_w, app.ui.fb_h],
            pixels: app.ui.fb_rgba.clone(),
            source_size: egui::vec2(app.ui.fb_w as f32, app.ui.fb_h as f32),
        };
        app.ui.tex = Some(ctx.load_texture("framebuffer", img, egui::TextureOptions::NEAREST));
    }
}

fn refresh_framebuffer_full(app: &mut EmuApp, ctx: &egui::Context) {
    ensure_texture(app, ctx);

    let words = app.emu.machine.bus.framebuffer_words_copy();
    let fb_width_words = app.ui.fb_w / 32;

    for y in 0..app.ui.fb_h {
        let src_line = (app.ui.fb_h - 1) - y;
        let base = src_line * fb_width_words;

        for xw in 0..fb_width_words {
            let mut bits = words[base + xw];
            let out_x = xw * 32;

            for b in 0..32 {
                let on = (bits & 1) != 0;
                bits >>= 1;
                app.ui.fb_rgba[y * app.ui.fb_w + (out_x + b)] =
                    if on { app.ui.white } else { app.ui.black };
            }
        }
    }

    if let Some(tex) = &mut app.ui.tex {
        let img = egui::ColorImage {
            size: [app.ui.fb_w, app.ui.fb_h],
            pixels: app.ui.fb_rgba.clone(),
            source_size: egui::vec2(app.ui.fb_w as f32, app.ui.fb_h as f32),
        };
        tex.set(img, egui::TextureOptions::NEAREST);
    }
}
