//! Catalogue of every part, matching the six gallery pages. No window: the host paints the scene.

use crate::color::Color;
use crate::design::{self, KnobMode, FACEPLATES, KNOB_STYLES};
use crate::draw::knob_named_mode;
use crate::hardware::{draw_panel, draw_screw};
use crate::meters::{self, FaderCap, VuFace};
use crate::retro::{self, Icon, SliderStyle};
use crate::scene::{Point, Rect, Scene};
use crate::spectrum_draw::{self, SpectrumSettings, SpectrumStyle};
use crate::switches::{self, PushStyle, RockerStyle, ToggleStyle};

pub const PAGE_NAMES: [&str; 6] = ["Knobs", "Plates", "Meters", "Sliders", "Switches & lamps", "Spectrum"];

pub fn paint_page(scene: &mut Scene, page: usize, flat: bool, glow: bool) {
    paint_page_mode(scene, page, if flat { KnobMode::Flat } else { KnobMode::Classic }, glow);
}

pub fn paint_page_mode(scene: &mut Scene, page: usize, mode: KnobMode, glow: bool) {
    match page {
        0 => {
            for (i, s) in KNOB_STYLES.iter().enumerate() {
                let col = (i % 8) as f32;
                let row = (i / 8) as f32;
                let c = Point::new(70.0 + col * 150.0, 70.0 + row * 120.0);
                knob_named_mode(scene, s.code, c, 36.0, 0.4, Color(0xffc96f4a), false, mode);
                scene.text(Point::new(c.x, c.y + 48.0), s.code, Color(0xffdde1d5), 11.0, true);
            }
            if mode == KnobMode::Lit {
                // One row across the 1280-wide page, however many presets there are.
                let n = crate::knob3d::PRESETS.len().max(1) as f32;
                let step = (1280.0 - 100.0) / n;
                let radius = (step * 0.32).min(32.0);
                for (i, &code) in crate::knob3d::PRESETS.iter().enumerate() {
                    let c = Point::new(50.0 + step * (i as f32 + 0.5), 790.0);
                    knob_named_mode(scene, code, c, radius, 0.4, Color(0xffc96f4a), false, mode);
                    scene.text(Point::new(c.x, c.y + radius + 16.0), code, Color(0xffdde1d5), 11.0, true);
                }
            }
        }
        1 => {
            for (i, p) in FACEPLATES.iter().enumerate() {
                let col = (i % 8) as f32;
                let row = (i / 8) as f32;
                let r = Rect::new(16.0 + col * 156.0, 16.0 + row * 110.0, 148.0, 96.0);
                draw_panel(scene, r, Color(p.panel_top), Color(p.panel_bottom), p.finish);
                scene.text(Point::new(r.x + 8.0, r.y + 8.0), p.code, Color(p.text), 12.0, false);
            }
        }
        2 => {
            meters::draw_vu_meter(scene, Rect::new(20.0, 20.0, 220.0, 140.0), 0.62, VuFace::Cream);
            meters::draw_vu_meter(scene, Rect::new(260.0, 20.0, 220.0, 140.0), 0.4, VuFace::Black);
            meters::draw_ppm(scene, Rect::new(500.0, 20.0, 280.0, 90.0), 0.7);
            meters::draw_led_ladder(scene, Rect::new(20.0, 180.0, 40.0, 220.0), false, -12.0, -6.0, 16);
            meters::draw_stereo_ladder(scene, Rect::new(80.0, 180.0, 70.0, 220.0), -8.0, -14.0, -3.0);
            meters::draw_segment_arc(scene, Rect::new(180.0, 180.0, 160.0, 120.0), 0.66, Color(0xff39ff7a));
            meters::draw_vector_fan(scene, Rect::new(360.0, 180.0, 160.0, 120.0), 0.5, Color(0xffffc34d));
            meters::draw_seven_segment(scene, Rect::new(540.0, 180.0, 200.0, 60.0), "-12.4", Color(0xffff9a1f), true);
            meters::draw_lamp(scene, Point::new(60.0, 440.0), 10.0, Color(0xffff3b4a), true);
            meters::draw_lit_key(scene, Rect::new(100.0, 420.0, 80.0, 28.0), "IN", true, Color(0xff5be85a), false);
            meters::draw_slot_fader(scene, Rect::new(220.0, 420.0, 28.0, 160.0), 0.35, FaderCap::Oval, Color(0xfff0f0ee));
            meters::draw_slot_fader(scene, Rect::new(270.0, 420.0, 36.0, 160.0), 0.6, FaderCap::Console, Color(0xffd0d0d0));
            retro::draw_bar_wedge(scene, Rect::new(340.0, 420.0, 180.0, 70.0), 0.55, Color(0xff39ff7a), 1.0, glow);
            retro::draw_mountains(scene, Rect::new(540.0, 420.0, 180.0, 70.0), 0.4, 0.7, Color(0xff39ff7a), 1.0, glow);
            retro::draw_lcd_panel(scene, Rect::new(740.0, 420.0, 180.0, 70.0), "440", "HZ", "-3.2", "DB", Color(0xffffc34d), 1.0, glow);
            retro::draw_arc_gauge(scene, Rect::new(20.0, 520.0, 140.0, 100.0), 0.5, Color(0xffff9a1f), 1.0, glow);
            retro::draw_colour_columns(scene, Rect::new(180.0, 520.0, 80.0, 120.0), 0.6, 0.4, 0.8, 0.5, 1.0, glow);
            retro::draw_neon_bars(scene, Rect::new(280.0, 520.0, 160.0, 80.0), 0.7, 0.85, Color(0xff39f2ff), Color(0xffff4fd8), 1.0, glow);
            retro::draw_hex_ring(scene, Rect::new(460.0, 520.0, 120.0, 120.0), 0.45, Color(0xff39f2ff), Color(0xffff4fd8), 1.0, glow);
            retro::draw_scope(scene, Rect::new(600.0, 520.0, 180.0, 80.0), &[0.1, 0.4, -0.2, 0.8, 0.0, -0.5, 0.3], false, Color(0xff39f2ff), 1.0, glow);
            retro::draw_chassis(scene, Rect::new(800.0, 520.0, 160.0, 90.0), 0.5, Color(0xffc7a812), Color(0xff16b4c4), 1.0, glow);
            retro::draw_flat_bars(scene, Rect::new(980.0, 520.0, 40.0, 120.0), 0.4, 0.6, 0.7, 0.8, Color(0xff5be85a));
        }
        3 => {
            for (i, st) in [SliderStyle::Neon, SliderStyle::Bar, SliderStyle::Chamfer, SliderStyle::Wedge, SliderStyle::Flat].iter().enumerate() {
                retro::draw_slider(scene, Rect::new(30.0, 30.0 + i as f32 * 70.0, 280.0, 28.0), 0.35 + i as f32 * 0.1, *st, Color(0xff39f2ff), false, 0.5, glow);
                retro::draw_slider(scene, Rect::new(340.0 + i as f32 * 50.0, 30.0, 28.0, 220.0), 0.6, *st, Color(0xffff4fd8), true, 0.5, glow);
            }
        }
        4 => {
            for (i, st) in [RockerStyle::Lamp, RockerStyle::Classic, RockerStyle::Slim, RockerStyle::Neon].iter().enumerate() {
                switches::draw_rocker(scene, Rect::new(30.0 + i as f32 * 90.0, 30.0, 70.0, 36.0), i % 2 == 0, Color(0xffff3b4a), *st, false, glow);
            }
            for (i, st) in [ToggleStyle::Bat, ToggleStyle::Lever, ToggleStyle::Mini].iter().enumerate() {
                switches::draw_toggle(scene, Rect::new(30.0 + i as f32 * 70.0, 90.0, 40.0, 70.0), true, *st, Color(0xfff2a31b));
            }
            for (i, st) in [PushStyle::Square, PushStyle::Round, PushStyle::Dome, PushStyle::Chamfer, PushStyle::Neon, PushStyle::Flat, PushStyle::Arcade, PushStyle::Pill].iter().enumerate() {
                switches::draw_push(scene, Rect::new(30.0 + (i % 4) as f32 * 100.0, 180.0 + (i / 4) as f32 * 50.0, 88.0, 36.0), "KEY", *st, Color(0xff3f6fd0), true, false, glow);
            }
            switches::draw_key_bank(scene, Rect::new(30.0, 300.0, 360.0, 36.0), &["A", "B", "C"], 1, PushStyle::Square, Color(0xffc96f4a), glow);
            for (i, icon) in [Icon::Battery, Icon::Warning, Icon::ArrowLeft, Icon::ArrowRight, Icon::Beam, Icon::Temp].iter().enumerate() {
                retro::draw_icon_lamp(scene, Rect::new(30.0 + i as f32 * 48.0, 360.0, 36.0, 36.0), *icon, Color(0xffffc34d), true);
            }
            draw_screw(scene, 30.0, 430.0);
        }
        _ => paint_spectrum_page(scene),
    }
}

fn paint_spectrum_page(scene: &mut Scene) {
    const NAMES: [&str; 6] = ["Standard", "Multicolour", "Bars", "Multicolour, 1/3-octave smoothing", "Spectrogram (heat map)", "Spectrogram (range colours, 1/3 oct)"];
    let looks = [
        (SpectrumStyle::Standard, 0, 0),
        (SpectrumStyle::Multicolour, 0, 0),
        (SpectrumStyle::Bars, 0, 0),
        (SpectrumStyle::Multicolour, 3, 0),
        (SpectrumStyle::Spectrogram, 0, 0),
        (SpectrumStyle::Spectrogram, 3, 1),
    ];
    let n = 96usize;
    let cols: Vec<f32> = (0..n).map(|i| {
        let f = i as f32 / n as f32;
        -78.0 + 36.0 * (-(f - 0.22).powi(2) / 0.008).exp() + 28.0 * (-(f - 0.55).powi(2) / 0.02).exp()
    }).collect();
    let hold: Vec<f32> = cols.iter().map(|v| (v + 6.0).min(0.0)).collect();
    let rows = 24usize;
    let mut gram = vec![0u8; rows * n];
    for r in 0..rows {
        for c in 0..n {
            let db = cols[c] - r as f32 * 1.6;
            gram[r * n + c] = (((db + 90.0) / 80.0).clamp(0.0, 1.0) * 255.0).round() as u8;
        }
    }
    for i in 0..6 {
        let col = (i % 2) as f32;
        let row = (i / 2) as f32;
        let cell = Rect::new(16.0 + col * 460.0, 16.0 + row * 200.0, 440.0, 180.0);
        scene.fill_round(cell, 6.0, Color(0xff0e161a), Color(0xff0e161a), true);
        let mut settings = SpectrumSettings::default();
        settings.style = looks[i as usize].0;
        settings.smoothing = looks[i as usize].1;
        settings.spectrogram_map = looks[i as usize].2;
        settings.size_index = 4;
        let plot = cell.reduced(8.0, 8.0);
        let plot = Rect::new(plot.x, plot.y, plot.w, (plot.h - 16.0).max(0.0));
        spectrum_draw::SpectrumView::paint_columns(scene, plot, &settings, &cols, Some(&hold), Some(&gram), 20.0, 30000.0, 48000.0);
        scene.text(Point::new(cell.center().x, cell.bottom() - 12.0), NAMES[i as usize], Color(0xff9fb0b8), 10.0, true);
    }
}

pub fn every_style_draws(flat: bool) -> bool {
    every_style_draws_mode(if flat { KnobMode::Flat } else { KnobMode::Classic })
}

pub fn every_style_draws_mode(mode: KnobMode) -> bool {
    for s in KNOB_STYLES {
        let mut scene = Scene::new();
        knob_named_mode(&mut scene, s.code, Point::new(40.0, 40.0), 32.0, 0.2, Color(0xffc96f4a), false, mode);
        if scene.cmds.is_empty() { return false; }
        let _ = design::find_style(s.code);
    }
    true
}
