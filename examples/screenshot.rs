//! Development helper: opens the app on a given week, waits, saves a
//! screenshot taken by the renderer itself and exits.
//!
//! Usage: `cargo run --example screenshot -- WEEK OUT.ppm [SECONDS]`

use std::path::PathBuf;
use std::time::{Duration, Instant};

use embryo::app::{App, Message};
use iced::window::{self, Screenshot};
use iced::{Element, Size, Subscription, Task};

struct Shot {
    app: App,
    out: PathBuf,
    delay: Duration,
    started: Instant,
    taken: bool,
}

#[derive(Debug, Clone)]
enum Msg {
    App(Message),
    Captured(Screenshot),
}

impl Shot {
    fn update(&mut self, msg: Msg) -> Task<Msg> {
        match msg {
            Msg::App(message) => {
                if let Message::Tick(now) = message
                    && !self.taken
                    && now.duration_since(self.started) >= self.delay
                {
                    self.taken = true;
                    return window::latest()
                        .and_then(window::screenshot)
                        .map(Msg::Captured);
                }
                self.app.update(message);
                Task::none()
            }
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
        self.app.subscription().map(Msg::App)
    }
}

fn save_ppm(path: &PathBuf, shot: &Screenshot) -> std::io::Result<()> {
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

fn main() -> iced::Result {
    let mut args = std::env::args().skip(1);
    let week = args.next().and_then(|a| a.parse().ok()).unwrap_or(1);
    let out = PathBuf::from(args.next().unwrap_or_else(|| "screenshot.ppm".to_owned()));
    let delay = args
        .next()
        .and_then(|a| a.parse().ok())
        .map_or(Duration::from_secs(2), Duration::from_secs_f32);

    iced::application(
        move || Shot {
            app: App::starting_at(week),
            out: out.clone(),
            delay,
            started: Instant::now(),
            taken: false,
        },
        Shot::update,
        Shot::view,
    )
    .subscription(Shot::subscription)
    .theme(embryo::theme::theme())
    .antialiasing(false)
    .window(window::Settings {
        size: Size::new(1320.0, 840.0),
        ..window::Settings::default()
    })
    .run()
}
