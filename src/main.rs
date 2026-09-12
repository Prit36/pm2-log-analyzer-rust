#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use iced::window;
use iced::Size;

use pm2_log_analyzer::app::App;

fn main() -> iced::Result {
    let window = window::Settings {
        size: Size::new(1280.0, 850.0),
        min_size: Some(Size::new(900.0, 600.0)),
        resizable: true,
        ..window::Settings::default()
    };

    let mut app = iced::application(App::boot, App::update, App::view)
        .title(App::title)
        .theme(App::theme)
        .subscription(App::subscription)
        .style(App::style)
        .default_font(pm2_log_analyzer::ui::style::REGULAR)
        .window(window);

    for face in pm2_log_analyzer::ui::fonts::FILES {
        app = app.font(face);
    }

    app.run()
}
