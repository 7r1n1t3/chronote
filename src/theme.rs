//! Colors and the application theme.

use iced::theme::Palette;
use iced::{Color, Theme, color};

use crate::data::Trimester;

pub const BACKGROUND: Color = color!(0x0b0e1a);
pub const SURFACE: Color = color!(0x141a2e);
pub const SURFACE_RAISED: Color = color!(0x1c2440);
pub const BORDER: Color = color!(0x2a3358);
pub const TEXT: Color = color!(0xe8ecf8);
pub const MUTED: Color = color!(0x8a93b2);
pub const ACCENT: Color = color!(0xff7a9c);
pub const DANGER: Color = color!(0xf56565);

pub const FIRST: Color = color!(0x4fd1c5);
pub const SECOND: Color = color!(0x9f7aea);
pub const THIRD: Color = color!(0xf6ad55);

/// Signature color of a trimester.
#[must_use]
pub fn trimester(trimester: Trimester) -> Color {
    match trimester {
        Trimester::First => FIRST,
        Trimester::Second => SECOND,
        Trimester::Third => THIRD,
    }
}

/// `color` with its alpha multiplied by `alpha`.
#[must_use]
pub fn fade(color: Color, alpha: f32) -> Color {
    Color {
        a: color.a * alpha.clamp(0.0, 1.0),
        ..color
    }
}

#[must_use]
pub fn theme() -> Theme {
    Theme::custom(
        "Embryo",
        Palette {
            background: BACKGROUND,
            text: TEXT,
            primary: ACCENT,
            success: FIRST,
            warning: THIRD,
            danger: DANGER,
        },
    )
}
