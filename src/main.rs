// Hide the console window on Windows release builds.
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

use chronote::app::App;
use chronote::project::Store;
use iced::{Size, window};

/// Usage: `embryo [WEEK]`. With WEEK (1-40) the Embryo demo opens directly
/// on that week; otherwise the project menu is shown.
fn main() -> iced::Result {
    chronote::crash::install();

    let start = std::env::args()
        .nth(1)
        .and_then(|arg| arg.parse::<u8>().ok());

    // The app-wide font is fixed at startup, so load the saved choice first.
    let custom_font = Store::default_location().and_then(|store| {
        let font = store.load().ok()?.font?;
        let bytes = store.read_font(&font).ok()?;
        Some((font.family, bytes))
    });
    if let Some((family, _)) = &custom_font {
        chronote::ui::set_family(Box::leak(family.clone().into_boxed_str()));
    }

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
    if let Some((_, bytes)) = custom_font {
        app = app.font(bytes);
    }
    let result = app.run();
    if let Err(error) = &result {
        chronote::crash::report("Chronote could not start", &error.to_string());
    }
    result
}
