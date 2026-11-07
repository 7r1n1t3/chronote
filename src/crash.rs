//! Makes startup failures visible.
//!
//! Release builds on Windows have no console, so a panic or renderer error
//! would otherwise close the app without a trace. Failures are written to
//! `crash.log` in the data directory and shown in a native error dialog.

use std::fmt::Write as _;
use std::path::PathBuf;

use crate::project::Store;

/// Where the crash log goes: the data directory, or the temp directory.
#[must_use]
pub fn log_path() -> PathBuf {
    Store::default_location()
        .map_or_else(std::env::temp_dir, |s| s.root().to_path_buf())
        .join("crash.log")
}

/// Reports panics through [`report`] in addition to the default handler.
pub fn install() {
    let default = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let mut details = info.to_string();
        let backtrace = std::backtrace::Backtrace::force_capture();
        let _ = write!(details, "\n\nBacktrace:\n{backtrace}");
        log::error!("{details}");
        report("Chronote crashed", &details);
        default(info);
    }));
}

/// Logs `details` and shows them in an error dialog.
pub fn report(title: &str, details: &str) {
    let path = log_path();
    let logged = std::fs::create_dir_all(path.parent().unwrap_or(&path))
        .and_then(|()| std::fs::write(&path, format!("{title}\n\n{details}\n")))
        .is_ok();

    // Keep the dialog short; the full text is in the log.
    let summary: String = details.lines().take(12).collect::<Vec<_>>().join("\n");
    let mut message = format!("{summary}\n\n");
    if logged {
        let _ = write!(message, "Full details were saved to:\n{}", path.display());
    }
    let _ = rfd::MessageDialog::new()
        .set_level(rfd::MessageLevel::Error)
        .set_title(title)
        .set_description(message)
        .set_buttons(rfd::MessageButtons::Ok)
        .show();
}
