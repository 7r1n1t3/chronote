//! Small animation helpers: easing curves and a time-based tween.

use std::time::{Duration, Instant};

use iced::Color;

#[must_use]
pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// Hermite smoothstep from `edge0` to `edge1`, clamped to `0..=1`.
#[must_use]
pub fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

#[must_use]
pub fn ease_out_cubic(t: f32) -> f32 {
    1.0 - (1.0 - t.clamp(0.0, 1.0)).powi(3)
}

/// Linear blend between two colors, alpha included.
#[must_use]
pub fn mix(a: Color, b: Color, t: f32) -> Color {
    Color {
        r: lerp(a.r, b.r, t),
        g: lerp(a.g, b.g, t),
        b: lerp(a.b, b.b, t),
        a: lerp(a.a, b.a, t),
    }
}

/// An `f32` value that glides toward its target with an ease-out curve.
#[derive(Debug, Clone, Copy)]
pub struct Tween {
    from: f32,
    to: f32,
    start: Instant,
    duration: Duration,
}

impl Tween {
    #[must_use]
    pub fn new(value: f32, duration: Duration) -> Self {
        Self {
            from: value,
            to: value,
            start: Instant::now(),
            duration,
        }
    }

    /// Starts a new transition from the current value toward `target`.
    pub fn go(&mut self, target: f32, now: Instant) {
        self.from = self.value(now);
        self.to = target;
        self.start = now;
    }

    /// Linear progress of the current transition in `0..=1`.
    #[must_use]
    pub fn progress(&self, now: Instant) -> f32 {
        let elapsed = now.saturating_duration_since(self.start).as_secs_f32();
        (elapsed / self.duration.as_secs_f32().max(f32::EPSILON)).min(1.0)
    }

    #[must_use]
    pub fn value(&self, now: Instant) -> f32 {
        lerp(self.from, self.to, ease_out_cubic(self.progress(now)))
    }

    #[must_use]
    pub fn target(&self) -> f32 {
        self.to
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoothstep_clamps() {
        assert!(smoothstep(1.0, 2.0, 0.0).abs() < f32::EPSILON);
        assert!((smoothstep(1.0, 2.0, 3.0) - 1.0).abs() < f32::EPSILON);
        assert!((smoothstep(0.0, 1.0, 0.5) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn tween_reaches_target() {
        let start = Instant::now();
        let mut tween = Tween::new(1.0, Duration::from_millis(100));
        tween.go(5.0, start);
        assert!((tween.value(start) - 1.0).abs() < 1e-6);
        let mid = tween.value(start + Duration::from_millis(50));
        assert!(mid > 1.0 && mid < 5.0);
        assert!((tween.value(start + Duration::from_secs(1)) - 5.0).abs() < 1e-6);
    }

    #[test]
    fn retarget_starts_from_current_value() {
        let start = Instant::now();
        let mut tween = Tween::new(0.0, Duration::from_millis(100));
        tween.go(10.0, start);
        let half = start + Duration::from_millis(50);
        let current = tween.value(half);
        tween.go(0.0, half);
        assert!((tween.value(half) - current).abs() < 1e-6);
    }
}
