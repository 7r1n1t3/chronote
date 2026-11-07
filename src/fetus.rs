//! Procedural, animated illustration of the embryo / fetus.
//!
//! Everything is driven by a continuous `week` value, so moving between
//! weeks morphs the drawing smoothly instead of swapping pictures.

use std::f32::consts::{PI, TAU};

use iced::widget::canvas::{self, Frame, Geometry, LineCap, Path, Stroke, gradient};
use iced::{Color, Point, Rectangle, Renderer, Theme, Vector, color, mouse};

use crate::anim::{lerp, mix, smoothstep};
use crate::theme::{self, fade};

/// Canvas program drawing the developing baby at a given (fractional) week.
#[derive(Debug, Clone, Copy)]
pub struct Fetus {
    /// Gestational week, may be fractional while animating.
    pub week: f32,
    /// Seconds since the app started, drives idle motion.
    pub time: f32,
}

impl<Message> canvas::Program<Message> for Fetus {
    type State = ();

    fn draw(
        &self,
        _state: &(),
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        let scene = Scene::new(&frame, self.week, self.time);

        scene.backdrop(&mut frame);
        scene.particles(&mut frame);
        scene.ovum(&mut frame);
        scene.cells(&mut frame);
        scene.body(&mut frame);

        vec![frame.into_geometry()]
    }
}

/// Precomputed layout shared by all the drawing passes of one frame.
struct Scene {
    center: Point,
    /// Radius of the amniotic sac "window".
    sac: f32,
    week: f32,
    time: f32,
}

impl Scene {
    fn new(frame: &Frame, week: f32, time: f32) -> Self {
        let size = frame.size();
        Self {
            center: frame.center(),
            sac: size.width.min(size.height) * 0.42,
            week,
            time,
        }
    }

    fn backdrop(&self, frame: &mut Frame) {
        let breathe = 1.0 + 0.015 * (self.time * 0.9).sin();

        // Soft outer glow built from stacked translucent discs.
        for i in 0..10 {
            let k = i as f32 / 10.0;
            let r = self.sac * breathe * lerp(1.32, 1.02, k);
            frame.fill(
                &Path::circle(self.center, r),
                fade(theme::ACCENT, 0.012 + 0.006 * k),
            );
        }

        let sac = Path::circle(self.center, self.sac * breathe);
        let top = self.center - Vector::new(self.sac * 0.6, self.sac);
        let bottom = self.center + Vector::new(self.sac * 0.6, self.sac);
        frame.fill(
            &sac,
            gradient::Linear::new(top, bottom)
                .add_stop(0.0, color!(0x3a2246))
                .add_stop(0.55, color!(0x221832))
                .add_stop(1.0, color!(0x140f22)),
        );
        frame.stroke(
            &sac,
            Stroke::default()
                .with_width(2.0)
                .with_color(fade(theme::ACCENT, 0.35)),
        );
        frame.stroke(
            &Path::circle(self.center, self.sac * breathe * 0.94),
            Stroke::default()
                .with_width(1.0)
                .with_color(fade(theme::ACCENT, 0.08)),
        );

        // Placenta on the upper-right wall, once the pregnancy is implanted.
        let placenta = smoothstep(4.0, 6.0, self.week);
        if placenta > 0.0 {
            let arc = Path::new(|p| {
                p.arc(canvas::path::Arc {
                    center: self.center,
                    radius: self.sac * 0.9,
                    start_angle: (-70f32).to_radians().into(),
                    end_angle: (-5f32).to_radians().into(),
                });
            });
            let width = self.sac * lerp(0.06, 0.14, smoothstep(5.0, 30.0, self.week));
            frame.stroke(
                &arc,
                Stroke::default()
                    .with_width(width)
                    .with_line_cap(LineCap::Round)
                    .with_color(fade(color!(0x8c2f55), 0.75 * placenta)),
            );
            frame.stroke(
                &arc,
                Stroke::default()
                    .with_width(width * 0.35)
                    .with_line_cap(LineCap::Round)
                    .with_color(fade(color!(0xc2507a), 0.5 * placenta)),
            );
        }
    }

    /// Slowly drifting specks suspended in the amniotic fluid.
    fn particles(&self, frame: &mut Frame) {
        for i in 0..28 {
            let seed = i as f32;
            let radius = self.sac * 0.85 * hash(seed * 1.7).sqrt();
            let drift = lerp(0.02, 0.07, hash(seed * 3.1)) * if i % 2 == 0 { 1.0 } else { -1.0 };
            let angle = hash(seed * 5.3) * TAU + self.time * drift;
            let wobble = (self.time * 0.6 + seed).sin() * self.sac * 0.02;
            let p = self.center + polar(radius + wobble, angle);
            let twinkle = 0.5 + 0.5 * (self.time * 1.3 + seed * 2.0).sin();
            frame.fill(
                &Path::circle(p, lerp(1.0, 2.6, hash(seed * 7.7))),
                fade(Color::WHITE, 0.05 + 0.12 * twinkle),
            );
        }
    }

    /// Weeks 1–3: the egg cell, with sperm approaching around ovulation.
    fn ovum(&self, frame: &mut Frame) {
        let alpha = 1.0 - smoothstep(2.6, 3.1, self.week);
        if alpha <= 0.0 {
            return;
        }
        // The egg is still maturing in week 1, so it starts faint.
        let alpha = alpha * lerp(0.4, 1.0, smoothstep(1.0, 2.0, self.week));
        let r = self.sac * 0.2 * (1.0 + 0.02 * (self.time * 1.6).sin());
        let c = self.center;

        // Corona radiata: a halo of small follicle cells.
        for i in 0..36 {
            let a = i as f32 / 36.0 * TAU + self.time * 0.05;
            let d = r * (1.32 + 0.06 * (i as f32 * 1.9 + self.time).sin());
            frame.fill(
                &Path::circle(c + polar(d, a), r * 0.09),
                fade(color!(0xf6c28b), 0.55 * alpha),
            );
        }
        frame.fill(
            &Path::circle(c, r * 1.18),
            fade(color!(0xffe3c4), 0.18 * alpha),
        );
        frame.fill(&Path::circle(c, r), fade(color!(0xffd1dc), 0.85 * alpha));
        frame.fill(
            &Path::circle(c - Vector::new(r * 0.25, r * 0.25), r * 0.55),
            fade(Color::WHITE, 0.18 * alpha),
        );
        frame.fill(
            &Path::circle(c + Vector::new(r * 0.15, r * 0.1), r * 0.28),
            fade(color!(0xd9638a), 0.8 * alpha),
        );

        let sperm = smoothstep(1.8, 2.3, self.week) * (1.0 - smoothstep(2.7, 3.0, self.week));
        for i in 0..5 {
            let seed = i as f32;
            let phase = (self.time * 0.12 + hash(seed * 9.1)).fract();
            let a = PI + lerp(-0.9, 0.9, hash(seed * 4.4));
            let dist = lerp(self.sac * 0.95, r * 1.45, phase);
            let head = c + polar(dist, a);
            draw_sperm(
                frame,
                head,
                a,
                r * 0.12,
                self.time * 9.0 + seed,
                sperm * alpha,
            );
        }
    }

    /// Weeks 3–5: cleavage into a morula, then a hollow blastocyst.
    fn cells(&self, frame: &mut Frame) {
        let alpha = smoothstep(2.6, 3.1, self.week) * (1.0 - smoothstep(4.6, 5.2, self.week));
        if alpha <= 0.0 {
            return;
        }
        let grow = ((self.week - 2.9) / 1.2).clamp(0.0, 1.0);
        let count = 2f32.powf(grow * 5.0).round().max(1.0) as usize;
        let blast = smoothstep(3.7, 4.4, self.week);
        let cluster = self.sac * lerp(0.17, 0.26, blast);
        let c = self.center + Vector::new(0.0, (self.time * 0.8).sin() * self.sac * 0.01);

        frame.fill(
            &Path::circle(c, cluster * 1.15),
            fade(color!(0xffe3c4), 0.12 * alpha),
        );

        let packed_r = cluster / (count as f32).sqrt() * 0.95;
        let ring_r = (cluster * PI / count as f32 * 1.05).min(packed_r * 1.4);
        for i in 0..count {
            let fi = i as f32;
            let jiggle = Vector::new(
                (self.time * 1.1 + fi).sin(),
                (self.time * 0.9 + fi * 1.3).cos(),
            ) * cluster
                * 0.012;
            // Golden-angle packing for the morula ...
            let packed = polar(
                cluster * 0.92 * ((fi + 0.5) / count as f32).sqrt(),
                fi * 2.399_963,
            );
            // ... morphing into a ring for the blastocyst.
            let ring = polar(cluster * 0.9, fi / count as f32 * TAU + self.time * 0.03);
            let p = c + lerp_vec(packed, ring, blast) + jiggle;
            let cr = lerp(packed_r, ring_r, blast);
            frame.fill(&Path::circle(p, cr), fade(color!(0xf7a1b5), 0.9 * alpha));
            frame.fill(
                &Path::circle(p - Vector::new(cr * 0.3, cr * 0.3), cr * 0.45),
                fade(Color::WHITE, 0.2 * alpha),
            );
        }

        // Inner cell mass, the part that becomes the embryo.
        if blast > 0.0 {
            for i in 0..7 {
                let fi = i as f32;
                let p = c
                    + Vector::new(cluster * 0.42, -cluster * 0.2)
                    + polar(cluster * 0.16 * (fi / 7.0).sqrt(), fi * 2.4);
                frame.fill(
                    &Path::circle(p, cluster * 0.14),
                    fade(color!(0xe0628a), 0.9 * alpha * blast),
                );
            }
        }
    }

    /// Week 5 onward: the curled embryo, then the fetus.
    fn body(&self, frame: &mut Frame) {
        let alpha = smoothstep(4.6, 5.2, self.week);
        if alpha <= 0.0 {
            return;
        }
        let body = Body::new(self, alpha);
        body.draw(frame);
    }
}

/// Geometry of the curled body for one frame.
///
/// The spine is an arc around `pivot`, running from the neck (top) down
/// the left side to the rump. The belly faces the inside of the curve,
/// which is where the limbs fold.
struct Body {
    week: f32,
    time: f32,
    alpha: f32,
    pivot: Point,
    radius: f32,
    start: f32,
    span: f32,
    tail: f32,
    thickness: f32,
    head: f32,
    skin: Color,
    shade: Color,
    sac_center: Point,
    sac: f32,
}

impl Body {
    fn new(scene: &Scene, alpha: f32) -> Self {
        let w = scene.week;
        let growth = ((w - 5.0) / 35.0).clamp(0.0, 1.0).powf(0.7);
        let size = scene.sac * lerp(0.42, 1.55, growth);
        let radius = size * 0.32;
        let bob = Vector::new(
            (scene.time * 0.45).sin() * scene.sac * 0.008,
            (scene.time * 0.8).sin() * scene.sac * 0.015,
        );
        let skin = mix(
            mix(color!(0xf28aa0), color!(0xf7a8a8), smoothstep(8.0, 20.0, w)),
            color!(0xf6bfa0),
            smoothstep(24.0, 36.0, w),
        );
        Self {
            week: w,
            time: scene.time,
            alpha,
            pivot: scene.center + Vector::new(-radius * 0.25, radius * 0.1) + bob,
            radius,
            start: (-70f32).to_radians(),
            span: lerp(255.0, 205.0, smoothstep(5.0, 20.0, w)).to_radians(),
            tail: 45f32.to_radians() * (1.0 - smoothstep(7.0, 9.5, w)),
            thickness: radius
                * (lerp(0.42, 0.6, smoothstep(5.0, 24.0, w)) + 0.1 * smoothstep(26.0, 38.0, w)),
            head: radius * lerp(0.95, 0.62, smoothstep(6.0, 36.0, w)),
            skin,
            shade: mix(skin, color!(0x6b2340), 0.35),
            sac_center: scene.center,
            sac: scene.sac,
        }
    }

    fn angle(&self, t: f32) -> f32 {
        self.start - self.span * t
    }

    fn spine(&self, t: f32) -> Point {
        self.pivot + polar(self.radius, self.angle(t))
    }

    /// Unit vector from the spine toward the belly side.
    fn inward(&self, t: f32) -> Vector {
        polar(-1.0, self.angle(t))
    }

    fn width(&self, t: f32) -> f32 {
        if t <= 1.0 {
            self.thickness * (0.55 + 0.45 * (PI * t).sin().max(0.0).powf(0.7))
        } else {
            let tail_t = self.tail / self.span;
            self.thickness * 0.55 * (1.0 - (t - 1.0) / tail_t).max(0.0)
        }
    }

    fn color(&self, c: Color) -> Color {
        fade(c, self.alpha)
    }

    fn draw(&self, frame: &mut Frame) {
        self.yolk_sac(frame);
        self.cord(frame);
        self.limbs(frame, false);
        self.torso(frame);
        self.head(frame);
        self.limbs(frame, true);
        self.heart(frame);
    }

    fn torso(&self, frame: &mut Frame) {
        let end = 1.0 + self.tail / self.span;
        let steps = 48;
        let sample = |i: usize| end * i as f32 / steps as f32;

        let outline = Path::new(|p| {
            for i in 0..=steps {
                let t = sample(i);
                let pt = self.spine(t) - self.inward(t) * (self.width(t) * 0.5);
                if i == 0 {
                    p.move_to(pt);
                } else {
                    p.line_to(pt);
                }
            }
            for i in (0..=steps).rev() {
                let t = sample(i);
                p.line_to(self.spine(t) + self.inward(t) * (self.width(t) * 0.5));
            }
            p.close();
        });

        let back = self.spine(0.5) - self.inward(0.5) * self.thickness;
        let belly = self.spine(0.5) + self.inward(0.5) * self.thickness;
        frame.fill(
            &outline,
            gradient::Linear::new(back, belly)
                .add_stop(0.0, self.color(mix(self.skin, Color::WHITE, 0.12)))
                .add_stop(1.0, self.color(self.shade)),
        );
        // Rounded rump.
        frame.fill(
            &Path::circle(self.spine(1.0), self.width(1.0) * 0.5),
            self.color(mix(self.skin, self.shade, 0.5)),
        );

        // Gentle highlight along the back.
        let highlight = Path::new(|p| {
            for i in 2..=steps - 8 {
                let t = sample(i).min(1.0);
                let pt = self.spine(t) - self.inward(t) * (self.width(t) * 0.3);
                if i == 2 {
                    p.move_to(pt);
                } else {
                    p.line_to(pt);
                }
            }
        });
        frame.stroke(
            &highlight,
            Stroke::default()
                .with_width(self.thickness * 0.12)
                .with_line_cap(LineCap::Round)
                .with_color(self.color(fade(Color::WHITE, 0.18))),
        );

        // Early on, the spinal segments (somites) show through the skin.
        let somites = 1.0 - smoothstep(8.0, 11.0, self.week);
        if somites > 0.0 {
            for i in 2..22 {
                let t = i as f32 / 22.0;
                let p = self.spine(t) - self.inward(t) * (self.width(t) * 0.28);
                frame.fill(
                    &Path::circle(p, self.thickness * 0.05),
                    self.color(fade(self.shade, 0.6 * somites)),
                );
            }
        }
    }

    fn head_center(&self) -> Point {
        let a = self.start;
        let tangent = Vector::new(-a.sin(), a.cos());
        self.spine(0.0) + tangent * (self.head * 0.55) - self.inward(0.0) * (self.head * 0.25)
    }

    fn head(&self, frame: &mut Frame) {
        let c = self.head_center();
        let r = self.head;
        let face =
            normalize(Vector::new(-self.start.sin(), self.start.cos()) + self.inward(0.0) * 0.5);

        frame.fill(
            &Path::circle(c, r),
            gradient::Linear::new(c - Vector::new(r, r), c + Vector::new(r, r))
                .add_stop(0.0, self.color(mix(self.skin, Color::WHITE, 0.15)))
                .add_stop(1.0, self.color(mix(self.skin, self.shade, 0.6))),
        );
        frame.fill(
            &Path::circle(c - Vector::new(r * 0.3, r * 0.35), r * 0.45),
            self.color(fade(Color::WHITE, 0.12)),
        );

        // Translucent early brain vesicles.
        let brain = 1.0 - smoothstep(10.0, 16.0, self.week);
        if brain > 0.0 {
            frame.fill(
                &Path::circle(c - face * (r * 0.2), r * 0.55),
                self.color(fade(color!(0xd75f86), 0.25 * brain)),
            );
        }

        // Eye: a dark spot, then closed lids (wk 9–26), then open again.
        let eye_alpha = smoothstep(6.5, 8.0, self.week);
        if eye_alpha > 0.0 {
            let side = Vector::new(-face.y, face.x);
            let eye = c + face * (r * 0.5) - side * (r * 0.12);
            let closed =
                smoothstep(8.5, 9.5, self.week) * (1.0 - smoothstep(25.5, 27.0, self.week));
            let ink = self.color(fade(color!(0x3a1426), eye_alpha));
            if closed > 0.5 {
                let a = side.y.atan2(side.x);
                let lid = Path::new(|p| {
                    p.arc(canvas::path::Arc {
                        center: eye - face * (r * 0.04),
                        radius: r * 0.11,
                        start_angle: (a - 0.2).into(),
                        end_angle: (a + PI + 0.2).into(),
                    });
                });
                frame.stroke(
                    &lid,
                    Stroke::default()
                        .with_width(r * 0.035)
                        .with_line_cap(LineCap::Round)
                        .with_color(ink),
                );
            } else {
                // Occasional blink once the eyes have opened.
                let blink: f32 = if self.week > 26.0 && (self.time * 0.5).fract() < 0.04 {
                    0.3
                } else {
                    1.0
                };
                frame.fill(&Path::circle(eye, r * 0.08 * blink), ink);
            }
        }

        // Ear, and a cheek blush late in pregnancy.
        let ear = smoothstep(8.0, 11.0, self.week);
        if ear > 0.0 {
            let pos = c - face * (r * 0.15) - Vector::new(-face.y, face.x) * (r * 0.05);
            frame.stroke(
                &Path::new(|p| {
                    p.arc(canvas::path::Arc {
                        center: pos,
                        radius: r * 0.13,
                        start_angle: (-PI * 0.4).into(),
                        end_angle: (PI * 0.7).into(),
                    });
                }),
                Stroke::default()
                    .with_width(r * 0.04)
                    .with_line_cap(LineCap::Round)
                    .with_color(self.color(fade(self.shade, ear))),
            );
        }
        let blush = smoothstep(26.0, 36.0, self.week);
        if blush > 0.0 {
            frame.fill(
                &Path::circle(c + face * (r * 0.62), r * 0.14),
                self.color(fade(color!(0xff8c9a), 0.35 * blush)),
            );
        }
    }

    /// Arms and legs; `near` selects the limb pair closer to the viewer.
    fn limbs(&self, frame: &mut Frame, near: bool) {
        let arm = smoothstep(5.6, 13.0, self.week);
        let leg = smoothstep(6.0, 14.0, self.week);
        if arm <= 0.0 {
            return;
        }
        // Small kicks and wiggles once movement starts (~week 8).
        let motion = smoothstep(7.5, 10.0, self.week) * if near { 1.0 } else { 0.6 };
        let kick = (self.time * 1.7).sin().max(0.0).powi(4) * 0.35 * motion;
        let wave = (self.time * 1.1 + 1.0).sin() * 0.12 * motion;

        let (offset, tint) = if near {
            (Vector::new(0.0, 0.0), self.skin)
        } else {
            (
                Vector::new(-self.radius * 0.08, -self.radius * 0.05),
                self.shade,
            )
        };
        let r = self.radius;

        let shoulder = self.spine(0.16) + self.inward(0.16) * (self.thickness * 0.15) + offset;
        let elbow = shoulder + polar(r * 0.5 * arm, (70f32).to_radians() + wave);
        let hand = elbow + polar(r * 0.45 * arm, (-45f32).to_radians() + wave * 1.5);
        self.limb(frame, &[shoulder, elbow, hand], self.thickness * 0.36, tint);

        if leg > 0.0 {
            let hip = self.spine(0.86) + self.inward(0.86) * (self.thickness * 0.1) + offset;
            let knee = hip + polar(r * 0.65 * leg, (-40f32).to_radians() - kick);
            let foot = knee + polar(r * 0.6 * leg, (115f32).to_radians() - kick * 1.6);
            self.limb(frame, &[hip, knee, foot], self.thickness * 0.48, tint);
        }
    }

    fn limb(&self, frame: &mut Frame, joints: &[Point], width: f32, tint: Color) {
        let path = Path::new(|p| {
            p.move_to(joints[0]);
            for j in &joints[1..] {
                p.line_to(*j);
            }
        });
        frame.stroke(
            &path,
            Stroke::default()
                .with_width(width)
                .with_line_cap(LineCap::Round)
                .with_line_join(canvas::LineJoin::Round)
                .with_color(self.color(tint)),
        );
        frame.stroke(
            &path,
            Stroke::default()
                .with_width(width * 0.25)
                .with_line_cap(LineCap::Round)
                .with_line_join(canvas::LineJoin::Round)
                .with_color(self.color(fade(Color::WHITE, 0.12))),
        );
        if let Some(end) = joints.last() {
            frame.fill(&Path::circle(*end, width * 0.62), self.color(tint));
        }
    }

    fn cord(&self, frame: &mut Frame) {
        let belly = self.spine(0.62) + self.inward(0.62) * (self.width(0.62) * 0.4);
        let placenta = self.sac_center + polar(self.sac * 0.88, (-35f32).to_radians());
        let sway = (self.time * 0.7).sin() * self.radius * 0.15;
        let c1 = belly + Vector::new(self.radius * 0.8, self.radius * 0.5 + sway);
        let c2 = placenta + Vector::new(-self.radius * 0.2, self.radius * 0.9 - sway);
        let path = Path::new(|p| {
            p.move_to(belly);
            p.bezier_curve_to(c1, c2, placenta);
        });
        let width = self.thickness * 0.2;
        frame.stroke(
            &path,
            Stroke::default()
                .with_width(width)
                .with_line_cap(LineCap::Round)
                .with_color(self.color(color!(0xb0507e))),
        );
        frame.stroke(
            &path,
            Stroke {
                line_dash: canvas::LineDash {
                    segments: &[width * 0.6, width * 0.9],
                    offset: (self.time * 4.0) as usize,
                },
                ..Stroke::default()
                    .with_width(width * 0.35)
                    .with_line_cap(LineCap::Round)
                    .with_color(self.color(fade(color!(0xf2a3c4), 0.6)))
            },
        );
    }

    /// The yolk sac nourishes the embryo until the placenta takes over.
    fn yolk_sac(&self, frame: &mut Frame) {
        let a = 1.0 - smoothstep(9.0, 12.0, self.week);
        if a <= 0.0 {
            return;
        }
        let belly = self.spine(0.55) + self.inward(0.55) * (self.width(0.55) * 0.4);
        let c = belly + Vector::new(self.radius * 0.9, self.radius * 0.35);
        let r = self.radius * 0.32;
        frame.stroke(
            &Path::line(belly, c),
            Stroke::default()
                .with_width(self.thickness * 0.08)
                .with_color(self.color(fade(color!(0xf6d38b), 0.6 * a))),
        );
        frame.fill(
            &Path::circle(c, r),
            self.color(fade(color!(0xf6d38b), 0.55 * a)),
        );
        frame.fill(
            &Path::circle(c - Vector::new(r * 0.3, r * 0.3), r * 0.4),
            self.color(fade(Color::WHITE, 0.25 * a)),
        );
    }

    /// Pulsing glow at the heart, beating at an age-appropriate rate.
    fn heart(&self, frame: &mut Frame) {
        let visible = smoothstep(5.5, 6.5, self.week);
        if visible <= 0.0 {
            return;
        }
        let bpm = heart_rate(self.week);
        let phase = (self.time * bpm / 60.0).fract();
        let beat = (-phase * 9.0).exp()
            + 0.6 * (-(phase - 0.18).max(0.0) * 12.0).exp() * f32::from(u8::from(phase > 0.18));
        // The heart is easy to see through thin early skin, less so later.
        let strength = visible * lerp(1.0, 0.45, smoothstep(12.0, 24.0, self.week));
        let p = self.spine(0.3) + self.inward(0.3) * (self.width(0.3) * 0.22);
        let r = self.thickness * 0.16 * (1.0 + 0.35 * beat);
        for (scale, a) in [(2.6, 0.08), (1.7, 0.18), (1.0, 0.85)] {
            frame.fill(
                &Path::circle(p, r * scale),
                self.color(fade(color!(0xff3d6e), a * strength * (0.55 + 0.45 * beat))),
            );
        }
    }
}

/// Typical fetal heart rate in beats per minute for a gestational week.
#[must_use]
pub fn heart_rate(week: f32) -> f32 {
    if week < 9.0 {
        lerp(100.0, 170.0, ((week - 6.0) / 3.0).clamp(0.0, 1.0))
    } else {
        lerp(170.0, 140.0, ((week - 9.0) / 12.0).clamp(0.0, 1.0))
    }
}

fn draw_sperm(frame: &mut Frame, head: Point, angle: f32, size: f32, phase: f32, alpha: f32) {
    if alpha <= 0.0 {
        return;
    }
    let dir = polar(1.0, angle);
    let normal = Vector::new(-dir.y, dir.x);
    let tail = Path::new(|p| {
        p.move_to(head);
        for i in 1..=12 {
            let t = i as f32 / 12.0;
            let wiggle = (phase + t * 7.0).sin() * size * 0.6 * t;
            p.line_to(head + dir * (size * 6.0 * t) + normal * wiggle);
        }
    });
    frame.stroke(
        &tail,
        Stroke::default()
            .with_width(size * 0.25)
            .with_line_cap(LineCap::Round)
            .with_color(fade(Color::WHITE, 0.45 * alpha)),
    );
    frame.fill(
        &Path::circle(head, size * 0.55),
        fade(Color::WHITE, 0.8 * alpha),
    );
}

fn polar(radius: f32, angle: f32) -> Vector {
    Vector::new(radius * angle.cos(), radius * angle.sin())
}

fn lerp_vec(a: Vector, b: Vector, t: f32) -> Vector {
    Vector::new(lerp(a.x, b.x, t), lerp(a.y, b.y, t))
}

fn normalize(v: Vector) -> Vector {
    let len = v.x.hypot(v.y).max(f32::EPSILON);
    Vector::new(v.x / len, v.y / len)
}

/// Deterministic pseudo-random number in `0..1`.
fn hash(x: f32) -> f32 {
    ((x * 12.989_8).sin() * 43_758.547).fract().abs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn heart_rate_peaks_around_week_nine() {
        assert!(heart_rate(6.0) < heart_rate(9.0));
        assert!(heart_rate(40.0) < heart_rate(9.0));
        assert!((heart_rate(9.0) - 170.0).abs() < f32::EPSILON);
    }

    #[test]
    fn body_geometry_is_finite_every_week() {
        // Regression: sin(PI) is slightly negative and powf() turned it into NaN.
        for tenth in 46..=400 {
            let scene = Scene {
                center: Point::new(400.0, 300.0),
                sac: 250.0,
                week: tenth as f32 / 10.0,
                time: tenth as f32,
            };
            let body = Body::new(&scene, 1.0);
            let end = 1.0 + body.tail / body.span;
            for i in 0..=100 {
                let t = end * i as f32 / 100.0;
                let p = body.spine(t);
                assert!(
                    body.width(t).is_finite(),
                    "width at week {} t {t}",
                    scene.week
                );
                assert!(p.x.is_finite() && p.y.is_finite());
            }
            let head = body.head_center();
            assert!(head.x.is_finite() && head.y.is_finite());
        }
    }

    #[test]
    fn hash_stays_in_unit_range() {
        for i in 0..1000 {
            let h = hash(i as f32 * 0.37);
            assert!((0.0..1.0).contains(&h));
        }
    }
}
