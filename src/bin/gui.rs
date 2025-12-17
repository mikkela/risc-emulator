use risc_emulator::ui::app::EmuApp;
use eframe::egui;
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
struct Args {
    /// Primary disk image (mounts on SPI1)
    #[arg(long)]
    disk1: Option<PathBuf>,

    /// Secondary disk image (mounts on SPI2)
    #[arg(long)]
    disk2: Option<PathBuf>,
}

fn main() -> eframe::Result<()> {
    let args = Args::parse();
    let mut disk1 = None;
    let mut disk2 = None;
    if let Some(path) = args.disk1 { disk1 = Some(path); }
    if let Some(path) = args.disk2 { disk2 = Some(path); }

    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "RISC Emulator",
        native_options,
        Box::new(|_cc| Ok(Box::new(EmuApp::new(1024, 768, disk1, disk2)))),
    )
}
