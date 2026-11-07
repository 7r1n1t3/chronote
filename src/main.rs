// Hide the console window on Windows release builds.
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

use std::time::Duration;

use chronote::app::App;
use chronote::project::Store;
use chronote::startup::{self, Options, Renderer};
use iced::{Size, window};

/// How long to wait for the first frame before warning in the log.
const FIRST_FRAME_TIMEOUT: Duration = Duration::from_secs(20);

/// Usage: `chronote [WEEK] [--software | --gpu]`.
///
/// With WEEK (1-40) the Embryo demo opens directly on that week; otherwise
/// the menu is shown. `--software` forces CPU rendering; `--gpu` retries
/// the GPU after a failed start switched to software rendering.
fn main() -> iced::Result {
    let options = Options::parse(std::env::args().skip(1));
    let store = Store::default_location();
    let renderer = startup::choose(store.as_ref().map(Store::root), &options);
    configure_renderer(&renderer);

    chronote::crash::install();
    let log_file = chronote::logging::init(store.as_ref().map(Store::root));
    log_startup(store.as_ref(), log_file.as_deref());

    match &renderer {
        Renderer::Gpu => log::info!(
            "starting the GPU renderer (backends: {})",
            std::env::var("WGPU_BACKEND").unwrap_or_else(|_| "all".to_owned())
        ),
        Renderer::Software(reason) => {
            log::warn!("starting the software renderer ({reason:?})");
        }
    }
    let notice = match &renderer {
        Renderer::Software(reason) => reason.notice(),
        Renderer::Gpu => None,
    };
    let start = options.week;

    let custom_font = load_custom_font(store.as_ref());

    let icon = chronote::ui::window_icon();
    if icon.is_none() {
        log::warn!("could not decode the window icon");
    }

    let mut app = iced::application(
        move || {
            let app = App::new().with_notice(notice);
            match start {
                Some(week) => app.with_demo(week),
                None => app,
            }
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
        icon,
        ..window::Settings::default()
    });
    for font in chronote::ui::BUNDLED_FONTS {
        app = app.font(font);
    }
    if let Some((_, bytes)) = custom_font {
        app = app.font(bytes);
    }
    if let Some(store) = &store {
        startup::begin(store.root());
    }
    spawn_first_frame_watchdog();

    let result = app.run();
    log::info!("exiting");
    if let Err(error) = &result {
        log::error!("could not start: {error}");
        chronote::crash::report("Chronote could not start", &error.to_string());
    }
    result
}

/// Picks the renderer through the environment variables iced and wgpu read.
fn configure_renderer(renderer: &Renderer) {
    let set = |name: &str, value: &str| {
        if std::env::var_os(name).is_none() {
            // SAFETY: called at the very start of `main`, before any other
            // thread exists, so nothing can read the environment concurrently.
            #[allow(unsafe_code)]
            unsafe {
                std::env::set_var(name, value);
            }
        }
    };
    match renderer {
        Renderer::Software(_) => set("ICED_BACKEND", "tiny-skia"),
        // Only load Direct3D 12 on Windows: probing Vulkan and OpenGL is
        // where broken drivers and overlay layers tend to hang or crash.
        Renderer::Gpu if cfg!(windows) => set("WGPU_BACKEND", "dx12"),
        Renderer::Gpu => {}
    }
}

/// Logs version, OS, overrides and where data and logs go.
fn log_startup(store: Option<&Store>, log_file: Option<&std::path::Path>) {
    log::info!(
        "Chronote {} starting on {}",
        env!("CARGO_PKG_VERSION"),
        chronote::logging::os_description()
    );
    let overrides = chronote::logging::overrides();
    if !overrides.is_empty() {
        log::info!("environment overrides: {}", overrides.join(", "));
    }
    match (store, log_file) {
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
}

/// Loads the saved custom font (the app-wide font is fixed at startup).
fn load_custom_font(store: Option<&Store>) -> Option<(String, Vec<u8>)> {
    let custom_font = store.and_then(|store| {
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
    custom_font
}

/// Warns in the log when the first frame takes suspiciously long.
fn spawn_first_frame_watchdog() {
    std::thread::spawn(|| {
        std::thread::sleep(FIRST_FRAME_TIMEOUT);
        if !chronote::app::first_frame_drawn() {
            log::warn!(
                "no frame drawn after {}s: the graphics driver may be stuck; \
                 the next start will use the software renderer",
                FIRST_FRAME_TIMEOUT.as_secs()
            );
        }
    });
}
