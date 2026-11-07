//! Minimal logger: writes to stderr and to a log file in the data directory.
//!
//! The level for Chronote's own messages comes from `CHRONOTE_LOG`
//! (`error`, `warn`, `info`, `debug` or `trace`; default `info`). Other
//! crates are limited to `warn`, except iced's own crates which log useful
//! startup details (such as the chosen renderer) at `info`.

use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::Instant;

use log::{Level, LevelFilter, Log, Metadata, Record};

const FILE_NAME: &str = "chronote.log";

struct Logger {
    own: LevelFilter,
    file: Option<Mutex<File>>,
    started: Instant,
}

impl Logger {
    fn limit(&self, target: &str) -> LevelFilter {
        if target.starts_with("chronote") {
            self.own
        } else if target.starts_with("iced") {
            self.own.min(LevelFilter::Info)
        } else {
            LevelFilter::Warn
        }
    }
}

impl Log for Logger {
    fn enabled(&self, metadata: &Metadata<'_>) -> bool {
        metadata.level() <= self.limit(metadata.target())
    }

    fn log(&self, record: &Record<'_>) {
        if !self.enabled(record.metadata()) {
            return;
        }
        let mut message = record.args().to_string();
        if !record.target().starts_with("chronote") {
            // Libraries log pretty-printed structs; keep them on one line.
            message = message.split_whitespace().collect::<Vec<_>>().join(" ");
        }
        let line = format_line(
            self.started.elapsed().as_secs_f64(),
            record.level(),
            record.target(),
            &message,
        );
        let _ = io::stderr().write_all(line.as_bytes());
        if let Some(file) = &self.file
            && let Ok(mut file) = file.lock()
        {
            let _ = file.write_all(line.as_bytes());
        }
    }

    fn flush(&self) {
        if let Some(file) = &self.file
            && let Ok(mut file) = file.lock()
        {
            let _ = file.flush();
        }
    }
}

fn format_line(elapsed: f64, level: Level, target: &str, message: &str) -> String {
    let tag = match level {
        Level::Error => "ERR",
        Level::Warn => "WAR",
        Level::Info => "INF",
        Level::Debug => "DBG",
        Level::Trace => "TRC",
    };
    format!("[{elapsed:>9.3}s {tag} {target}] {message}\n")
}

/// Describes the operating system, e.g. "windows 10.0.22631 (Windows 11)".
#[must_use]
pub fn os_description() -> String {
    let base = format!("{} ({})", std::env::consts::OS, std::env::consts::ARCH);
    match windows_ver() {
        Some(ver) => format!("{base}, {}", describe_windows_ver(&ver)),
        None => base,
    }
}

/// Output of `cmd /C ver`, without flashing a console window.
#[cfg(windows)]
fn windows_ver() -> Option<String> {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    let out = std::process::Command::new("cmd")
        .args(["/C", "ver"])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .ok()?;
    Some(String::from_utf8_lossy(&out.stdout).trim().to_owned())
}

#[cfg(not(windows))]
fn windows_ver() -> Option<String> {
    None
}

/// Turns "Microsoft Windows [Version 10.0.22631.4460]" into a description
/// that says whether it is Windows 11 (build 22000 and later).
#[must_use]
pub fn describe_windows_ver(ver: &str) -> String {
    let build = ver
        .split("Version ")
        .nth(1)
        .and_then(|v| v.trim_end_matches(']').split('.').nth(2))
        .and_then(|b| b.parse::<u32>().ok());
    match build {
        Some(b) if b >= 22_000 => format!("{ver} = Windows 11"),
        Some(_) => format!("{ver} = Windows 10 or older"),
        None => ver.to_owned(),
    }
}

/// Environment variables that change rendering or logging, if set.
#[must_use]
pub fn overrides() -> Vec<String> {
    [
        "ICED_BACKEND",
        "WGPU_BACKEND",
        "WGPU_ADAPTER_NAME",
        "CHRONOTE_LOG",
        "CHRONOTE_DATA_DIR",
    ]
    .into_iter()
    .filter_map(|name| std::env::var(name).ok().map(|v| format!("{name}={v}")))
    .collect()
}

/// Parses a `CHRONOTE_LOG` value; unknown values fall back to `info`.
#[must_use]
pub fn parse_level(value: Option<&str>) -> LevelFilter {
    match value.map(|v| v.trim().to_ascii_lowercase()).as_deref() {
        Some("off") => LevelFilter::Off,
        Some("error") => LevelFilter::Error,
        Some("warn" | "warning") => LevelFilter::Warn,
        Some("debug") => LevelFilter::Debug,
        Some("trace") => LevelFilter::Trace,
        _ => LevelFilter::Info,
    }
}

/// Opens `<dir>/chronote.log`, keeping the previous run as `.old`.
fn open_file(dir: &Path) -> io::Result<(File, PathBuf)> {
    fs::create_dir_all(dir)?;
    let path = dir.join(FILE_NAME);
    if path.exists() {
        let _ = fs::rename(&path, dir.join(format!("{FILE_NAME}.old")));
    }
    Ok((File::create(&path)?, path))
}

/// Installs the logger. `dir` is where the log file goes (usually the
/// data directory). Returns the log file path when one could be opened.
pub fn init(dir: Option<&Path>) -> Option<PathBuf> {
    let own = parse_level(std::env::var("CHRONOTE_LOG").ok().as_deref());
    let (file, path) = match dir.map(open_file) {
        Some(Ok((file, path))) => (Some(Mutex::new(file)), Some(path)),
        Some(Err(e)) => {
            eprintln!("could not open log file: {e}");
            (None, None)
        }
        None => (None, None),
    };
    let logger = Logger {
        own,
        file,
        started: Instant::now(),
    };
    let max = own.max(LevelFilter::Warn);
    if log::set_boxed_logger(Box::new(logger)).is_ok() {
        log::set_max_level(max);
    }
    path
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_levels() {
        assert_eq!(parse_level(None), LevelFilter::Info);
        assert_eq!(parse_level(Some("DEBUG")), LevelFilter::Debug);
        assert_eq!(parse_level(Some("warning")), LevelFilter::Warn);
        assert_eq!(parse_level(Some("nonsense")), LevelFilter::Info);
    }

    #[test]
    fn formats_lines_with_short_tags() {
        let line = format_line(1.5, Level::Warn, "chronote::app", "careful");
        assert_eq!(line, "[    1.500s WAR chronote::app] careful\n");
        assert!(format_line(0.0, Level::Info, "x", "y").contains(" INF x] y"));
    }

    #[test]
    fn filters_by_target() {
        let logger = Logger {
            own: LevelFilter::Debug,
            file: None,
            started: Instant::now(),
        };
        assert_eq!(logger.limit("chronote::editor"), LevelFilter::Debug);
        assert_eq!(logger.limit("iced_wgpu"), LevelFilter::Info);
        assert_eq!(logger.limit("wgpu_core::device"), LevelFilter::Warn);
    }

    #[test]
    fn recognises_windows_11() {
        let w11 = describe_windows_ver("Microsoft Windows [Version 10.0.22631.4460]");
        assert!(w11.ends_with("= Windows 11"));
        let w10 = describe_windows_ver("Microsoft Windows [Version 10.0.19045.3803]");
        assert!(w10.ends_with("Windows 10 or older"));
        assert_eq!(describe_windows_ver("garbage"), "garbage");
    }

    #[test]
    fn rotates_previous_log() {
        let dir = tempfile::tempdir().unwrap();
        let (mut first, path) = open_file(dir.path()).unwrap();
        first.write_all(b"run 1").unwrap();
        drop(first);
        let _second = open_file(dir.path()).unwrap();
        assert_eq!(
            fs::read(dir.path().join("chronote.log.old")).unwrap(),
            b"run 1"
        );
        assert_eq!(fs::read(path).unwrap(), b"");
    }
}
