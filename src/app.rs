//! Root of the application: owns the project library and switches screens.

use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

use iced::{Element, Subscription, Task, event, window};

use crate::demo::{self, Demo};
use crate::editor::{self, Editor, Outcome};
use crate::menu::{self, Menu};
use crate::project::{self, Entry, Library, ProjectId, Store, StoreError};
use crate::ui;

#[derive(Debug, Clone)]
pub enum Message {
    Menu(menu::Message),
    Demo(demo::Message),
    Editor(editor::Message),
    /// Window events, logged to diagnose display problems.
    Window(window::Id, window::Event),
    /// A display detail queried after the window opened.
    Display(String),
}

#[derive(Debug)]
enum Screen {
    Menu,
    Demo(Box<Demo>),
    Editor(Editor),
}

#[derive(Debug)]
pub struct App {
    store: Option<Store>,
    library: Library,
    menu: Menu,
    screen: Screen,
    /// Warning shown on the menu, e.g. when saving failed.
    notice: Option<String>,
    started: Instant,
}

impl App {
    /// Loads the library from the default data directory.
    #[must_use]
    pub fn new() -> Self {
        Self::with_store(Store::default_location())
    }

    /// Loads the library from `store`; `None` keeps everything in memory.
    #[must_use]
    pub fn with_store(store: Option<Store>) -> Self {
        let mut notice = None;
        let library = match store.as_ref().map(Store::load) {
            None => {
                log::warn!("no data directory found; timelines will not be saved");
                notice = Some("No data directory found; timelines will not be saved.".to_owned());
                Library::default()
            }
            Some(Ok(library)) => {
                log::info!(
                    "loaded {} timeline(s), sort {:?}, font {}",
                    library.projects.len(),
                    library.sort,
                    library
                        .font
                        .as_ref()
                        .map_or("default", |f| f.family.as_str())
                );
                library
            }
            Some(Err(error)) => {
                log::warn!("could not load library: {error}");
                notice = Some(match (&error, store.as_ref().map(Store::quarantine)) {
                    (StoreError::Parse(_), Some(Ok(path))) => format!(
                        "{error}. It was moved to {} and a new library was started.",
                        path.display()
                    ),
                    _ => error.to_string(),
                });
                Library::default()
            }
        };
        Self {
            store,
            library,
            menu: Menu::default(),
            screen: Screen::Menu,
            notice,
            started: Instant::now(),
        }
    }

    /// Opens straight into the demo at `week`.
    #[must_use]
    pub fn with_demo(mut self, week: u8) -> Self {
        log::info!("opening the Embryo demo at week {week}");
        self.screen = Screen::Demo(Box::new(Demo::starting_at(week)));
        self
    }

    #[must_use]
    pub fn title(&self) -> String {
        match &self.screen {
            Screen::Menu => "Chronote".to_owned(),
            Screen::Demo(demo) => demo.title(),
            Screen::Editor(editor) => self.library.project(editor.project).map_or_else(
                || "Chronote".to_owned(),
                |p| format!("{} — Chronote", p.name),
            ),
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Menu(message) => return self.update_menu(message),
            Message::Window(id, event) => return self.on_window(id, event),
            Message::Display(detail) => log::info!("display: {detail}"),
            Message::Demo(demo::Message::Back) => {
                log::info!("back to the menu");
                self.screen = Screen::Menu;
            }
            Message::Demo(message) => {
                if let Screen::Demo(demo) = &mut self.screen {
                    demo.update(message);
                }
            }
            Message::Editor(message) => {
                if let Screen::Editor(editor) = &mut self.screen {
                    match editor.update(&mut self.library, self.store.as_ref(), message) {
                        Outcome::None => {}
                        Outcome::Changed => self.save(),
                        Outcome::Back => {
                            log::info!("back to the menu");
                            self.screen = Screen::Menu;
                        }
                        Outcome::PickFiles(event) => {
                            log::debug!("showing attachment picker for event {event}");
                            return Task::perform(pick_attachments(), move |paths| {
                                Message::Editor(editor::Message::Attached(event, paths))
                            });
                        }
                    }
                }
            }
        }
        Task::none()
    }

    fn update_menu(&mut self, message: menu::Message) -> Task<Message> {
        if !matches!(message, menu::Message::Delete(_)) {
            self.menu.confirm_delete = None;
        }
        match message {
            menu::Message::Open(Entry::Demo) => {
                log::info!("opening the Embryo demo");
                self.screen = Screen::Demo(Box::default());
            }
            menu::Message::Open(Entry::Project(id)) => self.open_project(id),
            menu::Message::Create => {
                let id = self
                    .library
                    .create_project("Untitled timeline", project::now());
                log::info!("created timeline {id}");
                self.save();
                self.open_project(id);
            }
            menu::Message::Sort(mode) => {
                self.library.sort = mode;
                self.save();
            }
            menu::Message::Move(entry, delta) => {
                self.library.move_entry(entry, delta);
                self.save();
            }
            menu::Message::Delete(id) => {
                if self.menu.confirm_delete == Some(id) {
                    self.menu.confirm_delete = None;
                    self.library.remove_project(id);
                    log::info!("deleted timeline {id}");
                    if let Some(store) = &self.store
                        && let Err(e) = store.remove_project_files(id)
                    {
                        log::warn!("could not delete attachments of timeline {id}: {e}");
                        self.notice = Some(format!("Could not delete attachments: {e}"));
                    }
                    self.save();
                } else {
                    self.menu.confirm_delete = Some(id);
                }
            }
            menu::Message::CancelDelete | menu::Message::FontChosen(None) => {}
            menu::Message::ChooseFont => {
                log::debug!("showing font picker");
                return Task::perform(pick_font(), |path| {
                    Message::Menu(menu::Message::FontChosen(path))
                });
            }
            menu::Message::FontChosen(Some(path)) => self.set_font(&path),
            menu::Message::ResetFont => {
                log::info!("font reset to {}", ui::DEFAULT_FAMILY);
                if let (Some(store), Some(old)) = (&self.store, self.library.font.take())
                    && let Err(e) = store.remove_font(&old)
                {
                    log::warn!("could not delete the old font: {e}");
                    self.notice = Some(format!("Could not delete the old font: {e}"));
                }
                self.save();
            }
            menu::Message::Restart => return self.restart(),
        }
        Task::none()
    }

    /// Imports a font file and makes it the saved choice.
    fn set_font(&mut self, path: &std::path::Path) {
        let Some(store) = &self.store else {
            log::warn!("cannot store a custom font without a data directory");
            self.notice = Some("Custom fonts need a data directory.".to_owned());
            return;
        };
        match store.import_font(path) {
            Ok(font) => {
                log::info!("custom font set to {} ({})", font.family, font.file);
                if let Some(old) = self.library.font.replace(font) {
                    let _ = store.remove_font(&old);
                }
                self.notice = None;
                self.save();
            }
            Err(e) => {
                log::warn!("could not use font {}: {e}", path.display());
                self.notice = Some(format!("Could not use {}: {e}", path.display()));
            }
        }
    }

    /// Starts a fresh copy of the app and closes this one.
    fn restart(&mut self) -> Task<Message> {
        match std::env::current_exe().and_then(|exe| std::process::Command::new(exe).spawn()) {
            Ok(_) => {
                log::info!("restarting to apply settings");
                iced::exit()
            }
            Err(e) => {
                log::warn!("could not restart: {e}");
                self.notice = Some(format!("Could not restart, please reopen the app: {e}"));
                Task::none()
            }
        }
    }

    fn open_project(&mut self, id: ProjectId) {
        if let Some(project) = self.library.project(id) {
            log::info!("opening timeline {id} ({:?})", project.name);
            self.screen = Screen::Editor(Editor::new(project));
        }
    }

    /// Logs window events that matter for diagnosing display problems.
    fn on_window(&mut self, id: window::Id, event: window::Event) -> Task<Message> {
        match event {
            window::Event::Opened { position, size } => {
                log::info!(
                    "window opened: {:.0}x{:.0} at {position:?}",
                    size.width,
                    size.height
                );
                return Task::batch([
                    window::scale_factor(id).map(|f| Message::Display(format!("scale factor {f}"))),
                    window::monitor_size(id).map(|m| {
                        Message::Display(m.map_or_else(
                            || "monitor size unknown".to_owned(),
                            |s| format!("monitor {:.0}x{:.0}", s.width, s.height),
                        ))
                    }),
                ]);
            }
            window::Event::RedrawRequested(_) => log::info!(
                "first frame drawn {:.0} ms after start",
                self.started.elapsed().as_secs_f64() * 1000.0
            ),
            window::Event::Rescaled(factor) => log::info!("scale factor changed to {factor}"),
            window::Event::Resized(size) => {
                log::debug!("window resized to {:.0}x{:.0}", size.width, size.height);
            }
            window::Event::CloseRequested => log::info!("window close requested"),
            window::Event::Closed => log::info!("window closed"),
            other => log::debug!("window event: {other:?}"),
        }
        Task::none()
    }

    /// Persists the library, surfacing failures on the menu.
    fn save(&mut self) {
        if let Some(store) = &self.store
            && let Err(e) = store
                .save(&self.library)
                .inspect(|()| log::debug!("saved {} timeline(s)", self.library.projects.len()))
        {
            log::warn!("could not save timelines: {e}");
            self.notice = Some(format!("Could not save timelines: {e}"));
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        let screen = match &self.screen {
            Screen::Demo(demo) => demo.subscription().map(Message::Demo),
            Screen::Editor(_) => Editor::subscription().map(Message::Editor),
            Screen::Menu => Subscription::none(),
        };
        Subscription::batch([screen, event::listen_raw(window_event)])
    }

    pub fn view(&self) -> Element<'_, Message> {
        match &self.screen {
            Screen::Menu => self
                .menu
                .view(&self.library, self.notice.as_deref())
                .map(Message::Menu),
            Screen::Demo(demo) => demo.view().map(Message::Demo),
            Screen::Editor(editor) => match self.library.project(editor.project) {
                Some(project) => editor.view(project).map(Message::Editor),
                None => self
                    .menu
                    .view(&self.library, self.notice.as_deref())
                    .map(Message::Menu),
            },
        }
    }

    #[cfg(test)]
    fn library(&self) -> &Library {
        &self.library
    }
}

/// Shows the native file picker, limited to supported attachment types.
static FIRST_FRAME_SEEN: AtomicBool = AtomicBool::new(false);

/// Forwards window events for logging; only the first redraw is kept so
/// the app does not wake up on every frame.
fn window_event(event: iced::Event, _status: event::Status, id: window::Id) -> Option<Message> {
    let iced::Event::Window(event) = event else {
        return None;
    };
    if matches!(event, window::Event::RedrawRequested(_))
        && FIRST_FRAME_SEEN.swap(true, Ordering::Relaxed)
    {
        return None;
    }
    if matches!(event, window::Event::Moved(_)) {
        return None;
    }
    Some(Message::Window(id, event))
}

/// Shows the native file picker for TrueType/OpenType fonts.
async fn pick_font() -> Option<std::path::PathBuf> {
    rfd::AsyncFileDialog::new()
        .set_title("Choose a font")
        .add_filter("Fonts (TTF, OTF)", &project::FONT_EXTENSIONS)
        .pick_file()
        .await
        .map(|file| file.path().to_path_buf())
}

async fn pick_attachments() -> Vec<std::path::PathBuf> {
    rfd::AsyncFileDialog::new()
        .set_title("Attach files")
        .add_filter("Documents (PDF, DOCX)", &project::ATTACHMENT_EXTENSIONS)
        .pick_files()
        .await
        .map(|files| files.iter().map(|f| f.path().to_path_buf()).collect())
        .unwrap_or_default()
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn app(dir: &tempfile::TempDir) -> App {
        App::with_store(Some(Store::at(dir.path())))
    }

    #[test]
    fn menu_changes_are_saved() {
        let dir = tempfile::tempdir().unwrap();
        let mut a = app(&dir);
        let _ = a.update(Message::Menu(menu::Message::Create));
        let _ = a.update(Message::Menu(menu::Message::Sort(
            project::SortMode::Manual,
        )));
        assert_eq!(a.library().projects.len(), 1);

        let reloaded = app(&dir);
        assert_eq!(reloaded.library().projects.len(), 1);
        assert_eq!(reloaded.library().sort, project::SortMode::Manual);
    }

    #[test]
    fn creating_opens_the_editor_and_edits_persist() {
        let dir = tempfile::tempdir().unwrap();
        let mut a = app(&dir);
        let _ = a.update(Message::Menu(menu::Message::Create));
        assert!(matches!(a.screen, Screen::Editor(_)));
        let _ = a.update(Message::Editor(editor::Message::Rename("Thesis".into())));
        let _ = a.update(Message::Editor(editor::Message::AddEvent));
        let _ = a.update(Message::Editor(editor::Message::Back));
        assert!(matches!(a.screen, Screen::Menu));

        let reloaded = app(&dir);
        let p = &reloaded.library().projects[0];
        assert_eq!(p.name, "Thesis");
        assert_eq!(p.events.len(), 1);
    }

    #[test]
    fn choosing_and_resetting_a_font_is_saved() {
        let dir = tempfile::tempdir().unwrap();
        let mut a = app(&dir);
        let inter =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/fonts/Inter-Regular.ttf");
        let _ = a.update(Message::Menu(menu::Message::FontChosen(Some(inter))));
        let font = app(&dir).library().font.clone().unwrap();
        assert_eq!(font.family, "Inter");
        assert!(dir.path().join(&font.file).exists());

        let _ = a.update(Message::Menu(menu::Message::ResetFont));
        assert!(app(&dir).library().font.is_none());
        assert!(!dir.path().join(&font.file).exists());

        let _ = a.update(Message::Menu(menu::Message::FontChosen(Some(
            "x.txt".into(),
        ))));
        assert!(a.notice.is_some());
        assert!(app(&dir).library().font.is_none());
    }

    #[test]
    fn delete_needs_confirmation() {
        let dir = tempfile::tempdir().unwrap();
        let mut a = app(&dir);
        let _ = a.update(Message::Menu(menu::Message::Create));
        let id = a.library().projects[0].id;
        let _ = a.update(Message::Menu(menu::Message::Delete(id)));
        assert_eq!(a.library().projects.len(), 1);
        let _ = a.update(Message::Menu(menu::Message::Delete(id)));
        assert_eq!(a.library().projects.len(), 0);
    }

    #[test]
    fn demo_back_returns_to_menu() {
        let dir = tempfile::tempdir().unwrap();
        let mut a = app(&dir);
        let _ = a.update(Message::Menu(menu::Message::Open(Entry::Demo)));
        assert!(matches!(a.screen, Screen::Demo(_)));
        let _ = a.update(Message::Demo(demo::Message::Back));
        assert!(matches!(a.screen, Screen::Menu));
    }

    #[test]
    fn damaged_library_is_quarantined_with_notice() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("library.json"), b"garbage").unwrap();
        let a = app(&dir);
        assert!(a.notice.as_deref().unwrap_or("").contains("damaged"));
        assert!(dir.path().read_dir().unwrap().count() >= 1);
    }
}
