// Hide the console window on Windows release builds.
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

use embryo::demo::Demo;
use iced::{Size, window};

/// Usage: `embryo [WEEK]`, where WEEK (1-40) is the week to open on.
fn main() -> iced::Result {
    let start = std::env::args()
        .nth(1)
        .and_then(|arg| arg.parse::<u8>().ok())
        .unwrap_or(1);

    iced::application(move || Demo::starting_at(start), Demo::update, Demo::view)
        .title(Demo::title)
        .subscription(Demo::subscription)
        .theme(embryo::theme::theme())
        // MSAA makes iced 0.14's wgpu renderer drop canvas meshes on some
        // drivers, so it stays off.
        .antialiasing(false)
        .window(window::Settings {
            size: Size::new(1320.0, 840.0),
            min_size: Some(Size::new(1000.0, 680.0)),
            ..window::Settings::default()
        })
        .run()
}
