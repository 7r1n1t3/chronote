//! Shared widgets and styles used by every screen.

use iced::widget::{Space, button, container, text};
use iced::{
    Alignment, Background, Border, Color, Element, Fill, Font, Shadow, Theme, Vector, font,
};

use crate::theme::{self, fade};

pub const BOLD: Font = Font {
    weight: font::Weight::Bold,
    ..Font::DEFAULT
};

pub const SEMIBOLD: Font = Font {
    weight: font::Weight::Semibold,
    ..Font::DEFAULT
};

/// Rounded surface used for panels and list entries.
pub fn card_style(radius: f32) -> impl Fn(&Theme) -> container::Style {
    move |_| container::Style {
        background: Some(Background::Color(theme::SURFACE)),
        border: Border::default()
            .rounded(radius)
            .width(1)
            .color(theme::BORDER),
        ..container::Style::default()
    }
}

pub fn card<'a, Message: 'a>(
    content: impl Into<Element<'a, Message>>,
    padding: f32,
) -> Element<'a, Message> {
    container(content)
        .padding(padding)
        .width(Fill)
        .style(card_style(18.0))
        .into()
}

/// Small pill-shaped label.
pub fn chip<'a, Message: 'a>(
    label: impl text::IntoFragment<'a>,
    tint: Color,
) -> Element<'a, Message> {
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

/// Glowing brand dot shown next to screen titles.
pub fn dot<'a, Message: 'a>(tint: Color) -> Element<'a, Message> {
    container(Space::new().width(12).height(12))
        .style(move |_| container::Style {
            background: Some(Background::Color(tint)),
            border: Border::default().rounded(6),
            shadow: Shadow {
                color: fade(tint, 0.6),
                offset: Vector::ZERO,
                blur_radius: 12.0,
            },
            ..container::Style::default()
        })
        .into()
}

/// Visual weight of a button.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tone {
    /// Neutral raised surface.
    Normal,
    /// Filled with the accent color.
    Primary,
    /// Selected state of a toggle.
    Selected,
    /// Destructive action.
    Danger,
}

pub fn button_style(tone: Tone) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_, status| {
        let base = match tone {
            Tone::Normal => theme::SURFACE_RAISED,
            Tone::Primary => fade(theme::ACCENT, 0.85),
            Tone::Selected => fade(theme::ACCENT, 0.22),
            Tone::Danger => fade(theme::DANGER, 0.18),
        };
        let accent = if tone == Tone::Danger {
            theme::DANGER
        } else {
            theme::ACCENT
        };
        let (bg, fg) = match status {
            button::Status::Hovered => (mix_over(base, accent, 0.25), theme::TEXT),
            button::Status::Pressed => (mix_over(base, accent, 0.4), theme::TEXT),
            button::Status::Disabled => (theme::SURFACE_RAISED, fade(theme::MUTED, 0.5)),
            button::Status::Active => (
                base,
                match tone {
                    Tone::Primary => theme::BACKGROUND,
                    Tone::Danger => theme::DANGER,
                    _ => theme::TEXT,
                },
            ),
        };
        button::Style {
            background: Some(Background::Color(bg)),
            text_color: fg,
            border: Border::default()
                .rounded(12)
                .width(1)
                .color(if tone == Tone::Selected {
                    fade(theme::ACCENT, 0.6)
                } else {
                    theme::BORDER
                }),
            ..button::Style::default()
        }
    }
}

/// A labelled button that fills its width.
pub fn action<'a, Message: Clone + 'a>(
    label: impl text::IntoFragment<'a>,
    message: Option<Message>,
    tone: Tone,
) -> Element<'a, Message> {
    button(
        text(label)
            .size(14)
            .font(SEMIBOLD)
            .align_x(Alignment::Center)
            .width(Fill),
    )
    .padding([10, 14])
    .on_press_maybe(message)
    .style(button_style(tone))
    .into()
}

fn mix_over(base: Color, accent: Color, t: f32) -> Color {
    let base = Color { a: 1.0, ..base };
    crate::anim::mix(base, accent, t)
}
