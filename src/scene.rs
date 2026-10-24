//! Host-owned drawing list. GoodLookinDri never opens a window or touches audio parameters.
//! A plugin (Water Channel, an EQ, nih-plug, iced, egui, Cairo) replays [`Cmd`] on its GUI thread.
//!
//! Water Channel's C++ canvas only had five primitives and rejected every style except Brit.
//! This list is the same idea with the operations the full toolkit actually paints:
//! ellipses, round rects, lines, arcs, filled polygons, stroked polylines, and text runs.
//! Text is a run of glyphs the host draws with its own font; the toolkit does not bundle one.

use crate::color::Color;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Point { pub x: f32, pub y: f32 }
impl Point {
    pub fn new(x: f32, y: f32) -> Self { Self { x, y } }
    pub fn add(self, o: Point) -> Self { Self { x: self.x + o.x, y: self.y + o.y } }
    pub fn mul(self, s: f32) -> Self { Self { x: self.x * s, y: self.y * s } }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect { pub x: f32, pub y: f32, pub w: f32, pub h: f32 }
impl Rect {
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Self { Self { x, y, w, h } }
    pub fn right(self) -> f32 { self.x + self.w }
    pub fn bottom(self) -> f32 { self.y + self.h }
    pub fn center(self) -> Point { Point::new(self.x + self.w * 0.5, self.y + self.h * 0.5) }
    pub fn contains(self, p: Point) -> bool {
        p.x >= self.x && p.y >= self.y && p.x < self.right() && p.y < self.bottom()
    }
    pub fn translated(self, dx: f32, dy: f32) -> Self { Self { x: self.x + dx, y: self.y + dy, ..self } }
    pub fn reduced(self, dx: f32, dy: f32) -> Self {
        Self { x: self.x + dx, y: self.y + dy, w: (self.w - dx * 2.0).max(0.0), h: (self.h - dy * 2.0).max(0.0) }
    }
    pub fn expanded(self, d: f32) -> Self { self.reduced(-d, -d) }
}

/// Linear gradient stops are the two colours. `vertical` paints top→bottom; otherwise the
/// gradient runs from the top-left of the shape toward the bottom-right (JUCE knob discs).
#[derive(Clone, Debug, PartialEq)]
pub enum Cmd {
    FillEllipse { rect: Rect, top: Color, bottom: Color, diagonal: bool },
    StrokeEllipse { rect: Rect, color: Color, width: f32 },
    FillRoundRect { rect: Rect, radius: f32, top: Color, bottom: Color, vertical: bool },
    StrokeRoundRect { rect: Rect, radius: f32, color: Color, width: f32 },
    FillRect { rect: Rect, top: Color, bottom: Color, vertical: bool },
    Line { a: Point, b: Point, color: Color, width: f32 },
    /// Closed fill. `diagonal` uses a top-left → bottom-right gradient across the bounds of `pts`.
    FillPoly { pts: Vec<Point>, top: Color, bottom: Color, diagonal: bool },
    StrokePoly { pts: Vec<Point>, color: Color, width: f32, closed: bool },
    /// Angles in radians, 0 = +x in standard math, matching JUCE `addCentredArc` (0 = 3 o'clock, clockwise positive in screen space).
    StrokeArc { center: Point, radius: f32, from: f32, to: f32, color: Color, width: f32 },
    Text { at: Point, text: String, color: Color, size: f32, center: bool },
    /// Push/pop a clip rounded-rect. Hosts that cannot clip may ignore these.
    ClipRound { rect: Rect, radius: f32 },
    PopClip,
}

#[derive(Clone, Debug, Default)]
pub struct Scene { pub cmds: Vec<Cmd> }

impl Scene {
    pub fn new() -> Self { Self { cmds: Vec::new() } }
    pub fn clear(&mut self) { self.cmds.clear(); }

    pub fn fill_ellipse(&mut self, rect: Rect, top: Color, bottom: Color) {
        self.cmds.push(Cmd::FillEllipse { rect, top, bottom, diagonal: top != bottom });
    }
    pub fn fill_ellipse_solid(&mut self, rect: Rect, color: Color) {
        self.cmds.push(Cmd::FillEllipse { rect, top: color, bottom: color, diagonal: false });
    }
    pub fn stroke_ellipse(&mut self, rect: Rect, color: Color, width: f32) {
        self.cmds.push(Cmd::StrokeEllipse { rect, color, width });
    }
    pub fn fill_round(&mut self, rect: Rect, radius: f32, top: Color, bottom: Color, vertical: bool) {
        self.cmds.push(Cmd::FillRoundRect { rect, radius, top, bottom, vertical });
    }
    pub fn stroke_round(&mut self, rect: Rect, radius: f32, color: Color, width: f32) {
        self.cmds.push(Cmd::StrokeRoundRect { rect, radius, color, width });
    }
    pub fn fill_rect(&mut self, rect: Rect, top: Color, bottom: Color, vertical: bool) {
        self.cmds.push(Cmd::FillRect { rect, top, bottom, vertical });
    }
    pub fn line(&mut self, a: Point, b: Point, color: Color, width: f32) {
        self.cmds.push(Cmd::Line { a, b, color, width });
    }
    pub fn fill_poly(&mut self, pts: Vec<Point>, top: Color, bottom: Color) {
        self.cmds.push(Cmd::FillPoly { pts, top, bottom, diagonal: top != bottom });
    }
    pub fn stroke_poly(&mut self, pts: Vec<Point>, color: Color, width: f32, closed: bool) {
        self.cmds.push(Cmd::StrokePoly { pts, color, width, closed });
    }
    pub fn stroke_arc(&mut self, center: Point, radius: f32, from: f32, to: f32, color: Color, width: f32) {
        self.cmds.push(Cmd::StrokeArc { center, radius, from, to, color, width });
    }
    pub fn text(&mut self, at: Point, text: impl Into<String>, color: Color, size: f32, center: bool) {
        self.cmds.push(Cmd::Text { at, text: text.into(), color, size, center });
    }
    pub fn clip_round(&mut self, rect: Rect, radius: f32) { self.cmds.push(Cmd::ClipRound { rect, radius }); }
    pub fn pop_clip(&mut self) { self.cmds.push(Cmd::PopClip); }

    pub fn ellipse_at(c: Point, radius: f32) -> Rect {
        Rect::new(c.x - radius, c.y - radius, radius * 2.0, radius * 2.0)
    }
}

/// Five-primitive subset used by the Water Channel canvas. Hosts that only implement those
/// can still paint panels and the Brit/console knob through [`crate::draw::water_knob`].
#[derive(Clone, Debug, PartialEq)]
pub enum WaterCmd {
    GradientEllipse { rect: Rect, top: Color, bottom: Color },
    StrokeEllipse { rect: Rect, color: Color, width: f32 },
    Line { a: Point, b: Point, color: Color, width: f32 },
    GradientPanel { rect: Rect, radius: f32, top: Color, bottom: Color },
    StrokePanel { rect: Rect, radius: f32, color: Color, width: f32 },
}
