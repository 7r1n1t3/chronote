//! Root of the application: owns the project library and switches screens.

use iced::{Element, Subscription, Task};

use crate::demo::{self, Demo};
use crate::editor::{self, Editor, Outcome};
use crate::menu::{self, Menu};
use crate::project::{self, Entry, Library, ProjectId, Store, StoreError};

#[derive(Debug, Clone)]
pub enum Message {
    Menu(menu::Message),
    Demo(demo::Message),
    Editor(editor::Message),
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
                notice = Some("No data directory found; timelines will not be saved.".to_owned());
                Library::default()
            }
            Some(Ok(library)) => library,
            Some(Err(error)) => {
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
        }
    }

    /// Opens straight into the demo at `week`.
    #[must_use]
    pub fn with_demo(mut self, week: u8) -> Self {
        self.screen = Screen::Demo(Box::new(Demo::starting_at(week)));
        self
    }

    #[must_use]
    pub fn title(&self) -> String {
        match &self.screen {
            Screen::Menu => "Timelines".to_owned(),
            Screen::Demo(demo) => demo.title(),
            Screen::Editor(editor) => self.library.project(editor.project).map_or_else(
                || "Timelines".to_owned(),
                |p| format!("{} — Timelines", p.name),
            ),
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Menu(message) => self.update_menu(message),
            Message::Demo(demo::Message::Back) => self.screen = Screen::Menu,
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
                        Outcome::Back => self.screen = Screen::Menu,
                        Outcome::PickFiles(event) => {
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

    fn update_menu(&mut self, message: menu::Message) {
        if !matches!(message, menu::Message::Delete(_)) {
            self.menu.confirm_delete = None;
        }
        match message {
            menu::Message::Open(Entry::Demo) => {
                self.screen = Screen::Demo(Box::default());
            }
            menu::Message::Open(Entry::Project(id)) => self.open_project(id),
            menu::Message::Create => {
                let id = self
                    .library
                    .create_project("Untitled timeline", project::now());
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
                    if let Some(store) = &self.store
                        && let Err(e) = store.remove_project_files(id)
                    {
                        self.notice = Some(format!("Could not delete attachments: {e}"));
                    }
                    self.save();
                } else {
                    self.menu.confirm_delete = Some(id);
                }
            }
            menu::Message::CancelDelete => {}
        }
    }

    fn open_project(&mut self, id: ProjectId) {
        if let Some(project) = self.library.project(id) {
            self.screen = Screen::Editor(Editor::new(project));
        }
    }

    /// Persists the library, surfacing failures on the menu.
    fn save(&mut self) {
        if let Some(store) = &self.store
            && let Err(e) = store.save(&self.library)
        {
            self.notice = Some(format!("Could not save timelines: {e}"));
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        match &self.screen {
            Screen::Demo(demo) => demo.subscription().map(Message::Demo),
            Screen::Editor(_) => Editor::subscription().map(Message::Editor),
            Screen::Menu => Subscription::none(),
        }
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
