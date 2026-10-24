// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Marcos Deida

use crate::color::Color;
use crate::scene::{Point, Rect, Scene};

fn lim(v: f32, lo: f32, hi: f32) -> f32 { v.max(lo).min(hi) }

fn box_at(w: f32, h: f32, c: Point) -> Rect {
    Rect::new(c.x - w * 0.5, c.y - h * 0.5, w, h)
}
fn ellipse_c(c: Point, rx: f32, ry: f32) -> Rect {
    Rect::new(c.x - rx, c.y - ry, rx * 2.0, ry * 2.0)
}
fn trim_right(r: Rect, d: f32) -> Rect { Rect::new(r.x, r.y, (r.w - d).max(0.0), r.h) }
fn trim_top(r: Rect, d: f32) -> Rect { Rect::new(r.x, r.y + d, r.w, (r.h - d).max(0.0)) }
fn with_y(r: Rect, y: f32) -> Rect { Rect::new(r.x, y, r.w, r.h) }
fn with_h(r: Rect, h: f32) -> Rect { Rect::new(r.x, r.y, r.w, h) }
fn remove_bottom(r: &mut Rect, h: f32) -> Rect {
    let h = h.min(r.h).max(0.0);
    r.h -= h;
    Rect::new(r.x, r.bottom(), r.w, h)
}
fn remove_top(r: &mut Rect, h: f32) -> Rect {
    let h = h.min(r.h).max(0.0);
    let out = Rect::new(r.x, r.y, r.w, h);
    r.y += h;
    r.h -= h;
    out
}
fn text_c(scene: &mut Scene, r: Rect, text: &str, color: Color, size: f32) {
    scene.text(r.center(), text, color, size, true);
}
fn hline(scene: &mut Scene, y: f32, x0: f32, x1: f32, color: Color, w: f32) {
    scene.line(Point::new(x0, y), Point::new(x1, y), color, w);
}
fn vline(scene: &mut Scene, x: f32, y0: f32, y1: f32, color: Color, w: f32) {
    scene.line(Point::new(x, y0), Point::new(x, y1), color, w);
}

pub fn vu_position(amplitude_over_reference: f32) -> f32 {
    lim(amplitude_over_reference * 0.7079, 0.0, 1.0)
}
pub fn vu_position_for_db(vu: f32) -> f32 {
    vu_position(10f32.powf(vu / 20.0))
}
pub fn db_position(db: f32, floor_db: f32, ceil_db: f32) -> f32 {
    lim((db - floor_db) / (ceil_db - floor_db), 0.0, 1.0)
}
pub fn ladder_colour(db: f32) -> Color {
    if db > -3.0 { Color(0xffe5382a) } else if db > -12.0 { Color(0xfff0b323) } else { Color(0xff3ccf5a) }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VuFace { Cream, Black }

fn juce_arc(scene: &mut Scene, c: Point, rad: f32, from: f32, to: f32, color: Color, width: f32) {
    let k = -std::f32::consts::FRAC_PI_2;
    scene.stroke_arc(c, rad, from + k, to + k, color, width);
}

fn quad(p0: Point, ctrl: Point, p1: Point, n: i32) -> Vec<Point> {
    let mut pts = Vec::with_capacity(n as usize + 1);
    for i in 0..=n {
        let t = i as f32 / n as f32;
        let u = 1.0 - t;
        pts.push(Point::new(
            u * u * p0.x + 2.0 * u * t * ctrl.x + t * t * p1.x,
            u * u * p0.y + 2.0 * u * t * ctrl.y + t * t * p1.y,
        ));
    }
    pts
}

fn glass(f: Rect, h: f32, cx: f32) -> Vec<Point> {
    let y1 = f.y + h * 0.36;
    let mut pts = vec![Point::new(f.x, f.y), Point::new(f.right(), f.y), Point::new(f.right(), y1)];
    pts.extend(quad(Point::new(f.right(), y1), Point::new(cx, f.y + h * 0.50), Point::new(f.x, y1), 8).into_iter().skip(1));
    pts
}

pub fn draw_vu_meter(scene: &mut Scene, r: Rect, needle: f32, face: VuFace) {
    let cream = face == VuFace::Cream;
    scene.fill_round(r, 4.0, Color(0xff08090a), Color(0xff08090a), true);
    hline(scene, r.bottom() - 1.0, r.x + 2.0, r.right() - 2.0, Color(0x35ffffff), 1.0);
    let mut f = r.reduced(3.5, 3.5);
    let (top, bot) = if cream { (Color(0xfff8eece), Color(0xffd4c07e)) } else { (Color(0xff2d261a), Color(0xff0a0907)) };
    scene.fill_round(f, 2.5, top, bot, true);
    scene.clip_round(f, 2.5);
    scene.fill_rect(with_h(f, 9.0), Color(0x55000000), Color(0x00000000), true);
    let h = f.h;
    let cx = f.center().x;
    let radius = h * 1.55;
    let arc_top = f.y + h * 0.30;
    let py = arc_top + radius;
    let span = 0.62f32.min((f.w * 0.43) / radius);
    let angle_for = |pos: f32| -span + 2.0 * span * pos;
    let pt = |ang: f32, rad: f32| Point::new(cx + ang.sin() * rad, py - ang.cos() * rad);
    let ink = if cream { Color(0xff1c1812) } else { Color(0xffffb347) };
    let red = if cream { Color(0xffc0281c) } else { Color(0xffff6a3a) };
    let pivot = Point::new(cx, py);
    juce_arc(scene, pivot, radius, angle_for(0.0), angle_for(vu_position_for_db(0.0)), ink, h * 0.025);
    juce_arc(scene, pivot, radius, angle_for(vu_position_for_db(0.0)), angle_for(1.0), red, h * 0.07);
    const MARKS: [(f32, &str, bool); 11] = [
        (-20.0, "20", true), (-10.0, "10", true), (-7.0, "7", true), (-5.0, "5", true), (-3.0, "3", true),
        (-2.0, "2", false), (-1.0, "1", false), (0.0, "0", true), (1.0, "1", false), (2.0, "2", false), (3.0, "3", true),
    ];
    for m in MARKS {
        let a = angle_for(vu_position_for_db(m.0));
        let c = if m.0 > 0.0 { red } else { ink };
        scene.line(pt(a, radius), pt(a, radius - h * if m.2 { 0.11 } else { 0.07 }), c, if m.2 { 1.4 } else { 1.0 });
        text_c(scene, box_at(h * 0.4, h * 0.2, pt(a, radius + h * 0.12)), m.1, c, h * 0.15);
    }
    let label = trim_right(remove_bottom(&mut f, h * 0.40), h * 0.15);
    text_c(scene, label, "VU", ink, h * 0.30);
    let na = angle_for(lim(needle, 0.0, 1.0));
    let sh = Point::new(1.8, 2.0);
    scene.line(pt(na, radius * 0.45).add(sh), pt(na, radius * 1.01).add(sh), Color(0x40000000), 1.8);
    scene.line(pt(na, radius * 0.45), pt(na, radius * 1.01), if cream { Color(0xff0e0c09) } else { Color(0xffff8e2b) }, 1.6);
    let hub = box_at(h * 0.16, h * 0.16, Point::new(cx, py - radius * 0.45));
    scene.fill_ellipse_solid(hub.translated(0.8, 1.4), Color(0x66000000));
    scene.fill_ellipse(hub, Color(0xff687074), Color(0xff121517));
    scene.stroke_ellipse(hub, Color(0x55ffffff), 0.7);
    let gpoly = glass(f, h, cx);
    scene.fill_poly(gpoly, Color(0x28ffffff), Color(0x00ffffff));
    scene.pop_clip();
    scene.stroke_round(r, 4.0, Color(0xff040506), 1.5);
}

pub fn draw_ppm(scene: &mut Scene, r: Rect, level01: f32) {
    scene.fill_round(r, 4.0, Color(0xff08090a), Color(0xff08090a), true);
    hline(scene, r.bottom() - 1.0, r.x + 2.0, r.right() - 2.0, Color(0x35ffffff), 1.0);
    let mut f = r.reduced(3.5, 3.5);
    scene.fill_round(f, 2.5, Color(0xfff4f4f0), Color(0xffcbcdc6), true);
    scene.clip_round(f, 2.5);
    scene.fill_rect(with_h(f, 9.0), Color(0x55000000), Color(0x00000000), true);
    let h = f.h;
    let cx = f.center().x;
    let radius = h * 1.55;
    let py = f.y + h * 0.30 + radius;
    let span = 0.62f32.min((f.w * 0.43) / radius);
    let ang = |p: f32| -span + 2.0 * span * p;
    let pt = |a: f32, rad: f32| Point::new(cx + a.sin() * rad, py - a.cos() * rad);
    juce_arc(scene, Point::new(cx, py), radius, ang(0.0), ang(1.0), Color(0xff1b1d1c), h * 0.025);
    for i in 1..=7 {
        let p = (i - 1) as f32 / 6.0;
        let a = ang(p);
        let c = if i == 7 { Color(0xffc0281c) } else { Color(0xff1b1d1c) };
        scene.line(pt(a, radius), pt(a, radius - h * 0.12), c, 1.5);
        text_c(scene, box_at(h * 0.3, h * 0.22, pt(a, radius + h * 0.14)), &i.to_string(), c, h * 0.19);
    }
    let label = trim_right(remove_bottom(&mut f, h * 0.38), h * 0.15);
    text_c(scene, label, "PPM", Color(0xff1b1d1c), h * 0.28);
    let na = ang(lim(level01, 0.0, 1.0));
    let sh = Point::new(1.8, 2.0);
    scene.line(pt(na, radius * 0.45).add(sh), pt(na, radius * 1.01).add(sh), Color(0x40000000), 1.8);
    scene.line(pt(na, radius * 0.45), pt(na, radius * 1.01), Color(0xff0d0d0b), 1.6);
    let hub = box_at(h * 0.16, h * 0.16, Point::new(cx, py - radius * 0.45));
    scene.fill_ellipse_solid(hub.translated(0.8, 1.4), Color(0x66000000));
    scene.fill_ellipse(hub, Color(0xff687074), Color(0xff121517));
    scene.stroke_ellipse(hub, Color(0x55ffffff), 0.7);
    scene.fill_poly(glass(f, h, cx), Color(0x28ffffff), Color(0x00ffffff));
    scene.pop_clip();
    scene.stroke_round(r, 4.0, Color(0xff040506), 1.5);
}

pub fn draw_led_ladder(scene: &mut Scene, r: Rect, horizontal: bool, level_db: f32, peak_db: f32, segments: i32) {
    let gap = 1.5;
    let span = if horizontal { r.w } else { r.h };
    let seg_len = (span - gap * (segments - 1) as f32) / segments as f32;
    let peak_idx = (db_position(peak_db, -48.0, 0.0) * (segments - 1) as f32 + 0.5).floor();
    for i in 0..segments {
        let db = -48.0 + 48.0 * (i + 1) as f32 / segments as f32;
        let lit = level_db >= db - 48.0 / segments as f32 * 0.5 || i as f32 == peak_idx;
        let c = ladder_colour(db);
        let s = if horizontal {
            Rect::new(r.x + i as f32 * (seg_len + gap), r.y, seg_len, r.h)
        } else {
            Rect::new(r.x, r.bottom() - (i + 1) as f32 * seg_len - i as f32 * gap, r.w, seg_len)
        };
        if lit { scene.fill_round(s.expanded(1.2), 2.0, c.with_alpha(0.28), c.with_alpha(0.28), true); }
        let body = if lit && (level_db >= db - 1.5 || i as f32 == peak_idx) { c } else if lit { c.darker(0.2) } else { c.with_alpha(0.14) };
        scene.fill_round(s, 1.2, body, body, true);
    }
}

pub fn draw_stereo_ladder(scene: &mut Scene, r: Rect, db_l: f32, db_r: f32, peak_db: f32) {
    scene.fill_round(r, 4.0, Color(0xff0b0c0d), Color(0xff0b0c0d), true);
    let b = r.reduced(10.0, 8.0);
    let bar_w = 30.0f32.min(b.w * 0.26);
    let lb = Rect::new(b.x + b.w * 0.5 - bar_w - 16.0, b.y, bar_w, b.h);
    let rb = Rect::new(b.x + b.w * 0.5 + 16.0, b.y, bar_w, b.h);
    draw_led_ladder(scene, lb, false, db_l, peak_db, 24);
    draw_led_ladder(scene, rb, false, db_r, peak_db, 24);
    let ink = Color(0xff8b9498);
    for d in [0, -6, -12, -24, -36, -48] {
        let y = b.bottom() - db_position(d as f32, -48.0, 0.0) * b.h;
        let yc = lim(y, b.y + 5.0, b.bottom() - 5.0);
        text_c(scene, box_at(30.0, 9.0, Point::new(b.center().x, yc)), &d.to_string(), ink, 7.5);
    }
    text_c(scene, with_h(with_y(lb, r.bottom() - 11.0), 10.0), "L", ink, 8.0);
    text_c(scene, with_h(with_y(rb, r.bottom() - 11.0), 10.0), "R", ink, 8.0);
    scene.stroke_round(r, 4.0, Color(0xff2a3033), 1.0);
}

pub fn draw_segment_arc(scene: &mut Scene, r: Rect, level01: f32, lit: Color) {
    let n = 32;
    let rad = r.h * 1.05;
    let half = ((r.w * 0.46) / rad).min(0.98).asin();
    let centre = Point::new(r.center().x, r.bottom() + rad * 0.18);
    for i in 0..n {
        let f = (i as f32 + 0.5) / n as f32;
        let a = (-1.0 + 2.0 * f) * half;
        let on = f <= level01 + 0.5 / n as f32;
        let col = if f > 0.86 { Color(0xffe4412c) } else { lit };
        let dirv = Point::new(a.sin(), -a.cos());
        let p0 = centre.add(dirv.mul(rad * 0.70));
        let p1 = centre.add(dirv.mul(rad * (0.82 + 0.16 * f)));
        let w = r.w / n as f32;
        if on { scene.line(p0, p1, col.with_alpha(0.25), w * 0.95); }
        scene.line(p0, p1, if on { col } else { col.with_alpha(0.13) }, w * 0.55);
    }
}

pub fn draw_vector_fan(scene: &mut Scene, r: Rect, level01: f32, lit: Color) {
    scene.fill_round(r, 4.0, Color(0xff031009), Color(0xff031009), true);
    let b = r.reduced(6.0, 6.0);
    let n = 26;
    for i in 0..n {
        let f = i as f32 / (n - 1) as f32;
        let x = b.x + f * b.w;
        let hgt = b.h * (0.28 + 0.72 * f);
        let on = f <= level01 + 0.5 / n as f32;
        if on { vline(scene, x, b.bottom(), b.bottom() - hgt, lit.with_alpha(0.25), 3.4); }
        vline(scene, x, b.bottom(), b.bottom() - hgt, if on { lit.brighter(0.2) } else { lit.with_alpha(0.14) }, 1.2);
    }
    scene.stroke_round(r, 4.0, Color(0x22ffffff), 1.0);
}

fn seg_line(s: char, x0: f32, x1: f32, y0: f32, y1: f32, ym: f32, th: f32) -> (Point, Point) {
    match s {
        'a' => (Point::new(x0 + th, y0), Point::new(x1 - th, y0)),
        'b' => (Point::new(x1, y0 + th), Point::new(x1, ym - th * 0.5)),
        'c' => (Point::new(x1, ym + th * 0.5), Point::new(x1, y1 - th)),
        'd' => (Point::new(x0 + th, y1), Point::new(x1 - th, y1)),
        'e' => (Point::new(x0, ym + th * 0.5), Point::new(x0, y1 - th)),
        'f' => (Point::new(x0, y0 + th), Point::new(x0, ym - th * 0.5)),
        _ => (Point::new(x0 + th, ym), Point::new(x1 - th, ym)),
    }
}

pub fn draw_seven_segment(scene: &mut Scene, r: Rect, text: &str, lit: Color, background: bool) {
    const SEGS: [&str; 10] = ["abcdef", "bc", "abged", "abgcd", "fgbc", "afgcd", "afgedc", "abc", "abcdefg", "abcdfg"];
    let mut cells = String::new();
    let mut dots = Vec::new();
    for ch in text.chars() {
        if ch == '.' && !dots.is_empty() { *dots.last_mut().unwrap() = true; continue; }
        cells.push(ch);
        dots.push(false);
    }
    let n = cells.chars().count();
    if n == 0 { return; }
    if background { scene.fill_round(r, 3.0, Color(0xff0b0f0c), Color(0xff0b0f0c), true); }
    let b = r.reduced(r.h * 0.14, r.h * 0.14);
    let cw = b.w / n as f32;
    let ch = b.h;
    let th = (cw * 0.16).min(ch * 0.11);
    for (i, c) in cells.chars().enumerate() {
        let x0 = b.x + i as f32 * cw + cw * 0.12;
        let x1 = b.x + (i + 1) as f32 * cw - cw * 0.12;
        let y0 = b.y;
        let y1 = b.bottom();
        let ym = (y0 + y1) * 0.5;
        let on = if c.is_ascii_digit() { SEGS[c as usize - '0' as usize] } else if c == '-' || c == '+' { "g" } else { "" };
        for s in ['a', 'b', 'c', 'd', 'e', 'f', 'g'] {
            let is_on = on.contains(s);
            if !is_on && !background { continue; }
            let (a, bpt) = seg_line(s, x0, x1, y0, y1, ym, th);
            if is_on { scene.line(a, bpt, lit.with_alpha(0.22), th * 2.4); }
            scene.line(a, bpt, if is_on { lit } else { lit.with_alpha(0.07) }, th);
        }
        if dots[i] {
            scene.fill_ellipse_solid(Rect::new(x1 + th * 0.2 - th, y1 - th, th * 2.0, th * 2.0), lit.with_alpha(0.22));
            scene.fill_ellipse_solid(Rect::new(x1 + th * 0.2 - th * 0.6, y1 - th * 0.6, th * 1.2, th * 1.2), lit);
        }
    }
}

pub fn draw_lamp(scene: &mut Scene, c: Point, radius: f32, colour: Color, on: bool) {
    scene.fill_ellipse_solid(ellipse_c(c, radius + 1.5, radius + 1.5), Color(0xff0a0a0a));
    scene.fill_ellipse(ellipse_c(c, radius + 1.0, radius + 1.0), Color(0xffb9bec0), Color(0xff3a3f42));
    if on { scene.fill_ellipse_solid(ellipse_c(c, radius * 2.0, radius * 2.0), colour.with_alpha(0.30)); }
    let top = if on { colour.brighter(0.7) } else { colour.darker(0.9) };
    let bot = if on { colour.darker(0.2) } else { colour.darker(1.4) };
    scene.fill_ellipse(ellipse_c(c, radius, radius), top, bot);
    scene.fill_ellipse_solid(ellipse_c(Point::new(c.x - radius * 0.15, c.y - radius * 0.525), radius * 0.4, radius * 0.225), Color(0x66ffffff));
}

pub fn draw_lit_key(scene: &mut Scene, r: Rect, text: &str, lit: bool, lamp: Color, down: bool) {
    scene.fill_round(r.expanded(1.5), 3.0, Color(0xff0e1112), Color(0xff0e1112), true);
    let face = r.translated(0.0, if down { 1.0 } else { 0.0 });
    scene.fill_round(face.translated(0.0, 1.5), 2.0, Color(0xff060808), Color(0xff060808), true);
    scene.fill_round(face, 2.0, Color(0xffdfe2d0), Color(0xff9da79e), false);
    let strip_h = 2.5f32.max(face.h * 0.16);
    let mut strip_host = face.reduced(face.w * 0.14, 0.0);
    let strip = remove_top(&mut strip_host, strip_h).translated(0.0, 2.0);
    if lit { scene.fill_round(strip.expanded(2.0), 3.0, lamp.with_alpha(0.35), lamp.with_alpha(0.35), true); }
    let sc = if lit { lamp } else { Color(0xff3a423d) };
    scene.fill_round(strip, 1.5, sc, sc, true);
    let text_r = trim_top(face, strip.bottom() - face.y);
    text_c(scene, text_r, text, Color(0xff26312e), face.h * 0.42);
    hline(scene, face.y + 1.0, face.x + 2.0, face.right() - 2.0, Color(0x55ffffff), 1.0);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FaderCap { Oval, Console }

pub fn draw_slot_fader(scene: &mut Scene, r: Rect, prop01: f32, cap: FaderCap, cap_colour: Color) {
    let slot_h = (r.h * 0.42).min(12.0);
    let slot = box_at(r.w, slot_h, r.center());
    scene.fill_round(slot, slot_h * 0.5, Color(0xff030405), Color(0xff030405), true);
    scene.stroke_round(slot, slot_h * 0.5, Color(0xff2a3236), 1.0);
    let ticks = Color(0xff39b6c8).with_alpha(0.55);
    for i in 0..=8 {
        let x = r.x + r.w * i as f32 / 8.0;
        let len = if i % 4 == 0 { 5.0 } else { 3.0 };
        scene.line(Point::new(x, slot.bottom() + 1.5), Point::new(x, slot.bottom() + len), ticks, 0.9);
    }
    let x = r.x + lim(prop01, 0.0, 1.0) * r.w;
    let cy = r.center().y;
    if cap == FaderCap::Oval {
        let o = box_at(slot_h * 2.1, slot_h * 1.05, Point::new(x, cy));
        scene.fill_round(o.translated(0.0, 1.5), o.h * 0.5, Color(0x66000000), Color(0x66000000), true);
        scene.fill_round(o, o.h * 0.5, cap_colour.brighter(0.3), cap_colour.darker(0.35), true);
        hline(scene, o.y + 1.5, o.x + o.h * 0.4, o.right() - o.h * 0.4, Color(0x55ffffff), 1.0);
    } else {
        let c = box_at(slot_h * 1.5, slot_h * 2.2, Point::new(x, cy));
        scene.fill_round(c.translated(1.0, 2.0), 2.0, Color(0x77000000), Color(0x77000000), true);
        scene.fill_round(c, 2.0, cap_colour.brighter(0.25), cap_colour.darker(0.4), false);
        scene.line(Point::new(x, c.y + 2.0), Point::new(x, c.bottom() - 2.0), Color(0xaaffffff), 1.4);
        scene.stroke_round(c, 2.0, Color(0x55000000), 0.8);
    }
}

