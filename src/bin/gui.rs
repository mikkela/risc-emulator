use risc_emulator::ui::app::EmuApp;
use eframe::egui;

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "RISC Emulator",
        native_options,
        Box::new(|_cc| Ok(Box::new(EmuApp::new()))),
    )
}
