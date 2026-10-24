// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Marcos Deida

use crate::color::Color;
use crate::scene::{Point, Rect, Scene};

fn hline(scene: &mut Scene, y: f32, x0: f32, x1: f32, color: Color, w: f32) {
    scene.line(Point::new(x0, y), Point::new(x1, y), color, w);
}
fn vline(scene: &mut Scene, x: f32, y0: f32, y1: f32, color: Color, w: f32) {
    scene.line(Point::new(x, y0), Point::new(x, y1), color, w);
}
fn solid_round(scene: &mut Scene, r: Rect, radius: f32, c: Color) {
    scene.fill_round(r, radius, c, c, true);
}
fn solid_ellipse(scene: &mut Scene, r: Rect, c: Color) { scene.fill_ellipse_solid(r, c); }

pub fn draw_screw(scene: &mut Scene, x: f32, y: f32) {
    let r = Rect::new(x - 4.25, y - 4.25, 8.5, 8.5);
    solid_ellipse(scene, r.expanded(1.2), Color(0xff090c0e));
    scene.stroke_ellipse(r.expanded(1.2), Color(0x28ffffff), 0.7);
    scene.fill_ellipse(r, Color(0xff9aa4a8), Color(0xff22272a));
    scene.stroke_ellipse(r, Color(0x55ffffff), 0.6);
    scene.line(Point::new(x - 2.4, y + 1.2), Point::new(x + 2.4, y - 1.2), Color(0xff080b0c), 1.4);
    scene.line(Point::new(x - 2.4, y + 0.5), Point::new(x + 2.4, y - 1.9), Color(0x44ffffff), 0.7);
}

pub fn draw_finish(scene: &mut Scene, r: Rect, finish: i32) {
    if finish <= 0 { return; }
    scene.clip_round(r, 2.0);
    let (x0, x1, y0, y1) = (r.x, r.right(), r.y, r.bottom());
    match finish {
        1 => {
            let y_a = y0.trunc() as i32;
            let y_b = y1.trunc() as i32;
            for y in y_a..y_b {
                let h = (y as u32).wrapping_mul(2654435761);
                let c = if (h >> 7) & 1 == 1 { Color(0x0dffffff) } else { Color(0x0d000000) };
                hline(scene, y as f32, x0 + (h % 9) as f32, x1 - ((h >> 5) % 13) as f32, c, 1.0);
            }
        }
        2 => {
            let mut y = y0 + 3.0;
            while y < y1 {
                let mut pts = Vec::new();
                let mut x = x0;
                while x <= x1 {
                    let yy = y + (x * 0.045 + y * 0.31).sin() * 2.2;
                    pts.push(Point::new(x, yy));
                    x += 6.0;
                }
                scene.stroke_poly(pts, Color(0x26000000), 0.8, false);
                y += 5.0;
            }
        }
        3 => {
            let mut y = y0;
            while y < y1 {
                hline(scene, y.trunc(), x0, x1, Color(0x1c000000), 1.0);
                y += 3.0;
            }
            scene.fill_rect(Rect::new(x0, y0, x1 - x0, 60.0), Color(0x14ffffff), Color(0x00ffffff), true);
        }
        4 => {
            for y_band in [y0 + 1.0, y1 - 7.0] {
                let mut x = x0 - 8.0;
                while x < x1 {
                    scene.fill_poly(vec![
                        Point::new(x, y_band + 6.0), Point::new(x + 6.0, y_band + 6.0),
                        Point::new(x + 12.0, y_band), Point::new(x + 6.0, y_band),
                    ], Color(0xffe8c820), Color(0xffe8c820));
                    x += 12.0;
                }
            }
            scene.fill_rect(Rect::new(x0, y0 + 1.0, x1 - x0, 6.0), Color(0x55000000), Color(0x55000000), true);
        }
        6 => {
            for k in (1..=5).rev() {
                let a = 0.035 * (6 - k) as f32;
                scene.stroke_round(r.reduced(1.0 + k as f32 * 0.7, 1.0 + k as f32 * 0.7), 2.0, Color(0xff39f2ff).with_alpha(a), 1.6);
            }
            scene.stroke_round(r.reduced(2.0, 2.0), 2.0, Color(0xcc39f2ff), 1.0);
            scene.line(Point::new(x0 + 2.0, y0 + 2.0), Point::new(x0 + 22.0, y0 + 2.0), Color(0xffff4fd8), 2.0);
            scene.line(Point::new(x1 - 22.0, y1 - 2.0), Point::new(x1 - 2.0, y1 - 2.0), Color(0xffff4fd8), 2.0);
        }
        7 => {
            let hy = y0 + (y1 - y0) * 0.58;
            scene.fill_rect(Rect::new(x0, y0, x1 - x0, hy - y0), Color(0x00ff4fd8), Color(0x55ff4fd8), true);
            hline(scene, hy.trunc(), x0, x1, Color(0x8835f2ff), 1.0);
            for i in -8..=8 {
                scene.line(
                    Point::new(x0 + (x1 - x0) * 0.5 + i as f32 * 3.0, hy),
                    Point::new(x0 + (x1 - x0) * 0.5 + i as f32 * (x1 - x0) * 0.16, y1),
                    Color(0x8835f2ff), 0.7,
                );
            }
            for j in 1..=6 {
                let y = hy + ((j as f32) / 6.0).powi(2) * (y1 - hy);
                hline(scene, y.trunc(), x0, x1, Color(0x8835f2ff), 1.0);
            }
        }
        _ => {
            let mut x = x0 + 14.0;
            while x < x1 {
                vline(scene, x.trunc(), y0, y1, Color(0x12ffffff), 1.0);
                x += 14.0;
            }
            let mut y = y0 + 14.0;
            while y < y1 {
                hline(scene, y.trunc(), x0, x1, Color(0x12ffffff), 1.0);
                y += 14.0;
            }
            for c in [Point::new(x0, y0), Point::new(x1, y0), Point::new(x0, y1), Point::new(x1, y1)] {
                let sx = if c.x == x0 { 1.0 } else { -1.0 };
                let sy = if c.y == y0 { 1.0 } else { -1.0 };
                scene.line(Point::new(c.x, c.y + sy * 10.0), c, Color(0x66ffffff), 1.2);
                scene.line(c, Point::new(c.x + sx * 10.0, c.y), Color(0x66ffffff), 1.2);
            }
        }
    }
    scene.pop_clip();
}

pub fn draw_panel(scene: &mut Scene, r: Rect, top: Color, bottom: Color, finish: i32) {
    solid_round(scene, r.translated(1.0, 2.5), 3.5, Color(0x66000000));
    scene.fill_round(r, 2.5, top, bottom, true);
    draw_finish(scene, r, finish);
    hline(scene, (r.y + 0.5).trunc(), r.x + 2.0, r.right() - 2.0, Color(0x40ffffff), 1.0);
    vline(scene, (r.x + 0.5).trunc(), r.y + 2.0, r.bottom() - 2.0, Color(0x25ffffff), 1.0);
    hline(scene, (r.bottom() - 1.0).trunc(), r.x + 2.0, r.right() - 2.0, Color(0x80000000), 1.0);
    vline(scene, (r.right() - 1.0).trunc(), r.y + 2.0, r.bottom() - 2.0, Color(0x60000000), 1.0);
    scene.stroke_round(r, 2.5, Color(0xff0c0f11), 1.0);
}

pub fn draw_key(scene: &mut Scene, r: Rect, base: Color, down: bool) {
    solid_round(scene, r.expanded(2.5), 3.5, Color(0xff060809));
    scene.stroke_round(r.expanded(2.5), 3.5, Color(0x35ffffff), 0.7);
    let face = r.translated(0.0, if down { 2.0 } else { 0.0 });
    if !down { solid_round(scene, face.translated(0.0, 2.5), 2.0, Color(0x99000000)); }
    let hi = base.brighter(if down { 0.05 } else { 0.28 });
    let lo = base.darker(if down { 0.45 } else { 0.22 });
    scene.fill_round(face, 2.0, hi, lo, false);
    hline(scene, (face.y + 0.8).trunc(), face.x + 2.0, face.right() - 2.0, Color(0x77ffffff), 1.0);
    hline(scene, (face.bottom() - 0.8).trunc(), face.x + 2.0, face.right() - 2.0, Color(0x55000000), 1.0);
    scene.stroke_round(face, 2.0, Color(0x44000000), 0.8);
}

pub fn use_dark_marks(marks: &str, backdrop: Color) -> bool {
    if marks == "dark" { return true; }
    if marks == "light" { return false; }
    backdrop.a() > 0 && backdrop.perceived_brightness() > 0.55
}

fn hash_u(mut x: u32) -> u32 {
    x ^= x >> 16;
    x = x.wrapping_mul(0x7feb352d);
    x ^= x >> 15;
    x = x.wrapping_mul(0x846ca68b);
    x ^= x >> 16;
    x
}

pub fn draw_brushed_panel(scene: &mut Scene, r: Rect, top: Color, bottom: Color, seed: u32) {
    scene.fill_rect(r, top, bottom, true);
    scene.clip_round(r, 0.0);
    let y0 = r.y.trunc() as i32;
    let y1 = r.bottom().trunc() as i32;
    for y in y0..y1 {
        let h = hash_u((y as u32).wrapping_mul(2654435761).wrapping_add(seed));
        let a = (h & 255) as f32 / 255.0;
        let c = if h & 256 != 0 {
            Color::rgba(255, 255, 255, 255).with_alpha(0.008 + 0.02 * a)
        } else {
            Color::rgba(0, 0, 0, 255).with_alpha(0.03 + 0.06 * a)
        };
        let x0 = r.x + ((h >> 9) & 511) as f32 / 511.0 * r.w * 0.4;
        let x1 = x0 + r.w * (0.3 + 0.7 * ((h >> 18) & 255) as f32 / 255.0);
        hline(scene, y as f32, x0 - r.w * 0.2, x1, c, 1.0);
    }
    scene.fill_rect(Rect::new(r.x, r.y, r.w, r.h * 0.18), Color(0x14ffffff), Color(0x00ffffff), true);
    hline(scene, r.y.trunc(), r.x, r.right(), Color(0x40ffffff), 1.0);
    hline(scene, r.bottom().trunc() - 1.0, r.x, r.right(), Color(0xa0000000), 1.0);
    scene.pop_clip();
}

pub fn draw_rack_ear(scene: &mut Scene, r: Rect, left_side: bool) {
    draw_brushed_panel(scene, r, Color(0xff2d2e31), Color(0xff141516), if left_side { 11 } else { 12 });
    let x = if left_side { r.right() - 1.0 } else { r.x };
    vline(scene, x.trunc(), r.y, r.bottom(), Color(0xff050506), 1.0);
    let x2 = if left_side { r.right() - 2.0 } else { r.x + 1.0 };
    vline(scene, x2.trunc(), r.y + 1.0, r.bottom() - 1.0, Color(0x22ffffff), 1.0);
    let sw = (r.w * 0.55).min(18.0);
    let sh = (r.h * 0.26).min(26.0);
    let slot = Rect::new(r.center().x - sw * 0.5, r.center().y - sh * 0.5, sw, sh);
    solid_round(scene, slot.expanded(1.5), sw * 0.5, Color(0xff000000));
    scene.fill_round(slot, sw * 0.5, Color(0xff020203), Color(0xff17181a), true);
    scene.stroke_round(slot.expanded(1.5), sw * 0.5, Color(0x40ffffff), 0.8);
}

pub fn draw_led_key(scene: &mut Scene, r: Rect, led: Color, lit: bool, down: bool) {
    solid_round(scene, r.expanded(1.6), 2.6, Color(0xff030303));
    scene.stroke_round(r.expanded(1.6), 2.6, Color(0x30ffffff), 0.7);
    let cap = r.reduced(0.6, 0.6).translated(0.0, if down { 0.8 } else { 0.0 });
    if lit {
        solid_round(scene, cap.expanded(4.0), 5.0, led.with_alpha(0.22));
        solid_round(scene, cap.expanded(2.0), 3.5, led.with_alpha(0.20));
    }
    let hi = if lit { led.brighter(0.5) } else { led.darker(0.78) };
    let lo = if lit { led.darker(0.1) } else { led.darker(0.95) };
    scene.fill_round(cap, 2.0, hi, lo, true);
    let sheen = cap.reduced(1.5, 1.5);
    solid_round(scene, Rect::new(sheen.x, sheen.y, sheen.w, cap.h * 0.38), 1.5, Color(if lit { 0x55ffffff } else { 0x22ffffff }));
    scene.stroke_round(cap, 2.0, Color(0x66000000), 0.8);
}

pub fn draw_legend_line(scene: &mut Scene, ink: Color, x0: f32, x1: f32, y: f32, title: &str, font_size: f32) {
    let tw = font_size * 0.55 * title.chars().count() as f32 + 10.0;
    let cx = 0.5 * (x0 + x1);
    scene.fill_rect(Rect::new(x0, y, cx - tw * 0.5 - x0, 1.0), ink, ink, true);
    scene.fill_rect(Rect::new(cx + tw * 0.5, y, x1 - cx - tw * 0.5, 1.0), ink, ink, true);
    scene.text(Point::new(cx, y), title, ink, font_size, true);
}
