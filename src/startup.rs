//! Renderer selection and recovery from failed starts.
//!
//! Some Windows graphics drivers (or Vulkan layers installed by overlays)
//! hang or crash while the GPU renderer starts, before any Rust error can
//! be reported. To recover, a marker file is written before the window
//! opens and removed once the first frame is drawn. If the marker is still
//! there on the next launch, that start failed, and the app switches to
//! the software renderer until it is started with `--gpu`.

use std::fs;
use std::path::{Path, PathBuf};

const PENDING: &str = "startup.pending";
const SOFTWARE: &str = "software-renderer";

/// Which renderer to start with, and why.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Renderer {
    /// GPU via wgpu, with iced's own software fallback on errors.
    Gpu,
    /// CPU rendering (tiny-skia).
    Software(Reason),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reason {
    /// `--software` was passed.
    Requested,
    /// The previous start never drew a frame.
    PreviousStartFailed,
    /// An earlier failed start switched to software rendering.
    Remembered,
}

impl Reason {
    /// Message shown in the menu, if the user should know.
    #[must_use]
    pub fn notice(self) -> Option<&'static str> {
        match self {
            Self::Requested => None,
            Self::PreviousStartFailed | Self::Remembered => Some(
                "Chronote is using software rendering because the graphics driver failed \
                 to start the last time. Launch with --gpu to try the GPU again.",
            ),
        }
    }
}

/// Command-line options.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Options {
    /// Open the Embryo demo directly at this week.
    pub week: Option<u8>,
    pub software: bool,
    pub gpu: bool,
}

impl Options {
    #[must_use]
    pub fn parse(args: impl IntoIterator<Item = String>) -> Self {
        let mut options = Self::default();
        for arg in args {
            match arg.as_str() {
                "--software" => options.software = true,
                "--gpu" => options.gpu = true,
                other => {
                    if let Ok(week) = other.parse() {
                        options.week = Some(week);
                    }
                }
            }
        }
        options
    }
}

/// Decides the renderer from the options and the markers in `dir`.
pub fn choose(dir: Option<&Path>, options: &Options) -> Renderer {
    if options.software {
        return Renderer::Software(Reason::Requested);
    }
    let Some(dir) = dir else {
        return Renderer::Gpu;
    };
    let remembered = dir.join(SOFTWARE);
    if options.gpu {
        let _ = fs::remove_file(&remembered);
        return Renderer::Gpu;
    }
    if dir.join(PENDING).exists() {
        let _ = fs::write(&remembered, "software\n");
        return Renderer::Software(Reason::PreviousStartFailed);
    }
    if remembered.exists() {
        return Renderer::Software(Reason::Remembered);
    }
    Renderer::Gpu
}

fn pending(dir: &Path) -> PathBuf {
    dir.join(PENDING)
}

/// Records that a start is in progress.
pub fn begin(dir: &Path) {
    let _ = fs::create_dir_all(dir);
    if let Err(e) = fs::write(pending(dir), "starting\n") {
        log::warn!("could not write startup marker: {e}");
    }
}

/// Records that the start succeeded (the first frame was drawn).
pub fn succeeded(dir: &Path) {
    if let Err(e) = fs::remove_file(pending(dir))
        && e.kind() != std::io::ErrorKind::NotFound
    {
        log::warn!("could not remove startup marker: {e}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn opts(args: &[&str]) -> Options {
        Options::parse(args.iter().map(|s| (*s).to_owned()))
    }

    #[test]
    fn parses_arguments() {
        assert_eq!(opts(&["24"]).week, Some(24));
        assert!(opts(&["--software"]).software);
        let both = opts(&["--gpu", "12"]);
        assert!(both.gpu);
        assert_eq!(both.week, Some(12));
        assert_eq!(opts(&["nonsense"]), Options::default());
    }

    #[test]
    fn failed_start_switches_to_software_until_gpu_is_requested() {
        let dir = tempfile::tempdir().unwrap();
        let dir = dir.path();
        assert_eq!(choose(Some(dir), &Options::default()), Renderer::Gpu);

        begin(dir);
        assert_eq!(
            choose(Some(dir), &Options::default()),
            Renderer::Software(Reason::PreviousStartFailed)
        );
        succeeded(dir);
        assert_eq!(
            choose(Some(dir), &Options::default()),
            Renderer::Software(Reason::Remembered)
        );
        assert_eq!(choose(Some(dir), &opts(&["--gpu"])), Renderer::Gpu);
        assert_eq!(choose(Some(dir), &Options::default()), Renderer::Gpu);
    }

    #[test]
    fn software_flag_wins() {
        assert_eq!(
            choose(None, &opts(&["--software"])),
            Renderer::Software(Reason::Requested)
        );
        assert_eq!(choose(None, &Options::default()), Renderer::Gpu);
        assert!(Reason::PreviousStartFailed.notice().is_some());
        assert!(Reason::Requested.notice().is_none());
    }
}
