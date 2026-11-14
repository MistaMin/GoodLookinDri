//! Procedural studio environment and the surface shader. Everything is linear light.

use super::field::Mat;
use super::model::Rig;
use super::vec::{clamp01, cross, dot, grey, lerp3, mixf, norm, reflect, srgb, ss, to_linear, v3, V3};

/// Rig with derived vectors.
pub(crate) struct Lights {
    pub rig: Rig,
    /// Unit vector towards the key light (x right, y up, z out of the panel).
    pub key: V3,
    /// Unit vector towards the camera.
    pub view: V3,
    /// Linear panel colour.
    pub floor: V3,
}

impl Lights {
    pub fn new(rig: &Rig) -> Lights {
        let (se, ce) = rig.key_elevation.sin_cos();
        let (sa, ca) = rig.key_azimuth.sin_cos();
        let (st, ct) = rig.tilt.sin_cos();
        let f = rig.floor;
        let floor = v3(to_linear(f.r() as f32 / 255.0), to_linear(f.g() as f32 / 255.0), to_linear(f.b() as f32 / 255.0));
        Lights { rig: *rig, key: v3(-ce * sa, ce * ca, se), view: v3(0.0, -st, ct), floor }
    }
}

/// Soft rectangular light in gnomonic coordinates around `c`.
fn softbox(d: V3, c: V3, hint: V3, hu: f32, hv: f32, blur: f32) -> f32 {
    let cd = dot(d, c);
    if cd <= 0.02 {
        return 0.0;
    }
    let bx = norm(hint - c * dot(hint, c));
    let by = cross(c, bx);
    let u = dot(d, bx) / cd;
    let v = dot(d, by) / cd;
    let a = ss(hu + blur, hu - blur, u.abs()) * ss(hv + blur, hv - blur, v.abs());
    let fall = 1.0 - 0.35 * ((u / hu).powi(2) + (v / hv).powi(2)).min(2.0) * 0.5;
    // a blurred box spreads its energy: keep the integral roughly constant
    let spread = (hu * hv) / ((hu.max(blur)) * (hv.max(blur)));
    a * fall * spread
}

/// Returns (key light part, everything else) as linear radiance.
fn env(l: &Lights, d: V3, rough: f32, floor_gain: f32) -> (V3, V3) {
    let blur = 0.015 + 1.4 * rough * rough;
    let e = l.rig.environment;
    // studio room: warm-grey walls, brighter towards the top of the screen, panel below
    let horizon = ss(-0.40 - blur, 0.0 + blur, d.z);
    let wall = lerp3(srgb(0.22, 0.215, 0.21), srgb(0.52, 0.51, 0.50), clamp01(0.5 + 0.5 * d.y));
    // the panel is itself lit by the key, and keeps a little room light when dark
    let floor = (l.floor * 0.85 + grey(0.07)) * floor_gain * (0.55 + 0.45 * clamp01(0.5 - 0.5 * d.y));
    let mut other = lerp3(floor, wall * e, horizon);
    // big overhead diffuser near the camera, with a sweep so flat faces get a gradient
    let zen = ss(0.50 - blur, 0.96, d.z);
    let sweep = ss(-0.45 - blur, 0.45 + blur, d.y - 0.35 * d.x);
    other += srgb(0.66, 0.66, 0.67) * (zen * (0.22 + 0.78 * sweep) * e);
    // tall strip light at the right, slightly cool
    let strip = softbox(d, norm(v3(0.86, 0.1, 0.48)), v3(0.0, 1.0, 0.0), 0.8, 0.07, blur);
    other += v3(1.5, 1.6, 1.75) * (strip * e);
    // second, thinner strip left of the key: gives glossy caps a double streak
    let strip2 = softbox(d, norm(v3(-0.80, 0.25, 0.54)), v3(0.0, 1.0, 0.0), 0.55, 0.035, blur);
    other += v3(1.25, 1.2, 1.1) * (strip2 * e);
    // broad fill panel behind the key: the big soft shape glossy domes mirror
    let fill = softbox(d, norm(v3(-0.55, 0.62, 0.56)), v3(1.0, 0.0, 0.0), 0.55, 0.26, blur);
    other += v3(1.25, 1.22, 1.16) * (fill * e);
    // low warm kicker from the bottom, grazing
    let kick = softbox(d, norm(v3(-0.35, -0.85, 0.3)), v3(1.0, 0.0, 0.0), 0.5, 0.10, blur);
    other += v3(0.55, 0.47, 0.38) * (kick * e);
    // dark flag in front of the camera keeps mirror metal from going flat grey
    let rd = ((d.x - l.view.x).powi(2) + (d.y - l.view.y).powi(2)).sqrt() / d.z.max(0.05);
    let gobo = ss(0.20 + blur + 0.06, 0.04, rd);
    other = other * (1.0 - 0.25 * gobo);
    // key softbox
    let k = softbox(d, l.key, v3(1.0, 0.0, 0.0), l.rig.key_size * 1.25, l.rig.key_size, blur);
    let key = v3(3.6, 3.5, 3.35) * (k * l.rig.key_intensity);
    (key, other)
}

/// Fresnel with the grazing peak damped for rough surfaces.
fn schlick_r(f0: V3, c: f32, rough: f32) -> V3 {
    let m = (1.0 - c.clamp(0.0, 1.0)).powi(5);
    let g = (1.0 - rough).max(0.0).powi(2);
    let top = v3(g.max(f0.x), g.max(f0.y), g.max(f0.z));
    f0 + (top - f0) * m
}

/// Shade one surface point. `v` points to the camera, `jitter` is 0..1.
pub(crate) fn shade(l: &Lights, n: V3, v: V3, m: &Mat, ao: f32, sh: f32) -> V3 {
    let nv = dot(n, v).max(1e-3);
    let spec_ao = ao.powf(1.3) * mixf(1.0, sh, 0.15);
    // base specular; spun metal spreads the reflection across its grooves
    let taps = if m.aniso > 0.0 { 12 } else { 1 };
    // each tap must be at least as wide as the gap to the next, or the fan shows as bands
    let rough = if taps > 1 { m.rough.max((2.2 * m.aniso / taps as f32 / 1.4).sqrt()) } else { m.rough };
    let mut spec = grey(0.0);
    for j in 0..taps {
        let s = if taps > 1 { ((j as f32 + 0.5) / taps as f32 * 2.0 - 1.0) * m.aniso } else { 0.0 };
        let nj = norm(n + m.t * s);
        let r = reflect(v * -1.0, nj);
        let f = schlick_r(m.f0, dot(nj, v).max(0.0), m.rough);
        let (k, o) = env(l, r, rough, 1.0);
        spec += f * (k * sh + o * spec_ao);
    }
    spec = spec * (1.0 / taps as f32);
    // diffuse: wrapped key (it is a big softbox) plus ambient
    let ndl = clamp01((dot(n, l.key) + 0.2) / 1.2);
    // soft frontal fill from below the camera keeps the walls facing the viewer alive
    let fill = 0.7 * (0.8 * n.z - 0.6 * n.y).max(0.0);
    let amb = (grey(0.40) * (0.7 + 0.3 * n.z.max(0.0)) + grey(fill) + grey(0.05) * n.y.max(0.0) + (l.floor * 0.8 + grey(0.02)) * (-n.z).max(0.0).max(0.45 * (1.0 - n.z)))
        * l.rig.ambient;
    let fd = 1.0 - schlick_r(grey(0.04), nv, 0.0).x;
    let diffuse = m.diff * (grey(1.2 * l.rig.key_intensity * ndl * sh) + amb * (ao * mixf(1.0, sh, 0.2))) * fd;
    let mut col = diffuse + spec;
    if m.coat > 0.0 {
        let fc = m.coat * schlick_r(grey(0.04), nv, 0.0).x;
        let r = reflect(v * -1.0, n);
        // a thinner coat is a softer, dimmer mirror
        let (k, o) = env(l, r, 0.03 + 0.15 * (1.0 - m.coat), 0.45);
        // lacquer is a mirror: it shows the lights at their real (far brighter) level and
        // the dim room hardly at all, which is what makes black gloss read as black
        col = col * (1.0 - fc) + (k * (2.6 * sh) + o * (grey(0.3) + o * 2.0) * spec_ao) * fc;
    }
    col
}
