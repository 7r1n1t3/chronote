//! Interactive week scrubber: click, drag or scroll to pick a week.

use iced::alignment::Vertical;
use iced::mouse::{self, Button, ScrollDelta};
use iced::widget::canvas::{self, Action, Event, Frame, Geometry, Path, Stroke, Text};
use iced::widget::text::Alignment;
use iced::{Color, Font, Pixels, Point, Rectangle, Renderer, Size, Theme, font};

use crate::data::{Trimester, WEEKS};
use crate::theme::{self, fade};

const PADDING: f32 = 36.0;
const BAND: f32 = 6.0;

/// Canvas program for the bottom timeline.
#[derive(Debug, Clone, Copy)]
pub struct Timeline<Message> {
    /// Week currently selected.
    pub selected: u8,
    /// Animated (fractional) week used to place the marker.
    pub position: f32,
    /// Seconds since start, for the marker's glow.
    pub time: f32,
    pub on_select: fn(u8) -> Message,
}

/// Interaction state kept by the canvas between events.
#[derive(Debug, Default)]
pub struct State {
    dragging: bool,
    hover: Option<u8>,
    /// Accumulates fractional trackpad scrolling.
    scroll: f32,
}

struct Track {
    x0: f32,
    x1: f32,
    y: f32,
}

impl Track {
    fn new(bounds: Size) -> Self {
        Self {
            x0: PADDING,
            x1: (bounds.width - PADDING).max(PADDING + 1.0),
            y: bounds.height * 0.52,
        }
    }

    fn step(&self) -> f32 {
        (self.x1 - self.x0) / f32::from(WEEKS - 1)
    }

    fn x(&self, week: f32) -> f32 {
        self.x0 + (week - 1.0) * self.step()
    }

    fn week_at(&self, x: f32) -> u8 {
        let w = ((x - self.x0) / self.step()).round() + 1.0;
        w.clamp(1.0, f32::from(WEEKS)) as u8
    }
}

impl<Message> canvas::Program<Message> for Timeline<Message> {
    type State = State;

    fn update(
        &self,
        state: &mut State,
        event: &Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<Action<Message>> {
        let track = Track::new(bounds.size());
        let inside = cursor.position_in(bounds);

        let Event::Mouse(event) = event else {
            return None;
        };
        match event {
            mouse::Event::ButtonPressed(Button::Left) => {
                let p = inside?;
                state.dragging = true;
                Some(Action::publish((self.on_select)(track.week_at(p.x))).and_capture())
            }
            mouse::Event::ButtonReleased(Button::Left) if state.dragging => {
                state.dragging = false;
                Some(Action::capture())
            }
            mouse::Event::CursorMoved { .. } => {
                let hover = inside.map(|p| track.week_at(p.x));
                let changed = hover != state.hover;
                state.hover = hover;
                if state.dragging {
                    let x = cursor.position()?.x - bounds.x;
                    let week = track.week_at(x);
                    (week != self.selected)
                        .then(|| Action::publish((self.on_select)(week)).and_capture())
                } else {
                    changed.then(Action::request_redraw)
                }
            }
            mouse::Event::CursorLeft => {
                state.hover = None;
                Some(Action::request_redraw())
            }
            mouse::Event::WheelScrolled { delta } => {
                inside?;
                let amount = match delta {
                    ScrollDelta::Lines { x, y } => -(y + x),
                    ScrollDelta::Pixels { x, y } => -(y + x) / 40.0,
                };
                state.scroll += amount;
                let steps = state.scroll.trunc();
                if steps == 0.0 {
                    return Some(Action::capture());
                }
                state.scroll -= steps;
                let week = (f32::from(self.selected) + steps).clamp(1.0, f32::from(WEEKS)) as u8;
                Some(Action::publish((self.on_select)(week)).and_capture())
            }
            _ => None,
        }
    }

    fn draw(
        &self,
        state: &State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        let track = Track::new(bounds.size());
        let half = track.step() / 2.0;

        // Trimester bands, filled up to the current position.
        for trimester in [Trimester::First, Trimester::Second, Trimester::Third] {
            let (from, to) = trimester.weeks();
            let color = theme::trimester(trimester);
            let left = track.x(f32::from(from)) - half;
            let right = track.x(f32::from(to)) + half;
            let gap = 3.0;
            let band = |x0: f32, x1: f32| {
                Path::rounded_rectangle(
                    Point::new(x0 + gap, track.y - BAND / 2.0),
                    Size::new((x1 - x0 - 2.0 * gap).max(0.0), BAND),
                    (BAND / 2.0).into(),
                )
            };
            frame.fill(&band(left, right), fade(color, 0.18));
            let filled = (track.x(self.position) + 1.0).clamp(left, right);
            if filled > left + gap {
                frame.fill(&band(left, filled), fade(color, 0.9));
            }
            frame.fill_text(Text {
                content: trimester.label().to_uppercase(),
                position: Point::new(f32::midpoint(left, right), track.y - 30.0),
                color: fade(color, 0.85),
                size: Pixels(11.0),
                font: Font {
                    weight: font::Weight::Semibold,
                    ..crate::ui::regular()
                },
                align_x: Alignment::Center,
                align_y: Vertical::Center,
                ..Text::default()
            });
        }

        // Week ticks with labels every five weeks.
        for week in 1..=WEEKS {
            let x = track.x(f32::from(week));
            let hovered = state.hover == Some(week);
            let major = week == 1 || week % 5 == 0;
            let past = f32::from(week) <= self.position + 0.01;
            let r = if hovered {
                4.5
            } else if major {
                3.0
            } else {
                2.0
            };
            let color = if past { theme::TEXT } else { theme::MUTED };
            frame.fill(
                &Path::circle(Point::new(x, track.y + 16.0), r),
                fade(color, if hovered { 1.0 } else { 0.55 }),
            );
            if major || hovered {
                frame.fill_text(Text {
                    content: week.to_string(),
                    position: Point::new(x, track.y + 32.0),
                    color: if hovered { theme::TEXT } else { theme::MUTED },
                    size: Pixels(12.0),
                    align_x: Alignment::Center,
                    align_y: Vertical::Center,
                    font: crate::ui::regular(),
                    ..Text::default()
                });
            }
        }

        // Marker with a softly pulsing halo.
        let x = track.x(self.position);
        let center = Point::new(x, track.y);
        let color = theme::trimester(Trimester::of(self.position.round() as u8));
        let pulse = 0.5 + 0.5 * (self.time * 2.4).sin();
        frame.fill(&Path::circle(center, 18.0 + 4.0 * pulse), fade(color, 0.10));
        frame.fill(&Path::circle(center, 13.0), fade(color, 0.25));
        frame.fill(&Path::circle(center, 10.0), color);
        frame.stroke(
            &Path::circle(center, 10.0),
            Stroke::default().with_width(2.0).with_color(Color::WHITE),
        );
        frame.fill_text(Text {
            content: self.selected.to_string(),
            position: center,
            color: theme::BACKGROUND,
            size: Pixels(10.0),
            font: Font {
                weight: font::Weight::Bold,
                ..crate::ui::regular()
            },
            align_x: Alignment::Center,
            align_y: Vertical::Center,
            ..Text::default()
        });

        vec![frame.into_geometry()]
    }

    fn mouse_interaction(
        &self,
        state: &State,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        if state.dragging {
            mouse::Interaction::Grabbing
        } else if cursor.is_over(bounds) {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn track_maps_positions_to_weeks() {
        let track = Track::new(Size::new(1000.0, 100.0));
        assert_eq!(track.week_at(0.0), 1);
        assert_eq!(track.week_at(10_000.0), WEEKS);
        for week in 1..=WEEKS {
            assert_eq!(track.week_at(track.x(f32::from(week))), week);
        }
    }
}
