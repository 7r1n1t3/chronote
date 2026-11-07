// Hide the console window on Windows release builds.
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

use chronote::app::App;
use iced::{Size, window};

/// Usage: `embryo [WEEK]`. With WEEK (1-40) the Embryo demo opens directly
/// on that week; otherwise the project menu is shown.
fn main() -> iced::Result {
    let start = std::env::args()
        .nth(1)
        .and_then(|arg| arg.parse::<u8>().ok());

    let mut app = iced::application(
        move || match start {
            Some(week) => App::new().with_demo(week),
            None => App::new(),
        },
        App::update,
        App::view,
    )
    .title(App::title)
    .subscription(App::subscription)
    .theme(chronote::theme::theme())
    .default_font(chronote::ui::regular())
    // MSAA makes iced 0.14's wgpu renderer drop canvas meshes on some
    // drivers, so it stays off.
    .antialiasing(false)
    .window(window::Settings {
        size: Size::new(1320.0, 840.0),
        min_size: Some(Size::new(1000.0, 680.0)),
        ..window::Settings::default()
    });
    for font in chronote::ui::BUNDLED_FONTS {
        app = app.font(font);
    }
    app.run()
}
