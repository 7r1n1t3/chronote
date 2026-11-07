//! Startup screen: a scrollable list of projects.

use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Alignment, Background, Border, Color, Element, Fill, Length, Theme};

use crate::project::{self, Entry, Library, ProjectId, SortMode};
use crate::theme::{self, fade};
use crate::ui::{self, BOLD, SEMIBOLD, Tone};

#[derive(Debug, Clone, Copy)]
pub enum Message {
    Open(Entry),
    Create,
    Sort(SortMode),
    /// Move an entry up (negative) or down (positive) in manual order.
    Move(Entry, isize),
    /// First click asks for confirmation, second click deletes.
    Delete(ProjectId),
    CancelDelete,
}

/// Transient menu state (not saved).
#[derive(Debug, Default)]
pub struct Menu {
    pub confirm_delete: Option<ProjectId>,
}

impl Menu {
    pub fn view<'a>(
        &'a self,
        library: &'a Library,
        notice: Option<&'a str>,
    ) -> Element<'a, Message> {
        let header = row![
            ui::dot(theme::ACCENT),
            column![
                text("Timelines").size(30).font(BOLD),
                text("Open the Embryo demo or build your own timeline.")
                    .size(14)
                    .color(theme::MUTED),
            ]
            .spacing(2),
        ]
        .spacing(16)
        .align_y(Alignment::Center);

        let sort = row![
            text("SORT BY").size(11).font(SEMIBOLD).color(theme::MUTED),
            sort_button("Created", SortMode::Created, library.sort),
            sort_button("A–Z", SortMode::Alphabetical, library.sort),
            sort_button("Manual", SortMode::Manual, library.sort),
        ]
        .spacing(8)
        .align_y(Alignment::Center);

        let entries = library.entries();
        let final_index = entries.len().saturating_sub(1);
        let mut list = column![].spacing(10);
        for (index, entry) in entries.iter().copied().enumerate() {
            list = list.push(self.entry(library, entry, index, final_index));
        }
        list = list.push(create_entry());

        let mut content = column![header, sort].spacing(22).padding(36).max_width(820);
        if let Some(notice) = notice {
            content = content.push(
                container(text(notice).size(13).color(theme::THIRD))
                    .padding([10, 14])
                    .width(Fill)
                    .style(|_| container::Style {
                        background: Some(Background::Color(fade(theme::THIRD, 0.12))),
                        border: Border::default().rounded(12),
                        ..container::Style::default()
                    }),
            );
        }
        content = content.push(
            scrollable(list.padding(iced::Padding::ZERO.right(14)))
                .direction(scrollable::Direction::Vertical(
                    scrollable::Scrollbar::new().width(4).scroller_width(4),
                ))
                .height(Fill),
        );

        container(content)
            .center_x(Fill)
            .height(Fill)
            .style(|_| container::Style {
                background: Some(Background::Color(theme::BACKGROUND)),
                text_color: Some(theme::TEXT),
                ..container::Style::default()
            })
            .into()
    }

    fn entry<'a>(
        &'a self,
        library: &'a Library,
        entry: Entry,
        index: usize,
        last: usize,
    ) -> Element<'a, Message> {
        let (tint, subtitle) = match entry {
            Entry::Demo => (
                theme::ACCENT,
                "Demo · 40 weeks of prenatal development".to_owned(),
            ),
            Entry::Project(id) => {
                let p = library.project(id);
                let count = p.map_or(0, |p| p.events.len());
                (
                    tint_for(id),
                    p.map_or_else(String::new, |p| {
                        format!(
                            "{count} event{} · {} {} · created {}",
                            if count == 1 { "" } else { "s" },
                            project::format_units(p.length),
                            plural(&p.unit, p.length),
                            project::format_date(p.created),
                        )
                    }),
                )
            }
        };

        let body = row![
            ui::dot(tint),
            column![
                text(library.name(entry).to_owned()).size(18).font(BOLD),
                text(subtitle).size(13).color(theme::MUTED),
            ]
            .spacing(4),
        ]
        .spacing(16)
        .align_y(Alignment::Center);

        let open = button(body)
            .padding([18, 20])
            .width(Fill)
            .on_press(Message::Open(entry))
            .style(entry_style);

        let mut controls = row![].spacing(6).align_y(Alignment::Center);
        if library.sort == SortMode::Manual {
            controls = controls
                .push(small("↑", (index > 0).then_some(Message::Move(entry, -1))))
                .push(small(
                    "↓",
                    (index < last).then_some(Message::Move(entry, 1)),
                ));
        }
        if let Entry::Project(id) = entry {
            controls = if self.confirm_delete == Some(id) {
                controls
                    .push(ui::action(
                        "Delete?",
                        Some(Message::Delete(id)),
                        Tone::Danger,
                    ))
                    .push(ui::action(
                        "Keep",
                        Some(Message::CancelDelete),
                        Tone::Normal,
                    ))
            } else {
                controls.push(small("✕", Some(Message::Delete(id))))
            };
        }

        container(
            row![open, controls]
                .spacing(8)
                .align_y(Alignment::Center)
                .padding(iced::Padding::ZERO.right(12)),
        )
        .style(ui::card_style(18.0))
        .into()
    }
}

/// "New project" entry, styled like the project entries.
fn create_entry<'a>() -> Element<'a, Message> {
    let body = row![
        container(text("+").size(18).font(BOLD).color(theme::ACCENT))
            .center_x(28)
            .center_y(28)
            .style(|_| container::Style {
                border: Border::default()
                    .rounded(14)
                    .width(1)
                    .color(fade(theme::ACCENT, 0.6)),
                ..container::Style::default()
            }),
        column![
            text("New project").size(18).font(BOLD),
            text("Start a blank timeline with your own unit of time")
                .size(13)
                .color(theme::MUTED),
        ]
        .spacing(4),
    ]
    .spacing(16)
    .align_y(Alignment::Center);

    container(
        button(body)
            .padding([18, 20])
            .width(Fill)
            .on_press(Message::Create)
            .style(entry_style),
    )
    .style(|_| container::Style {
        background: Some(Background::Color(theme::SURFACE)),
        border: Border::default()
            .rounded(18)
            .width(1)
            .color(fade(theme::ACCENT, 0.35)),
        ..container::Style::default()
    })
    .into()
}

fn entry_style(_: &Theme, status: button::Status) -> button::Style {
    let bg = match status {
        button::Status::Hovered => fade(theme::ACCENT, 0.08),
        button::Status::Pressed => fade(theme::ACCENT, 0.14),
        _ => Color::TRANSPARENT,
    };
    button::Style {
        background: Some(Background::Color(bg)),
        text_color: theme::TEXT,
        border: Border::default().rounded(18),
        ..button::Style::default()
    }
}

fn sort_button(label: &str, mode: SortMode, current: SortMode) -> Element<'_, Message> {
    button(text(label).size(13).font(SEMIBOLD))
        .padding([6, 14])
        .on_press(Message::Sort(mode))
        .style(ui::button_style(if mode == current {
            Tone::Selected
        } else {
            Tone::Normal
        }))
        .into()
}

fn small(label: &str, message: Option<Message>) -> Element<'_, Message> {
    button(
        text(label)
            .size(14)
            .align_x(Alignment::Center)
            .width(Length::Fill),
    )
    .width(34)
    .padding([8, 0])
    .on_press_maybe(message)
    .style(ui::button_style(Tone::Normal))
    .into()
}

/// Stable accent color per project.
#[must_use]
pub fn tint_for(id: ProjectId) -> Color {
    const TINTS: [Color; 4] = [theme::FIRST, theme::SECOND, theme::THIRD, theme::ACCENT];
    TINTS[(id % 4) as usize]
}

/// Naive English plural for a unit name.
#[must_use]
pub fn plural(unit: &str, amount: f64) -> String {
    if (amount - 1.0).abs() < f64::EPSILON || unit.ends_with('s') || unit.is_empty() {
        unit.to_owned()
    } else {
        format!("{unit}s")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pluralizes_units() {
        assert_eq!(plural("day", 3.0), "days");
        assert_eq!(plural("day", 1.0), "day");
        assert_eq!(plural("hours", 2.0), "hours");
    }
}
