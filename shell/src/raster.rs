//! Software replay of [`goodlookindri::scene::Cmd`].
//!
//! Every shape is a signed-distance function; coverage is `0.5 - d` clamped to 0..1, which gives
//! one pixel of anti-aliasing with no supersampling. Slow for very large scenes, but it has no
//! dependencies and no requirement beyond the standard library.

use crate::font;
use goodlookindri::color::Color;
use goodlookindri::scene::{Cmd, Point, Scene};

/// Opaque `0x00RRGGBB` pixels, top row first.
pub struct Framebuffer {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<u32>,
}

#[derive(Clone, Copy)]
struct Clip { cx: f32, cy: f32, hx: f32, hy: f32, r: f32 }

enum Fill {
    Solid(Color),
    Grad { top: Color, bottom: Color, vertical: bool, bx: f32, by: f32, inv_v: f32, inv_d: f32 },
}

impl Fill {
    fn grad(top: Color, bottom: Color, vertical: bool, bx: f32, by: f32, bw: f32, bh: f32) -> Fill {
        if top == bottom { return Fill::Solid(top); }
        Fill::Grad { top, bottom, vertical, bx, by, inv_v: 1.0 / bh.max(1e-3), inv_d: 1.0 / (bw + bh).max(1e-3) }
    }
    fn at(&self, x: f32, y: f32) -> Color {
        match self {
            Fill::Solid(c) => *c,
            Fill::Grad { top, bottom, vertical, bx, by, inv_v, inv_d } => {
                let t = if *vertical { (y - by) * inv_v } else { ((x - bx) + (y - by)) * inv_d };
                top.mix(*bottom, t)
            }
        }
    }
}

fn sd_round(px: f32, py: f32, cx: f32, cy: f32, hx: f32, hy: f32, r: f32) -> f32 {
    let r = r.min(hx).min(hy).max(0.0);
    let qx = (px - cx).abs() - (hx - r);
    let qy = (py - cy).abs() - (hy - r);
    let (ox, oy) = (qx.max(0.0), qy.max(0.0));
    (ox * ox + oy * oy).sqrt() + qx.max(qy).min(0.0) - r
}

fn sd_ellipse(px: f32, py: f32, cx: f32, cy: f32, a: f32, b: f32) -> f32 {
    let (a, b) = (a.max(1e-3), b.max(1e-3));
    let (dx, dy) = ((px - cx) / a, (py - cy) / b);
    ((dx * dx + dy * dy).sqrt() - 1.0) * a.min(b)
}

fn seg_dist(px: f32, py: f32, a: Point, b: Point) -> f32 {
    let (vx, vy) = (b.x - a.x, b.y - a.y);
    let (wx, wy) = (px - a.x, py - a.y);
    let l2 = vx * vx + vy * vy;
    let t = if l2 > 0.0 { ((wx * vx + wy * vy) / l2).clamp(0.0, 1.0) } else { 0.0 };
    let (dx, dy) = (wx - vx * t, wy - vy * t);
    (dx * dx + dy * dy).sqrt()
}

fn poly_dist(pts: &[Point], closed: bool, px: f32, py: f32) -> f32 {
    let n = pts.len();
    let mut d = f32::MAX;
    let last = if closed { n } else { n.saturating_sub(1) };
    for i in 0..last { d = d.min(seg_dist(px, py, pts[i], pts[(i + 1) % n])); }
    d
}

fn poly_signed(pts: &[Point], px: f32, py: f32) -> f32 {
    let n = pts.len();
    let mut inside = false;
    let mut j = n - 1;
    for i in 0..n {
        let (a, b) = (pts[i], pts[j]);
        if (a.y > py) != (b.y > py) && px < (b.x - a.x) * (py - a.y) / (b.y - a.y) + a.x { inside = !inside; }
        j = i;
    }
    let d = poly_dist(pts, true, px, py);
    if inside { -d } else { d }
}

fn bounds(pts: &[Point]) -> [f32; 4] {
    let mut b = [f32::MAX, f32::MAX, f32::MIN, f32::MIN];
    for p in pts { b = [b[0].min(p.x), b[1].min(p.y), b[2].max(p.x), b[3].max(p.y)]; }
    b
}

impl Framebuffer {
    pub fn new(width: usize, height: usize) -> Self {
        Self { width, height, pixels: vec![0; width * height] }
    }

    pub fn clear(&mut self, c: Color) {
        let v = c.0 & 0x00ff_ffff;
        for p in self.pixels.iter_mut() { *p = v; }
    }

    /// PPM (P6) bytes, for tests and CI artefacts.
    pub fn to_ppm(&self) -> Vec<u8> {
        let mut out = format!("P6\n{} {}\n255\n", self.width, self.height).into_bytes();
        for p in &self.pixels { out.extend_from_slice(&[(p >> 16) as u8, (p >> 8) as u8, *p as u8]); }
        out
    }

    /// Paints `scene`. Scene units are multiplied by `scale` (the backing scale of the display).
    pub fn replay(&mut self, scene: &Scene, scale: f32) {
        let s = scale;
        let mut clips: Vec<Clip> = Vec::new();
        for cmd in &scene.cmds {
            match cmd {
                Cmd::FillEllipse { rect, top, bottom, diagonal } => {
                    let (x, y, w, h) = (rect.x * s, rect.y * s, rect.w * s, rect.h * s);
                    let fill = Fill::grad(*top, *bottom, !*diagonal, x, y, w, h);
                    let (cx, cy, a, b) = (x + w * 0.5, y + h * 0.5, w * 0.5, h * 0.5);
                    self.draw(&clips, [x, y, x + w, y + h], |px, py| sd_ellipse(px, py, cx, cy, a, b), &fill);
                }
                Cmd::StrokeEllipse { rect, color, width } => {
                    let (x, y, w, h) = (rect.x * s, rect.y * s, rect.w * s, rect.h * s);
                    let hw = (width * s).max(1.0) * 0.5;
                    let (cx, cy, a, b) = (x + w * 0.5, y + h * 0.5, w * 0.5, h * 0.5);
                    self.draw(&clips, [x - hw, y - hw, x + w + hw, y + h + hw], |px, py| sd_ellipse(px, py, cx, cy, a, b).abs() - hw, &Fill::Solid(*color));
                }
                Cmd::FillRoundRect { rect, radius, top, bottom, vertical } => {
                    let (x, y, w, h) = (rect.x * s, rect.y * s, rect.w * s, rect.h * s);
                    let fill = Fill::grad(*top, *bottom, *vertical, x, y, w, h);
                    let (cx, cy, hx, hy, r) = (x + w * 0.5, y + h * 0.5, w * 0.5, h * 0.5, radius * s);
                    self.draw(&clips, [x, y, x + w, y + h], |px, py| sd_round(px, py, cx, cy, hx, hy, r), &fill);
                }
                Cmd::FillRect { rect, top, bottom, vertical } => {
                    let (x, y, w, h) = (rect.x * s, rect.y * s, rect.w * s, rect.h * s);
                    let fill = Fill::grad(*top, *bottom, *vertical, x, y, w, h);
                    let (cx, cy, hx, hy) = (x + w * 0.5, y + h * 0.5, w * 0.5, h * 0.5);
                    self.draw(&clips, [x, y, x + w, y + h], |px, py| sd_round(px, py, cx, cy, hx, hy, 0.0), &fill);
                }
                Cmd::StrokeRoundRect { rect, radius, color, width } => {
                    let (x, y, w, h) = (rect.x * s, rect.y * s, rect.w * s, rect.h * s);
                    let hw = (width * s).max(1.0) * 0.5;
                    let (cx, cy, hx, hy, r) = (x + w * 0.5, y + h * 0.5, w * 0.5, h * 0.5, radius * s);
                    self.draw(&clips, [x - hw, y - hw, x + w + hw, y + h + hw], |px, py| sd_round(px, py, cx, cy, hx, hy, r).abs() - hw, &Fill::Solid(*color));
                }
                Cmd::Line { a, b, color, width } => {
                    let (a, b) = (a.mul(s), b.mul(s));
                    let hw = (width * s).max(1.0) * 0.5;
                    let bb = bounds(&[a, b]);
                    self.draw(&clips, [bb[0] - hw, bb[1] - hw, bb[2] + hw, bb[3] + hw], |px, py| seg_dist(px, py, a, b) - hw, &Fill::Solid(*color));
                }
                Cmd::FillPoly { pts, top, bottom, diagonal } => {
                    if pts.len() < 3 { continue; }
                    let pts: Vec<Point> = pts.iter().map(|p| p.mul(s)).collect();
                    let bb = bounds(&pts);
                    let fill = Fill::grad(*top, *bottom, !*diagonal, bb[0], bb[1], bb[2] - bb[0], bb[3] - bb[1]);
                    self.draw(&clips, bb, |px, py| poly_signed(&pts, px, py), &fill);
                }
                Cmd::StrokePoly { pts, color, width, closed } => {
                    if pts.len() < 2 { continue; }
                    let pts: Vec<Point> = pts.iter().map(|p| p.mul(s)).collect();
                    let hw = (width * s).max(1.0) * 0.5;
                    let bb = bounds(&pts);
                    self.draw(&clips, [bb[0] - hw, bb[1] - hw, bb[2] + hw, bb[3] + hw], |px, py| poly_dist(&pts, *closed, px, py) - hw, &Fill::Solid(*color));
                }
                Cmd::StrokeArc { center, radius, from, to, color, width } => {
                    let (c, r) = (center.mul(s), radius * s);
                    let steps = (((to - from).abs() * r / 2.0).ceil() as usize).clamp(4, 512);
                    let pts: Vec<Point> = (0..=steps)
                        .map(|i| {
                            let a = from + (to - from) * i as f32 / steps as f32;
                            Point::new(c.x + r * a.cos(), c.y + r * a.sin())
                        })
                        .collect();
                    let hw = (width * s).max(1.0) * 0.5;
                    let bb = bounds(&pts);
                    self.draw(&clips, [bb[0] - hw, bb[1] - hw, bb[2] + hw, bb[3] + hw], |px, py| poly_dist(&pts, false, px, py) - hw, &Fill::Solid(*color));
                }
                Cmd::Text { at, text, color, size, center } => {
                    let cell = (size * s / font::GLYPH_ROWS as f32).max(0.5);
                    let width = font::width_cells(text) as f32 * cell;
                    let x0 = if *center { at.x * s - width * 0.5 } else { at.x * s };
                    let y0 = at.y * s - font::GLYPH_ROWS as f32 * cell * 0.5;
                    let fill = Fill::Solid(*color);
                    for (i, ch) in text.chars().enumerate() {
                        let gx = x0 + (i * (font::GLYPH_W + 1)) as f32 * cell;
                        for (col, bits) in font::glyph(ch).iter().enumerate() {
                            for row in 0..font::GLYPH_ROWS {
                                if bits >> row & 1 == 0 { continue; }
                                let (bx, by) = (gx + col as f32 * cell, y0 + row as f32 * cell);
                                let (cx, cy, h) = (bx + cell * 0.5, by + cell * 0.5, cell * 0.5);
                                self.draw(&clips, [bx, by, bx + cell, by + cell], |px, py| sd_round(px, py, cx, cy, h, h, 0.0), &fill);
                            }
                        }
                    }
                }
                Cmd::ClipRound { rect, radius } => {
                    clips.push(Clip { cx: (rect.x + rect.w * 0.5) * s, cy: (rect.y + rect.h * 0.5) * s, hx: rect.w * 0.5 * s, hy: rect.h * 0.5 * s, r: radius * s });
                }
                Cmd::PopClip => { clips.pop(); }
            }
        }
    }

    fn draw<D: Fn(f32, f32) -> f32>(&mut self, clips: &[Clip], bb: [f32; 4], dist: D, fill: &Fill) {
        let x0 = ((bb[0].floor() as i64) - 1).max(0) as usize;
        let y0 = ((bb[1].floor() as i64) - 1).max(0) as usize;
        let x1 = (((bb[2].ceil() as i64) + 1).max(0) as usize).min(self.width);
        let y1 = (((bb[3].ceil() as i64) + 1).max(0) as usize).min(self.height);
        for y in y0..y1 {
            let py = y as f32 + 0.5;
            for x in x0..x1 {
                let px = x as f32 + 0.5;
                let mut cov = (0.5 - dist(px, py)).clamp(0.0, 1.0);
                if cov <= 0.0 { continue; }
                for c in clips {
                    cov *= (0.5 - sd_round(px, py, c.cx, c.cy, c.hx, c.hy, c.r)).clamp(0.0, 1.0);
                    if cov <= 0.0 { break; }
                }
                if cov <= 0.0 { continue; }
                let col = fill.at(px, py);
                let a = col.alpha_f() * cov;
                if a <= 0.0 { continue; }
                let dst = &mut self.pixels[y * self.width + x];
                let mix = |s: u8, d: u32| (d as f32 + (s as f32 - d as f32) * a).round() as u32;
                let (dr, dg, db) = ((*dst >> 16) & 255, (*dst >> 8) & 255, *dst & 255);
                *dst = (mix(col.r(), dr) << 16) | (mix(col.g(), dg) << 8) | mix(col.b(), db);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use goodlookindri::scene::Rect;

    #[test]
    fn fills_and_clips() {
        let mut sc = Scene::new();
        sc.fill_rect(Rect::new(2.0, 2.0, 4.0, 4.0), Color(0xffff0000), Color(0xffff0000), true);
        let mut fb = Framebuffer::new(8, 8);
        fb.replay(&sc, 1.0);
        assert_eq!(fb.pixels[3 * 8 + 3], 0x00ff0000);
        assert_eq!(fb.pixels[0], 0);
        let mut sc = Scene::new();
        sc.clip_round(Rect::new(0.0, 0.0, 3.0, 8.0), 0.0);
        sc.fill_rect(Rect::new(0.0, 0.0, 8.0, 8.0), Color(0xff00ff00), Color(0xff00ff00), true);
        sc.pop_clip();
        let mut fb = Framebuffer::new(8, 8);
        fb.replay(&sc, 1.0);
        assert_eq!(fb.pixels[4], 0);
        assert_eq!(fb.pixels[1], 0x0000ff00);
    }

    #[test]
    fn scale_doubles_geometry() {
        let mut sc = Scene::new();
        sc.fill_rect(Rect::new(1.0, 1.0, 2.0, 2.0), Color(0xffffffff), Color(0xffffffff), true);
        let mut fb = Framebuffer::new(8, 8);
        fb.replay(&sc, 2.0);
        assert_eq!(fb.pixels[2 * 8 + 2], 0x00ffffff);
        assert_eq!(fb.pixels[5 * 8 + 5], 0x00ffffff);
        assert_eq!(fb.pixels[6 * 8 + 6], 0);
    }

    #[test]
    fn every_knob_style_draws_pixels() {
        for flat in [false, true] {
            let mut sc = Scene::new();
            goodlookindri::gallery::paint_page(&mut sc, 0, flat, false);
            let mut fb = Framebuffer::new(700, 500);
            fb.clear(Color(0xff202020));
            fb.replay(&sc, 1.0);
            assert!(fb.pixels.iter().any(|p| *p != 0x00202020));
        }
    }
}
