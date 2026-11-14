//! Ray-marches the baked height field along the tilted view direction and assembles the sprite.

use std::f32::consts::PI;

use super::field::Field;
use super::model::{KnobModel, Rig, Sprite};
use super::shade::{shade, Lights};
use super::vec::{dot, grey, hash2, mixf, norm, ss, to_srgb, v3, Rng, V3};
use crate::color::Color;

/// Supersampling grid per axis.
const SS_N: usize = 3;

struct Ctx<'a> {
    f: &'a Field,
    l: &'a Lights,
    tan_tilt: f32,
    /// Size scale relative to the 90 px knob the sampling patterns were tuned on.
    sc: f32,
}

impl Ctx<'_> {
    /// First intersection of the view ray through panel point (px, py) with the knob.
    fn trace(&self, px: f32, py: f32) -> Option<(f32, f32)> {
        let f = self.f;
        let b = f.bound();
        let reach = f.zmax * self.tan_tilt;
        if px.abs() > b || py < -b || py > b + reach {
            return None;
        }
        let over = |z: f32| f.height(px, py - z * self.tan_tilt) - z;
        let steps = ((reach / 0.5).ceil() as usize).max(1);
        let mut z_hi = f.zmax + 0.5;
        for i in 1..=steps {
            let z = (f.zmax + 0.5) * (1.0 - i as f32 / steps as f32);
            if over(z) > 0.02 {
                let (mut lo, mut hi) = (z, z_hi);
                for _ in 0..9 {
                    let mid = 0.5 * (lo + hi);
                    if over(mid) > 0.02 {
                        lo = mid;
                    } else {
                        hi = mid;
                    }
                }
                let z = 0.5 * (lo + hi);
                return Some((px, py - z * self.tan_tilt));
            }
            z_hi = z;
        }
        None
    }

    fn ao(&self, x: f32, y: f32, h0: f32, rot: f32) -> f32 {
        const DIRS: usize = 6;
        const DIST: [f32; 6] = [1.5, 4.5, 10.0, 19.0, 34.0, 56.0];
        let mut occ = 0.0;
        for d in 0..DIRS {
            let a = rot + d as f32 * (2.0 * PI / DIRS as f32);
            let (s, c) = a.sin_cos();
            let mut maxs = 0.0f32;
            for &t in DIST.iter() {
                let t = t * self.sc;
                maxs = maxs.max((self.f.height(x + c * t, y + s * t) - h0) / t);
            }
            occ += maxs / (1.0 + maxs * maxs).sqrt();
        }
        1.0 - occ / DIRS as f32
    }

    /// Soft shadow of the key softbox: 0 = fully shadowed, 1 = lit. A tight lobe and a
    /// wide one are tracked along the same ray; `min_w` is a floor on the penumbra width
    /// in pixels so the shadow has no hard core where it meets the base.
    fn shadow(&self, x: f32, y: f32, h0: f32, min_w: f32) -> f32 {
        let l = self.l.key;
        let lh = (l.x * l.x + l.y * l.y).sqrt().max(1e-4);
        let (dx, dy) = (l.x / lh, l.y / lh);
        let rise = l.z / lh;
        let (tan_a, tan_b) = wide_tans(&self.l.rig, rise);
        let (mut ra, mut rb) = (1.0f32, 1.0f32);
        let mut t = 0.6 * self.sc.max(0.5);
        // stop once even the wide penumbra cone has cleared the highest point
        let t_max = (self.f.zmax - h0) / (rise - tan_b) + 1.0;
        while t < t_max {
            let hq = self.f.height(x + dx * t, y + dy * t);
            if hq <= h0 + 0.01 {
                t += (t * 0.12).max(0.7 * self.sc.max(0.5));
                continue; // nothing stands here
            }
            let clear = h0 + t * rise + 0.25 - hq;
            ra = ra.min(clear / (t * tan_a + min_w));
            rb = rb.min(clear / (t * tan_b + 2.0 * min_w));
            if rb < -1.0 {
                break;
            }
            t += (t * 0.12).max(0.7 * self.sc.max(0.5));
        }
        0.55 * ss(-0.6, 1.0, ra.max(-1.0)) + 0.45 * ss(-0.6, 1.0, rb.max(-1.0))
    }

    /// One sub-sample: (linear colour, knob coverage, shadow alpha when the panel is seen).
    fn sample(&self, px: f32, py: f32, rng: &mut Rng) -> (V3, f32, f32) {
        let f = self.f;
        let rot = rng.next() * PI;
        if let Some((x, y)) = self.trace(px, py) {
            let h = f.height(x, y);
            let e = 0.35;
            let hx = (f.height(x + e, y) - f.height(x - e, y)) / (2.0 * e);
            let hy = (f.height(x, y + e) - f.height(x, y - e)) / (2.0 * e);
            let (m, bump) = f.material(x, y);
            let n = norm(norm(v3(-hx, -hy, 1.0)) + bump);
            // camera at a finite distance: flat faces get a gentle gradient
            let v = norm(self.l.view + v3(-x, -y, 0.0) * (0.16 / f.r));
            // never shade a facet that faces away from the eye (bump can tip it over)
            let n = if dot(n, v) < 0.02 { norm(n + v * (0.02 - dot(n, v))) } else { n };
            // the height-field horizon test over-darkens walls: there, use height above the panel
            let wall_ao = 0.72 + 0.28 * ss(0.0, 0.45 * f.r, h);
            let ao = mixf(wall_ao, self.ao(x, y, h, rot), ss(0.3, 0.8, n.z));
            let sh = self.shadow(x, y, h, 0.0);
            (shade(self.l, n, v, &m, ao, sh), 1.0, 0.0)
        } else {
            let rig = &self.l.rig;
            let sh = self.shadow(px, py, 0.0, 0.045 * f.r);
            let ao = self.ao(px, py, 0.0, rot);
            let keep = (1.0 - rig.shadow * (1.0 - sh)) * (1.0 - rig.occlusion * (1.0 - ao).powf(1.3));
            (grey(0.0), 0.0, 1.0 - keep)
        }
    }
}

/// Penumbra slopes of the tight and the wide shadow lobe.
fn wide_tans(rig: &Rig, rise: f32) -> (f32, f32) {
    let a = rig.shadow_softness.min(rise - 0.6).max(0.05);
    (a, (2.0 * a).min(rise - 0.6).max(a))
}

/// Sprite rectangle around the knob, in pixels.
struct Layout {
    w: usize,
    h: usize,
    ox: f32,
    oy: f32,
    fade: f32,
}

fn layout(field: &Field, l: &Lights) -> Layout {
    let b = field.bound();
    let k = l.key;
    let kh = (k.x * k.x + k.y * k.y).sqrt().max(1e-4);
    let cast = 0.8 * field.zmax / (k.z / kh - wide_tans(&l.rig, k.z / kh).1);
    let soft = 0.45 * field.r + 3.0;
    let left = (b + soft + (cast * k.x / kh).max(0.0)).ceil();
    let right = (b + soft + (-cast * k.x / kh).max(0.0)).ceil();
    let up = (b + (field.zmax * l.rig.tilt.tan()).max(soft + (-cast * k.y / kh).max(0.0))).ceil() + 1.0;
    let down = (b + soft + (cast * k.y / kh).max(0.0)).ceil();
    Layout { w: (left + right) as usize, h: (up + down) as usize, ox: left, oy: up, fade: (0.2 * field.r).max(3.0) }
}

/// Render one knob to a premultiplied sprite with its shadow.
pub fn render(model: &KnobModel, rig: &Rig, radius_px: f32, angle: f32, accent: Color) -> Sprite {
    let field = Field::new(model, radius_px, angle, accent);
    render_field(&field, &Lights::new(rig), false).0
}

/// Full render. With `want_hits`, also returns per pixel the plan-space point its centre
/// ray lands on (NaN where it sees the panel).
pub(crate) fn render_field(field: &Field, lights: &Lights, want_hits: bool) -> (Sprite, Vec<[f32; 2]>) {
    let ctx = Ctx { f: field, l: lights, tan_tilt: lights.rig.tilt.tan(), sc: field.r / 90.0 };
    let lay = layout(field, lights);
    let (w, h) = (lay.w, lay.h);
    let mut px = vec![0u32; w * h];
    let mut hits = vec![[f32::NAN; 2]; if want_hits { w * h } else { 0 }];
    let threads = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4).min(h.max(1));
    let rows_per = h.div_ceil(threads).max(1);
    let mut hit_chunks = hits.chunks_mut(rows_per * w);
    std::thread::scope(|s| {
        for (ci, chunk) in px.chunks_mut(rows_per * w).enumerate() {
            let (ctx, lay) = (&ctx, &lay);
            let hc = hit_chunks.next();
            s.spawn(move || {
                for (i, out) in chunk.iter_mut().enumerate() {
                    let (ix, iy) = (i % w, ci * rows_per + i / w);
                    *out = pixel(ctx, ix, iy, lay.ox, lay.oy, w, h, lay.fade);
                }
                if let Some(hc) = hc {
                    for (i, out) in hc.iter_mut().enumerate() {
                        let (ix, iy) = (i % w, ci * rows_per + i / w);
                        if let Some((x, y)) = ctx.trace(ix as f32 + 0.5 - lay.ox, lay.oy - iy as f32 - 0.5) {
                            *out = [x, y];
                        }
                    }
                }
            });
        }
    });
    (Sprite { w: w as u32, h: h as u32, px, origin: (lay.ox, lay.oy) }, hits)
}

/// Re-render only the pixels that see the pointer of `field`, over a pointer-less base.
pub(crate) fn repaint_pointer(field: &Field, lights: &Lights, sprite: &mut Sprite, hits: &[[f32; 2]]) {
    const MARGIN: f32 = 1.5;
    let Some((x0, y0, x1, y1)) = field.pointer_box(MARGIN) else { return };
    let ctx = Ctx { f: field, l: lights, tan_tilt: lights.rig.tilt.tan(), sc: field.r / 90.0 };
    let lay = layout(field, lights);
    let (w, h) = (lay.w, lay.h);
    let lift = field.zmax * ctx.tan_tilt;
    let cx0 = ((lay.ox + x0).floor().max(0.0)) as usize;
    let cx1 = ((lay.ox + x1).ceil().min(w as f32 - 1.0)) as usize;
    let cy0 = ((lay.oy - y1 - lift).floor().max(0.0)) as usize;
    let cy1 = ((lay.oy - y0).ceil().min(h as f32 - 1.0)) as usize;
    let near = |i: usize| !hits[i][0].is_nan() && field.near_pointer(hits[i][0], hits[i][1], MARGIN);
    for iy in cy0..=cy1 {
        for ix in cx0..=cx1 {
            let i = iy * w + ix;
            let hit = near(i) || (ix > 0 && near(i - 1)) || (ix + 1 < w && near(i + 1)) || (iy > 0 && near(i - w)) || (iy + 1 < h && near(i + w));
            if hit {
                sprite.px[i] = pixel(&ctx, ix, iy, lay.ox, lay.oy, w, h, lay.fade);
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn pixel(ctx: &Ctx, ix: usize, iy: usize, ox: f32, oy: f32, w: usize, h: usize, fade: f32) -> u32 {
    let mut rng = Rng((ix as u32).wrapping_mul(9781).wrapping_add((iy as u32).wrapping_mul(6271)) | 1);
    let mut col = grey(0.0);
    let (mut cov, mut dark) = (0.0f32, 0.0f32);
    for j in 0..SS_N {
        for i in 0..SS_N {
            let sx = ix as f32 + (i as f32 + rng.next()) / SS_N as f32;
            let sy = iy as f32 + (j as f32 + rng.next()) / SS_N as f32;
            let (c, k, a) = ctx.sample(sx - ox, oy - sy, &mut rng);
            col += c;
            cov += k;
            dark += a;
        }
    }
    let n = (SS_N * SS_N) as f32;
    if cov == 0.0 && dark < 1e-4 {
        return 0;
    }
    // the shadow must die out before the sprite border
    let edge = (ix as f32 + 0.5).min(w as f32 - ix as f32 - 0.5).min(iy as f32 + 0.5).min(h as f32 - iy as f32 - 0.5);
    let dark = dark / n * ss(0.5, fade, edge);
    let cov = cov / n;
    let alpha = (cov + dark).min(1.0);
    let c = if cov > 0.0 { col * (1.0 / (cov * n)) } else { col };
    let a8 = (alpha * 255.0 + 0.5).floor().min(255.0);
    let mut out = (a8 as u32) << 24;
    for (shift, lin) in [(16, c.x), (8, c.y), (0, c.z)] {
        // triangular dither, about one LSB, from a per-pixel hash
        let d = hash2(ix as i32 * 3 + shift, iy as i32) + hash2(ix as i32, iy as i32 * 3 + shift + 77) - 1.0;
        let v = (to_srgb(lin) * cov * 255.0 + 0.5 + d * 0.5 * cov).floor().clamp(0.0, a8);
        out |= (v as u32) << shift;
    }
    out
}
