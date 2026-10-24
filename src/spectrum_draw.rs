//! Spectrum looks from `SpectrumRenderer.h`: standard, multicolour, bars, spectrogram.
//! Painting is command-list only. The FFT stays in [`crate::spectrum`].

use crate::color::Color;
use crate::scene::{Point, Rect, Scene};
use crate::spectrum::{self, colour_at, heat_colour, shade_colour, Analyser, SampleRing, SpectrogramHistory, FLOOR_DB};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpectrumStyle { Standard, Multicolour, Bars, Spectrogram }

impl SpectrumStyle {
    pub fn code(self) -> &'static str {
        match self { Self::Multicolour => "multi", Self::Bars => "bars", Self::Spectrogram => "gram", Self::Standard => "standard" }
    }
    pub fn from_code(c: &str) -> Self {
        match c { "multi" => Self::Multicolour, "bars" => Self::Bars, "gram" => Self::Spectrogram, _ => Self::Standard }
    }
}

#[derive(Clone, Debug)]
pub struct SpectrumSettings {
    pub on: bool,
    pub size_index: i32,
    pub style: SpectrumStyle,
    pub use_output: bool,
    pub tilt: f32,
    pub fall: f32,
    pub opacity: f32,
    pub peak_hold: bool,
    pub smoothing: i32,
    pub spectrogram_map: i32,
    pub standard: u32,
    pub range: [u32; 7],
}
impl Default for SpectrumSettings {
    fn default() -> Self {
        Self { on: true, size_index: 3, style: SpectrumStyle::Standard, use_output: true, tilt: 3.0, fall: 24.0, opacity: 0.85, peak_hold: true, smoothing: 0, spectrogram_map: 0, standard: 0xff8fe0c0, range: spectrum::DEFAULT_COLOURS }
    }
}
impl SpectrumSettings {
    pub const SMOOTHING_CHOICES: [i32; 6] = [0, 1, 3, 6, 12, 24];
    pub fn to_string(&self) -> String {
        let mut s = format!("{},{},{},{},{},{},{},{},{:x}",
            if self.on { 1 } else { 0 }, self.size_index, self.style.code(), if self.use_output { 1 } else { 0 },
            self.tilt, self.fall, self.opacity, if self.peak_hold { 1 } else { 0 }, self.standard);
        for c in self.range { s.push(','); s.push_str(&format!("{c:x}")); }
        s.push(','); s.push_str(&self.smoothing.to_string());
        s.push(','); s.push_str(&self.spectrogram_map.to_string());
        s
    }
    pub fn from_string(text: &str) -> Self {
        let mut d = Self::default();
        let t: Vec<&str> = text.split(',').collect();
        if t.len() != 16 && t.len() != 18 { return d; }
        d.on = t[0] != "0";
        d.size_index = t[1].parse::<i32>().unwrap_or(0).clamp(0, spectrum::FFT_SIZES.len() as i32 - 1);
        d.style = SpectrumStyle::from_code(t[2]);
        d.use_output = t[3] != "0";
        d.tilt = t[4].parse::<f32>().unwrap_or(d.tilt).clamp(-6.0, 9.0);
        d.fall = t[5].parse::<f32>().unwrap_or(d.fall).clamp(2.0, 200.0);
        d.opacity = t[6].parse::<f32>().unwrap_or(d.opacity).clamp(0.1, 1.0);
        d.peak_hold = t[7] != "0";
        d.standard = 0xff00_0000 | u32::from_str_radix(t[8], 16).unwrap_or(0);
        for i in 0..7 { d.range[i] = 0xff00_0000 | u32::from_str_radix(t[9 + i], 16).unwrap_or(0); }
        if t.len() == 18 {
            d.smoothing = t[16].parse::<i32>().unwrap_or(0).clamp(0, 48);
            d.spectrogram_map = t[17].parse::<i32>().unwrap_or(0).clamp(0, 1);
        }
        d
    }
}

pub struct SpectrumView {
    pub settings: SpectrumSettings,
    analyser: Analyser,
    cols: Vec<f32>,
    hold_cols: Vec<f32>,
    scratch: Vec<f32>,
    prefix: Vec<f64>,
    smoothed: Vec<f32>,
    hist: SpectrogramHistory,
    pub tick: u64,
}
impl SpectrumView {
    pub fn new(settings: SpectrumSettings) -> Self {
        let mut v = Self { settings, analyser: Analyser::default(), cols: vec![], hold_cols: vec![], scratch: vec![], prefix: vec![], smoothed: vec![], hist: SpectrogramHistory::new(), tick: 0 };
        v.apply();
        v
    }
    pub fn analyser(&self) -> &Analyser { &self.analyser }
    fn apply(&mut self) {
        let idx = self.settings.size_index.clamp(0, spectrum::FFT_SIZES.len() as i32 - 1) as usize;
        let size = spectrum::FFT_SIZES[idx] as usize;
        if self.analyser.fft_size() != size { self.analyser.configure(size, 48000.0); }
        let tilt = self.settings.tilt; let fall = self.settings.fall;
        self.analyser.set_tilt(tilt); self.analyser.set_fall(fall);
    }
    pub fn set_settings(&mut self, s: SpectrumSettings) { self.settings = s; self.apply(); }
    pub fn update(&mut self, ring: &SampleRing, seconds: f32, sample_rate: f64) {
        if !self.settings.on { return; }
        if sample_rate > 1000.0 && sample_rate != self.analyser.sample_rate() { self.analyser.set_sample_rate(sample_rate); }
        let _ = self.analyser.analyse(ring, 128);
        self.analyser.decay(seconds);
        self.tick += 1;
    }
    pub fn paint(&mut self, scene: &mut Scene, plot: Rect, f_min: f32, f_max: f32) {
        if !self.settings.on || plot.w < 8.0 { return; }
        let n = (plot.w as i32).clamp(8, 2048) as usize;
        self.compute(n, f_min, f_max);
        let step = plot.w / n as f32;
        let alpha = self.settings.opacity;
        let nyq = self.analyser.sample_rate() * 0.5;
        let span = (f_max as f64 / f_min as f64).ln();
        let nv = ((nyq.min(f_max as f64) / f_min as f64).ln() / span * n as f64).ceil() as usize;
        let nv = nv.clamp(2, n);
        let y_for = |db: f32| plot.bottom() - ((db - (-90.0)) / 90.0).clamp(0.0, 1.0) * plot.h;
        let colour_for = |c: usize| Color(colour_at(f_min * (f_max / f_min).powf((c as f32 + 0.5) / n as f32), &self.settings.range));
        scene.clip_round(plot, 0.0);
        match self.settings.style {
            SpectrumStyle::Spectrogram => self.paint_gram(scene, plot, n, f_min, f_max, alpha),
            SpectrumStyle::Bars => {
                let group = 3;
                let w = step * group as f32;
                let mut c = 0;
                while c + group <= nv {
                    let mut m = FLOOR_DB; let mut pk = FLOOR_DB;
                    for k in 0..group { m = m.max(self.cols[c + k]); pk = pk.max(self.hold_cols[c + k]); }
                    if m > FLOOR_DB + 1.0 {
                        let col = colour_for(c + group / 2);
                        let x = plot.x + c as f32 * step;
                        let y = y_for(m);
                        scene.fill_rect(Rect::new(x, y, w - 1.0, plot.bottom() - y), col.with_alpha(alpha), col.darker(0.7).with_alpha(alpha * 0.45), true);
                        if self.settings.peak_hold {
                            scene.fill_rect(Rect::new(x, y_for(pk) - 1.0, w - 1.0, 1.5), col.brighter(0.6).with_alpha(alpha), col.brighter(0.6).with_alpha(alpha), false);
                        }
                    }
                    c += group;
                }
            }
            SpectrumStyle::Standard | SpectrumStyle::Multicolour => {
                if self.settings.style == SpectrumStyle::Standard {
                    let c = Color(self.settings.standard);
                    let mut fill = vec![Point::new(plot.x + 0.5 * step, plot.bottom())];
                    let mut line = Vec::new();
                    for i in 0..nv {
                        let x = plot.x + (i as f32 + 0.5) * step;
                        let y = y_for(self.cols[i]);
                        line.push(Point::new(x, y));
                        fill.push(Point::new(x, y));
                    }
                    fill.push(Point::new(plot.x + (nv as f32 - 0.5) * step, plot.bottom()));
                    scene.fill_poly(fill, c.with_alpha(alpha * 0.45), c.with_alpha(alpha * 0.04));
                    scene.stroke_poly(line, c.with_alpha(alpha), 1.3, false);
                } else {
                    for i in 0..nv {
                        let x = plot.x + i as f32 * step;
                        let y = y_for(self.cols[i]);
                        if y >= plot.bottom() - 0.5 { continue; }
                        let col = colour_for(i);
                        scene.fill_rect(Rect::new(x, y, step + 0.5, plot.bottom() - y), col.with_alpha(alpha * 0.55), col.with_alpha(alpha * 0.04), true);
                    }
                    let mut line = Vec::new();
                    for i in 0..nv { line.push(Point::new(plot.x + (i as f32 + 0.5) * step, y_for(self.cols[i]))); }
                    for i in 0..line.len().saturating_sub(1) {
                        let col = colour_for(i).with_alpha(alpha);
                        scene.line(line[i], line[i + 1], col, 5.0);
                        scene.line(line[i], line[i + 1], col, 1.6);
                    }
                }
                if self.settings.peak_hold {
                    let mut hold = Vec::new();
                    for i in 0..nv { hold.push(Point::new(plot.x + (i as f32 + 0.5) * step, y_for(self.hold_cols[i]))); }
                    scene.stroke_poly(hold, Color(0xffffffff).with_alpha(0.28 * alpha), 0.8, false);
                }
            }
        }
        scene.pop_clip();
    }
    fn compute(&mut self, n: usize, f_min: f32, f_max: f32) {
        self.cols.resize(n, FLOOR_DB);
        self.hold_cols.resize(n, FLOOR_DB);
        let db = self.analyser.display_db().to_vec();
        let hold = self.analyser.hold_db().to_vec();
        let sr = self.analyser.sample_rate();
        spectrum::to_columns(&db, sr, n, f_min, f_max, &mut self.cols);
        spectrum::to_columns(&hold, sr, n, f_min, f_max, &mut self.hold_cols);
        if self.settings.smoothing > 0 {
            self.scratch.resize(n, 0.0);
            self.smoothed.resize(n, 0.0);
            self.prefix.resize(n + 1, 0.0);
            let octaves = (f_max as f64 / f_min as f64).log2() as f32;
            let sm = self.settings.smoothing;
            let cols = self.cols.clone();
            spectrum::smooth_columns(&cols, &mut self.smoothed, octaves, sm, &mut self.scratch, &mut self.prefix);
            self.cols.copy_from_slice(&self.smoothed);
            let hold = self.hold_cols.clone();
            spectrum::smooth_columns(&hold, &mut self.smoothed, octaves, sm, &mut self.scratch, &mut self.prefix);
            self.hold_cols.copy_from_slice(&self.smoothed);
        }
    }
    /// Column / bar / line / spectrogram paint from already-reduced bins.
/// `gram` is row-major levels, `columns.len()` wide, age 0 at the top. `sample_rate` 0 draws every column.
pub fn paint_columns(
    scene: &mut Scene,
    plot: Rect,
    settings: &SpectrumSettings,
    columns: &[f32],
    hold: Option<&[f32]>,
    gram: Option<&[u8]>,
    f_min: f32,
    f_max: f32,
    sample_rate: f64,
) {
    if !settings.on || columns.is_empty() || f_min <= 0.0 || f_max <= f_min { return; }
    let n = columns.len();
    let step = plot.w / n as f32;
    let alpha = settings.opacity;
    let nv = if sample_rate > 0.0 {
        let nyq = (f_max as f64).min(sample_rate * 0.5);
        let nv = ((nyq / f_min as f64).ln() / ((f_max / f_min) as f64).ln() * n as f64).ceil() as i32;
        nv.clamp(2, n as i32) as usize
    } else { n };
    let y_for = |db: f32| plot.bottom() - ((db - (-90.0)) / 90.0).clamp(0.0, 1.0) * plot.h;
    let colour_for = |c: usize| Color(colour_at(f_min * (f_max / f_min).powf((c as f32 + 0.5) / n as f32), &settings.range));
    scene.clip_round(plot, 0.0);
    if settings.style == SpectrumStyle::Spectrogram {
        if let Some(rows) = gram {
            let nrows = rows.len() / n;
            if nrows > 0 {
                let a = (alpha * 255.0).round() as u32;
                let cw = plot.w / n as f32;
                let ch = plot.h / nrows as f32;
                for r in 0..nrows {
                    for c in 0..n {
                        let base = colour_at(f_min * (f_max / f_min).powf((c as f32 + 0.5) / n as f32), &settings.range);
                        let col = if settings.spectrogram_map == 1 { shade_colour(base, rows[r * n + c]) } else { heat_colour(rows[r * n + c]) };
                        let color = Color((a << 24) | (col & 0x00ff_ffff));
                        scene.fill_rect(Rect::new(plot.x + c as f32 * cw, plot.y + r as f32 * ch, cw, ch), color, color, true);
                    }
                }
            }
        }
    } else if settings.style == SpectrumStyle::Bars {
        let group = 3usize;
        let w = step * group as f32;
        let mut c = 0;
        while c + group <= nv {
            let mut m = FLOOR_DB;
            let mut pk = FLOOR_DB;
            for k in 0..group {
                m = m.max(columns[c + k]);
                if let Some(h) = hold { if c + k < h.len() { pk = pk.max(h[c + k]); } }
            }
            if m > FLOOR_DB + 1.0 {
                let col = colour_for(c + group / 2);
                let x = plot.x + c as f32 * step;
                let y = y_for(m);
                scene.fill_rect(Rect::new(x, y, w - 1.0, plot.bottom() - y), col.with_alpha(alpha), col.darker(0.7).with_alpha(alpha * 0.45), true);
                if settings.peak_hold {
                    let cap = col.brighter(0.6).with_alpha(alpha);
                    scene.fill_rect(Rect::new(x, y_for(pk) - 1.0, w - 1.0, 1.5), cap, cap, true);
                }
            }
            c += group;
        }
    } else {
        let mut line = Vec::new();
        let mut fill = vec![];
        for i in 0..nv {
            let x = plot.x + (i as f32 + 0.5) * step;
            let y = y_for(columns[i]);
            if i == 0 { fill.push(Point::new(x, plot.bottom())); }
            line.push(Point::new(x, y));
            fill.push(Point::new(x, y));
        }
        if nv > 0 { fill.push(Point::new(plot.x + (nv as f32 - 0.5) * step, plot.bottom())); }
        if settings.style == SpectrumStyle::Standard {
            let c = Color(settings.standard);
            scene.fill_poly(fill, c.with_alpha(alpha * 0.45), c.with_alpha(alpha * 0.04));
            scene.stroke_poly(line, c.with_alpha(alpha), 1.3, false);
        } else {
            for i in 0..nv {
                let x = plot.x + i as f32 * step;
                let y = y_for(columns[i]);
                if y >= plot.bottom() - 0.5 { continue; }
                let col = colour_for(i);
                scene.fill_rect(Rect::new(x, y, step + 0.5, plot.bottom() - y), col.with_alpha(alpha * 0.55), col.with_alpha(alpha * 0.04), true);
            }
            for i in 0..line.len().saturating_sub(1) {
                let col = colour_for(i).with_alpha(alpha);
                scene.line(line[i], line[i + 1], col, 5.0);
                scene.line(line[i], line[i + 1], col, 1.6);
            }
        }
        if settings.peak_hold {
            if let Some(h) = hold {
                let hold_line: Vec<Point> = (0..nv.min(h.len())).map(|i| Point::new(plot.x + (i as f32 + 0.5) * step, y_for(h[i]))).collect();
                scene.stroke_poly(hold_line, Color(0xffffffff).with_alpha(0.28 * alpha), 0.8, false);
            }
        }
    }
    scene.pop_clip();
}

fn paint_gram(&mut self, scene: &mut Scene, plot: Rect, n: usize, f_min: f32, f_max: f32, alpha: f32) {
        let rows = (plot.h as i32).clamp(8, 256) as usize;
        if self.hist.columns() != n || self.hist.rows() != rows { self.hist.configure(n, rows); }
        if !self.cols.is_empty() { self.hist.push(&self.cols, -90.0, -10.0); }
        let cw = plot.w / n as f32;
        let rh = plot.h / rows as f32;
        let a = (alpha * 255.0).round() as u32;
        for r in 0..self.hist.filled().min(rows) {
            let src = self.hist.row(r);
            for c in 0..n {
                let base = colour_at(f_min * (f_max / f_min).powf((c as f32 + 0.5) / n as f32), &self.settings.range);
                let col = if self.settings.spectrogram_map == 1 { shade_colour(base, src[c]) } else { heat_colour(src[c]) };
                let color = Color((a << 24) | (col & 0x00ff_ffff));
                scene.fill_rect(Rect::new(plot.x + c as f32 * cw, plot.y + r as f32 * rh, cw + 0.25, rh + 0.25), color, color, false);
            }
        }
    }
}
