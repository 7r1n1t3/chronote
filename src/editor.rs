//! Screen for editing a study timeline and the notes on its events.

use iced::widget::{
    Space, button, canvas, column, container, row, scrollable, text, text_editor, text_input,
};
use std::path::PathBuf;

use iced::{Alignment, Background, Border, Element, Fill, Length, Padding, Subscription, window};

use crate::menu::{plural, tint_for};
use crate::project::{self, Event, EventId, EventKind, Library, Project, ProjectId, Store};
use crate::theme::{self, fade};
use crate::track::Track;
use crate::ui::{self, BOLD, SEMIBOLD, Tone};

#[derive(Debug, Clone)]
pub enum Message {
    Back,
    Rename(String),
    Unit(String),
    Length(String),
    AddEvent,
    Select(Option<EventId>),
    Title(String),
    SetLasting(bool),
    Start(String),
    End(String),
    Description(text_editor::Action),
    DeleteEvent,
    /// Ask the parent to show a file picker for the selected event.
    Attach,
    /// Files chosen for an event (from the picker).
    Attached(EventId, Vec<PathBuf>),
    /// A file dropped onto the window; goes to the selected event.
    Dropped(PathBuf),
    OpenAttachment(usize),
    RemoveAttachment(usize),
}

/// What the parent should do after an update.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    None,
    /// The library changed and should be saved.
    Changed,
    Back,
    /// Show a file picker; send the result back as `Message::Attached`.
    PickFiles(EventId),
}

/// Editing state for one project. Numeric fields keep the raw text so
/// partial input like `1.` can be typed.
#[derive(Debug)]
pub struct Editor {
    pub project: ProjectId,
    selected: Option<EventId>,
    length: String,
    start: String,
    end: String,
    description: text_editor::Content,
    /// Last attachment problem, shown in the side panel.
    error: Option<String>,
}

impl Editor {
    #[must_use]
    pub fn new(project: &Project) -> Self {
        Self {
            project: project.id,
            selected: None,
            length: project::format_units(project.length),
            start: String::new(),
            end: String::new(),
            description: text_editor::Content::new(),
            error: None,
        }
    }

    #[must_use]
    pub fn selected(&self) -> Option<EventId> {
        self.selected
    }

    fn select(&mut self, event: Option<&Event>) {
        self.selected = event.map(|e| e.id);
        self.error = None;
        if let Some(e) = event {
            self.start = project::format_units(e.kind.start());
            self.end = project::format_units(e.kind.end());
            self.description = text_editor::Content::with_text(&e.description);
        }
    }

    /// Files dropped onto the window while the editor is open.
    pub fn subscription() -> Subscription<Message> {
        iced::event::listen_with(|event, _status, _window| match event {
            iced::Event::Window(window::Event::FileDropped(path)) => Some(Message::Dropped(path)),
            _ => None,
        })
    }

    pub fn update(
        &mut self,
        library: &mut Library,
        store: Option<&Store>,
        message: Message,
    ) -> Outcome {
        if let Message::Back = message {
            return Outcome::Back;
        }
        if let Message::AddEvent = message {
            let id = library.next_id();
            let Some(project) = library.project_mut(self.project) else {
                return Outcome::Back;
            };
            let at = next_free_spot(project);
            project.events.push(Event {
                id,
                title: "New event".to_owned(),
                description: String::new(),
                kind: EventKind::Instant { at },
                attachments: Vec::new(),
            });
            let event = project.event(id);
            self.select(event);
            return Outcome::Changed;
        }

        let Some(project) = library.project_mut(self.project) else {
            return Outcome::Back;
        };
        match message {
            Message::Back | Message::AddEvent => Outcome::None,
            Message::Rename(name) => {
                project.name = name;
                Outcome::Changed
            }
            Message::Unit(unit) => {
                project.unit = unit;
                Outcome::Changed
            }
            Message::Length(raw) => {
                let parsed = parse_units(&raw).filter(|v| *v > 0.0);
                self.length = raw;
                match parsed {
                    Some(v) => {
                        project.length = v;
                        Outcome::Changed
                    }
                    None => Outcome::None,
                }
            }
            Message::Select(id) => {
                let event = id.and_then(|id| project.event(id));
                self.select(event);
                Outcome::None
            }
            message @ (Message::Attach
            | Message::Attached(..)
            | Message::Dropped(_)
            | Message::OpenAttachment(_)
            | Message::RemoveAttachment(_)
            | Message::DeleteEvent) => self.update_files(project, store, message),
            message => self.update_event(project, message),
        }
    }

    /// Attachment handling and event deletion (which removes files too).
    fn update_files(
        &mut self,
        project: &mut Project,
        store: Option<&Store>,
        message: Message,
    ) -> Outcome {
        match message {
            Message::Attach => match self.selected {
                Some(id) => Outcome::PickFiles(id),
                None => Outcome::None,
            },
            Message::Attached(id, paths) => self.attach(project, store, id, &paths),
            Message::Dropped(path) => {
                if let Some(id) = self.selected {
                    self.attach(project, store, id, &[path])
                } else {
                    self.error = Some("Select an event before dropping files.".to_owned());
                    Outcome::None
                }
            }
            Message::OpenAttachment(index) => {
                let file = self
                    .selected
                    .and_then(|id| project.event(id))
                    .and_then(|e| e.attachments.get(index));
                if let (Some(file), Some(store)) = (file, store)
                    && let Err(e) = opener::open(store.attachment_path(file))
                {
                    self.error = Some(format!("Could not open {}: {e}", file.name));
                }
                Outcome::None
            }
            Message::RemoveAttachment(index) => {
                let Some(event) = self.selected.and_then(|id| project.event_mut(id)) else {
                    return Outcome::None;
                };
                if index >= event.attachments.len() {
                    return Outcome::None;
                }
                let file = event.attachments.remove(index);
                if let Some(store) = store
                    && let Err(e) = store.remove_attachment(&file)
                {
                    self.error = Some(format!("Could not delete {}: {e}", file.name));
                }
                Outcome::Changed
            }
            Message::DeleteEvent => {
                let Some(id) = self.selected.take() else {
                    return Outcome::None;
                };
                project.remove_event(id);
                if let Some(store) = store
                    && let Err(e) = store.remove_event_files(project.id, id)
                {
                    self.error = Some(format!("Could not delete attachments: {e}"));
                }
                Outcome::Changed
            }
            _ => Outcome::None,
        }
    }

    fn update_event(&mut self, project: &mut Project, message: Message) -> Outcome {
        let span = (project.length / 10.0).max(1.0);
        let Some(event) = self.selected.and_then(|id| project.event_mut(id)) else {
            return Outcome::None;
        };
        match message {
            Message::Title(title) => event.title = title,
            Message::SetLasting(lasting) => {
                if event.kind.is_lasting() == lasting {
                    return Outcome::None;
                }
                event.kind = event.kind.toggled(span);
                self.start = project::format_units(event.kind.start());
                self.end = project::format_units(event.kind.end());
            }
            Message::Start(raw) => {
                let parsed = parse_units(&raw);
                self.start = raw;
                let Some(v) = parsed else {
                    return Outcome::None;
                };
                event.kind = match event.kind {
                    EventKind::Instant { .. } => EventKind::Instant { at: v },
                    EventKind::Lasting { end, .. } => EventKind::Lasting { start: v, end },
                };
            }
            Message::End(raw) => {
                let parsed = parse_units(&raw);
                self.end = raw;
                let (Some(v), EventKind::Lasting { start, .. }) = (parsed, event.kind) else {
                    return Outcome::None;
                };
                event.kind = EventKind::Lasting { start, end: v };
            }
            Message::Description(action) => {
                let edit = action.is_edit();
                self.description.perform(action);
                if !edit {
                    return Outcome::None;
                }
                event.description = self.description.text();
            }
            _ => return Outcome::None,
        }
        Outcome::Changed
    }

    /// Copies files into the store and links them to event `id`.
    fn attach(
        &mut self,
        project: &mut Project,
        store: Option<&Store>,
        id: EventId,
        paths: &[PathBuf],
    ) -> Outcome {
        let Some(store) = store else {
            self.error = Some("Attachments need a data directory.".to_owned());
            return Outcome::None;
        };
        let project_id = project.id;
        let Some(event) = project.event_mut(id) else {
            return Outcome::None;
        };
        let mut failures = Vec::new();
        let before = event.attachments.len();
        for path in paths {
            match store.import_attachment(project_id, id, path) {
                Ok(file) => event.attachments.push(file),
                Err(e) => failures.push(format!(
                    "{}: {e}",
                    path.file_name().map_or_else(
                        || path.display().to_string(),
                        |n| n.to_string_lossy().into_owned()
                    )
                )),
            }
        }
        self.error = (!failures.is_empty()).then(|| failures.join("\n"));
        if event.attachments.len() > before {
            Outcome::Changed
        } else {
            Outcome::None
        }
    }

    pub fn view<'a>(&'a self, project: &'a Project) -> Element<'a, Message> {
        let header = row![
            ui::action("← Timelines", Some(Message::Back), Tone::Normal),
            ui::dot(tint_for(project.id)),
            text_input("Timeline name, e.g. Heart development", &project.name)
                .on_input(Message::Rename)
                .size(22)
                .font(BOLD)
                .padding([6, 10])
                .style(ui::input_style)
                .width(Fill),
            field(
                "UNIT",
                text_input("day", &project.unit).on_input(Message::Unit),
                120
            ),
            field(
                "LENGTH",
                text_input("30", &self.length).on_input(Message::Length),
                90,
            ),
        ]
        .spacing(14)
        .align_y(Alignment::End);

        let track = canvas(Track {
            project,
            selected: self.selected,
            on_select: Message::Select,
        })
        .width(Fill)
        .height(Fill);

        let body = row![
            ui::card(track, 4.0),
            container(self.side(project))
                .width(380)
                .height(Fill)
                .padding(22)
                .style(ui::card_style(20.0)),
        ]
        .spacing(20)
        .height(Fill);

        container(column![header, body].spacing(20).padding(28))
            .width(Fill)
            .height(Fill)
            .style(|_| container::Style {
                background: Some(Background::Color(theme::BACKGROUND)),
                text_color: Some(theme::TEXT),
                ..container::Style::default()
            })
            .into()
    }

    fn side<'a>(&'a self, project: &'a Project) -> Element<'a, Message> {
        let selected = self.selected.and_then(|id| project.event(id));
        match selected {
            Some(event) => self.form(project, event),
            None => event_list(project),
        }
    }

    fn form<'a>(&'a self, project: &'a Project, event: &'a Event) -> Element<'a, Message> {
        let lasting = event.kind.is_lasting();
        let unit = plural(&project.unit, 2.0);

        let kind = row![
            toggle("Instant", !lasting, Message::SetLasting(false)),
            toggle("Lasting", lasting, Message::SetLasting(true)),
        ]
        .spacing(8);

        let mut timing = row![field(
            if lasting { "START" } else { "AT" },
            text_input("0", &self.start).on_input(Message::Start),
            Length::Fill,
        )]
        .spacing(10)
        .align_y(Alignment::End);
        if lasting {
            timing = timing.push(field(
                "END",
                text_input("1", &self.end).on_input(Message::End),
                Length::Fill,
            ));
        }
        timing = timing.push(
            text(unit)
                .size(12)
                .color(theme::MUTED)
                .height(36)
                .align_y(Alignment::Center),
        );

        let description = column![
            label("STUDY NOTES (OPTIONAL)"),
            text_editor(&self.description)
                .placeholder("What happens, why it matters, mnemonics…")
                .on_action(Message::Description)
                .height(140)
                .padding(10)
                .style(ui::editor_style),
        ]
        .spacing(6);

        let content = column![
            row![
                label("EVENT"),
                Space::new().width(Fill),
                button(text("Done").size(12).font(SEMIBOLD))
                    .padding([4, 10])
                    .on_press(Message::Select(None))
                    .style(ui::button_style(Tone::Normal)),
            ]
            .align_y(Alignment::Center),
            text_input("Title", &event.title)
                .on_input(Message::Title)
                .size(18)
                .padding(10)
                .style(ui::input_style),
            kind,
            timing,
            description,
            attachments(event, self.error.as_deref()),
        ]
        .spacing(16);

        column![
            scrollable(content.padding(Padding::ZERO.right(10)))
                .direction(scrollable::Direction::Vertical(
                    scrollable::Scrollbar::new().width(4).scroller_width(4),
                ))
                .height(Fill),
            ui::wide_action("Delete event", Some(Message::DeleteEvent), Tone::Danger),
        ]
        .spacing(12)
        .into()
    }
}

fn event_list(project: &Project) -> Element<'_, Message> {
    let mut events: Vec<&Event> = project.events.iter().collect();
    events.sort_by(|a, b| a.kind.start().total_cmp(&b.kind.start()));

    let mut list = column![].spacing(8);
    for event in events {
        let (color, when) = match event.kind {
            EventKind::Instant { at } => (
                theme::FIRST,
                format!(
                    "at {} {}",
                    project::format_units(at),
                    plural(&project.unit, at)
                ),
            ),
            EventKind::Lasting { .. } => (
                theme::SECOND,
                format!(
                    "{}–{} {}",
                    project::format_units(event.kind.start()),
                    project::format_units(event.kind.end()),
                    plural(&project.unit, 2.0)
                ),
            ),
        };
        let marker: Element<'_, Message> = container(Space::new().width(10).height(10))
            .style(move |_| container::Style {
                background: Some(Background::Color(color)),
                border: Border::default().rounded(if event.kind.is_lasting() { 3 } else { 5 }),
                ..container::Style::default()
            })
            .into();
        let clips = if event.attachments.is_empty() {
            String::new()
        } else {
            format!(
                " · {} file{}",
                event.attachments.len(),
                if event.attachments.len() == 1 {
                    ""
                } else {
                    "s"
                }
            )
        };
        list = list.push(
            button(
                row![
                    marker,
                    column![
                        text(event.title.as_str()).size(15).font(SEMIBOLD),
                        text(format!("{when}{clips}")).size(12).color(theme::MUTED),
                    ]
                    .spacing(2),
                ]
                .spacing(12)
                .align_y(Alignment::Center),
            )
            .width(Fill)
            .padding([10, 12])
            .on_press(Message::Select(Some(event.id)))
            .style(ui::button_style(Tone::Normal)),
        );
    }

    column![
        ui::wide_action("+ Add event", Some(Message::AddEvent), Tone::Primary),
        label("EVENTS & NOTES"),
        scrollable(list.padding(Padding::ZERO.right(10)))
            .direction(scrollable::Direction::Vertical(
                scrollable::Scrollbar::new().width(4).scroller_width(4),
            ))
            .height(Fill),
        text("Click an event on the timeline or in this list to edit its notes.")
            .size(11)
            .color(theme::MUTED),
    ]
    .spacing(14)
    .into()
}

fn attachments<'a>(event: &'a Event, error: Option<&'a str>) -> Element<'a, Message> {
    let mut list = column![label("ATTACHMENTS")].spacing(8);
    for (index, file) in event.attachments.iter().enumerate() {
        let kind = std::path::Path::new(&file.name)
            .extension()
            .map(|e| e.to_string_lossy().to_uppercase())
            .unwrap_or_default();
        let tint = if kind == "PDF" {
            theme::DANGER
        } else {
            theme::FIRST
        };
        list = list.push(
            container(
                row![
                    ui::chip(kind, tint),
                    text(file.name.as_str()).size(13).width(Fill),
                    button(text("Open").size(12).font(SEMIBOLD))
                        .padding([4, 10])
                        .on_press(Message::OpenAttachment(index))
                        .style(ui::button_style(Tone::Normal)),
                    button(text("✕").size(12))
                        .padding([4, 8])
                        .on_press(Message::RemoveAttachment(index))
                        .style(ui::button_style(Tone::Danger)),
                ]
                .spacing(8)
                .align_y(Alignment::Center),
            )
            .padding([8, 10])
            .style(|_| container::Style {
                background: Some(Background::Color(theme::SURFACE_RAISED)),
                border: Border::default().rounded(10),
                ..container::Style::default()
            }),
        );
    }
    list = list
        .push(ui::wide_action(
            "Attach PDF / DOCX…",
            Some(Message::Attach),
            Tone::Normal,
        ))
        .push(
            text("…or drop files onto the window.")
                .size(11)
                .color(theme::MUTED),
        );
    if let Some(error) = error {
        list = list.push(text(error).size(12).color(fade(theme::DANGER, 0.9)));
    }
    list.into()
}

fn label(content: &str) -> Element<'_, Message> {
    text(content)
        .size(11)
        .font(SEMIBOLD)
        .color(theme::MUTED)
        .into()
}

fn field<'a>(
    name: &'a str,
    input: text_input::TextInput<'a, Message>,
    width: impl Into<Length>,
) -> Element<'a, Message> {
    column![
        label(name),
        input.padding([8, 10]).size(14).style(ui::input_style),
    ]
    .spacing(6)
    .width(width)
    .into()
}

fn toggle(name: &str, active: bool, message: Message) -> Element<'_, Message> {
    button(
        text(name)
            .size(13)
            .font(SEMIBOLD)
            .align_x(Alignment::Center)
            .width(Fill),
    )
    .width(Fill)
    .padding([8, 0])
    .on_press(message)
    .style(ui::button_style(if active {
        Tone::Selected
    } else {
        Tone::Normal
    }))
    .into()
}

/// Parses a timeline position; accepts `,` as decimal separator.
fn parse_units(raw: &str) -> Option<f64> {
    raw.trim()
        .replace(',', ".")
        .parse::<f64>()
        .ok()
        .filter(|v| v.is_finite() && *v >= 0.0)
}

/// Places a new event just after the last one, wrapping to the start.
fn next_free_spot(project: &Project) -> f64 {
    let last = project
        .events
        .iter()
        .map(|e| e.kind.end())
        .fold(None, |acc: Option<f64>, v| {
            Some(acc.map_or(v, |a| a.max(v)))
        });
    match last {
        Some(end) if end + 1.0 <= project.length => (end + 1.0).floor(),
        Some(_) => 0.0,
        None => (project.length / 2.0).floor(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> (Library, Editor) {
        let mut library = Library::default();
        let id = library.create_project("Test", 0);
        let editor = Editor::new(library.project(id).unwrap());
        (library, editor)
    }

    fn event<'a>(library: &'a Library, editor: &Editor) -> &'a Event {
        library
            .project(editor.project)
            .and_then(|p| p.event(editor.selected().unwrap()))
            .unwrap()
    }

    #[test]
    fn add_and_edit_event() {
        let (mut lib, mut ed) = setup();
        assert_eq!(
            ed.update(&mut lib, None, Message::AddEvent),
            Outcome::Changed
        );
        assert_eq!(event(&lib, &ed).kind, EventKind::Instant { at: 20.0 });

        ed.update(&mut lib, None, Message::Title("Neural tube closes".into()));
        ed.update(&mut lib, None, Message::SetLasting(true));
        ed.update(&mut lib, None, Message::Start("2,5".into()));
        ed.update(&mut lib, None, Message::End("8".into()));
        let e = event(&lib, &ed);
        assert_eq!(e.title, "Neural tube closes");
        assert_eq!(
            e.kind,
            EventKind::Lasting {
                start: 2.5,
                end: 8.0
            }
        );

        ed.update(&mut lib, None, Message::SetLasting(false));
        assert_eq!(event(&lib, &ed).kind, EventKind::Instant { at: 2.5 });
    }

    #[test]
    fn invalid_numbers_are_kept_as_drafts() {
        let (mut lib, mut ed) = setup();
        assert_eq!(
            ed.update(&mut lib, None, Message::Length("abc".into())),
            Outcome::None
        );
        assert_eq!(ed.length, "abc");
        assert!((lib.projects[0].length - 40.0).abs() < f64::EPSILON);
        assert_eq!(
            ed.update(&mut lib, None, Message::Length("12".into())),
            Outcome::Changed
        );
        assert!((lib.projects[0].length - 12.0).abs() < f64::EPSILON);
        assert_eq!(
            ed.update(&mut lib, None, Message::Length("0".into())),
            Outcome::None
        );
    }

    #[test]
    fn description_edits_are_stored() {
        let (mut lib, mut ed) = setup();
        ed.update(&mut lib, None, Message::AddEvent);
        let outcome = ed.update(
            &mut lib,
            None,
            Message::Description(text_editor::Action::Edit(text_editor::Edit::Insert('x'))),
        );
        assert_eq!(outcome, Outcome::Changed);
        assert!(event(&lib, &ed).description.starts_with('x'));

        // Re-selecting the event loads its saved description back.
        let id = ed.selected();
        ed.update(&mut lib, None, Message::Select(None));
        ed.update(&mut lib, None, Message::Select(id));
        assert!(ed.description.text().starts_with('x'));
    }

    #[test]
    fn delete_clears_selection() {
        let (mut lib, mut ed) = setup();
        ed.update(&mut lib, None, Message::AddEvent);
        ed.update(&mut lib, None, Message::DeleteEvent);
        assert!(ed.selected().is_none());
        assert_eq!(lib.projects[0].events.len(), 0);
    }

    #[test]
    fn attachments_are_imported_filtered_and_removed() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::at(dir.path().join("data"));
        let pdf = dir.path().join("plan.pdf");
        let txt = dir.path().join("plan.txt");
        std::fs::write(&pdf, b"%PDF").unwrap();
        std::fs::write(&txt, b"text").unwrap();

        let (mut lib, mut ed) = setup();
        assert_eq!(ed.update(&mut lib, None, Message::Attach), Outcome::None);
        ed.update(&mut lib, None, Message::AddEvent);
        let id = ed.selected().unwrap();
        assert_eq!(
            ed.update(&mut lib, None, Message::Attach),
            Outcome::PickFiles(id)
        );

        let outcome = ed.update(
            &mut lib,
            Some(&store),
            Message::Attached(id, vec![pdf, txt]),
        );
        assert_eq!(outcome, Outcome::Changed);
        assert_eq!(event(&lib, &ed).attachments.len(), 1);
        assert!(ed.error.as_deref().unwrap_or("").contains("plan.txt"));
        let stored = store.attachment_path(&event(&lib, &ed).attachments[0]);
        assert!(stored.exists());

        ed.update(&mut lib, Some(&store), Message::RemoveAttachment(0));
        assert_eq!(event(&lib, &ed).attachments.len(), 0);
        assert!(!stored.exists());
    }

    #[test]
    fn dropping_without_selection_reports_error() {
        let (mut lib, mut ed) = setup();
        let outcome = ed.update(&mut lib, None, Message::Dropped("x.pdf".into()));
        assert_eq!(outcome, Outcome::None);
        assert!(ed.error.is_some());
    }

    #[test]
    fn new_events_are_spread_out() {
        let (mut lib, mut ed) = setup();
        ed.update(&mut lib, None, Message::AddEvent);
        ed.update(&mut lib, None, Message::AddEvent);
        let starts: Vec<f64> = lib.projects[0]
            .events
            .iter()
            .map(|e| e.kind.start())
            .collect();
        assert_eq!(starts, [20.0, 21.0]);
    }
}
