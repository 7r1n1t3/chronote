//! Screen for editing a user project and its events.

use iced::widget::{
    Space, button, canvas, column, container, row, scrollable, text, text_editor, text_input,
};
use iced::{Alignment, Background, Border, Element, Fill, Length, Padding};

use crate::menu::{plural, tint_for};
use crate::project::{self, Event, EventId, EventKind, Library, Project, ProjectId};
use crate::theme;
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
}

/// What the parent should do after an update.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    None,
    /// The library changed and should be saved.
    Changed,
    Back,
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
        }
    }

    #[must_use]
    pub fn selected(&self) -> Option<EventId> {
        self.selected
    }

    fn select(&mut self, event: Option<&Event>) {
        self.selected = event.map(|e| e.id);
        if let Some(e) = event {
            self.start = project::format_units(e.kind.start());
            self.end = project::format_units(e.kind.end());
            self.description = text_editor::Content::with_text(&e.description);
        }
    }

    pub fn update(&mut self, library: &mut Library, message: Message) -> Outcome {
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
            message => self.update_event(project, message),
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
            Message::DeleteEvent => {
                let id = event.id;
                project.remove_event(id);
                self.selected = None;
            }
            _ => return Outcome::None,
        }
        Outcome::Changed
    }

    pub fn view<'a>(&'a self, project: &'a Project) -> Element<'a, Message> {
        let header = row![
            ui::action("← Projects", Some(Message::Back), Tone::Normal),
            ui::dot(tint_for(project.id)),
            text_input("Project name", &project.name)
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
            label("DESCRIPTION (OPTIONAL)"),
            text_editor(&self.description)
                .placeholder("Notes about this event…")
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
        ]
        .spacing(16);

        column![
            scrollable(content.padding(Padding::ZERO.right(10)))
                .direction(scrollable::Direction::Vertical(
                    scrollable::Scrollbar::new().width(4).scroller_width(4),
                ))
                .height(Fill),
            ui::action("Delete event", Some(Message::DeleteEvent), Tone::Danger),
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
        ui::action("+ Add event", Some(Message::AddEvent), Tone::Primary),
        label("EVENTS"),
        scrollable(list.padding(Padding::ZERO.right(10)))
            .direction(scrollable::Direction::Vertical(
                scrollable::Scrollbar::new().width(4).scroller_width(4),
            ))
            .height(Fill),
        text("Click an event on the timeline or in this list to edit it.")
            .size(11)
            .color(theme::MUTED),
    ]
    .spacing(14)
    .into()
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
        assert_eq!(ed.update(&mut lib, Message::AddEvent), Outcome::Changed);
        assert_eq!(event(&lib, &ed).kind, EventKind::Instant { at: 15.0 });

        ed.update(&mut lib, Message::Title("Kickoff".into()));
        ed.update(&mut lib, Message::SetLasting(true));
        ed.update(&mut lib, Message::Start("2,5".into()));
        ed.update(&mut lib, Message::End("8".into()));
        let e = event(&lib, &ed);
        assert_eq!(e.title, "Kickoff");
        assert_eq!(
            e.kind,
            EventKind::Lasting {
                start: 2.5,
                end: 8.0
            }
        );

        ed.update(&mut lib, Message::SetLasting(false));
        assert_eq!(event(&lib, &ed).kind, EventKind::Instant { at: 2.5 });
    }

    #[test]
    fn invalid_numbers_are_kept_as_drafts() {
        let (mut lib, mut ed) = setup();
        assert_eq!(
            ed.update(&mut lib, Message::Length("abc".into())),
            Outcome::None
        );
        assert_eq!(ed.length, "abc");
        assert!((lib.projects[0].length - 30.0).abs() < f64::EPSILON);
        assert_eq!(
            ed.update(&mut lib, Message::Length("12".into())),
            Outcome::Changed
        );
        assert!((lib.projects[0].length - 12.0).abs() < f64::EPSILON);
        assert_eq!(
            ed.update(&mut lib, Message::Length("0".into())),
            Outcome::None
        );
    }

    #[test]
    fn description_edits_are_stored() {
        let (mut lib, mut ed) = setup();
        ed.update(&mut lib, Message::AddEvent);
        let outcome = ed.update(
            &mut lib,
            Message::Description(text_editor::Action::Edit(text_editor::Edit::Insert('x'))),
        );
        assert_eq!(outcome, Outcome::Changed);
        assert!(event(&lib, &ed).description.starts_with('x'));
    }

    #[test]
    fn delete_clears_selection() {
        let (mut lib, mut ed) = setup();
        ed.update(&mut lib, Message::AddEvent);
        ed.update(&mut lib, Message::DeleteEvent);
        assert!(ed.selected().is_none());
        assert_eq!(lib.projects[0].events.len(), 0);
    }

    #[test]
    fn new_events_are_spread_out() {
        let (mut lib, mut ed) = setup();
        ed.update(&mut lib, Message::AddEvent);
        ed.update(&mut lib, Message::AddEvent);
        let starts: Vec<f64> = lib.projects[0]
            .events
            .iter()
            .map(|e| e.kind.start())
            .collect();
        assert_eq!(starts, [15.0, 16.0]);
    }
}
