//! Development helper: opens a screen, waits, saves a screenshot taken
//! by the renderer itself and exits.
//!
//! Usage: `cargo run --example screenshot -- SCREEN OUT.ppm [SECONDS]`
//! where SCREEN is a demo week (`1`–`40`), `menu` or `editor`. The menu
//! and editor use a temporary data directory seeded with sample events.

use std::path::{Path, PathBuf};
use std::time::Duration;

use chronote::app::{App, Message};
use chronote::editor;
use chronote::menu;
use chronote::project::{Entry, Event, EventKind, Library, Store};
use iced::window::{self, Screenshot};
use iced::{Element, Size, Subscription, Task, time};

struct Shot {
    app: App,
    out: PathBuf,
    delay: Duration,
    taken: bool,
    _data: Option<tempfile::TempDir>,
}

#[derive(Debug, Clone)]
enum Msg {
    App(Message),
    Capture,
    Captured(Screenshot),
}

impl Shot {
    fn update(&mut self, msg: Msg) -> Task<Msg> {
        match msg {
            Msg::App(message) => self.app.update(message).map(Msg::App),
            Msg::Capture if !self.taken => {
                self.taken = true;
                window::latest()
                    .and_then(window::screenshot)
                    .map(Msg::Captured)
            }
            Msg::Capture => Task::none(),
            Msg::Captured(shot) => {
                if let Err(error) = save_ppm(&self.out, &shot) {
                    eprintln!("failed to save {}: {error}", self.out.display());
                }
                iced::exit()
            }
        }
    }

    fn view(&self) -> Element<'_, Msg> {
        self.app.view().map(Msg::App)
    }

    fn subscription(&self) -> Subscription<Msg> {
        Subscription::batch([
            self.app.subscription().map(Msg::App),
            time::every(self.delay).map(|_| Msg::Capture),
        ])
    }
}

fn save_ppm(path: &Path, shot: &Screenshot) -> std::io::Result<()> {
    let mut bytes = format!("P6\n{} {}\n255\n", shot.size.width, shot.size.height).into_bytes();
    bytes.extend(
        shot.rgba
            .as_chunks::<4>()
            .0
            .iter()
            .flat_map(|px| [px[0], px[1], px[2]]),
    );
    std::fs::write(path, bytes)
}

/// A library with one project and a few events, saved to `dir`.
fn seed(dir: &Path) -> std::io::Result<()> {
    let mut library = Library::default();
    let id = library.create_project("Heart development", 1_790_000_000);
    library.create_project("Sepsis: clinical course", 1_791_000_000);
    let ids: Vec<u64> = (0..5).map(|_| library.next_id()).collect();
    if let Some(project) = library.project_mut(id) {
        "week".clone_into(&mut project.unit);
        project.length = 12.0;
        let event = |id, title: &str, kind| Event {
            id,
            title: title.to_owned(),
            description: "Derived from splanchnic mesoderm.".to_owned(),
            kind,
            attachments: Vec::new(),
        };
        let lasting = |start, end| EventKind::Lasting { start, end };
        project.events = vec![
            event(ids[0], "Cardiogenic field", lasting(3.0, 4.0)),
            event(ids[1], "Heart tube looping", lasting(4.0, 5.0)),
            event(ids[2], "First heartbeat", EventKind::Instant { at: 5.5 }),
            event(ids[3], "Septation", lasting(5.0, 8.0)),
            event(ids[4], "Valves formed", EventKind::Instant { at: 9.0 }),
        ];
    }
    Store::at(dir).save(&library)
}

fn main() -> iced::Result {
    let mut args = std::env::args().skip(1);
    let screen = args.next().unwrap_or_else(|| "menu".to_owned());
    let out = PathBuf::from(args.next().unwrap_or_else(|| "screenshot.ppm".to_owned()));
    let delay = args
        .next()
        .and_then(|a| a.parse().ok())
        .map_or(Duration::from_secs(2), Duration::from_secs_f32);

    iced::application(
        move || {
            let mut data = None;
            let app = if let Ok(week) = screen.parse::<u8>() {
                App::with_store(None).with_demo(week)
            } else {
                let dir = tempfile::tempdir().expect("temporary directory");
                seed(dir.path()).expect("seed library");
                let mut app = App::with_store(Some(Store::at(dir.path())));
                if screen == "editor" {
                    let entry = Entry::Project(1);
                    let _ = app.update(Message::Menu(menu::Message::Open(entry)));
                    let _ = app.update(Message::Editor(editor::Message::Select(Some(5))));
                }
                data = Some(dir);
                app
            };
            Shot {
                app,
                out: out.clone(),
                delay,
                taken: false,
                _data: data,
            }
        },
        Shot::update,
        Shot::view,
    )
    .subscription(Shot::subscription)
    .theme(chronote::theme::theme())
    .antialiasing(false)
    .window(window::Settings {
        size: Size::new(1320.0, 840.0),
        ..window::Settings::default()
    })
    .run()
}
