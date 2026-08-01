#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // Hide console window on Windows release builds

use eframe::NativeOptions;
use pm2_log_analyzer::app::Pm2App;

fn main() -> eframe::Result<()> {
    let native_options = NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("PM2 Log Analyzer Native - Pure Rust Ops Console")
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([900.0, 600.0])
            .with_resizable(true),
        multisampling: 0,
        vsync: true,
        ..Default::default()
    };

    eframe::run_native(
        "PM2 Log Analyzer Native",
        native_options,
        Box::new(|cc| Ok(Box::new(Pm2App::new(cc)))),
    )
}

