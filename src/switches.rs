// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Marcos Deida

use crate::color::Color;
use crate::retro;
use crate::scene::{Cmd, Point, Rect, Scene};

const HALF_PI: f32 = std::f32::consts::FRAC_PI_2;
const PI: f32 = std::f32::consts::PI;

fn solid_round(scene: &mut Scene, r: Rect, radius: f32, c: Color) { scene.fill_round(r, radius, c, c, true); }
fn hline(scene: &mut Scene, y: f32, x0: f32, x1: f32, c: Color, w: f32) { scene.line(Point::new(x0, y), Point::new(x1, y), c, w); }
fn text_c(scene: &mut Scene, r: Rect, text: &str, c: Color, size: f32) { scene.text(r.center(), text, c, size, true); }
fn box_at(w: f32, h: f32, c: Point) -> Rect { Rect::new(c.x - w * 0.5, c.y - h * 0.5, w, h) }

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RockerStyle { Lamp, Classic, Slim, Neon }
pub static ROCKER_NAMES: [&str; 4] = ["Lamp rocker", "Classic rocker", "Slim rocker", "Neon rocker"];

pub fn draw_rocker(scene: &mut Scene, r: Rect, on: bool, lamp: Color, style: RockerStyle, horizontal: bool, glow: bool) {
    let mut tmp = Scene::new();
    let g = &mut tmp;
    let b = if horizontal { box_at(r.h, r.w, r.center()) } else { r };
    let w = b.w;
    let h = b.h;
    let cx = b.center().x;
    solid_round(g, b.expanded(1.5), 4.0, Color(0xff0a0b0c));
    let plate = b.reduced(w * 0.08, h * 0.05);
    let split = plate.center().y;
    let top = Rect::new(plate.x, plate.y, plate.w, split + 1.0 - plate.y);
    let bottom = Rect::new(plate.x, split - 1.0, plate.w, plate.bottom() - (split - 1.0));
    let (raised, sunk) = if on { (top, bottom) } else { (bottom, top) };
    let body = match style {
        RockerStyle::Classic => Color(0xff25272a),
        RockerStyle::Slim => Color(0xffd8dad6),
        RockerStyle::Neon => Color(0xff05090d),
        RockerStyle::Lamp => Color(0xff2a2c2f),
    };
    let light = style == RockerStyle::Slim;
    g.fill_round(sunk, 3.0, body.darker(0.5), body.darker(0.2), true);
    g.fill_round(raised, 3.0, body.brighter(if light { 0.05 } else { 0.35 }), body.darker(0.15), true);
    hline(g, raised.y + 1.5, raised.x + 3.0, raised.right() - 3.0, Color(0x55ffffff), 1.0);
    hline(g, split, plate.x, plate.right(), Color(0xff000000).with_alpha(0.7), 1.0);
    let ink = if light { Color(0xff222426) } else { Color(0xfff2f2ee) };
    let mark = w.min(h) * 0.17;
    solid_round(g, box_at(mark * 0.45, mark * 2.0, Point::new(cx, top.center().y)), 1.0, ink);
    g.stroke_ellipse(box_at(mark * 1.7, mark * 1.7, Point::new(cx, bottom.center().y)), ink, mark * 0.38);
    if style == RockerStyle::Lamp || style == RockerStyle::Neon {
        let win = box_at(plate.w * 0.62, h * 0.075, Point::new(cx, top.y + h * 0.075));
        if on { solid_round(g, win.expanded(3.0), 3.0, lamp.with_alpha(0.30)); }
        let lc = if on { lamp } else { lamp.darker(0.9) };
        solid_round(g, win, 1.5, lc);
        if on { solid_round(g, Rect::new(win.x, win.y, win.w, win.h * 0.4), 1.0, Color(0x66ffffff)); }
    }
    if style == RockerStyle::Neon {
        retro::glow_round(g, plate, 3.0, if on { lamp } else { lamp.with_alpha(0.35) }, 1.2, if on { 1.0 } else { 0.2 }, glow);
    }
    if horizontal {
        blit(scene, tmp, &Rot::around(r.center().x, r.center().y, HALF_PI));
    } else {
        scene.cmds.append(&mut tmp.cmds);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToggleStyle { Bat, Lever, Mini }
pub static TOGGLE_NAMES: [&str; 3] = ["Bat toggle", "Lever toggle", "Mini toggle"];

pub fn draw_toggle(scene: &mut Scene, r: Rect, on: bool, style: ToggleStyle, accent: Color) {
    let c = r.center();
    let s = r.w.min(r.h * 0.8);
    let nut_r = s * if style == ToggleStyle::Mini { 0.30 } else { 0.38 };
    let mut nut = Vec::new();
    for i in 0..6 {
        let a = i as f32 * PI / 3.0;
        nut.push(Point::new(c.x + a.cos() * nut_r * 1.15, c.y + a.sin() * nut_r * 1.15));
    }
    let shifted: Vec<Point> = nut.iter().map(|p| Point::new(p.x + 1.0, p.y + 2.0)).collect();
    scene.fill_poly(shifted, Color(0x66000000), Color(0x66000000));
    scene.fill_poly(nut.clone(), Color(0xffe9ecee), Color(0xff6b7378));
    scene.stroke_poly(nut, Color(0xff2a2f32), 0.8, true);
    let d = nut_r * 1.24;
    scene.fill_ellipse(Rect::new(c.x - nut_r * 0.62, c.y - nut_r * 0.62, d, d), Color(0xff40454a), Color(0xff0f1113));
    let len = r.h * if style == ToggleStyle::Bat { 0.30 } else { 0.42 };
    let dir = if on { -1.0 } else { 1.0 };
    let tip = Point::new(c.x, c.y + dir * len);
    let w0 = s * if style == ToggleStyle::Bat { 0.18 } else { 0.10 };
    let w1 = s * if style == ToggleStyle::Bat { 0.30 } else { 0.13 };
    let lev = vec![Point::new(c.x - w0, c.y), Point::new(tip.x - w1, tip.y), Point::new(tip.x + w1, tip.y), Point::new(c.x + w0, c.y)];
    let sh: Vec<Point> = lev.iter().map(|p| Point::new(p.x + 1.5, p.y + 1.5)).collect();
    scene.fill_poly(sh, Color(0x66000000), Color(0x66000000));
    scene.fill_poly(lev.clone(), Color(0xfff7f9f9), Color(0xff7f8689));
    scene.stroke_poly(lev, Color(0xff2a2f32), 0.8, true);
    let tr = w1 * if style == ToggleStyle::Bat { 1.1 } else { 1.0 };
    scene.fill_ellipse(Rect::new(tip.x - tr, tip.y - tr * 0.8, tr * 2.0, tr * 1.6), Color(0xffffffff), Color(0xff8f979b));
    if style == ToggleStyle::Lever && on {
        scene.fill_ellipse_solid(Rect::new(tip.x - tr * 0.5, tip.y - tr * 0.4, tr, tr * 0.8), accent);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PushStyle { Square, Round, Dome, Chamfer, Neon, Flat, Arcade, Pill }
pub static PUSH_NAMES: [&str; 8] = ["Square key", "Round chrome", "Illuminated dome", "Chamfer pad", "Neon outline", "Flat label", "Arcade", "Pill"];
pub const NUM_PUSH_STYLES: i32 = 8;

pub fn draw_push(scene: &mut Scene, r: Rect, text: &str, style: PushStyle, colour: Color, lit: bool, down: bool, glow: bool) {
    let off = if down { 2.0 } else { 0.0 };
    let face = r.translated(0.0, off);
    match style {
        PushStyle::Square => {
            solid_round(scene, r.expanded(2.0), 3.5, Color(0xff08090a));
            scene.stroke_round(r.expanded(2.0), 3.5, Color(0x30ffffff), 0.7);
            if !down { solid_round(scene, face.translated(0.0, 2.5), 2.5, Color(0x88000000)); }
            let base = if lit { colour } else { Color(0xffd6d9c8) };
            scene.fill_round(face, 2.0, base.brighter(if down { 0.05 } else { 0.28 }), base.darker(if down { 0.40 } else { 0.20 }), false);
            hline(scene, face.y + 1.0, face.x + 2.0, face.right() - 2.0, Color(0x77ffffff), 1.0);
            hline(scene, face.bottom() - 1.0, face.x + 2.0, face.right() - 2.0, Color(0x55000000), 1.0);
            scene.stroke_round(face, 2.0, Color(0x40000000), 0.8);
            text_c(scene, face, text, if lit { Color(0xff0d0f11) } else { Color(0xff222a28) }, (face.h * 0.42).min(13.0));
        }
        PushStyle::Round => {
            let d = r.w.min(r.h);
            let cir = box_at(d, d, r.center());
            scene.fill_ellipse_solid(cir.expanded(2.5), Color(0xff08090a));
            scene.fill_ellipse(cir, Color(0xffffffff), Color(0xff454c50));
            scene.stroke_ellipse(cir, Color(0x40000000), 1.0);
            let inner = cir.reduced(d * 0.12, d * 0.12).translated(0.0, off);
            if !down { scene.fill_ellipse_solid(inner.translated(0.0, 2.0), Color(0x77000000)); }
            scene.fill_ellipse(inner, Color(0xfff0f3f4), Color(0xff757d82));
            scene.stroke_ellipse(inner.reduced(0.8, 0.8), Color(0x88ffffff), 0.7);
            if lit {
                scene.stroke_ellipse(cir.reduced(d * 0.03, d * 0.03), colour.with_alpha(0.38), d * 0.07);
                scene.stroke_ellipse(cir.reduced(d * 0.07, d * 0.07), colour, d * 0.03);
            }
            text_c(scene, inner, text, Color(0xff1d2124), (inner.h * 0.42).min(13.0));
        }
        PushStyle::Dome => {
            let d = r.w.min(r.h);
            let cir = box_at(d, d, r.center()).translated(0.0, off * 0.5);
            scene.fill_ellipse_solid(cir.expanded(3.0), Color(0xff050607));
            scene.stroke_ellipse(cir.expanded(3.0), Color(0x25ffffff), 0.8);
            if lit { scene.fill_ellipse_solid(cir.expanded(d * 0.26), colour.with_alpha(0.32)); }
            let base = if lit { colour } else { colour.darker(0.85) };
            scene.fill_ellipse(cir, base.brighter(if lit { 0.95 } else { 0.25 }), base.darker(0.55));
            scene.fill_ellipse_solid(box_at(d * 0.52, d * 0.28, Point::new(cir.center().x - d * 0.06, cir.y + d * 0.30)), Color(0x88ffffff));
            text_c(scene, cir, text, if lit { Color(0xff0f1113) } else { Color(0xffeaeae4) }, (cir.h * 0.42).min(13.0));
        }
        PushStyle::Chamfer => {
            let path = retro::chamfer(face.reduced(1.0, 1.0), face.h * 0.28, false);
            scene.fill_poly(retro::chamfer(r.expanded(1.5), r.h * 0.28, false), Color(0xff070809), Color(0xff070809));
            if !down {
                let sh: Vec<Point> = path.iter().map(|p| Point::new(p.x, p.y + 2.5)).collect();
                scene.fill_poly(sh, Color(0x88000000), Color(0x88000000));
            }
            let base = if lit { colour } else { Color(0xff33383e) };
            scene.fill_poly(path.clone(), base.brighter(if down { 0.05 } else { 0.25 }), base.darker(0.45));
            scene.stroke_poly(path, colour.with_alpha(if lit { 0.95 } else { 0.35 }), 1.2, true);
            text_c(scene, face, text, if lit { Color(0xff090b0d) } else { Color(0xffdbe0e2) }, (face.h * 0.42).min(13.0));
        }
        PushStyle::Neon => {
            let path = retro::chamfer(face.reduced(2.0, 2.0), face.h * 0.3, false);
            scene.fill_poly(path.clone(), Color(0xff030608), Color(0xff030608));
            if lit { scene.fill_poly(path.clone(), colour.with_alpha(0.20), colour.with_alpha(0.20)); }
            retro::glow_stroke(scene, &path, if lit { colour } else { colour.with_alpha(0.4) }, 1.3, if lit { 1.0 } else { 0.15 }, glow, true);
            let ink = if lit { colour.brighter(0.85) } else { colour.with_alpha(0.6) };
            text_c(scene, face, text, ink, (face.h * 0.42).min(13.0));
        }
        PushStyle::Flat => {
            solid_round(scene, r.expanded(1.5), 2.5, Color(0xff08090a));
            if !down { solid_round(scene, face.translated(0.0, 2.0), 2.0, Color(0x66000000)); }
            let base = if lit { colour } else { Color(0xff2b3033) };
            scene.fill_round(face, 2.0, base.brighter(if down { 0.05 } else { 0.20 }), base.darker(0.35), false);
            hline(scene, face.y + 0.8, face.x + 1.0, face.right() - 1.0, if lit { colour.brighter(0.4) } else { Color(0x35ffffff) }, 1.0);
            scene.stroke_round(face, 2.0, Color(0xff090a0b), 0.8);
            text_c(scene, face, text, if lit { Color(0xff0a0c0e) } else { Color(0xffb8bfc2) }, (face.h * 0.42).min(13.0));
        }
        PushStyle::Arcade => {
            let d = r.w.min(r.h);
            let cir = box_at(d, d, r.center());
            scene.fill_ellipse_solid(cir.expanded(2.0), Color(0xff0f1113));
            scene.fill_ellipse(cir, Color(0xff4a5056), Color(0xff090a0b));
            let cap = cir.reduced(d * 0.12, d * 0.12).translated(0.0, off);
            let base = if lit { colour } else { colour.darker(0.55) };
            if !down { scene.fill_ellipse_solid(cap.translated(0.0, 2.5), Color(0x88000000)); }
            scene.fill_ellipse(cap, base.brighter(if down { 0.15 } else { 0.55 }), base.darker(0.25));
            scene.fill_ellipse_solid(box_at(cap.w * 0.52, cap.h * 0.26, Point::new(cap.center().x, cap.y + cap.h * 0.25)), Color(0x77ffffff));
            text_c(scene, cap, text, Color(0xff0d0f11), (cap.h * 0.42).min(13.0));
        }
        PushStyle::Pill => {
            let rad = face.h * 0.5;
            solid_round(scene, r.expanded(2.0), rad + 2.0, Color(0xff08090a));
            scene.stroke_round(r.expanded(2.0), rad + 2.0, Color(0x28ffffff), 0.7);
            if !down { solid_round(scene, face.translated(0.0, 2.5), rad, Color(0x88000000)); }
            let base = if lit { colour } else { Color(0xffccd0ce) };
            scene.fill_round(face, rad, base.brighter(if down { 0.08 } else { 0.32 }), base.darker(if down { 0.40 } else { 0.28 }), true);
            hline(scene, face.y + 1.5, face.x + rad, face.right() - rad, Color(0x77ffffff), 1.0);
            scene.stroke_round(face, rad, Color(0x40000000), 0.8);
            text_c(scene, face, text, Color(0xff181c1e), (face.h * 0.42).min(13.0));
        }
    }
}

pub fn draw_key_bank(scene: &mut Scene, r: Rect, names: &[&str], selected: i32, style: PushStyle, colour: Color, glow: bool) {
    let n = names.len();
    if n == 0 { return; }
    let gap = 4.0;
    let w = (r.w - gap * (n - 1) as f32) / n as f32;
    for (i, name) in names.iter().enumerate() {
        draw_push(scene, Rect::new(r.x + i as f32 * (w + gap), r.y, w, r.h), name, style, colour, i as i32 == selected, false, glow);
    }
}
