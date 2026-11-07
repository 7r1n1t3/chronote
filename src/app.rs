//! Application state, update logic and layout.

use std::time::{Duration, Instant};

use iced::keyboard::{self, Key, key::Named};
use iced::widget::{Space, button, canvas, column, container, row, scrollable, text};
use iced::{
    Alignment, Background, Border, Color, Element, Fill, Font, Length, Padding, Shadow,
    Subscription, Vector, font, time,
};

use crate::anim::{Tween, ease_out_cubic};
use crate::data::{self, WEEKS, Week};
use crate::fetus::{Fetus, heart_rate};
use crate::theme::{self, fade};
use crate::timeline::Timeline;

/// How long the illustration takes to morph between weeks.
const TRANSITION: Duration = Duration::from_millis(900);
/// Animation tick interval (~60 fps).
const FRAME: Duration = Duration::from_millis(16);
/// Time spent on each week while autoplaying.
const AUTOPLAY_STEP: Duration = Duration::from_millis(2600);

const BOLD: Font = Font {
    weight: font::Weight::Bold,
    ..Font::DEFAULT
};
const SEMIBOLD: Font = Font {
    weight: font::Weight::Semibold,
    ..Font::DEFAULT
};

#[derive(Debug, Clone, Copy)]
pub enum Message {
    Tick(Instant),
    Select(u8),
    Previous,
    Next,
    First,
    Last,
    TogglePlay,
}

#[derive(Debug)]
pub struct App {
    selected: u8,
    week: Tween,
    started: Instant,
    now: Instant,
    playing: bool,
    last_step: Instant,
}

impl Default for App {
    fn default() -> Self {
        let now = Instant::now();
        Self {
            selected: 1,
            week: Tween::new(1.0, TRANSITION),
            started: now,
            now,
            playing: false,
            last_step: now,
        }
    }
}

impl App {
    /// Starts the app on `week` (clamped to the valid range).
    #[must_use]
    pub fn starting_at(week: u8) -> Self {
        let week = week.clamp(1, WEEKS);
        Self {
            selected: week,
            week: Tween::new(f32::from(week), TRANSITION),
            ..Self::default()
        }
    }

    #[must_use]
    pub fn title(&self) -> String {
        format!("Embryo — Week {}", self.selected)
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::Tick(now) => {
                self.now = now;
                if self.playing && now.duration_since(self.last_step) >= AUTOPLAY_STEP {
                    if self.selected < WEEKS {
                        self.select(self.selected + 1);
                    } else {
                        self.playing = false;
                    }
                }
            }
            Message::Select(week) => self.select(week),
            Message::Previous => self.select(self.selected.saturating_sub(1)),
            Message::Next => self.select(self.selected + 1),
            Message::First => self.select(1),
            Message::Last => self.select(WEEKS),
            Message::TogglePlay => {
                self.playing = !self.playing;
                if self.playing && self.selected == WEEKS {
                    self.select(1);
                }
                self.last_step = self.now;
            }
        }
    }

    fn select(&mut self, week: u8) {
        let week = week.clamp(1, WEEKS);
        self.last_step = self.now;
        if week != self.selected {
            self.selected = week;
            self.week.go(f32::from(week), self.now);
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            // A fixed timer keeps the idle animation running even when the
            // compositor throttles redraw-driven `window::frames()`.
            time::every(FRAME).map(Message::Tick),
            keyboard::listen().filter_map(|event| match event {
                keyboard::Event::KeyPressed { key, .. } => shortcut(&key),
                _ => None,
            }),
        ])
    }

    #[must_use]
    pub fn view(&self) -> Element<'_, Message> {
        let time = self.now.duration_since(self.started).as_secs_f32();
        let position = self.week.value(self.now);
        let reveal = self.week.progress(self.now);

        let illustration = canvas(Fetus {
            week: position,
            time,
        })
        .width(Fill)
        .height(Fill);

        let timeline = canvas(Timeline {
            selected: self.selected,
            position,
            time,
            on_select: Message::Select,
        })
        .width(Fill)
        .height(110);

        let body = row![
            container(illustration).width(Fill).height(Fill),
            self.panel(data::week(self.selected), reveal, position),
        ]
        .spacing(24)
        .height(Fill);

        container(
            column![header(), body, card(timeline, 4.0)]
                .spacing(20)
                .padding(28),
        )
        .style(|_| container::Style {
            background: Some(Background::Color(theme::BACKGROUND)),
            text_color: Some(theme::TEXT),
            ..container::Style::default()
        })
        .width(Fill)
        .height(Fill)
        .into()
    }

    fn panel<'a>(&self, week: &'a Week, reveal: f32, position: f32) -> Element<'a, Message> {
        let trimester = week.trimester();
        let tint = theme::trimester(trimester);
        let shown = ease_out_cubic(reveal);
        let ink = |c: Color| fade(c, shown);

        let chips = row![
            chip(trimester.label(), tint),
            text(week.stage().label()).size(13).color(theme::MUTED),
        ]
        .spacing(12)
        .align_y(Alignment::Center);

        let heading = column![
            text(format!("Week {}", week.number))
                .size(42)
                .font(BOLD)
                .color(ink(theme::TEXT)),
            text(week.title).size(22).font(SEMIBOLD).color(ink(tint)),
        ]
        .spacing(2);

        let first_day = (u16::from(week.number) - 1) * 7 + 1;
        let to_go = WEEKS - week.number;
        let days = text(format!(
            "Days {first_day}–{} · {}",
            first_day + 6,
            match to_go {
                0 => "due this week".to_owned(),
                1 => "1 week to go".to_owned(),
                n => format!("{n} weeks to go"),
            }
        ))
        .size(13)
        .color(theme::MUTED);

        let stats = row![
            stat("LENGTH", week.length_text(), week.measure.label(), ink),
            stat("WEIGHT", week.weight_text(), "", ink),
            stat("SIZE OF", capitalize(week.size_of), "", ink),
        ]
        .spacing(10);

        let heart = (week.number >= 6).then(|| {
            text(format!(
                "Heart rate ≈ {:.0} bpm",
                heart_rate(f32::from(week.number))
            ))
            .size(13)
            .color(ink(color_heart()))
        });

        let milestones = milestones(week, reveal, tint);
        let controls = self.controls();

        // Only the milestone list scrolls, so the panel never outgrows the window.
        let details = scrollable(
            column![
                text("KEY DEVELOPMENTS")
                    .size(11)
                    .font(SEMIBOLD)
                    .color(theme::MUTED),
                milestones,
            ]
            .push(heart)
            .spacing(14)
            .padding(Padding::ZERO.right(12)),
        )
        .direction(scrollable::Direction::Vertical(
            scrollable::Scrollbar::new().width(4).scroller_width(4),
        ))
        .height(Fill);

        let content = column![
            chips,
            heading,
            progress(position, tint),
            days,
            stats,
            details,
            controls,
            text("Approximate averages for education only, not medical advice.")
                .size(11)
                .color(fade(theme::MUTED, 0.7)),
        ]
        .spacing(16)
        .padding(Padding::new(26.0).top(26.0 + 6.0 * (1.0 - shown)));

        container(content)
            .width(440)
            .height(Fill)
            .style(card_style(20.0))
            .into()
    }
    fn controls(&self) -> Element<'static, Message> {
        row![
            control("Prev", Message::Previous, self.selected > 1),
            control(
                if self.playing { "Pause" } else { "Play" },
                Message::TogglePlay,
                true
            ),
            control("Next", Message::Next, self.selected < WEEKS),
        ]
        .spacing(8)
        .into()
    }
}

/// Milestone list; each line fades and slides in slightly after the previous one.
fn milestones(week: &Week, reveal: f32, tint: Color) -> Element<'_, Message> {
    column(week.milestones.iter().enumerate().map(|(i, line)| {
        // Stagger each line's fade-in a little.
        let a = ease_out_cubic((reveal - i as f32 * 0.12) / 0.6);
        row![
            container(Space::new().width(8).height(8)).style(move |_| container::Style {
                background: Some(Background::Color(fade(tint, a))),
                border: Border::default().rounded(4),
                ..container::Style::default()
            }),
            text(*line)
                .size(15)
                .color(fade(theme::TEXT, 0.15 + 0.85 * a)),
        ]
        .spacing(12)
        .align_y(Alignment::Center)
        .padding(Padding::ZERO.left(8.0 * (1.0 - a)))
        .into()
    }))
    .spacing(12)
    .into()
}

fn shortcut(key: &Key) -> Option<Message> {
    match key {
        Key::Named(Named::ArrowRight | Named::ArrowDown | Named::PageDown) => Some(Message::Next),
        Key::Named(Named::ArrowLeft | Named::ArrowUp | Named::PageUp) => Some(Message::Previous),
        Key::Named(Named::Home) => Some(Message::First),
        Key::Named(Named::End) => Some(Message::Last),
        Key::Named(Named::Space) => Some(Message::TogglePlay),
        _ => None,
    }
}

fn header<'a>() -> Element<'a, Message> {
    row![
        container(Space::new().width(12).height(12)).style(|_| container::Style {
            background: Some(Background::Color(theme::ACCENT)),
            border: Border::default().rounded(6),
            shadow: Shadow {
                color: fade(theme::ACCENT, 0.6),
                offset: Vector::ZERO,
                blur_radius: 12.0,
            },
            ..container::Style::default()
        }),
        text("embryo").size(20).font(BOLD),
        text("A week-by-week journey through prenatal development")
            .size(14)
            .color(theme::MUTED),
        Space::new().width(Fill),
        text("← → or scroll to navigate · Space to play · Home / End to jump")
            .size(12)
            .color(theme::MUTED),
    ]
    .spacing(14)
    .align_y(Alignment::Center)
    .into()
}

/// Thin bar showing progress through the 40 weeks.
fn progress<'a>(position: f32, tint: Color) -> Element<'a, Message> {
    let done = ((position / f32::from(WEEKS)) * 1000.0).round() as u16;
    let bar = |portion: u16, color: Color| {
        container(Space::new().height(4))
            .width(Length::FillPortion(portion.max(1)))
            .style(move |_| container::Style {
                background: Some(Background::Color(color)),
                border: Border::default().rounded(2),
                ..container::Style::default()
            })
    };
    row![
        bar(done, tint),
        bar(1000 - done.min(999), theme::SURFACE_RAISED)
    ]
    .spacing(4)
    .into()
}

fn chip(label: &str, tint: Color) -> Element<'_, Message> {
    container(text(label).size(12).font(SEMIBOLD).color(tint))
        .padding([4, 12])
        .style(move |_| container::Style {
            background: Some(Background::Color(fade(tint, 0.14))),
            border: Border::default()
                .rounded(999)
                .width(1)
                .color(fade(tint, 0.4)),
            ..container::Style::default()
        })
        .into()
}

fn stat<'a>(
    label: &'a str,
    value: String,
    note: &'a str,
    ink: impl Fn(Color) -> Color,
) -> Element<'a, Message> {
    container(
        column![
            text(label).size(10).font(SEMIBOLD).color(theme::MUTED),
            text(value).size(18).font(BOLD).color(ink(theme::TEXT)),
            text(note).size(10).color(theme::MUTED),
        ]
        .spacing(4),
    )
    .padding([12, 14])
    .width(Fill)
    .style(move |_| container::Style {
        background: Some(Background::Color(theme::SURFACE_RAISED)),
        border: Border::default().rounded(14),
        ..container::Style::default()
    })
    .into()
}

fn control(label: &'static str, message: Message, enabled: bool) -> Element<'static, Message> {
    button(
        text(label)
            .size(14)
            .font(SEMIBOLD)
            .align_x(Alignment::Center)
            .width(Fill),
    )
    .padding([10, 0])
    .width(Fill)
    .on_press_maybe(enabled.then_some(message))
    .style(|_, status| {
        let (bg, fg) = match status {
            button::Status::Hovered => (fade(theme::ACCENT, 0.28), theme::TEXT),
            button::Status::Pressed => (fade(theme::ACCENT, 0.4), theme::TEXT),
            button::Status::Disabled => (theme::SURFACE_RAISED, fade(theme::MUTED, 0.5)),
            button::Status::Active => (theme::SURFACE_RAISED, theme::TEXT),
        };
        button::Style {
            background: Some(Background::Color(bg)),
            text_color: fg,
            border: Border::default().rounded(12).width(1).color(theme::BORDER),
            ..button::Style::default()
        }
    })
    .into()
}

fn card<'a>(content: impl Into<Element<'a, Message>>, padding: f32) -> Element<'a, Message> {
    container(content)
        .padding(padding)
        .width(Fill)
        .style(card_style(18.0))
        .into()
}

fn card_style(radius: f32) -> impl Fn(&iced::Theme) -> container::Style {
    move |_| container::Style {
        background: Some(Background::Color(theme::SURFACE)),
        border: Border::default()
            .rounded(radius)
            .width(1)
            .color(theme::BORDER),
        ..container::Style::default()
    }
}

fn color_heart() -> Color {
    iced::color!(0xff6b8b)
}

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    chars.next().map_or_else(String::new, |first| {
        first.to_uppercase().chain(chars).collect()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn navigation_is_clamped() {
        let mut app = App::default();
        app.update(Message::Previous);
        assert_eq!(app.selected, 1);
        app.update(Message::Last);
        assert_eq!(app.selected, WEEKS);
        app.update(Message::Next);
        assert_eq!(app.selected, WEEKS);
        app.update(Message::Select(12));
        assert_eq!(app.selected, 12);
    }

    #[test]
    fn play_from_the_end_restarts() {
        let mut app = App::default();
        app.update(Message::Last);
        app.update(Message::TogglePlay);
        assert!(app.playing);
        assert_eq!(app.selected, 1);
    }

    #[test]
    fn autoplay_advances_and_stops_at_term() {
        let mut app = App::default();
        app.update(Message::TogglePlay);
        let mut now = app.now;
        for _ in 0..WEEKS + 2 {
            now += AUTOPLAY_STEP;
            app.update(Message::Tick(now));
        }
        assert_eq!(app.selected, WEEKS);
        assert!(!app.playing);
    }

    #[test]
    fn starting_week_is_clamped() {
        assert_eq!(App::starting_at(0).selected, 1);
        assert_eq!(App::starting_at(20).selected, 20);
        assert_eq!(App::starting_at(200).selected, WEEKS);
    }

    #[test]
    fn capitalizes_first_letter() {
        assert_eq!(capitalize("a plum"), "A plum");
        assert_eq!(capitalize(""), "");
    }
}
