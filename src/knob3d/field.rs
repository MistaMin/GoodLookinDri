//! A knob baked for one size and angle: blurred profile tables, angular detail, materials.

use std::f32::consts::PI;

use super::model::{Angular, AngularKind, KnobModel, Material, Pointer, PointerKind, Segment};
use super::vec::{grey, lerp3, mixf, ss, v3, vnoise, V3};
use crate::color::Color;

const LUT_N: usize = 2048;
const LUT_SPAN: f32 = 1.12;

/// Material in the form the shader wants.
#[derive(Clone, Copy)]
pub(crate) struct Mat {
    pub diff: V3,
    pub f0: V3,
    pub rough: f32,
    pub aniso: f32,
    /// Direction across the grooves.
    pub t: V3,
    pub coat: f32,
}

#[derive(Clone)]
struct Mod {
    a: Angular,
    amp: f32,
    /// Scallop cutter: centre distance and radius (radius units).
    cut: (f32, f32),
    /// Radial width over which the detail fades in.
    fade: f32,
}

#[derive(Clone)]
struct Ptr {
    groove: bool,
    r0: f32,
    r1: f32,
    hw: f32,
    depth: f32,
    mat: Material,
    sin_a: f32,
    cos_a: f32,
}

/// One blurred radial height table with its materials.
#[derive(Clone)]
struct Table {
    lut: Vec<f32>,
    segs: Vec<(f32, Material)>,
    zmax: f32,
}

impl Table {
    fn new(profile: &[Segment], r: f32, hs: f32, edge: f32, accent: Color) -> Table {
        let raw_at = |u: f32| -> f32 {
            for s in profile {
                if u < s.r1 {
                    return if u >= s.r0 { s.height(u) * hs } else { 0.0 };
                }
            }
            0.0
        };
        // tabulate, then blur to round every edge (never sharper than about half a pixel)
        let raw: Vec<f32> = (0..LUT_N).map(|i| raw_at(i as f32 / (LUT_N - 1) as f32 * LUT_SPAN)).collect();
        let bins_per_px = (LUT_N - 1) as f32 / (LUT_SPAN * r);
        let sigma = (edge * r).max(0.55) * bins_per_px;
        let rad = (sigma * 3.0).ceil() as i32;
        let wts: Vec<f32> = (-rad..=rad).map(|k| (-(k as f32).powi(2) / (2.0 * sigma * sigma)).exp()).collect();
        let wsum: f32 = wts.iter().sum();
        let mut lut = vec![0.0f32; LUT_N];
        let mut zmax = 0.0f32;
        for i in 0..LUT_N as i32 {
            let mut acc = 0.0;
            for k in -rad..=rad {
                let j = (i + k).abs().min(LUT_N as i32 - 1);
                acc += raw[j as usize] * wts[(k + rad) as usize];
            }
            lut[i as usize] = acc / wsum;
            zmax = zmax.max(lut[i as usize]);
        }
        Table { lut, segs: profile.iter().map(|s| (s.r1, s.material.tinted(accent))).collect(), zmax }
    }
    fn at(&self, rr: f32, r: f32) -> f32 {
        let f = rr / (LUT_SPAN * r) * (LUT_N - 1) as f32;
        if f >= (LUT_N - 1) as f32 {
            return 0.0;
        }
        let i = f as usize;
        mixf(self.lut[i], self.lut[i + 1], f - i as f32)
    }
    fn mat(&self, u: f32) -> Material {
        for (r1, m) in &self.segs {
            if u < *r1 {
                return *m;
            }
        }
        self.segs.last().map(|s| s.1).unwrap_or(Material::plastic(0.5, 0.5, 0.5, 0.5))
    }
}

#[derive(Clone)]
pub(crate) struct Field {
    /// Base radius in pixels.
    pub r: f32,
    /// Highest point in pixels.
    pub zmax: f32,
    body: Table,
    under: Option<Table>,
    mods: Vec<Mod>,
    mod_from: f32,
    ptr: Option<Ptr>,
    /// Rotation of the body detail.
    ang: f32,
}

impl Field {
    /// Body at `angle`, no pointer yet.
    pub fn body(model: &KnobModel, radius_px: f32, angle: f32, accent: Color) -> Field {
        let r = radius_px.max(2.0);
        let hs = model.height * r;
        let body = Table::new(&model.profile, r, hs, model.edge, accent);
        let under = if model.under.is_empty() { None } else { Some(Table::new(&model.under, r, hs, model.edge, accent)) };
        let mods: Vec<Mod> = model
            .angular
            .iter()
            .filter(|a| matches!(a.kind, AngularKind::Bar { .. }) || (a.count > 0 && a.depth > 0.0))
            .map(|a| {
                let mut amp = a.depth;
                let mut cut = (0.0, 0.0);
                let mut fade = 0.22;
                match a.kind {
                    AngularKind::Knurl => {
                        // ridge pitch in pixels; fade before it can alias
                        let pitch = 2.0 * PI * r * a.r_in.max(0.3) / a.count as f32;
                        amp *= ss(2.0, 4.0, pitch);
                        fade = 0.04;
                    }
                    AngularKind::Scallops => {
                        let e = 1.0 - a.depth;
                        let ca = (0.3 * 2.0 * PI / a.count as f32).cos();
                        let rc = if e < ca - 1e-3 { (2.0 * e * ca - 1.0 - e * e) / (2.0 * (e - ca)) } else { 0.2 };
                        cut = (e + rc, rc);
                        fade = 0.03;
                    }
                    AngularKind::Flutes => fade = 0.08,
                    AngularKind::Bar { .. } => {}
                }
                Mod { a: *a, amp, cut, fade }
            })
            .collect();
        let mod_from = mods.iter().map(|m| m.a.r_in - m.fade).fold(f32::MAX, f32::min);
        let zmax = body.zmax.max(under.as_ref().map_or(0.0, |u| u.zmax));
        Field { r, zmax, body, under, mods, mod_from, ptr: None, ang: angle }
    }

    /// Same body with the pointer added at `angle`.
    pub fn with_pointer(&self, pointer: Option<Pointer>, angle: f32, accent: Color) -> Field {
        let mut f = self.clone();
        let r = self.r;
        f.ptr = pointer.map(|p| Ptr {
            groove: p.kind == PointerKind::Groove,
            r0: p.r0 * r,
            r1: p.r1 * r,
            hw: (p.width * r * 0.5).max(0.75),
            depth: p.depth * r,
            mat: p.material.tinted(accent),
            sin_a: angle.sin(),
            cos_a: angle.cos(),
        });
        f
    }

    pub fn new(model: &KnobModel, radius_px: f32, angle: f32, accent: Color) -> Field {
        Field::body(model, radius_px, angle, accent).with_pointer(model.pointer, angle, accent)
    }

    pub fn bound(&self) -> f32 {
        self.r * 1.04 + 1.5
    }

    /// Radius after the angular detail has pulled the profile inwards.
    fn warped(&self, lx: f32, ly: f32, rr: f32) -> f32 {
        let u = rr / self.r;
        if self.mods.is_empty() || u < self.mod_from {
            return rr;
        }
        let phi = lx.atan2(ly) - self.ang;
        let mut out = rr;
        for m in &self.mods {
            let w = ss(m.a.r_in - m.fade, m.a.r_in, u) * if m.a.r_out >= 1.0 { 1.0 } else { ss(m.a.r_out + 0.08, m.a.r_out, u) };
            if w <= 0.0 {
                continue;
            }
            let n = m.a.count as f32;
            let scale = match m.a.kind {
                AngularKind::Flutes => 1.0 - m.amp * (0.5 + 0.5 * (n * phi).cos()),
                AngularKind::Knurl => {
                    let c = (0.5 * n * phi).sin().abs();
                    1.0 - m.amp * (1.0 - c) * (1.0 - c)
                }
                AngularKind::Scallops => {
                    // cuts sit between lobes; a lobe is centred on the pointer
                    let pitch = 2.0 * PI / n;
                    let d = (phi / pitch).rem_euclid(1.0) * pitch - 0.5 * pitch;
                    let (dc, rc) = m.cut;
                    let disc = rc * rc - dc * dc * d.sin() * d.sin();
                    if disc > 0.0 {
                        (dc * d.cos() - disc.sqrt()).min(1.0)
                    } else {
                        1.0
                    }
                }
                AngularKind::Bar { tip_width, tail_width, tail } => bar_radius(phi, tip_width, tail_width, tail),
            };
            out /= mixf(1.0, scale, w).max(0.04);
        }
        out
    }

    /// True when the point lies on or within `margin` pixels of the pointer in plan.
    pub fn near_pointer(&self, lx: f32, ly: f32, margin: f32) -> bool {
        match &self.ptr {
            Some(p) => {
                let along = lx * p.sin_a + ly * p.cos_a;
                let perp = (lx * p.cos_a - ly * p.sin_a).abs();
                along > p.r0 - margin && along < p.r1 + margin && perp < p.hw + margin
            }
            None => false,
        }
    }

    /// Plan-space bounding box (x0, y0, x1, y1) of the pointer, y up.
    pub fn pointer_box(&self, margin: f32) -> Option<(f32, f32, f32, f32)> {
        let p = self.ptr.as_ref()?;
        let mut b = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
        for (al, pe) in [(p.r0, -p.hw), (p.r0, p.hw), (p.r1, -p.hw), (p.r1, p.hw)] {
            let (x, y) = (al * p.sin_a + pe * p.cos_a, al * p.cos_a - pe * p.sin_a);
            b = (b.0.min(x), b.1.min(y), b.2.max(x), b.3.max(y));
        }
        Some((b.0 - margin, b.1 - margin, b.2 + margin, b.3 + margin))
    }

    /// Pointer coverage 0..1 at a point.
    fn ptr_mask(&self, p: &Ptr, lx: f32, ly: f32) -> f32 {
        let along = lx * p.sin_a + ly * p.cos_a;
        if along < p.r0 - 1.0 || along > p.r1 + 1.0 {
            return 0.0;
        }
        let perp = (lx * p.cos_a - ly * p.sin_a).abs();
        ss(p.hw + 0.5, p.hw - 0.5, perp) * ss(p.r0 - 0.7, p.r0 + 0.7, along) * ss(p.r1 + 0.7, p.r1 - 0.7, along)
    }

    /// Height in pixels at a point in knob space (y up, origin at the base centre).
    pub fn height(&self, lx: f32, ly: f32) -> f32 {
        let rr = (lx * lx + ly * ly).sqrt();
        if rr > self.bound() {
            return 0.0;
        }
        let mut h = self.body.at(self.warped(lx, ly, rr), self.r);
        if let Some(u) = &self.under {
            h = h.max(u.at(rr, self.r));
        }
        if let Some(p) = &self.ptr {
            if p.groove && h > 0.0 {
                h = (h - p.depth * self.ptr_mask(p, lx, ly)).max(0.0);
            }
        }
        h
    }

    /// Material and a small normal perturbation at a surface point.
    pub fn material(&self, lx: f32, ly: f32) -> (Mat, V3) {
        let rr = (lx * lx + ly * ly).sqrt().max(1e-4);
        let wr = self.warped(lx, ly, rr);
        let mut src = self.body.mat(wr / self.r);
        if let Some(u) = &self.under {
            if u.at(rr, self.r) > self.body.at(wr, self.r) {
                src = u.mat(rr / self.r);
            }
        }
        let radial = v3(lx / rr, ly / rr, 0.0);
        let (mut mat, mut bump) = self.convert(&src, lx, ly, rr, radial);
        if let Some(p) = &self.ptr {
            let k = self.ptr_mask(p, lx, ly);
            if k > 0.0 {
                let (pm, _) = self.convert(&p.mat, lx, ly, rr, radial);
                mat = Mat {
                    diff: lerp3(mat.diff, pm.diff, k),
                    f0: lerp3(mat.f0, pm.f0, k),
                    rough: mixf(mat.rough, pm.rough, k),
                    aniso: mixf(mat.aniso, pm.aniso, k),
                    t: mat.t,
                    coat: mixf(mat.coat, pm.coat, k),
                };
                bump = bump * (1.0 - k);
            }
        }
        (mat, bump)
    }

    fn convert(&self, m: &Material, lx: f32, ly: f32, rr: f32, radial: V3) -> (Mat, V3) {
        let base = v3(m.base[0], m.base[1], m.base[2]);
        let mut tint = 1.0;
        let mut rough = m.roughness;
        let mut bump = grey(0.0);
        if m.grain > 0.0 {
            if m.anisotropy > 0.0 {
                // lathe rings: concentric grooves of varying depth
                let ring = vnoise(rr * 1.3, 3.0) * 0.6 + vnoise(rr * 0.33, 9.0) * 0.4;
                tint = 1.0 + m.grain * (ring - 0.5);
                bump = radial * ((vnoise(rr * 2.3, 1.0) - 0.5) * 0.25 * m.grain);
            } else if m.clear_coat > 0.0 {
                // slow waviness of the moulding plus a little orange peel in the lacquer
                let (sx, sy) = (lx / self.r, ly / self.r);
                let slow = v3(vnoise(sx * 2.3 + 4.0, sy * 2.3) - 0.5, vnoise(sx * 2.3, sy * 2.3 + 7.0) - 0.5, 0.0);
                let fine = v3(vnoise(lx * 0.45, ly * 0.45) - 0.5, vnoise(lx * 0.45 + 9.0, ly * 0.45) - 0.5, 0.0);
                bump = slow * (0.6 * m.grain) + fine * (0.05 * m.grain);
            } else {
                // blasted / satin speckle
                let s = vnoise(lx * 1.1, ly * 1.1) * 0.6 + vnoise(lx * 0.31 + 5.0, ly * 0.31) * 0.4;
                tint = 1.0 + m.grain * (s - 0.5);
                rough = (rough + 0.3 * m.grain * (s - 0.5)).max(0.02);
                bump = v3(vnoise(lx * 0.9 + 3.0, ly * 0.9) - 0.5, vnoise(lx * 0.9, ly * 0.9 + 3.0) - 0.5, 0.0) * (0.12 * m.grain);
            }
        }
        let mat = Mat {
            diff: base * ((1.0 - m.metallic) * tint),
            f0: lerp3(grey(m.f0), base * tint, m.metallic),
            rough,
            aniso: m.anisotropy,
            t: radial,
            coat: m.clear_coat,
        };
        (mat, bump)
    }
}

/// Polar radius of a tapered bar (tip at 1 ahead, tail behind), corners rounded.
fn bar_radius(phi: f32, tip_w: f32, tail_w: f32, tail: f32) -> f32 {
    let (s, c) = phi.sin_cos();
    let s = s.abs();
    let mut acc = 0.0f32;
    if c > 1e-3 {
        acc += c.powi(8);
    }
    if c < -1e-3 {
        acc += (-c / tail.max(0.05)).powi(8);
    }
    let b = (tip_w - tail_w) / (1.0 + tail);
    let a = (tail_w + b * tail).max(0.02);
    let den = s - b * c;
    if den > 1e-4 {
        acc += (den / a).powi(8);
    }
    acc.max(1e-6).powf(-0.125).min(1.0)
}
