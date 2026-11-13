// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Marcos Quirino Deida

use crate::color::Color;
use crate::meters;
use crate::scene::{Cmd, Point, Rect, Scene};

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

pub fn glow_stroke(scene: &mut Scene, pts: &[Point], c: Color, w: f32, strength: f32, glow: bool, closed: bool) {
    if pts.len() < 2 { return; }
    if !glow {
        scene.stroke_poly(pts.to_vec(), c, w, closed);
        return;
    }
    for k in (1..=4).rev() {
        scene.stroke_poly(pts.to_vec(), c.with_alpha(0.05 * strength * (5 - k) as f32), w * (1.0 + 1.6 * k as f32), closed);
    }
    scene.stroke_poly(pts.to_vec(), c, w, closed);
    scene.stroke_poly(pts.to_vec(), c.brighter(0.7).with_alpha(0.9), w * 0.35, closed);
}
pub fn glow_line(scene: &mut Scene, a: Point, b: Point, c: Color, w: f32, strength: f32, glow: bool) {
    glow_stroke(scene, &[a, b], c, w, strength, glow, false);
}
pub fn glow_dot(scene: &mut Scene, c: Point, r: f32, col: Color, glow: bool) {
    if !glow { scene.fill_ellipse_solid(box_at(r * 2.0, r * 2.0, c), col); return; }
    scene.fill_ellipse_solid(box_at(r * 6.0, r * 6.0, c), col.with_alpha(0.18));
    scene.fill_ellipse_solid(box_at(r * 3.6, r * 3.6, c), col.with_alpha(0.35));
    scene.fill_ellipse_solid(box_at(r * 1.4, r * 1.4, c), col.brighter(0.8));
}
pub fn glow_round(scene: &mut Scene, r: Rect, radius: f32, c: Color, w: f32, strength: f32, glow: bool) {
    if !glow { scene.stroke_round(r, radius, c, w); return; }
    for k in (1..=4).rev() {
        scene.stroke_round(r, radius, c.with_alpha(0.05 * strength * (5 - k) as f32), w * (1.0 + 1.6 * k as f32));
    }
    scene.stroke_round(r, radius, c, w);
    scene.stroke_round(r, radius, c.brighter(0.7).with_alpha(0.9), w * 0.35);
}
fn juce_arc(scene: &mut Scene, c: Point, rad: f32, from: f32, to: f32, color: Color, width: f32) {
    let k = -HALF_PI;
    scene.stroke_arc(c, rad, from + k, to + k, color, width);
}
fn glow_juce_arc(scene: &mut Scene, c: Point, rad: f32, from: f32, to: f32, color: Color, w: f32, strength: f32, glow: bool) {
    if !glow { juce_arc(scene, c, rad, from, to, color, w); return; }
    for k in (1..=4).rev() {
        juce_arc(scene, c, rad, from, to, color.with_alpha(0.05 * strength * (5 - k) as f32), w * (1.0 + 1.6 * k as f32));
    }
    juce_arc(scene, c, rad, from, to, color, w);
    juce_arc(scene, c, rad, from, to, color.brighter(0.7).with_alpha(0.9), w * 0.35);
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
    scene.fill_rect(Rect::new(r.x, sweep - 14.0, r.w, 14.0), Color(0x00ffffff), Color(0x16ffffff), true);
}

pub fn draw_bar_wedge(scene: &mut Scene, r: Rect, level: f32, lit: Color, t: f32, glow: bool) {
    scene.fill_round(r, 5.0, Color(0xff17140a), Color(0xff060502), true);
    let mut b = r.reduced(8.0, 6.0);
    b.h = (b.h - 10.0).max(0.0);
    let n = 26;
    for i in 0..n {
        let f = i as f32 / (n - 1) as f32;
        let x = b.x + f * (b.w - 4.0);
        let h = b.h * (0.14 + 0.86 * f.powf(1.35));
        let w = b.w / n as f32 * 0.68;
        let on = f <= level + 0.5 / n as f32;
        let c = if f > 0.84 { Color(0xffff3b2a) } else if f > 0.68 { lit.mix(Color(0xffff6a1a), 0.55) } else { lit };
        let bar = Rect::new(x, b.bottom() - h, w, h);
        if on && glow { solid_round(scene, bar.expanded(2.5), 2.0, c.with_alpha(0.22 * flicker(t + f, true))); }
        let body = if on { c.with_alpha(flicker(t + f * 3.0, glow)) } else { c.with_alpha(0.10) };
        solid_round(scene, bar, 1.2, body);
        if on { solid_rect(scene, Rect::new(bar.x, bar.y, bar.w * 0.35, bar.h), Color(0x33ffffff)); }
    }
    let mut q = r.reduced(8.0, 2.0);
    let labels = remove_bottom(&mut q, 10.0);
    for i in 0..=5 {
        text_c(scene, Rect::new(b.x + b.w * i as f32 / 5.0 - 10.0, labels.y, 20.0, labels.h), &(i * 12).to_string(), lit.with_alpha(0.7), 7.5);
    }
    scanlines(scene, r, t, 0.14, glow);
    scene.stroke_round(r, 5.0, lit.with_alpha(0.35), 1.2);
}

pub fn draw_mountains(scene: &mut Scene, r: Rect, left: f32, right: f32, lit: Color, t: f32, glow: bool) {
    solid_round(scene, r, 5.0, Color(0xff020a06));
    let b = r.reduced(8.0, 7.0);
    let cx = b.center().x;
    let half = b.w * 0.5 - 12.0;
    let n = 22;
    for side in 0..2 {
        let level = if side == 0 { left } else { right };
        let dirn = if side == 0 { -1.0 } else { 1.0 };
        for i in 0..n {
            let f = i as f32 / (n - 1) as f32;
            let x = cx + dirn * (10.0 + f * half);
            let h = b.h * (0.92 - 0.80 * f);
            let on = (1.0 - f) <= level + 0.5 / n as f32;
            let base = Point::new(x, b.bottom());
            let top = Point::new(x - dirn * (b.w * 0.02), b.bottom() - h);
            if on && glow { scene.line(base, top, lit.with_alpha(0.22 * flicker(t + f, true)), 3.6); }
            scene.line(base, top, if on { lit.brighter(0.15) } else { lit.with_alpha(0.12) }, 1.2);
        }
    }
    for i in 0..6 {
        let y = b.bottom() - b.h * i as f32 / 5.0;
        text_c(scene, box_at(20.0, 9.0, Point::new(cx, y - 3.0)), &(i * 10).to_string(), lit.with_alpha(0.8), 7.5);
        hline(scene, y.trunc(), cx - 8.0, cx - 5.0, lit.with_alpha(0.8), 1.0);
        hline(scene, y.trunc(), cx + 5.0, cx + 8.0, lit.with_alpha(0.8), 1.0);
    }
    hline(scene, (b.bottom() + 1.0).trunc(), b.x, b.right(), lit.with_alpha(0.6), 1.0);
    scanlines(scene, r, t, 0.2, glow);
    scene.stroke_round(r, 5.0, lit.with_alpha(0.3), 1.0);
}

pub fn draw_lcd_panel(scene: &mut Scene, r: Rect, a: &str, a_label: &str, b: &str, b_label: &str, lit: Color, t: f32, glow: bool) {
    scene.fill_round(r, 6.0, Color(0xff1b1108), Color(0xff070402), true);
    scene.stroke_round(r, 6.0, lit.with_alpha(0.55), 1.4);
    let inn = r.reduced(7.0, 7.0);
    let half = inn.w * 0.5 - 3.0;
    for i in 0..2 {
        let mut cell = Rect::new(inn.x + i as f32 * (half + 6.0), inn.y, half, inn.h);
        let lab = remove_bottom(&mut cell, 12.0);
        scene.stroke_round(cell, 4.0, lit.with_alpha(0.35), 1.0);
        meters::draw_seven_segment(scene, cell.reduced(5.0, 5.0), if i == 0 { a } else { b }, lit.with_alpha(flicker(t + i as f32, glow)), true);
        text_c(scene, lab, if i == 0 { a_label } else { b_label }, lit.with_alpha(0.85), 8.5);
    }
    scanlines(scene, r, t, 0.12, glow);
}

pub fn draw_arc_gauge(scene: &mut Scene, r: Rect, level: f32, lit: Color, t: f32, glow: bool) {
    scene.fill_round(r, 6.0, Color(0xff15110c), Color(0xff050404), true);
    let rad = (r.w * 0.46).min(r.h * 0.92);
    let c = Point::new(r.center().x, r.bottom() - 6.0);
    let arc = |scene: &mut Scene, f0: f32, f1: f32, rr: f32, w: f32, col: Color| {
        juce_arc(scene, c, rr, (-0.5 + f0) * PI, (-0.5 + f1) * PI, col, w);
    };
    arc(scene, 0.0, 1.0, rad, rad * 0.22, lit.with_alpha(0.12));
    arc(scene, 0.0, level.min(0.86), rad, rad * 0.30, lit.with_alpha(0.22 * flicker(t, glow)));
    arc(scene, 0.0, level.min(0.86), rad, rad * 0.20, lit);
    if level > 0.86 { arc(scene, 0.86, level, rad, rad * 0.20, Color(0xffff3b2a)); }
    arc(scene, 0.86, 1.0, rad * 0.78, rad * 0.06, Color(0xffff3b2a).with_alpha(0.85));
    for i in 0..=9 {
        let f = i as f32 / 9.0;
        let a = (-0.5 + f) * PI;
        let tip0 = Point::new(c.x + a.sin() * rad * 0.82, c.y - a.cos() * rad * 0.82);
        let tip1 = Point::new(c.x + a.sin() * rad * 0.74, c.y - a.cos() * rad * 0.74);
        let col = if f > 0.86 { Color(0xffff6a4a) } else { lit.with_alpha(0.9) };
        scene.line(tip0, tip1, col, 1.3);
        text_c(scene, box_at(14.0, 10.0, Point::new(c.x + a.sin() * rad * 0.64, c.y - a.cos() * rad * 0.64)), &i.to_string(), col, 9.0);
    }
    meters::draw_seven_segment(scene, box_at(rad * 0.55, rad * 0.30, Point::new(c.x, c.y - rad * 0.22)), &((level * 99.0) as i32).to_string(), lit, false);
    scanlines(scene, r, t, 0.12, glow);
    scene.stroke_round(r, 6.0, lit.with_alpha(0.4), 1.2);
}

pub fn draw_colour_columns(scene: &mut Scene, r: Rect, l: f32, rr: f32, peak_l: f32, peak_r: f32, t: f32, glow: bool) {
    solid_round(scene, r, 5.0, Color(0xff060907));
    let mut b = r.reduced(10.0, 7.0);
    let _left = remove_left(&mut b, 12.0);
    let n = 22;
    let seg = b.w / n as f32;
    let row_h = (b.h - 6.0) * 0.5;
    for c in 0..2 {
        let lv = if c == 0 { l } else { rr };
        let pk = if c == 0 { peak_l } else { peak_r };
        let row = Rect::new(b.x, b.y + c as f32 * (row_h + 6.0), b.w, row_h);
        text_c(scene, box_at(10.0, row_h, Point::new(r.x + 10.0, row.center().y)), if c == 0 { "L" } else { "R" }, Color(0xff8b9498), 8.0);
        for i in 0..n {
            let f = i as f32 / (n - 1) as f32;
            let cc = if f > 0.85 { Color(0xffff3b2a) } else if f > 0.62 { Color(0xffffcf2a) } else { Color(0xff35e04a) };
            let on = f <= lv + 0.5 / n as f32 || (pk > 0.0 && (f - pk).abs() < 0.5 / n as f32);
            let s = Rect::new(row.x + i as f32 * seg + 1.0, row.y, seg - 2.0, row.h);
            if on && glow { solid_rect(scene, s.expanded(1.6), cc.with_alpha(0.25 * flicker(t + f, true))); }
            solid_rect(scene, s, if on { cc } else { cc.with_alpha(0.10) });
            if on { solid_rect(scene, Rect::new(s.x, s.y, s.w, 2.0), Color(0x40ffffff)); }
        }
    }
    scanlines(scene, r, t, 0.10, glow);
    scene.stroke_round(r, 5.0, Color(0x44ffffff), 1.0);
}

pub fn draw_neon_bars(scene: &mut Scene, r: Rect, level: f32, peak: f32, lo: Color, hi: Color, t: f32, glow: bool) {
    let frame = chamfer(r.reduced(1.5, 1.5), 10.0, false);
    scene.fill_poly(frame.clone(), Color(0xff06161c), Color(0xff020609));
    scene.clip_round(r, 0.0);
    let grid = lo.with_alpha(0.07);
    let mut x = r.x;
    while x < r.right() { vline(scene, x.trunc(), r.y, r.bottom(), grid, 1.0); x += 12.0; }
    let mut y = r.y;
    while y < r.bottom() { hline(scene, y.trunc(), r.x, r.right(), grid, 1.0); y += 12.0; }
    let b = r.reduced(14.0, 12.0);
    let n = 24;
    for i in 0..n {
        let f = i as f32 / (n - 1) as f32;
        let x = b.x + f * b.w;
        let w = b.w / n as f32 * 0.55;
        let shape = 0.45 + 0.55 * (f * PI * 0.5 + 0.4).sin();
        let h = b.h * shape;
        let fill = lim((level - f * 0.6) / 0.4, 0.0, 1.0);
        let c = heat(f, lo, hi);
        let bar = Rect::new(x, b.bottom() - h, w, h);
        solid_rect(scene, bar, c.with_alpha(0.10));
        if fill > 0.0 {
            let lit = with_top(bar, bar.bottom() - h * fill);
            if glow { solid_rect(scene, Rect::new(lit.x - 2.5, lit.y - 1.0, lit.w + 5.0, lit.h + 2.0), c.with_alpha(0.28 * flicker(t + f * 2.0, true))); }
            solid_rect(scene, lit, c);
            solid_rect(scene, Rect::new(lit.x, lit.y, lit.w, 2.0), c.brighter(0.9));
        }
    }
    let px = b.x + lim(peak, 0.0, 1.0) * b.w;
    glow_line(scene, Point::new(px, b.y), Point::new(px, b.bottom()), hi, 1.2, 0.8, glow);
    scanlines(scene, r, t, 0.16, glow);
    scene.pop_clip();
    glow_stroke(scene, &frame, lo, 1.4, 1.0, glow, true);
    glow_line(scene, Point::new(r.x + 14.0, r.y + 1.0), Point::new(r.x + 44.0, r.y + 1.0), hi, 2.2, 1.0, glow);
}

pub fn draw_hex_ring(scene: &mut Scene, r: Rect, level: f32, lo: Color, hi: Color, t: f32, glow: bool) {
    let c = r.center();
    let radius = r.w.min(r.h) * 0.48;
    let mut hx = Vec::new();
    for i in 0..6 {
        let a = i as f32 * PI / 3.0 + PI / 6.0;
        hx.push(Point::new(c.x + a.cos() * radius * 1.12, c.y + a.sin() * radius * 1.12));
    }
    solid_round(scene, r, 6.0, Color(0xff03080d));
    scene.fill_poly(hx.clone(), lo.darker(0.9), Color(0xff020406));
    glow_stroke(scene, &hx, lo.with_alpha(0.7), 1.2, 0.8, glow, true);
    let a0 = -PI * 1.25;
    let a1 = PI * 0.25;
    let ring = radius * 0.80;
    juce_arc(scene, c, ring, a0 + HALF_PI, a1 + HALF_PI, lo.with_alpha(0.18), radius * 0.10);
    glow_juce_arc(scene, c, ring, a0 + HALF_PI, a0 + (a1 - a0) * level + HALF_PI, heat(level, lo, hi), radius * 0.10, 1.0, glow);
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
    meters::draw_seven_segment(scene, box_at(radius * 0.9, radius * 0.46, Point::new(c.x, c.y + radius * 0.05)), &((level * 100.0) as i32).to_string(), heat(level, lo, hi).with_alpha(flicker(t, glow)), false);
    scanlines(scene, r, t, 0.12, glow);
}

pub fn draw_scope(scene: &mut Scene, r: Rect, hist: &[f32], clip: bool, lit: Color, t: f32, glow: bool) {
    let frame = chamfer(r.reduced(1.5, 1.5), 8.0, true);
    scene.fill_poly(frame.clone(), Color(0xff020b0d), Color(0xff020b0d));
    scene.clip_round(r, 0.0);
    let grid = lit.with_alpha(0.09);
    let mut x = r.x;
    while x < r.right() { vline(scene, x.trunc(), r.y, r.bottom(), grid, 1.0); x += 14.0; }
    hline(scene, r.center().y.trunc(), r.x, r.right(), grid, 1.0);
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
        scene.fill_poly(fill, lit.with_alpha(0.30), lit.with_alpha(0.05));
        glow_stroke(scene, &top, lit, 1.2, 0.9, glow, false);
        glow_stroke(scene, &bot, lit.with_alpha(0.7), 1.0, 0.6, glow, false);
        let ex = r.right() - 3.0;
        glow_dot(scene, Point::new(ex, cy - hist[hist.len() - 1] * amp), 2.0, lit, glow);
        let ghost: Vec<Point> = top.iter().map(|p| Point::new(p.x + 1.5, p.y)).collect();
        scene.stroke_poly(ghost, Color(0x30ff2a6a), 1.0, false);
    }
    if clip && (t.rem_euclid(0.8)) < 0.5 {
        let bar = r.translated(0.0, 3.0);
        solid_rect(scene, Rect::new(bar.x, bar.y, bar.w, 14.0), Color(0xaaff2a3a));
        text_c(scene, Rect::new(bar.x, bar.y, bar.w, 14.0), "! ATTENTION  CLIP", Color(0xffffffff), 9.0);
    }
    scanlines(scene, r, t, 0.2, glow);
    scene.pop_clip();
    glow_stroke(scene, &frame, lit.with_alpha(0.8), 1.2, 0.7, glow, true);
}

pub fn draw_chassis(scene: &mut Scene, r: Rect, level: f32, body: Color, lit: Color, t: f32, glow: bool) {
    solid_round(scene, r.translated(0.0, 2.0), 9.0, Color(0x66000000));
    scene.fill_round(r, 9.0, body.brighter(0.25), body.darker(0.45), true);
    scene.stroke_round(r, 9.0, body.darker(0.8), 1.4);
    for p in [Point::new(r.x + 7.0, r.y + 7.0), Point::new(r.right() - 7.0, r.y + 7.0), Point::new(r.x + 7.0, r.bottom() - 7.0), Point::new(r.right() - 7.0, r.bottom() - 7.0)] {
        scene.fill_ellipse_solid(box_at(4.4, 4.4, p), body.darker(0.7));
    }
    let mut s = r.reduced(13.0, 11.0);
    solid_round(scene, s, 5.0, Color(0xff04080a));
    scene.stroke_round(s, 5.0, lit.with_alpha(0.4), 1.0);
    let bars = trim_top(trim_bottom(s.reduced(8.0, 0.0), 14.0), 8.0);
    let n = 28;
    for i in 0..n {
        let f = i as f32 / (n - 1) as f32;
        let x = bars.x + f * bars.w;
        let on = f <= level + 0.5 / n as f32;
        let c = if f > 0.85 { Color(0xffff4a5a) } else { lit };
        let inset = if i % 4 == 0 { 0.0 } else { 5.0 };
        let seg = Rect::new(x, bars.y + inset, bars.w / n as f32 * 0.62, bars.h - inset);
        if on && glow { solid_rect(scene, seg.expanded(2.0), c.with_alpha(0.25 * flicker(t + f, true))); }
        solid_rect(scene, seg, if on { c } else { c.with_alpha(0.10) });
    }
    let foot = remove_bottom(&mut s, 13.0);
    text_c(scene, Rect::new(foot.x + 8.0, foot.y, foot.w - 8.0, foot.h), "OUTPUT  //  LEVEL", lit.with_alpha(0.8), 8.0);
    scanlines(scene, s, t, 0.14, glow);
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

pub fn draw_icon_lamp(scene: &mut Scene, r: Rect, icon: Icon, c: Color, on: bool) {
    solid_round(scene, r, 3.0, Color(0xff050606));
    scene.stroke_round(r, 3.0, if on { c.with_alpha(0.7) } else { Color(0xff2a2f31) }, 1.0);
    if on { solid_round(scene, r.expanded(2.0), 5.0, c.with_alpha(0.18)); }
    let col = if on { c } else { c.with_alpha(0.18) };
    let a = r.reduced(r.w * 0.24, r.w * 0.24);
    match icon {
        Icon::Battery => {
            scene.stroke_poly(rect_pts(trim_top(a, a.h * 0.2)), col, 1.4, true);
            let nub = box_at(a.w * 0.3, a.h * 0.16, Point::new(a.center().x, a.y + a.h * 0.08));
            scene.stroke_poly(rect_pts(nub), col, 1.4, true);
            text_c(scene, trim_top(a, a.h * 0.2), "+", col, a.h * 0.55);
        }
        Icon::Warning => {
            scene.stroke_poly(vec![Point::new(a.center().x, a.y), Point::new(a.right(), a.bottom()), Point::new(a.x, a.bottom())], col, 1.5, true);
            text_c(scene, trim_top(a, a.h * 0.25), "!", col, a.h * 0.62);
        }
        Icon::ArrowLeft => scene.fill_poly(vec![Point::new(a.x, a.center().y), Point::new(a.right(), a.y), Point::new(a.right(), a.bottom())], col, col),
        Icon::ArrowRight => scene.fill_poly(vec![Point::new(a.right(), a.center().y), Point::new(a.x, a.y), Point::new(a.x, a.bottom())], col, col),
        Icon::Beam => {
            for i in 0..4 {
                let y = a.y + a.h * (0.15 + 0.23 * i as f32);
                scene.line(Point::new(a.x + a.w * 0.35, y), Point::new(a.right(), y), col, 1.6);
            }
            scene.fill_ellipse_solid(Rect::new(a.x, a.center().y - a.h * 0.3, a.w * 0.4, a.h * 0.6), col);
        }
        Icon::Temp => {
            scene.line(Point::new(a.center().x, a.y), Point::new(a.center().x, a.bottom() - a.w * 0.3), col, 2.4);
            scene.fill_ellipse_solid(Rect::new(a.center().x - a.w * 0.28, a.bottom() - a.w * 0.56, a.w * 0.56, a.w * 0.56), col);
        }
    }
}
fn rect_pts(r: Rect) -> Vec<Point> {
    vec![Point::new(r.x, r.y), Point::new(r.right(), r.y), Point::new(r.right(), r.bottom()), Point::new(r.x, r.bottom())]
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SliderStyle { Neon, Bar, Chamfer, Wedge, Flat }

pub fn draw_slider_h(scene: &mut Scene, r: Rect, mut prop: f32, st: SliderStyle, c: Color, t: f32, glow: bool) {
    prop = lim(prop, 0.0, 1.0);
    let cy = r.center().y;
    let x = r.x + prop * r.w;
    match st {
        SliderStyle::Neon => {
            glow_line(scene, Point::new(r.x, cy), Point::new(r.right(), cy), c.with_alpha(0.28), 1.4, 0.3, glow);
            for i in 0..=10 {
                let tx = r.x + r.w * i as f32 / 10.0;
                scene.line(Point::new(tx, cy + 5.0), Point::new(tx, cy + if i % 5 == 0 { 11.0 } else { 8.0 }), c.with_alpha(0.35), 0.9);
            }
            glow_line(scene, Point::new(r.x, cy), Point::new(x, cy), c, 2.4, 1.0, glow);
            solid_rect(scene, box_at(8.0, 18.0, Point::new(x, cy)), Color(0xff04090c));
            let d = vec![
                Point::new(x, cy - 10.0), Point::new(x + 6.0, cy), Point::new(x, cy + 10.0),
                Point::new(x, cy - 10.0), Point::new(x - 6.0, cy), Point::new(x, cy + 10.0),
            ];
            scene.fill_poly(vec![Point::new(x, cy - 10.0), Point::new(x + 6.0, cy), Point::new(x, cy + 10.0)], Color(0xff06161c), Color(0xff06161c));
            scene.fill_poly(vec![Point::new(x, cy - 10.0), Point::new(x - 6.0, cy), Point::new(x, cy + 10.0)], Color(0xff06161c), Color(0xff06161c));
            glow_stroke(scene, &d[..3], c.brighter(0.2), 1.3, 0.9, glow, true);
            glow_stroke(scene, &d[3..], c.brighter(0.2), 1.3, 0.9, glow, true);
            glow_dot(scene, Point::new(x, cy), 1.6, Color(0xffffffff), glow);
        }
        SliderStyle::Bar => {
            let n = 30;
            let seg = r.w / n as f32;
            for i in 0..n {
                let f = i as f32 / (n - 1) as f32;
                let on = f <= prop + 0.5 / n as f32;
                let cc = if f > 0.85 { Color(0xffff3b2a) } else if f > 0.66 { Color(0xffffcf2a) } else { c };
                let s = box_at(seg - 1.8, 12.0 + 10.0 * f, Point::new(r.x + i as f32 * seg + 0.8 + (seg - 1.8) * 0.5, cy));
                if on && glow { solid_rect(scene, s.expanded(2.0), cc.with_alpha(0.22 * flicker(t + f, true))); }
                solid_rect(scene, s, if on { cc } else { cc.with_alpha(0.10) });
            }
            solid_rect(scene, box_at(2.5, 28.0, Point::new(x, cy)), Color(0xfff6f6f0));
        }
        SliderStyle::Chamfer => {
            let track = box_at(r.w, 12.0, Point::new(r.center().x, cy));
            let frame = chamfer(track, 4.0, false);
            scene.fill_poly(frame.clone(), Color(0xff04080b), Color(0xff04080b));
            scene.clip_round(track, 0.0);
            scene.fill_rect(with_right(track, x), c.darker(0.2), c.brighter(0.3), false);
            scene.pop_clip();
            glow_stroke(scene, &frame, c, 1.2, 0.7, glow, true);
            let hex = chamfer(box_at(16.0, 22.0, Point::new(x, cy)), 5.0, false);
            scene.fill_poly(hex.clone(), Color(0xff0a1218), Color(0xff0a1218));
            glow_stroke(scene, &hex, c.brighter(0.3), 1.4, 1.0, glow, true);
            solid_rect(scene, box_at(2.0, 10.0, Point::new(x, cy)), c.brighter(0.6));
        }
        SliderStyle::Flat => {
            let track = box_at(r.w, 6.0, Point::new(r.center().x, cy));
            solid_round(scene, track, 1.5, Color(0xff0b0d0e));
            solid_round(scene, with_right(track, x), 1.5, c);
            solid_round(scene, box_at(7.0, 20.0, Point::new(x, cy)), 1.5, Color(0xfff1f1ec));
            scene.stroke_round(box_at(7.0, 20.0, Point::new(x, cy)), 1.5, Color(0xff1b1d1e), 0.8);
        }
        SliderStyle::Wedge => {
            let ramp = vec![Point::new(r.x, cy + 7.0), Point::new(r.right(), cy - 11.0), Point::new(r.right(), cy + 7.0)];
            scene.fill_poly(ramp.clone(), c.with_alpha(0.12), c.with_alpha(0.12));
            let tpos = if r.w.abs() < 1.0e-4 { 0.0 } else { (x - r.x) / r.w };
            let y_at = cy + 7.0 + tpos * ((cy - 11.0) - (cy + 7.0));
            scene.fill_poly(vec![Point::new(r.x, cy + 7.0), Point::new(x, y_at), Point::new(x, cy + 7.0)], c, Color(0xffff4a2a));
            let mut sx = r.x;
            while sx < r.right() {
                let u = if r.w.abs() < 1.0e-4 { 0.0 } else { (sx - r.x) / r.w };
                let yt = cy + 7.0 + u * -18.0;
                vline(scene, sx.trunc(), yt, cy + 7.0, Color(0x66000000), 1.0);
                sx += 4.0;
            }
            scene.stroke_poly(ramp, c.with_alpha(0.7), 1.0, true);
            solid_rect(scene, box_at(2.5, 22.0, Point::new(x, cy - 2.0)), Color(0xffffffff));
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
    draw_slider_h(&mut tmp, local, prop, st, c, t, glow);
    blit(scene, tmp, &Rot::around(r.center().x, r.center().y, -HALF_PI));
}
