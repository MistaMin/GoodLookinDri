// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Marcos Quirino Deida

//! Neon, VFD and LCD parts. Lit elements bloom through feathered additive fills
//! ([`Blend::Add`]): a tight core hugging the element plus a wider, fainter halo. Display
//! windows are recesses under glass, and raised parts take their bevel and soft shadow from
//! [`LightRig::studio`], also when a slider is drawn rotated.

use crate::color::Color;
use crate::light::LightRig;
use crate::meters;
use crate::scene::{Blend, Cmd, Paint, Point, Rect, Scene, Shape, Stops};

const PI: f32 = std::f32::consts::PI;
const HALF_PI: f32 = std::f32::consts::FRAC_PI_2;

fn lim(v: f32, lo: f32, hi: f32) -> f32 { v.max(lo).min(hi) }
fn solid_round(scene: &mut Scene, r: Rect, radius: f32, c: Color) { scene.fill_round(r, radius, c, c, true); }
fn solid_rect(scene: &mut Scene, r: Rect, c: Color) { scene.fill_rect(r, c, c, true); }
fn hline(scene: &mut Scene, y: f32, x0: f32, x1: f32, c: Color, w: f32) { scene.line(Point::new(x0, y), Point::new(x1, y), c, w); }
fn vline(scene: &mut Scene, x: f32, y0: f32, y1: f32, c: Color, w: f32) { scene.line(Point::new(x, y0), Point::new(x, y1), c, w); }
fn text_c(scene: &mut Scene, r: Rect, text: &str, c: Color, size: f32) { scene.text(r.center(), text, c, size, true); }
fn box_at(w: f32, h: f32, c: Point) -> Rect { Rect::new(c.x - w * 0.5, c.y - h * 0.5, w, h) }
fn with_right(r: Rect, right: f32) -> Rect { Rect::new(r.x, r.y, right - r.x, r.h) }
fn with_top(r: Rect, top: f32) -> Rect { Rect::new(r.x, top, r.w, r.bottom() - top) }
fn trim_bottom(r: Rect, d: f32) -> Rect { Rect::new(r.x, r.y, r.w, (r.h - d).max(0.0)) }
fn trim_top(r: Rect, d: f32) -> Rect { Rect::new(r.x, r.y + d, r.w, (r.h - d).max(0.0)) }
fn remove_bottom(r: &mut Rect, h: f32) -> Rect {
    let h = h.min(r.h).max(0.0);
    r.h -= h;
    Rect::new(r.x, r.bottom(), r.w, h)
}
fn remove_left(r: &mut Rect, w: f32) -> Rect {
    let w = w.min(r.w).max(0.0);
    let out = Rect::new(r.x, r.y, w, r.h);
    r.x += w;
    r.w -= w;
    out
}

struct Rot { cx: f32, cy: f32, sin: f32, cos: f32, ang: f32 }
impl Rot {
    fn around(cx: f32, cy: f32, ang: f32) -> Self { Self { cx, cy, sin: ang.sin(), cos: ang.cos(), ang } }
    fn pt(&self, p: Point) -> Point {
        let x = p.x - self.cx;
        let y = p.y - self.cy;
        Point::new(self.cos * x - self.sin * y + self.cx, self.sin * x + self.cos * y + self.cy)
    }
    fn vec(&self, p: Point) -> Point { Point::new(self.cos * p.x - self.sin * p.y, self.sin * p.x + self.cos * p.y) }
    fn shape(&self, s: Shape) -> Shape {
        match s { Shape::Ellipse(r) => Shape::Ellipse(self.rect(r)), Shape::Round(r, k) => Shape::Round(self.rect(r), k) }
    }
    fn paint(&self, p: Paint) -> Paint {
        match p {
            Paint::Solid(c) => Paint::Solid(c),
            Paint::Linear { stops, angle } => Paint::Linear { stops, angle: angle + self.ang },
            Paint::Radial { stops, center, rx, ry, focus } => {
                let (s, c) = (self.sin.abs(), self.cos.abs());
                Paint::Radial { stops, center: self.pt(center), rx: rx * c + ry * s, ry: rx * s + ry * c, focus: self.pt(focus) }
            }
            Paint::Conic { stops, center, start } => Paint::Conic { stops, center: self.pt(center), start: start + self.ang },
        }
    }
    fn rect(&self, r: Rect) -> Rect {
        let p = [self.pt(Point::new(r.x, r.y)), self.pt(Point::new(r.right(), r.y)), self.pt(Point::new(r.x, r.bottom())), self.pt(Point::new(r.right(), r.bottom()))];
        let minx = p.iter().map(|q| q.x).fold(f32::MAX, f32::min);
        let miny = p.iter().map(|q| q.y).fold(f32::MAX, f32::min);
        let maxx = p.iter().map(|q| q.x).fold(f32::MIN, f32::max);
        let maxy = p.iter().map(|q| q.y).fold(f32::MIN, f32::max);
        Rect::new(minx, miny, maxx - minx, maxy - miny)
    }
}
fn blit(scene: &mut Scene, tmp: Scene, rot: &Rot) {
    for c in tmp.cmds {
        scene.cmds.push(match c {
            Cmd::FillEllipse { rect, top, bottom, diagonal } => Cmd::FillEllipse { rect: rot.rect(rect), top, bottom, diagonal },
            Cmd::StrokeEllipse { rect, color, width } => Cmd::StrokeEllipse { rect: rot.rect(rect), color, width },
            Cmd::FillRoundRect { rect, radius, top, bottom, vertical } => Cmd::FillRoundRect { rect: rot.rect(rect), radius, top, bottom, vertical },
            Cmd::StrokeRoundRect { rect, radius, color, width } => Cmd::StrokeRoundRect { rect: rot.rect(rect), radius, color, width },
            Cmd::FillRect { rect, top, bottom, vertical } => Cmd::FillRect { rect: rot.rect(rect), top, bottom, vertical },
            Cmd::Line { a, b, color, width } => Cmd::Line { a: rot.pt(a), b: rot.pt(b), color, width },
            Cmd::FillPoly { pts, top, bottom, diagonal } => Cmd::FillPoly { pts: pts.into_iter().map(|p| rot.pt(p)).collect(), top, bottom, diagonal },
            Cmd::StrokePoly { pts, color, width, closed } => Cmd::StrokePoly { pts: pts.into_iter().map(|p| rot.pt(p)).collect(), color, width, closed },
            Cmd::StrokeArc { center, radius, from, to, color, width } => Cmd::StrokeArc { center: rot.pt(center), radius, from: from + rot.ang, to: to + rot.ang, color, width },
            Cmd::Text { at, text, color, size, center } => Cmd::Text { at: rot.pt(at), text, color, size, center },
            Cmd::ClipRound { rect, radius } => Cmd::ClipRound { rect: rot.rect(rect), radius },
            Cmd::PopClip => Cmd::PopClip,
            Cmd::Fill { shape, paint, feather, blend } => Cmd::Fill { shape: rot.shape(shape), paint: rot.paint(paint), feather, blend },
            Cmd::Shadow { shape, sigma, color } => Cmd::Shadow { shape: rot.shape(shape), sigma, color },
            Cmd::InnerShadow { shape, offset, sigma, color } => Cmd::InnerShadow { shape: rot.shape(shape), offset: rot.vec(offset), sigma, color },
            Cmd::Image { img, dest, opacity } => Cmd::Image { img, dest: rot.rect(dest), opacity },
        });
    }
}

pub fn flicker(t: f32, glow: bool) -> f32 {
    if !glow { return 1.0; }
    0.94 + 0.06 * (t * 31.0).sin() * (t * 7.3).sin()
}
pub fn heat(f: f32, lo: Color, hi: Color) -> Color { lo.mix(hi, lim(f, 0.0, 1.0)) }
pub fn neon(i: i32) -> Color {
    const C: [u32; 6] = [0xff39f2ff, 0xffff4fd8, 0xff9dff3a, 0xffffb02e, 0xffff3b4a, 0xff7a5cff];
    Color(C[(i.rem_euclid(6)) as usize])
}

pub fn chamfer(r: Rect, ch: f32, top_right_only: bool) -> Vec<Point> {
    if top_right_only {
        vec![
            Point::new(r.x, r.y), Point::new(r.right() - ch, r.y), Point::new(r.right(), r.y + ch),
            Point::new(r.right(), r.bottom()), Point::new(r.x + ch, r.bottom()), Point::new(r.x, r.bottom() - ch),
        ]
    } else {
        vec![
            Point::new(r.x + ch, r.y), Point::new(r.right() - ch, r.y), Point::new(r.right(), r.y + ch),
            Point::new(r.right(), r.bottom() - ch), Point::new(r.right() - ch, r.bottom()), Point::new(r.x + ch, r.bottom()),
            Point::new(r.x, r.bottom() - ch), Point::new(r.x, r.y + ch),
        ]
    }
}

/// The shared light seen from a frame that [`blit`] will rotate by `turn` afterwards, so
/// shadows and bevels still agree with the rest of the panel once they are on screen.
#[derive(Clone, Copy)]
struct Lt { rig: LightRig, turn: f32 }
impl Lt {
    fn screen() -> Self { Self { rig: LightRig::studio(), turn: 0.0 } }
    fn turned(turn: f32) -> Self { Self { rig: LightRig::studio(), turn } }
    fn v(&self, p: Point) -> Point {
        let (s, c) = (-self.turn).sin_cos();
        Point::new(c * p.x - s * p.y, s * p.x + c * p.y)
    }
    /// Shadow displacement of something `height` above the surface.
    fn offset(&self, height: f32) -> Point { self.v(self.rig.shadow_offset(height)) }
    fn sigma(&self, height: f32) -> f32 { self.rig.shadow_sigma(height) }
    /// [`Paint::Linear`] angle whose t = 0 is the side nearest the light.
    fn away(&self) -> f32 {
        let d = self.v(self.rig.screen_dir());
        if d.x == 0.0 && d.y == 0.0 { HALF_PI } else { (-d.y).atan2(-d.x) }
    }
}

/// Additive feathered glow around `r`: full strength on its outline, gone `spread` outside it.
fn bloom_rect(scene: &mut Scene, r: Rect, radius: f32, c: Color, spread: f32, alpha: f32) {
    if alpha < 0.004 || r.w <= 0.0 || r.h <= 0.0 { return; }
    scene.glow_round(r.expanded(spread), radius + spread, c.with_alpha(alpha), spread, Blend::Add);
}
/// The two bloom layers every lit block shares: a tight core and a wider faint halo.
fn bloom_block(scene: &mut Scene, r: Rect, radius: f32, c: Color, k: f32) {
    let c = c.with_alpha(1.0);
    let thin = r.w.min(r.h);
    let halo = lim(thin * 0.9, 2.6, 6.0);
    // Hairline segments sit closer together than their halo is wide; keep the sum in check.
    let k = k * lim(thin / 3.0, 0.3, 1.0);
    bloom_rect(scene, r, radius, c, halo, 0.17 * k);
    bloom_rect(scene, r, radius, c, 1.6, 0.15 * k);
}
/// Halo along a path of width `w`: feathered discs stamped closer than their radius, which sum
/// to an even band. A single near axis-aligned segment is one feathered round rect instead.
fn bloom_path(scene: &mut Scene, pts: &[Point], closed: bool, c: Color, w: f32, spread: f32, alpha: f32) {
    if pts.len() < 2 || alpha < 0.004 { return; }
    let c = c.with_alpha(1.0);
    if pts.len() == 2 {
        let (a, b) = (pts[0], pts[1]);
        let bb = Rect::new(a.x.min(b.x), a.y.min(b.y), (a.x - b.x).abs(), (a.y - b.y).abs());
        if bb.w.min(bb.h) <= 4.0 {
            bloom_rect(scene, bb.expanded(w * 0.5), w * 0.5, c, spread, alpha);
            return;
        }
    }
    let mut total = 0.0;
    let n = if closed { pts.len() } else { pts.len() - 1 };
    for i in 0..n { let d = pts[(i + 1) % pts.len()].sub(pts[i]); total += (d.x * d.x + d.y * d.y).sqrt(); }
    if total <= 0.0 { return; }
    let rad = w * 0.5 + spread;
    let step = (spread * 0.5).max(total / 220.0).min(rad);
    let col = c.with_alpha((alpha * step / (2.0 * rad - spread)).min(alpha));
    let mut carry = 0.0;
    for i in 0..n {
        let (a, b) = (pts[i], pts[(i + 1) % pts.len()]);
        let d = b.sub(a);
        let len = (d.x * d.x + d.y * d.y).sqrt();
        if len <= 0.0 { continue; }
        let mut at = carry;
        while at <= len {
            scene.glow_ellipse(box_at(rad * 2.0, rad * 2.0, a.add(d.mul(at / len))), col, spread, Blend::Add);
            at += step;
        }
        carry = at - len;
    }
}
fn round_pts(r: Rect, radius: f32) -> Vec<Point> {
    let k = radius.min(r.w * 0.5).min(r.h * 0.5).max(0.0);
    if k < 0.5 { return rect_pts(r); }
    let mut out = Vec::with_capacity(20);
    let corners = [(r.right() - k, r.y + k, -HALF_PI), (r.right() - k, r.bottom() - k, 0.0), (r.x + k, r.bottom() - k, HALF_PI), (r.x + k, r.y + k, PI)];
    for (cx, cy, a0) in corners {
        for j in 0..=4 {
            let a = a0 + HALF_PI * j as f32 / 4.0;
            out.push(Point::new(cx + a.cos() * k, cy + a.sin() * k));
        }
    }
    out
}

/// Neon tube look for a polyline: additive halo, a soft sleeve, the tube and a hot centre.
/// `c`'s alpha and `strength` both scale the light, so a dim tube does not bloom like a lit one.
pub fn glow_stroke(scene: &mut Scene, pts: &[Point], c: Color, w: f32, strength: f32, glow: bool, closed: bool) {
    if pts.len() < 2 { return; }
    if !glow {
        scene.stroke_poly(pts.to_vec(), c, w, closed);
        return;
    }
    let k = strength * c.alpha_f();
    bloom_path(scene, pts, closed, c, w, 3.5 + w * 1.6, 0.20 * k);
    scene.stroke_poly(pts.to_vec(), c.with_alpha(0.30 * k), w + 2.2, closed);
    scene.stroke_poly(pts.to_vec(), c, w, closed);
    scene.stroke_poly(pts.to_vec(), c.brighter(0.7).with_alpha(0.9 * c.alpha_f().sqrt()), w * 0.35, closed);
}
pub fn glow_line(scene: &mut Scene, a: Point, b: Point, c: Color, w: f32, strength: f32, glow: bool) {
    glow_stroke(scene, &[a, b], c, w, strength, glow, false);
}
pub fn glow_dot(scene: &mut Scene, c: Point, r: f32, col: Color, glow: bool) {
    if !glow { scene.fill_ellipse_solid(box_at(r * 2.0, r * 2.0, c), col); return; }
    scene.glow_ellipse(box_at(r * 9.0, r * 9.0, c), col.with_alpha(0.30), r * 4.5, Blend::Add);
    scene.glow_ellipse(box_at(r * 4.4, r * 4.4, c), col.with_alpha(0.55), r * 1.6, Blend::Add);
    scene.fill_ellipse_solid(box_at(r * 1.5, r * 1.5, c), col.brighter(0.8));
}
pub fn glow_round(scene: &mut Scene, r: Rect, radius: f32, c: Color, w: f32, strength: f32, glow: bool) {
    if !glow { scene.stroke_round(r, radius, c, w); return; }
    let k = strength * c.alpha_f();
    bloom_path(scene, &round_pts(r, radius), true, c, w, 3.5 + w * 1.6, 0.20 * k);
    scene.stroke_round(r, radius, c.with_alpha(0.30 * k), w + 2.2);
    scene.stroke_round(r, radius, c, w);
    scene.stroke_round(r, radius, c.brighter(0.7).with_alpha(0.9 * c.alpha_f().sqrt()), w * 0.35);
}
fn juce_arc(scene: &mut Scene, c: Point, rad: f32, from: f32, to: f32, color: Color, width: f32) {
    let k = -HALF_PI;
    scene.stroke_arc(c, rad, from + k, to + k, color, width);
}
/// Points along a [`juce_arc`], for [`bloom_path`].
fn juce_arc_pts(c: Point, rad: f32, from: f32, to: f32) -> Vec<Point> {
    let n = (((to - from).abs() * rad / 4.0).ceil() as usize).clamp(2, 96);
    (0..=n).map(|i| {
        let a = from + (to - from) * i as f32 / n as f32 - HALF_PI;
        Point::new(c.x + a.cos() * rad, c.y + a.sin() * rad)
    }).collect()
}
fn glow_juce_arc(scene: &mut Scene, c: Point, rad: f32, from: f32, to: f32, color: Color, w: f32, strength: f32, glow: bool) {
    if !glow { juce_arc(scene, c, rad, from, to, color, w); return; }
    let k = strength * color.alpha_f();
    bloom_path(scene, &juce_arc_pts(c, rad, from, to), false, color, w, 3.0 + w * 0.8, 0.20 * k);
    juce_arc(scene, c, rad, from, to, color.with_alpha(0.28 * k), w + 2.4);
    juce_arc(scene, c, rad, from, to, color, w);
    juce_arc(scene, c, rad, from, to, color.brighter(0.7).with_alpha(0.9 * color.alpha_f().sqrt()), w * 0.35);
}
/// Faint halo behind a centred text run (the glyphs themselves cannot be feathered).
fn text_bloom(scene: &mut Scene, at: Point, text: &str, c: Color, size: f32, glow: bool) {
    if !glow || text.is_empty() { return; }
    let r = box_at(size * 0.5 * text.chars().count() as f32, size * 0.45, at);
    bloom_rect(scene, r, size * 0.2, c.with_alpha(1.0), size * 0.55, 0.10 * c.alpha_f());
}

pub fn scanlines(scene: &mut Scene, r: Rect, t: f32, alpha: f32, glow: bool) {
    if !glow { return; }
    let ink = Color::rgba(0, 0, 0, 255).with_alpha(alpha);
    let mut y = r.y;
    while y < r.bottom() {
        hline(scene, y.trunc(), r.x, r.right(), ink, 1.0);
        y += 2.5;
    }
    let sweep = r.y + (t * 0.35).rem_euclid(1.0) * r.h;
    let (y0, y1) = ((sweep - 12.0).max(r.y), (sweep + 12.0).min(r.bottom()));
    if y1 - y0 > 1.0 {
        let (f0, f1) = ((y0 - (sweep - 12.0)) / 24.0, (y1 - (sweep - 12.0)) / 24.0);
        let peak = |f: f32| Color(0xffffffff).with_alpha(0.035 * (1.0 - (f * 2.0 - 1.0).abs()));
        let mut st = vec![(0.0, peak(f0))];
        if f0 < 0.5 && f1 > 0.5 { st.push(((0.5 - f0) / (f1 - f0), peak(0.5))); }
        st.push((1.0, peak(f1)));
        scene.fill_shape(Shape::Round(Rect::new(r.x, y0, r.w, y1 - y0), 0.0), Paint::Linear { stops: Stops::new(&st), angle: HALF_PI }, 0.0, Blend::Screen);
    }
}

/// A cut-out in the panel with a dark floor: the lip nearest the light is in shade, the far lip
/// catches it. Follow the contents with [`recess_shade`] (or [`window_glass`] for a display).
fn recess(scene: &mut Scene, r: Rect, radius: f32, top: Color, bottom: Color, lt: Lt) {
    let lip = Stops::new(&[(0.0, Color(0xc8000000)), (0.5, Color(0x50000000)), (1.0, Color(0x30ffffff))]);
    scene.fill_round_linear(r.expanded(1.1), radius + 1.1, lip, lt.away());
    scene.fill_round(r, radius, top, bottom, true);
}
fn recess_shade(scene: &mut Scene, r: Rect, radius: f32, depth: f32, lt: Lt) {
    scene.inner_shadow(Shape::Round(r, radius), lt.offset(depth), (depth * 0.9).max(1.0), Color(0xd8000000));
}
fn window_ground(scene: &mut Scene, r: Rect, radius: f32, top: Color, bottom: Color) {
    recess(scene, r, radius, top, bottom, Lt::screen());
}
/// What sits in front of a display: the panel's shadow falling onto it, a faint reflection of
/// the room in the glass, and a dark edge where the glass meets the cut-out.
fn window_glass(scene: &mut Scene, r: Rect, radius: f32) {
    let lt = Lt::screen();
    recess_shade(scene, r, radius, 3.2, lt);
    let g = Rect::new(r.x + 1.0, r.y + 1.0, (r.w - 2.0).max(0.0), (r.h * 0.6).max(0.0));
    let sheen = Stops::new(&[(0.0, Color(0x1effffff)), (0.4, Color(0x09ffffff)), (1.0, Color(0x00ffffff))]);
    scene.fill_shape(Shape::Round(g, (radius - 1.0).max(0.0)), Paint::Linear { stops: sheen, angle: HALF_PI }, 0.0, Blend::Screen);
    scene.stroke_round(r, radius, Color(0xe6000000), 1.0);
}

struct Seg { r: Rect, c: Color, on: bool }
/// Display segments: unlit ones stay as 10% ghosts, lit ones get bloom underneath (in its own
/// pass, so no segment's halo washes over a neighbour) and a slight top-to-bottom gradient.
fn segments(scene: &mut Scene, segs: &[Seg], radius: f32, t: f32, glow: bool) {
    for s in segs.iter().filter(|s| !s.on) {
        solid_round(scene, s.r, radius, s.c.with_alpha(0.10));
    }
    if glow {
        for (i, s) in segs.iter().enumerate().filter(|(_, s)| s.on) {
            bloom_block(scene, s.r, radius, s.c, flicker(t + i as f32 * 0.37, true));
        }
    }
    for (i, s) in segs.iter().enumerate().filter(|(_, s)| s.on) {
        let a = flicker(t + i as f32 * 1.1, glow);
        let stops = Stops::new(&[(0.0, s.c.brighter(0.5).with_alpha(a)), (0.22, s.c.brighter(0.14).with_alpha(a)), (1.0, s.c.darker(0.16).with_alpha(a))]);
        scene.fill_round_linear(s.r, radius, stops, HALF_PI);
    }
}

pub fn draw_bar_wedge(scene: &mut Scene, r: Rect, level: f32, lit: Color, t: f32, glow: bool) {
    window_ground(scene, r, 5.0, Color(0xff0d0b06), Color(0xff040302));
    let mut b = r.reduced(8.0, 6.0);
    b.h = (b.h - 10.0).max(0.0);
    let n = 26;
    let mut segs = Vec::with_capacity(n);
    for i in 0..n {
        let f = i as f32 / (n - 1) as f32;
        let x = b.x + f * (b.w - 4.0);
        let h = b.h * (0.14 + 0.86 * f.powf(1.35));
        let w = b.w / n as f32 * 0.68;
        let on = f <= level + 0.5 / n as f32;
        let c = if f > 0.84 { Color(0xffff3b2a) } else if f > 0.68 { lit.mix(Color(0xffff6a1a), 0.55) } else { lit };
        segs.push(Seg { r: Rect::new(x, b.bottom() - h, w, h), c, on });
    }
    segments(scene, &segs, 1.2, t, glow);
    let mut q = r.reduced(8.0, 2.0);
    let labels = remove_bottom(&mut q, 10.0);
    for i in 0..=5 {
        text_c(scene, Rect::new(b.x + b.w * i as f32 / 5.0 - 10.0, labels.y, 20.0, labels.h), &(i * 12).to_string(), lit.with_alpha(0.7), 7.5);
    }
    scanlines(scene, r, t, 0.14, glow);
    window_glass(scene, r, 5.0);
    scene.stroke_round(r.reduced(0.5, 0.5), 4.5, lit.with_alpha(0.20), 1.0);
}

pub fn draw_mountains(scene: &mut Scene, r: Rect, left: f32, right: f32, lit: Color, t: f32, glow: bool) {
    window_ground(scene, r, 5.0, Color(0xff030b07), Color(0xff010402));
    let b = r.reduced(8.0, 7.0);
    let cx = b.center().x;
    let half = b.w * 0.5 - 12.0;
    let n = 22;
    let mut lines = Vec::with_capacity(n * 2);
    for side in 0..2 {
        let level = if side == 0 { left } else { right };
        let dirn = if side == 0 { -1.0 } else { 1.0 };
        for i in 0..n {
            let f = i as f32 / (n - 1) as f32;
            let x = cx + dirn * (10.0 + f * half);
            let h = b.h * (0.92 - 0.80 * f);
            let on = (1.0 - f) <= level + 0.5 / n as f32;
            lines.push((Point::new(x, b.bottom()), Point::new(x - dirn * (b.w * 0.02), b.bottom() - h), on, f));
        }
    }
    for &(base, top, _, _) in lines.iter().filter(|l| !l.2) { scene.line(base, top, lit.with_alpha(0.11), 1.2); }
    if glow {
        for &(base, top, _, f) in lines.iter().filter(|l| l.2) {
            let k = flicker(t + f, true);
            bloom_path(scene, &[base, top], false, lit, 1.2, 3.2, 0.09 * k);
            bloom_path(scene, &[base, top], false, lit, 1.2, 1.2, 0.08 * k);
        }
    }
    for &(base, top, _, _) in lines.iter().filter(|l| l.2) {
        scene.line(base, top, lit.brighter(0.12), 1.2);
        scene.line(top, top.add(base.sub(top).mul(0.3)), lit.brighter(0.55).with_alpha(0.8), 0.7);
    }
    for i in 0..6 {
        let y = b.bottom() - b.h * i as f32 / 5.0;
        text_c(scene, box_at(20.0, 9.0, Point::new(cx, y - 3.0)), &(i * 10).to_string(), lit.with_alpha(0.8), 7.5);
        hline(scene, y.trunc(), cx - 8.0, cx - 5.0, lit.with_alpha(0.8), 1.0);
        hline(scene, y.trunc(), cx + 5.0, cx + 8.0, lit.with_alpha(0.8), 1.0);
    }
    hline(scene, (b.bottom() + 1.0).trunc(), b.x, b.right(), lit.with_alpha(0.6), 1.0);
    scanlines(scene, r, t, 0.2, glow);
    window_glass(scene, r, 5.0);
    scene.stroke_round(r.reduced(0.5, 0.5), 4.5, lit.with_alpha(0.18), 1.0);
}

pub fn draw_lcd_panel(scene: &mut Scene, r: Rect, a: &str, a_label: &str, b: &str, b_label: &str, lit: Color, t: f32, glow: bool) {
    window_ground(scene, r, 6.0, Color(0xff140c05), Color(0xff060301));
    let inn = r.reduced(7.0, 7.0);
    let half = inn.w * 0.5 - 3.0;
    for i in 0..2 {
        let mut cell = Rect::new(inn.x + i as f32 * (half + 6.0), inn.y, half, inn.h);
        let lab = remove_bottom(&mut cell, 12.0);
        let k = flicker(t + i as f32, glow);
        scene.fill_round(cell, 4.0, Color(0xff0a0603), Color(0xff040201), true);
        if glow { bloom_rect(scene, cell.reduced(cell.w * 0.3, cell.h * 0.34), 3.0, lit.with_alpha(1.0), cell.h * 0.42, 0.10 * k); }
        scene.stroke_round(cell, 4.0, lit.with_alpha(0.26), 1.0);
        meters::draw_seven_segment(scene, cell.reduced(5.0, 5.0), if i == 0 { a } else { b }, lit.with_alpha(k), true);
        text_c(scene, lab, if i == 0 { a_label } else { b_label }, lit.with_alpha(0.85), 8.5);
    }
    scanlines(scene, r, t, 0.12, glow);
    window_glass(scene, r, 6.0);
    scene.stroke_round(r.reduced(0.5, 0.5), 5.5, lit.with_alpha(0.32), 1.0);
}

pub fn draw_arc_gauge(scene: &mut Scene, r: Rect, level: f32, lit: Color, t: f32, glow: bool) {
    window_ground(scene, r, 6.0, Color(0xff100d09), Color(0xff040303));
    let rad = (r.w * 0.46).min(r.h * 0.92);
    let c = Point::new(r.center().x, r.bottom() - 6.0);
    let red = Color(0xffff3b2a);
    let (w, n) = (rad * 0.20, 30);
    let ang = |f: f32| (-0.5 + f) * PI;
    let gap = 0.14 / n as f32;
    let lit_to = lim(level, 0.0, 1.0);
    // Annular wedges as polygons: square ends whatever the host does with arc caps.
    let wedge = |f0: f32, f1: f32, r0: f32, r1: f32| -> Vec<Point> {
        let at = |f: f32, rr: f32| { let a = ang(f); Point::new(c.x + a.sin() * rr, c.y - a.cos() * rr) };
        let fm = (f0 + f1) * 0.5;
        vec![at(f0, r1), at(fm, r1), at(f1, r1), at(f1, r0), at(fm, r0), at(f0, r0)]
    };
    let (r0, r1) = (rad - w * 0.5, rad + w * 0.5);
    let cell = |i: usize| (i as f32 / n as f32 + gap, (i + 1) as f32 / n as f32 - gap, if i as f32 / n as f32 >= 0.86 { red } else { lit });
    let on = |i: usize| (i as f32 + 0.5) / n as f32 <= lit_to;
    for i in (0..n).filter(|i| !on(*i)) {
        let (f0, f1, col) = cell(i);
        let g = col.with_alpha(0.10);
        scene.fill_poly(wedge(f0, f1, r0, r1), g, g);
    }
    if glow && lit_to > 0.0 {
        let k = flicker(t, true);
        for (f0, f1, col) in [(0.0, lit_to.min(0.86), lit), (0.86, lit_to, red)] {
            if f1 > f0 {
                bloom_path(scene, &juce_arc_pts(c, rad, ang(f0), ang(f1)), false, col, w, 6.0, 0.22 * k);
                bloom_path(scene, &juce_arc_pts(c, rad, ang(f0), ang(f1)), false, col, w, 1.8, 0.26 * k);
            }
        }
    }
    for i in (0..n).filter(|i| on(*i)) {
        let (f0, f1, col) = cell(i);
        scene.fill_poly(wedge(f0, f1, r0, r1), col, col.darker(0.14));
        let hi = col.brighter(0.45).with_alpha(0.8);
        scene.fill_poly(wedge(f0, f1, r1 - w * 0.36, r1 - w * 0.08), hi, hi);
    }
    juce_arc(scene, c, rad * 0.78, ang(0.86), ang(1.0), red.with_alpha(0.85), rad * 0.06);
    for i in 0..=9 {
        let f = i as f32 / 9.0;
        let a = (-0.5 + f) * PI;
        let tip0 = Point::new(c.x + a.sin() * rad * 0.82, c.y - a.cos() * rad * 0.82);
        let tip1 = Point::new(c.x + a.sin() * rad * 0.74, c.y - a.cos() * rad * 0.74);
        let col = if f > 0.86 { Color(0xffff6a4a) } else { lit.with_alpha(0.9) };
        scene.line(tip0, tip1, col, 1.3);
        text_c(scene, box_at(14.0, 10.0, Point::new(c.x + a.sin() * rad * 0.64, c.y - a.cos() * rad * 0.64)), &i.to_string(), col, 9.0);
    }
    let digits = box_at(rad * 0.55, rad * 0.30, Point::new(c.x, c.y - rad * 0.22));
    if glow { bloom_rect(scene, digits.reduced(digits.w * 0.25, digits.h * 0.3), 2.0, lit.with_alpha(1.0), digits.h * 0.6, 0.10); }
    meters::draw_seven_segment(scene, digits, &((level * 99.0) as i32).to_string(), lit, false);
    scanlines(scene, r, t, 0.12, glow);
    window_glass(scene, r, 6.0);
    scene.stroke_round(r.reduced(0.5, 0.5), 5.5, lit.with_alpha(0.22), 1.0);
}

pub fn draw_colour_columns(scene: &mut Scene, r: Rect, l: f32, rr: f32, peak_l: f32, peak_r: f32, t: f32, glow: bool) {
    window_ground(scene, r, 5.0, Color(0xff070a08), Color(0xff020403));
    let mut b = r.reduced(10.0, 7.0);
    let _left = remove_left(&mut b, 12.0);
    let n = 22;
    let seg = b.w / n as f32;
    let row_h = (b.h - 6.0) * 0.5;
    let mut segs = Vec::with_capacity(n * 2);
    for c in 0..2 {
        let lv = if c == 0 { l } else { rr };
        let pk = if c == 0 { peak_l } else { peak_r };
        let row = Rect::new(b.x, b.y + c as f32 * (row_h + 6.0), b.w, row_h);
        text_c(scene, box_at(10.0, row_h, Point::new(r.x + 10.0, row.center().y)), if c == 0 { "L" } else { "R" }, Color(0xff8b9498), 8.0);
        for i in 0..n {
            let f = i as f32 / (n - 1) as f32;
            let cc = if f > 0.85 { Color(0xffff3b2a) } else if f > 0.62 { Color(0xffffcf2a) } else { Color(0xff35e04a) };
            let on = f <= lv + 0.5 / n as f32 || (pk > 0.0 && (f - pk).abs() < 0.5 / n as f32);
            segs.push(Seg { r: Rect::new(row.x + i as f32 * seg + 1.0, row.y, (seg - 2.0).max(0.6), row.h), c: cc, on });
        }
    }
    segments(scene, &segs, 0.4, t, glow);
    scanlines(scene, r, t, 0.10, glow);
    window_glass(scene, r, 5.0);
    scene.stroke_round(r.reduced(0.5, 0.5), 4.5, Color(0x22ffffff), 1.0);
}

pub fn draw_neon_bars(scene: &mut Scene, r: Rect, level: f32, peak: f32, lo: Color, hi: Color, t: f32, glow: bool) {
    let frame = chamfer(r.reduced(1.5, 1.5), 10.0, false);
    scene.fill_poly(frame.clone(), Color(0xff051217), Color(0xff010507));
    scene.clip_round(r, 0.0);
    let grid = lo.with_alpha(0.06);
    let mut x = r.x;
    while x < r.right() { vline(scene, x.trunc(), r.y, r.bottom(), grid, 1.0); x += 12.0; }
    let mut y = r.y;
    while y < r.bottom() { hline(scene, y.trunc(), r.x, r.right(), grid, 1.0); y += 12.0; }
    let b = r.reduced(14.0, 12.0);
    let n = 24;
    let mut segs = Vec::with_capacity(n * 2);
    for i in 0..n {
        let f = i as f32 / (n - 1) as f32;
        let x = b.x + f * b.w;
        let w = b.w / n as f32 * 0.55;
        let shape = 0.45 + 0.55 * (f * PI * 0.5 + 0.4).sin();
        let h = b.h * shape;
        let fill = lim((level - f * 0.6) / 0.4, 0.0, 1.0);
        let c = heat(f, lo, hi);
        let bar = Rect::new(x, b.bottom() - h, w, h);
        segs.push(Seg { r: bar, c, on: false });
        if fill > 0.0 { segs.push(Seg { r: with_top(bar, bar.bottom() - h * fill), c, on: true }); }
    }
    segments(scene, &segs, 0.0, t, glow);
    for s in segs.iter().filter(|s| s.on) {
        solid_rect(scene, Rect::new(s.r.x, s.r.y, s.r.w, 1.6), s.c.brighter(0.9));
    }
    let px = b.x + lim(peak, 0.0, 1.0) * b.w;
    glow_line(scene, Point::new(px, b.y), Point::new(px, b.bottom()), hi, 1.2, 0.8, glow);
    scanlines(scene, r, t, 0.16, glow);
    chamfer_glass(scene, r.reduced(1.5, 1.5), 10.0, false);
    scene.pop_clip();
    glow_stroke(scene, &frame, lo, 1.4, 1.0, glow, true);
    glow_line(scene, Point::new(r.x + 14.0, r.y + 1.0), Point::new(r.x + 44.0, r.y + 1.0), hi, 2.2, 1.0, glow);
}
/// [`window_glass`] for a [`chamfer`] frame, built from pieces that stay inside the cut corners.
fn chamfer_glass(scene: &mut Scene, r: Rect, ch: f32, top_right_only: bool) {
    let near = if top_right_only { 0.0 } else { ch };
    let o = Lt::screen().offset(3.2);
    let shade = |a: f32| Stops::new(&[(0.0, Color(0xff000000).with_alpha(a)), (0.45, Color(0xff000000).with_alpha(a * 0.35)), (1.0, Color(0x00000000))]);
    let top = Rect::new(r.x + near, r.y, (r.w - near - ch).max(0.0), 3.0 + o.y * 3.0);
    scene.fill_shape(Shape::Round(top, 0.0), Paint::Linear { stops: shade(0.8), angle: HALF_PI }, 0.0, Blend::Over);
    let left = Rect::new(r.x, r.y + near, 3.0 + o.x * 3.0, (r.h - near - ch).max(0.0));
    scene.fill_shape(Shape::Round(left, 0.0), Paint::Linear { stops: shade(0.6), angle: 0.0 }, 0.0, Blend::Over);
    let yb = r.y + r.h * 0.55;
    let sheen = vec![Point::new(r.x + near, r.y), Point::new(r.right() - ch, r.y), Point::new(r.right(), r.y + ch), Point::new(r.right(), yb), Point::new(r.x, yb), Point::new(r.x, r.y + near)];
    scene.cmds.push(Cmd::FillPoly { pts: sheen, top: Color(0x16ffffff), bottom: Color(0x00ffffff), diagonal: false });
}

pub fn draw_hex_ring(scene: &mut Scene, r: Rect, level: f32, lo: Color, hi: Color, t: f32, glow: bool) {
    let c = r.center();
    let radius = r.w.min(r.h) * 0.48;
    let mut hx = Vec::new();
    for i in 0..6 {
        let a = i as f32 * PI / 3.0 + PI / 6.0;
        hx.push(Point::new(c.x + a.cos() * radius * 1.12, c.y + a.sin() * radius * 1.12));
    }
    window_ground(scene, r, 6.0, Color(0xff040a10), Color(0xff010305));
    scene.clip_round(r, 6.0);
    scene.fill_poly(hx.clone(), lo.darker(0.9), Color(0xff020406));
    glow_stroke(scene, &hx, lo.with_alpha(0.7), 1.2, 0.8, glow, true);
    let a0 = -PI * 1.25;
    let a1 = PI * 0.25;
    let ring = radius * 0.80;
    let lit = heat(level, lo, hi);
    juce_arc(scene, c, ring, a0 + HALF_PI, a1 + HALF_PI, lo.with_alpha(0.12), radius * 0.10);
    if level > 0.0 {
        glow_juce_arc(scene, c, ring, a0 + HALF_PI, a0 + (a1 - a0) * lim(level, 0.0, 1.0) + HALF_PI, lit, radius * 0.10, 1.0, glow);
    }
    for i in 0..24 {
        let a = a0 + (a1 - a0) * i as f32 / 23.0 + HALF_PI;
        let on = i as f32 / 23.0 <= level;
        let col = if on { Color(0xffffffff).with_alpha(0.7) } else { lo.with_alpha(0.3) };
        scene.line(
            Point::new(c.x + a.cos() * ring * 0.80, c.y + a.sin() * ring * 0.80),
            Point::new(c.x + a.cos() * ring * 0.70, c.y + a.sin() * ring * 0.70),
            col, 1.0,
        );
    }
    let digits = box_at(radius * 0.9, radius * 0.46, Point::new(c.x, c.y + radius * 0.05));
    if glow { bloom_rect(scene, digits.reduced(digits.w * 0.25, digits.h * 0.3), 2.0, lit, digits.h * 0.55, 0.10); }
    meters::draw_seven_segment(scene, digits, &((level * 100.0) as i32).to_string(), lit.with_alpha(flicker(t, glow)), false);
    scanlines(scene, r, t, 0.12, glow);
    scene.pop_clip();
    window_glass(scene, r, 6.0);
}

pub fn draw_scope(scene: &mut Scene, r: Rect, hist: &[f32], clip: bool, lit: Color, t: f32, glow: bool) {
    let frame = chamfer(r.reduced(1.5, 1.5), 8.0, true);
    scene.fill_poly(frame.clone(), Color(0xff031013), Color(0xff010608));
    scene.clip_round(r, 0.0);
    let grid = lit.with_alpha(0.08);
    let mut x = r.x;
    while x < r.right() { vline(scene, x.trunc(), r.y, r.bottom(), grid, 1.0); x += 14.0; }
    let mut y = r.center().y.trunc();
    while y > r.y { y -= 14.0; }
    while y < r.bottom() { hline(scene, y, r.x, r.right(), if (y - r.center().y.trunc()).abs() < 1.0 { lit.with_alpha(0.16) } else { grid }, 1.0); y += 14.0; }
    if hist.len() >= 2 {
        let cy = r.center().y;
        let amp = r.h * 0.44;
        let mut top = Vec::new();
        let mut bot = Vec::new();
        for (i, &h) in hist.iter().enumerate() {
            let x = r.x + r.w * i as f32 / (hist.len() - 1) as f32;
            let v = h * amp;
            let jitter = if h > 0.02 { 0.5 * (i as f32 * 1.7 + t * 30.0).sin() } else { 0.0 };
            top.push(Point::new(x, cy - v - if i == 0 { 0.0 } else { jitter }));
            bot.push(Point::new(x, cy + v + if i == 0 { 0.0 } else { jitter }));
        }
        let mut fill = top.clone();
        fill.push(Point::new(r.right(), cy));
        for i in (0..hist.len()).rev() {
            fill.push(Point::new(r.x + r.w * i as f32 / (hist.len() - 1) as f32, cy + hist[i] * amp));
        }
        scene.fill_poly(fill, lit.with_alpha(0.22), lit.with_alpha(0.04));
        let ghost: Vec<Point> = top.iter().map(|p| Point::new(p.x + 1.5, p.y)).collect();
        scene.stroke_poly(ghost, Color(0x30ff2a6a), 1.0, false);
        glow_stroke(scene, &bot, lit.with_alpha(0.6), 1.0, 0.6, glow, false);
        glow_stroke(scene, &top, lit, 1.2, 0.9, glow, false);
        let ex = r.right() - 3.0;
        glow_dot(scene, Point::new(ex, cy - hist[hist.len() - 1] * amp), 2.0, lit, glow);
    }
    if clip && (t.rem_euclid(0.8)) < 0.5 {
        let bar = r.translated(0.0, 3.0);
        let band = Rect::new(bar.x, bar.y, bar.w, 14.0);
        if glow { bloom_rect(scene, band, 0.0, Color(0xffff2a3a), 6.0, 0.25); }
        solid_rect(scene, band, Color(0xaaff2a3a));
        text_c(scene, band, "! ATTENTION  CLIP", Color(0xffffffff), 9.0);
    }
    scanlines(scene, r, t, 0.2, glow);
    chamfer_glass(scene, r.reduced(1.5, 1.5), 8.0, true);
    scene.pop_clip();
    glow_stroke(scene, &frame, lit.with_alpha(0.8), 1.2, 0.7, glow, true);
}

pub fn draw_chassis(scene: &mut Scene, r: Rect, level: f32, body: Color, lit: Color, t: f32, glow: bool) {
    let lt = Lt::screen();
    let o = lt.offset(5.0);
    scene.shadow_round(r.translated(o.x, o.y), 9.0, lt.sigma(9.0), Color(0x99000000));
    scene.shadow_round(r.translated(o.x * 0.25, o.y * 0.25), 9.0, 1.0, Color(0x66000000));
    // Bevel: the rim runs from the lit corner to the shaded one, the face inside it is flatter.
    let rim = Stops::new(&[(0.0, body.brighter(0.75)), (0.35, body.brighter(0.2)), (0.7, body.darker(0.3)), (1.0, body.darker(0.62))]);
    scene.fill_round_linear(r, 9.0, rim, lt.away());
    let face = r.reduced(1.8, 1.8);
    let plate = Stops::new(&[(0.0, body.brighter(0.26)), (0.45, body.brighter(0.04)), (1.0, body.darker(0.32))]);
    scene.fill_round_linear(face, 7.4, plate, lt.away() * 0.25 + HALF_PI * 0.75);
    let sheen = Stops::new(&[(0.0, Color(0x30ffffff)), (1.0, Color(0x00ffffff))]);
    scene.fill_shape(Shape::Round(Rect::new(face.x, face.y, face.w, face.h * 0.4), 7.4), Paint::Linear { stops: sheen, angle: HALF_PI }, 0.0, Blend::Screen);
    scene.stroke_round(r, 9.0, body.darker(0.82).with_alpha(0.9), 0.8);
    let toward = lt.rig.screen_dir();
    for p in [Point::new(r.x + 7.0, r.y + 7.0), Point::new(r.right() - 7.0, r.y + 7.0), Point::new(r.x + 7.0, r.bottom() - 7.0), Point::new(r.right() - 7.0, r.bottom() - 7.0)] {
        let head = box_at(5.0, 5.0, p);
        let so = lt.offset(1.2);
        scene.shadow_ellipse(head.translated(so.x, so.y), 0.8, Color(0x90000000));
        scene.fill_ellipse_radial(head, Stops::new(&[(0.0, body.brighter(0.55)), (0.6, body.darker(0.35)), (1.0, body.darker(0.72))]), p.add(toward.mul(1.2)));
        scene.line(Point::new(p.x - 1.5, p.y + 1.0), Point::new(p.x + 1.5, p.y - 1.0), body.darker(0.85), 0.7);
    }
    let mut s = r.reduced(13.0, 11.0);
    let win = s;
    window_ground(scene, win, 5.0, Color(0xff050a0c), Color(0xff020405));
    let bars = trim_top(trim_bottom(s.reduced(8.0, 0.0), 14.0), 8.0);
    let n = 28;
    let mut segs = Vec::with_capacity(n);
    for i in 0..n {
        let f = i as f32 / (n - 1) as f32;
        let x = bars.x + f * bars.w;
        let on = f <= level + 0.5 / n as f32;
        let c = if f > 0.85 { Color(0xffff4a5a) } else { lit };
        let inset = if i % 4 == 0 { 0.0 } else { 5.0 };
        segs.push(Seg { r: Rect::new(x, bars.y + inset, bars.w / n as f32 * 0.62, bars.h - inset), c, on });
    }
    segments(scene, &segs, 0.3, t, glow);
    let foot = remove_bottom(&mut s, 13.0);
    let label = Rect::new(foot.x + 8.0, foot.y, foot.w - 8.0, foot.h);
    text_bloom(scene, label.center(), "OUTPUT  //  LEVEL", lit.with_alpha(0.5), 8.0, glow);
    text_c(scene, label, "OUTPUT  //  LEVEL", lit.with_alpha(0.8), 8.0);
    scanlines(scene, win, t, 0.14, glow);
    window_glass(scene, win, 5.0);
    scene.stroke_round(win.reduced(0.5, 0.5), 4.5, lit.with_alpha(0.22), 1.0);
}

pub fn draw_flat_bars(scene: &mut Scene, r: Rect, l: f32, rr: f32, peak_l: f32, peak_r: f32, lo: Color) {
    solid_round(scene, r, 4.0, Color(0xff0d0f10));
    let mut b = r.reduced(12.0, 9.0);
    b.h = (b.h - 10.0).max(0.0);
    let h = (b.h - 8.0) * 0.5;
    let n = 30;
    let seg = b.w / n as f32;
    for c in 0..2 {
        let lv = if c == 0 { l } else { rr };
        let pk = if c == 0 { peak_l } else { peak_r };
        let row = Rect::new(b.x, b.y + c as f32 * (h + 8.0), b.w, h);
        for i in 0..n {
            let f = i as f32 / (n - 1) as f32;
            let cc = if f > 0.88 { Color(0xffe5382a) } else if f > 0.70 { Color(0xffe8b923) } else { lo };
            let on = f <= lv + 0.5 / n as f32 || (f - pk).abs() < 0.5 / n as f32;
            solid_rect(scene, Rect::new(row.x + i as f32 * seg + 0.6, row.y, seg - 1.2, row.h), if on { cc } else { cc.with_alpha(0.13) });
        }
    }
    for i in 0..=6 {
        text_c(scene, box_at(24.0, 9.0, Point::new(b.x + b.w * i as f32 / 6.0, r.bottom() - 9.0)), &(-48 + i * 8).to_string(), Color(0xff8b9498), 7.5);
    }
    scene.stroke_round(r, 4.0, Color(0xff2a3033), 1.0);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Icon { Battery, Warning, ArrowLeft, ArrowRight, Beam, Temp }

/// Icon stroke on a lamp lens: a ghost when off, a lit filament with a soft sleeve when on.
fn lamp_poly(scene: &mut Scene, pts: Vec<Point>, c: Color, w: f32, closed: bool, on: bool) {
    if !on { scene.stroke_poly(pts, c.with_alpha(0.12), w, closed); return; }
    scene.stroke_poly(pts.clone(), c.with_alpha(0.26), w + 2.4, closed);
    scene.stroke_poly(pts.clone(), c, w, closed);
    scene.stroke_poly(pts, c.brighter(0.65).with_alpha(0.85), w * 0.4, closed);
}

pub fn draw_icon_lamp(scene: &mut Scene, r: Rect, icon: Icon, c: Color, on: bool) {
    let hue = c.with_alpha(1.0);
    if on { bloom_rect(scene, r, 3.0, hue, 7.0, 0.13); }
    window_ground(scene, r, 3.0, if on { hue.darker(0.90) } else { Color(0xff070808) }, if on { hue.darker(0.96) } else { Color(0xff030404) });
    if on {
        // The lamp behind the lens: brightest in the middle, falling off toward the bezel.
        scene.glow_ellipse(r.expanded(r.w * 0.1), hue.with_alpha(0.20), r.w * 0.6, Blend::Add);
        scene.clip_round(r, 3.0);
        scene.glow_ellipse(r.reduced(r.w * 0.12, r.w * 0.12), hue.with_alpha(0.16), r.w * 0.38, Blend::Add);
        scene.pop_clip();
    }
    let col = if on { c } else { c.with_alpha(0.12) };
    let hot = if on { c.brighter(0.35) } else { col };
    let a = r.reduced(r.w * 0.24, r.w * 0.24);
    match icon {
        Icon::Battery => {
            lamp_poly(scene, rect_pts(trim_top(a, a.h * 0.2)), c, 1.4, true, on);
            let nub = box_at(a.w * 0.3, a.h * 0.16, Point::new(a.center().x, a.y + a.h * 0.08));
            lamp_poly(scene, rect_pts(nub), c, 1.4, true, on);
            text_c(scene, trim_top(a, a.h * 0.2), "+", hot, a.h * 0.55);
        }
        Icon::Warning => {
            lamp_poly(scene, vec![Point::new(a.center().x, a.y), Point::new(a.right(), a.bottom()), Point::new(a.x, a.bottom())], c, 1.5, true, on);
            text_c(scene, trim_top(a, a.h * 0.25), "!", hot, a.h * 0.62);
        }
        Icon::ArrowLeft | Icon::ArrowRight => {
            let (tip, back) = if icon == Icon::ArrowLeft { (a.x, a.right()) } else { (a.right(), a.x) };
            let tri = vec![Point::new(tip, a.center().y), Point::new(back, a.y), Point::new(back, a.bottom())];
            if on { scene.stroke_poly(tri.clone(), c.with_alpha(0.26), 2.6, true); }
            scene.fill_poly(tri, if on { c.brighter(0.4) } else { col }, if on { c.darker(0.12) } else { col });
        }
        Icon::Beam => {
            for i in 0..4 {
                let y = a.y + a.h * (0.15 + 0.23 * i as f32);
                lamp_poly(scene, vec![Point::new(a.x + a.w * 0.35, y), Point::new(a.right(), y)], c, 1.6, false, on);
            }
            let lens = Rect::new(a.x, a.center().y - a.h * 0.3, a.w * 0.4, a.h * 0.6);
            if on { scene.fill_ellipse_solid(lens.expanded(1.2), c.with_alpha(0.26)); }
            scene.fill_ellipse(lens, hot, col);
        }
        Icon::Temp => {
            lamp_poly(scene, vec![Point::new(a.center().x, a.y), Point::new(a.center().x, a.bottom() - a.w * 0.3)], c, 2.4, false, on);
            let bulb = Rect::new(a.center().x - a.w * 0.28, a.bottom() - a.w * 0.56, a.w * 0.56, a.w * 0.56);
            if on { scene.fill_ellipse_solid(bulb.expanded(1.2), c.with_alpha(0.26)); }
            scene.fill_ellipse(bulb, hot, col);
        }
    }
    window_glass(scene, r, 3.0);
    scene.stroke_round(r.reduced(0.5, 0.5), 2.5, if on { c.with_alpha(0.40) } else { Color(0xff24292b) }, 1.0);
}
fn rect_pts(r: Rect) -> Vec<Point> {
    vec![Point::new(r.x, r.y), Point::new(r.right(), r.y), Point::new(r.right(), r.bottom()), Point::new(r.x, r.bottom())]
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SliderStyle { Neon, Bar, Chamfer, Wedge, Flat }

/// Raised slider cap: soft shadow thrown away from the light, a bevelled rim and a flatter face.
fn cap(scene: &mut Scene, r: Rect, radius: f32, face: Color, lt: Lt) {
    let o = lt.offset(3.5);
    scene.shadow_round(r.translated(o.x, o.y), radius, lt.sigma(8.0), Color(0x9c000000));
    let rim = Stops::new(&[(0.0, face.brighter(0.9)), (0.5, face.darker(0.14)), (1.0, face.darker(0.55))]);
    scene.fill_round_linear(r, radius, rim, lt.away());
    let top = Stops::new(&[(0.0, face.brighter(0.25)), (1.0, face.darker(0.24))]);
    scene.fill_round_linear(r.reduced(1.0, 1.0), (radius - 1.0).max(0.0), top, lt.away());
    scene.stroke_round(r, radius, Color(0xb0000000), 0.6);
}
/// The lit part of a track: bloom below, then a bar that is brightest along its upper third.
fn lit_bar(scene: &mut Scene, r: Rect, radius: f32, c: Color, k: f32, glow: bool) {
    if r.w <= 0.0 || r.h <= 0.0 { return; }
    if glow { bloom_block(scene, r, radius, c, 2.0 * k); }
    let stops = Stops::new(&[(0.0, c.brighter(0.2)), (0.3, c.brighter(0.6)), (0.6, c), (1.0, c.darker(0.25))]);
    scene.fill_round_linear(r, radius, stops, HALF_PI);
}

pub fn draw_slider_h(scene: &mut Scene, r: Rect, prop: f32, st: SliderStyle, c: Color, t: f32, glow: bool) {
    slider_h(scene, r, prop, st, c, t, glow, Lt::screen());
}

fn slider_h(scene: &mut Scene, r: Rect, mut prop: f32, st: SliderStyle, c: Color, t: f32, glow: bool, lt: Lt) {
    prop = lim(prop, 0.0, 1.0);
    let cy = r.center().y;
    let x = r.x + prop * r.w;
    let k = flicker(t, glow);
    match st {
        SliderStyle::Neon => {
            let groove = box_at(r.w + 8.0, 5.0, Point::new(r.center().x, cy));
            recess(scene, groove, 2.5, Color(0xff05090b), Color(0xff020405), lt);
            recess_shade(scene, groove, 2.5, 1.4, lt);
            hline(scene, cy, r.x, r.right(), c.with_alpha(0.22), 1.0);
            for i in 0..=10 {
                let tx = r.x + r.w * i as f32 / 10.0;
                scene.line(Point::new(tx, cy + 6.0), Point::new(tx, cy + if i % 5 == 0 { 12.0 } else { 9.0 }), c.with_alpha(0.35), 0.9);
            }
            lit_bar(scene, Rect::new(r.x, cy - 1.2, x - r.x, 2.4), 1.2, c, k, glow);
            let o = lt.offset(3.0);
            scene.shadow_ellipse(box_at(11.0, 19.0, Point::new(x + o.x, cy + o.y)), 2.2, Color(0xa0000000));
            let d = vec![Point::new(x, cy - 10.0), Point::new(x + 6.0, cy), Point::new(x, cy + 10.0), Point::new(x - 6.0, cy)];
            scene.fill_poly(d.clone(), c.darker(0.72), Color(0xff04090c));
            glow_stroke(scene, &d, c.brighter(0.2), 1.3, 0.9, glow, true);
            scene.line(Point::new(x, cy - 10.0), Point::new(x, cy + 10.0), c.brighter(0.2).with_alpha(0.55), 0.9);
            glow_dot(scene, Point::new(x, cy), 1.6, Color(0xffffffff), glow);
        }
        SliderStyle::Bar => {
            let win = box_at(r.w + 10.0, 30.0, Point::new(r.center().x, cy));
            recess(scene, win, 3.0, Color(0xff080a0b), Color(0xff030405), lt);
            let n = 30;
            let seg = r.w / n as f32;
            let mut segs = Vec::with_capacity(n);
            for i in 0..n {
                let f = i as f32 / (n - 1) as f32;
                let on = f <= prop + 0.5 / n as f32;
                let cc = if f > 0.85 { Color(0xffff3b2a) } else if f > 0.66 { Color(0xffffcf2a) } else { c };
                segs.push(Seg { r: box_at(seg - 1.8, 12.0 + 10.0 * f, Point::new(r.x + i as f32 * seg + 0.8 + (seg - 1.8) * 0.5, cy)), c: cc, on });
            }
            segments(scene, &segs, 0.5, t, glow);
            recess_shade(scene, win, 3.0, 2.6, lt);
            cap(scene, box_at(5.0, 34.0, Point::new(x, cy)), 1.5, Color(0xffe9e9e2), lt);
        }
        SliderStyle::Chamfer => {
            let track = box_at(r.w, 12.0, Point::new(r.center().x, cy));
            let frame = chamfer(track, 4.0, false);
            let lit = with_right(track, x);
            if glow && lit.w > 0.0 { bloom_rect(scene, lit, 3.0, c.with_alpha(1.0), 6.0, 0.16 * k); }
            scene.fill_poly(frame.clone(), Color(0xff04080b), Color(0xff020405));
            scene.clip_round(track.reduced(1.0, 1.0), 3.0);
            let stops = Stops::new(&[(0.0, c.darker(0.1)), (0.28, c.brighter(0.5)), (0.6, c.darker(0.05)), (1.0, c.darker(0.4))]);
            scene.fill_round_linear(lit, 0.0, stops, HALF_PI);
            scene.pop_clip();
            recess_shade(scene, track, 4.0, 2.0, lt);
            glow_stroke(scene, &frame, c, 1.2, 0.7, glow, true);
            let o = lt.offset(3.5);
            scene.shadow_round(box_at(16.0, 22.0, Point::new(x + o.x, cy + o.y)), 5.0, lt.sigma(8.0), Color(0xa0000000));
            let hex = chamfer(box_at(16.0, 22.0, Point::new(x, cy)), 5.0, false);
            scene.fill_poly(hex.clone(), c.darker(0.62), Color(0xff070d12));
            glow_stroke(scene, &hex, c.brighter(0.3), 1.4, 1.0, glow, true);
            lit_bar(scene, box_at(2.0, 10.0, Point::new(x, cy)), 1.0, c.brighter(0.6), 0.6 * k, glow);
        }
        SliderStyle::Flat => {
            let track = box_at(r.w, 6.0, Point::new(r.center().x, cy));
            recess(scene, track, 3.0, Color(0xff0b0d0e), Color(0xff050607), lt);
            lit_bar(scene, with_right(track, x), 3.0, c, 0.8 * k, glow);
            recess_shade(scene, track, 3.0, 1.3, lt);
            cap(scene, box_at(8.0, 20.0, Point::new(x, cy)), 2.0, Color(0xffe6e6e0), lt);
        }
        SliderStyle::Wedge => {
            let ramp = vec![Point::new(r.x, cy + 7.0), Point::new(r.right(), cy - 11.0), Point::new(r.right(), cy + 7.0)];
            scene.fill_poly(ramp.clone(), Color(0xff06090b), Color(0xff06090b));
            scene.fill_poly(ramp.clone(), c.with_alpha(0.10), c.with_alpha(0.10));
            let tpos = if r.w.abs() < 1.0e-4 { 0.0 } else { (x - r.x) / r.w };
            let y_at = cy + 7.0 + tpos * ((cy - 11.0) - (cy + 7.0));
            let edge = [Point::new(r.x, cy + 7.0), Point::new(x, y_at)];
            if glow && tpos > 0.0 {
                bloom_path(scene, &edge, false, c.mix(Color(0xffff4a2a), 0.5), 1.0, 5.0, 0.16 * k);
                bloom_rect(scene, Rect::new(r.x, cy + 6.0, x - r.x, 1.0), 0.5, c.mix(Color(0xffff4a2a), 0.5), 4.0, 0.12 * k);
            }
            scene.fill_poly(vec![Point::new(r.x, cy + 7.0), Point::new(x, y_at), Point::new(x, cy + 7.0)], c, Color(0xffff4a2a));
            let mut sx = r.x;
            while sx < r.right() {
                let u = if r.w.abs() < 1.0e-4 { 0.0 } else { (sx - r.x) / r.w };
                let yt = cy + 7.0 + u * -18.0;
                vline(scene, sx.trunc(), yt, cy + 7.0, Color(0x70000000), 1.0);
                sx += 4.0;
            }
            scene.stroke_poly(ramp, c.with_alpha(0.45), 1.0, true);
            if tpos > 0.0 { scene.line(edge[0], edge[1], Color(0xffffffff).with_alpha(0.55), 0.8); }
            cap(scene, box_at(5.0, 26.0, Point::new(x, cy - 2.0)), 1.5, Color(0xffeeeeea), lt);
        }
    }
}

pub fn draw_slider(scene: &mut Scene, r: Rect, prop: f32, st: SliderStyle, c: Color, vertical: bool, t: f32, glow: bool) {
    if !vertical {
        draw_slider_h(scene, r, prop, st, c, t, glow);
        return;
    }
    let mut tmp = Scene::new();
    let local = box_at(r.h, r.w, r.center());
    slider_h(&mut tmp, local, prop, st, c, t, glow, Lt::turned(-HALF_PI));
    blit(scene, tmp, &Rot::around(r.center().x, r.center().y, -HALF_PI));
}
