//! Entry points a host calls. `flat_` and the global flat flag select the legacy vector knobs.

use crate::color::Color;
use crate::design::{self, Item};
use crate::knobs;
use crate::knobs_flat;
use crate::scene::{Point, Rect, Scene};

pub use crate::hardware::use_dark_marks;

/// When true, [`knob`] draws the flat family. A `flat_` style prefix forces flat for that one knob.
pub static FLAT_KNOBS: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

pub fn knob(scene: &mut Scene, bounds: Rect, proportion: f64, item: &Item, active: bool, dark_marks: bool) {
    let d = bounds.w.min(bounds.h);
    let c = bounds.center();
    let r = d * 0.5;
    let a = ((proportion.clamp(0.0, 1.0) as f32) * 1.6 - 0.8) * std::f32::consts::PI;
    let accent = Color::from_hex(&item.colour).unwrap_or(Color(0xff697170));
    let accent = if active { accent } else { Color(0xff697170) };
    let flat = item.style == "flat" || item.style.starts_with("flat_") || FLAT_KNOBS.load(std::sync::atomic::Ordering::Relaxed);
    let code = item.style.strip_prefix("flat_").unwrap_or(item.style.as_str());
    let code = if code == "flat" { "Brit" } else { code };
    if flat { knobs_flat::draw_style_flat(scene, code, c, r, a, accent, dark_marks); }
    else { knobs::draw_style(scene, code, c, r, a, accent, dark_marks); }
}

pub fn knob_named(scene: &mut Scene, style: &str, c: Point, r: f32, angle: f32, accent: Color, dark_marks: bool, flat: bool) {
    let code = style.strip_prefix("flat_").unwrap_or(style);
    if flat || style.starts_with("flat_") { knobs_flat::draw_style_flat(scene, code, c, r, angle, accent, dark_marks); }
    else { knobs::draw_style(scene, code, c, r, angle, accent, dark_marks); }
}

pub fn style_count() -> usize { design::KNOB_STYLES.len() }
