#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use pm2_log_analyzer::egui_app::EguiApp;

fn capture_window_size() -> [f32; 2] {
    let capture_mode = std::env::var_os("PM2_CAPTURE_SCREENSHOT").is_some()
        || std::env::var_os("PM2_CAPTURE_MONGO_SCREENSHOT").is_some();
    if !capture_mode {
        return [1280.0, 850.0];
    }

    let dimension = |name: &str, default: f32| {
        std::env::var(name)
            .ok()
            .and_then(|value| value.parse::<f32>().ok())
            .filter(|value| value.is_finite() && *value > 0.0)
            .unwrap_or(default)
    };

    [
        dimension("PM2_CAPTURE_WIDTH", 1280.0),
        dimension("PM2_CAPTURE_HEIGHT", 850.0),
    ]
}

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size(capture_window_size())
            .with_min_inner_size([900.0, 600.0])
            .with_title("PM2 Log Analyzer"),
        ..Default::default()
    };

    eframe::run_native(
        "PM2 Log Analyzer",
        options,
        Box::new(|cc| Ok(Box::new(EguiApp::new(cc)))),
    )
}
