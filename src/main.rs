#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use dioxus::prelude::*;
use dioxus_desktop::{Config, LogicalSize, WindowBuilder};
use pm2_log_analyzer::app::App;

fn main() {
    let window = WindowBuilder::new()
        .with_title("PM2 Log Analyzer")
        .with_inner_size(LogicalSize::new(1280.0, 850.0))
        .with_min_inner_size(LogicalSize::new(900.0, 600.0))
        .with_resizable(true);

    let mut config = Config::new().with_window(window);
    // Test harness hook: isolated WebView2 storage per run (fresh localStorage).
    if let Ok(dir) = std::env::var("PM2_ANALYZER_DATA_DIR") {
        config = config.with_data_directory(std::path::PathBuf::from(dir));
    }

    LaunchBuilder::new()
        .with_cfg(config)
        .launch(App);
}
