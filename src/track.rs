//! Canvas timeline for a user project: lasting events are drawn as
//! continuous bars, instantaneous events as points.

use iced::alignment::Vertical;
use iced::mouse::{self, Button};
use iced::widget::canvas::{self, Action, Event, Frame, Geometry, LineCap, Path, Stroke, Text};
use iced::widget::text::Alignment;
use iced::{Color, Pixels, Point, Rectangle, Renderer, Size, Theme};

use crate::menu::plural;
use crate::project::{self, EventId, EventKind, Project};
use crate::theme::{self, fade};

const PAD_X: f32 = 32.0;
const TOP: f32 = 28.0;
const AXIS_SPACE: f32 = 44.0;
const LANE: f32 = 56.0;
const BAR: f32 = 10.0;
const DOT: f32 = 7.0;

/// Canvas program drawing one project's events.
#[derive(Debug)]
pub struct Track<'a, Message> {
    pub project: &'a Project,
    pub selected: Option<EventId>,
    pub on_select: fn(Option<EventId>) -> Message,
}

#[derive(Debug, Default)]
pub struct State {
    hover: Option<EventId>,
}

/// Screen geometry of an event, used for both drawing and hit testing.
struct Placed<'a> {
    event: &'a project::Event,
    y: f32,
    x0: f32,
    x1: f32,
}

struct Layout {
    x0: f32,
    x1: f32,
    axis: f32,
    length: f64,
}

impl Layout {
    fn new(size: Size, length: f64) -> Self {
        Self {
            x0: PAD_X,
            x1: (size.width - PAD_X).max(PAD_X + 1.0),
            axis: (size.height - AXIS_SPACE).max(TOP + LANE),
            length: length.max(f64::EPSILON),
        }
    }

    fn x(&self, t: f64) -> f32 {
        let f = (t / self.length).clamp(0.0, 1.0) as f32;
        self.x0 + f * (self.x1 - self.x0)
    }
}

impl<Message> Track<'_, Message> {
    fn place(&self, layout: &Layout) -> Vec<Placed<'_>> {
        let (lanes, count) = self.project.lanes();
        // Squeeze lanes when there are many, so everything stays visible.
        let room = layout.axis - TOP - 16.0;
        let lane_h = (room / count.max(1) as f32).min(LANE);
        lanes
            .into_iter()
            .filter_map(|(id, lane)| {
                let event = self.project.event(id)?;
                Some(Placed {
                    event,
                    y: TOP + 12.0 + lane as f32 * lane_h,
                    x0: layout.x(event.kind.start()),
                    x1: layout.x(event.kind.end()),
                })
            })
            .collect()
    }

    fn hit(&self, layout: &Layout, p: Point) -> Option<EventId> {
        self.place(layout)
            .into_iter()
            .filter(|e| {
                let dy = (p.y - e.y).abs();
                if e.event.kind.is_lasting() {
                    dy <= BAR && p.x >= e.x0 - BAR && p.x <= e.x1 + BAR
                } else {
                    (p.x - e.x0).hypot(p.y - e.y) <= DOT + 5.0
                }
            })
            .min_by(|a, b| (a.y - p.y).abs().total_cmp(&(b.y - p.y).abs()))
            .map(|e| e.event.id)
    }
}

impl<Message> canvas::Program<Message> for Track<'_, Message> {
    type State = State;

    fn update(
        &self,
        state: &mut State,
        event: &Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<Action<Message>> {
        let layout = Layout::new(bounds.size(), self.project.length);
        match event {
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                let hover = cursor
                    .position_in(bounds)
                    .and_then(|p| self.hit(&layout, p));
                (hover != state.hover).then(|| {
                    state.hover = hover;
                    Action::request_redraw()
                })
            }
            Event::Mouse(mouse::Event::ButtonPressed(Button::Left)) => {
                let p = cursor.position_in(bounds)?;
                Some(Action::publish((self.on_select)(self.hit(&layout, p))).and_capture())
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
        let layout = Layout::new(bounds.size(), self.project.length);
        self.axis(&mut frame, &layout);

        let placed = self.place(&layout);
        if placed.is_empty() {
            frame.fill_text(Text {
                content: "No events yet — use “Add event” to place one on the timeline.".into(),
                position: Point::new(bounds.width / 2.0, layout.axis / 2.0),
                color: theme::MUTED,
                size: Pixels(14.0),
                align_x: Alignment::Center,
                align_y: Vertical::Center,
                font: crate::ui::regular(),
                ..Text::default()
            });
        }
        for e in &placed {
            let active = self.selected == Some(e.event.id);
            let hovered = state.hover == Some(e.event.id);
            draw_event(&mut frame, e, active, hovered);
        }
        vec![frame.into_geometry()]
    }

    fn mouse_interaction(
        &self,
        state: &State,
        _bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        if state.hover.is_some() {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::default()
        }
    }
}

impl<Message> Track<'_, Message> {
    fn axis(&self, frame: &mut Frame, layout: &Layout) {
        let step = tick_step(layout.length);
        let mut t = 0.0;
        while t <= layout.length + step * 1e-6 {
            let x = layout.x(t);
            frame.stroke(
                &Path::line(Point::new(x, TOP), Point::new(x, layout.axis)),
                Stroke::default()
                    .with_width(1.0)
                    .with_color(fade(theme::BORDER, 0.6)),
            );
            frame.fill_text(Text {
                content: project::format_units(t),
                position: Point::new(x, layout.axis + 16.0),
                color: theme::MUTED,
                size: Pixels(11.0),
                align_x: Alignment::Center,
                align_y: Vertical::Center,
                font: crate::ui::regular(),
                ..Text::default()
            });
            t += step;
        }
        frame.stroke(
            &Path::line(
                Point::new(layout.x0, layout.axis),
                Point::new(layout.x1, layout.axis),
            ),
            Stroke::default()
                .with_width(2.0)
                .with_line_cap(LineCap::Round)
                .with_color(theme::BORDER),
        );
        frame.fill_text(Text {
            content: plural(&self.project.unit, 2.0).to_uppercase(),
            position: Point::new(layout.x1, layout.axis + 32.0),
            color: fade(theme::MUTED, 0.8),
            size: Pixels(10.0),
            align_x: Alignment::Right,
            align_y: Vertical::Center,
            font: crate::ui::regular(),
            ..Text::default()
        });
    }
}

fn draw_event(frame: &mut Frame, e: &Placed<'_>, active: bool, hovered: bool) {
    let base = if e.event.kind.is_lasting() {
        theme::SECOND
    } else {
        theme::FIRST
    };
    let color = if active { theme::ACCENT } else { base };
    let glow = if active {
        0.35
    } else if hovered {
        0.22
    } else {
        0.0
    };

    match e.event.kind {
        EventKind::Lasting { .. } => {
            let line = Path::line(Point::new(e.x0, e.y), Point::new(e.x1, e.y));
            if glow > 0.0 {
                frame.stroke(
                    &line,
                    Stroke::default()
                        .with_width(BAR * 2.2)
                        .with_line_cap(LineCap::Round)
                        .with_color(fade(color, glow)),
                );
            }
            frame.stroke(
                &line,
                Stroke::default()
                    .with_width(BAR)
                    .with_line_cap(LineCap::Round)
                    .with_color(color),
            );
        }
        EventKind::Instant { .. } => {
            let center = Point::new(e.x0, e.y);
            if glow > 0.0 {
                frame.fill(&Path::circle(center, DOT * 2.0), fade(color, glow));
            }
            frame.fill(&Path::circle(center, DOT), color);
            frame.stroke(
                &Path::circle(center, DOT),
                Stroke::default().with_width(2.0).with_color(Color::WHITE),
            );
        }
    }

    let label_x = if e.event.kind.is_lasting() {
        e.x0
    } else {
        e.x0 + DOT + 6.0
    };
    let (label_y, align_y) = if e.event.kind.is_lasting() {
        (e.y - BAR - 2.0, Vertical::Bottom)
    } else {
        (e.y, Vertical::Center)
    };
    frame.fill_text(Text {
        content: if e.event.title.is_empty() {
            "Untitled".to_owned()
        } else {
            e.event.title.clone()
        },
        position: Point::new(label_x, label_y),
        color: if active || hovered {
            theme::TEXT
        } else {
            fade(theme::TEXT, 0.75)
        },
        size: Pixels(12.0),
        align_y,
        font: crate::ui::regular(),
        ..Text::default()
    });
}

/// A "nice" tick spacing (1, 2 or 5 × 10ⁿ) giving at most ~12 ticks.
#[must_use]
pub fn tick_step(length: f64) -> f64 {
    let raw = (length / 12.0).max(f64::EPSILON);
    let magnitude = 10f64.powf(raw.log10().floor());
    [1.0, 2.0, 5.0, 10.0]
        .into_iter()
        .map(|m| m * magnitude)
        .find(|step| *step >= raw)
        .unwrap_or(10.0 * magnitude)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tick_steps_are_nice() {
        assert!((tick_step(10.0) - 1.0).abs() < 1e-9);
        assert!((tick_step(30.0) - 5.0).abs() < 1e-9);
        assert!((tick_step(100.0) - 10.0).abs() < 1e-9);
        assert!((tick_step(1.0) - 0.1).abs() < 1e-9);
        assert!(30.0 / tick_step(30.0) <= 12.0);
    }

    #[test]
    fn layout_clamps_positions() {
        let layout = Layout::new(Size::new(600.0, 300.0), 10.0);
        assert!((layout.x(-5.0) - layout.x0).abs() < f32::EPSILON);
        assert!((layout.x(50.0) - layout.x1).abs() < f32::EPSILON);
    }
}
