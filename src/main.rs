// Hide the console window on Windows release builds.
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

use chronote::app::App;
use chronote::project::Store;
use iced::{Size, window};

/// Usage: `embryo [WEEK]`. With WEEK (1-40) the Embryo demo opens directly
/// on that week; otherwise the project menu is shown.
fn main() -> iced::Result {
    chronote::crash::install();
    let store = Store::default_location();
    let log_file = chronote::logging::init(store.as_ref().map(Store::root));
    log::info!(
        "Chronote {} starting on {}",
        env!("CARGO_PKG_VERSION"),
        chronote::logging::os_description()
    );
    let overrides = chronote::logging::overrides();
    if !overrides.is_empty() {
        log::info!("environment overrides: {}", overrides.join(", "));
    }
    match (&store, &log_file) {
        (Some(store), Some(file)) => log::info!(
            "data directory {}, log file {}",
            store.root().display(),
            file.display()
        ),
        (Some(store), None) => log::warn!(
            "data directory {} (log file unavailable)",
            store.root().display()
        ),
        (None, _) => log::warn!("no data directory found; nothing will be saved"),
    }

    let start = std::env::args()
        .nth(1)
        .and_then(|arg| arg.parse::<u8>().ok());

    // The app-wide font is fixed at startup, so load the saved choice first.
    let custom_font = store.as_ref().and_then(|store| {
        let font = store.load().ok()?.font?;
        match store.read_font(&font) {
            Ok(bytes) => {
                log::info!("using custom font {}", font.family);
                Some((font.family, bytes))
            }
            Err(e) => {
                log::warn!("custom font {} unavailable, using Inter: {e}", font.family);
                None
            }
        }
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
    log::info!("exiting");
    if let Err(error) = &result {
        log::error!("could not start: {error}");
        chronote::crash::report("Chronote could not start", &error.to_string());
    }
    result
}
