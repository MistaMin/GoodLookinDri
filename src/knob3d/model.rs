//! Public description of a knob: radial profile, angular detail, materials, light rig.

use crate::color::Color;
use super::vec::to_linear;

/// Surface description. `base` is linear RGB.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Material {
    pub base: [f32; 3],
    /// 0 = dielectric (paint, plastic), 1 = bare metal.
    pub metallic: f32,
    pub roughness: f32,
    /// Reflectance at normal incidence for the dielectric part.
    pub f0: f32,
    /// Spread of the reflection across lathe grooves (spun metal), 0 = isotropic.
    pub anisotropy: f32,
    /// Glossy lacquer layer on top, 0..1.
    pub clear_coat: f32,
    /// Strength of the fine surface texture (lathe rings or blasting speckle).
    pub grain: f32,
    /// How much of the caller's accent colour replaces `base`, 0..1.
    pub accent: f32,
}

impl Material {
    /// Bare metal with an sRGB tint.
    pub fn metal(r: f32, g: f32, b: f32, roughness: f32) -> Self {
        Self { base: [to_linear(r), to_linear(g), to_linear(b)], metallic: 1.0, roughness, f0: 0.04, anisotropy: 0.0, clear_coat: 0.0, grain: 0.0, accent: 0.0 }
    }
    /// Paint or plastic with an sRGB colour.
    pub fn plastic(r: f32, g: f32, b: f32, roughness: f32) -> Self {
        Self { metallic: 0.0, ..Self::metal(r, g, b, roughness) }
    }
    pub fn with_aniso(mut self, a: f32) -> Self {
        self.anisotropy = a;
        self
    }
    pub fn with_coat(mut self, c: f32) -> Self {
        self.clear_coat = c;
        self
    }
    pub fn with_grain(mut self, g: f32) -> Self {
        self.grain = g;
        self
    }
    pub fn with_accent(mut self, a: f32) -> Self {
        self.accent = a;
        self
    }
    pub(crate) fn tinted(mut self, accent: Color) -> Self {
        if self.accent > 0.0 {
            let c = [accent.r(), accent.g(), accent.b()];
            for i in 0..3 {
                let l = to_linear(c[i] as f32 / 255.0);
                self.base[i] += (l - self.base[i]) * self.accent.min(1.0);
            }
        }
        self
    }
}

/// How the height moves from `h0` (at `r0`) to `h1` (at `r1`) inside a segment.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Shape {
    /// Constant `h0`.
    Flat,
    /// Straight: cone, chamfer or wall.
    Line,
    /// Convex quarter ellipse, level at `r0`, steep at `r1`.
    Dome,
    /// Concave quarter ellipse, steep at `r0`, level at `r1`.
    Fillet,
}

/// One ring of the radial profile. Radii and heights are in units of the knob radius.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Segment {
    pub r0: f32,
    pub r1: f32,
    pub h0: f32,
    pub h1: f32,
    pub shape: Shape,
    pub material: Material,
}

impl Segment {
    pub fn new(shape: Shape, r0: f32, r1: f32, h0: f32, h1: f32, material: Material) -> Self {
        Self { r0, r1, h0, h1, shape, material }
    }
    pub(crate) fn height(&self, u: f32) -> f32 {
        let t = ((u - self.r0) / (self.r1 - self.r0).max(1e-6)).clamp(0.0, 1.0);
        match self.shape {
            Shape::Flat => self.h0,
            Shape::Line => self.h0 + (self.h1 - self.h0) * t,
            Shape::Dome => self.h1 + (self.h0 - self.h1) * (1.0 - t * t).sqrt(),
            Shape::Fillet => self.h1 + (self.h0 - self.h1) * (1.0 - (1.0 - (1.0 - t) * (1.0 - t)).sqrt()),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AngularKind {
    /// Fine straight ridges; fades out when ridges get close to a pixel wide.
    Knurl,
    /// Smooth sinusoidal finger flutes.
    Flutes,
    /// Circular cuts between rounded lobes.
    Scallops,
    /// Asymmetric plan shape: the profile is squeezed into a tapered bar that points along
    /// the knob's zero direction. The tip sits at the full radius, the tail at `tail`
    /// behind the axis; widths are half-widths. All in fractions of the radius.
    /// `count` and `depth` are ignored.
    Bar { tip_width: f32, tail_width: f32, tail: f32 },
}

/// Angular detail that pulls the profile inwards, so it also shapes the silhouette.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Angular {
    pub kind: AngularKind,
    pub count: u32,
    /// Deepest inward cut as a fraction of the radius.
    pub depth: f32,
    /// Radial band (fractions of the radius) in which the detail acts.
    pub r_in: f32,
    pub r_out: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PointerKind {
    /// Painted line, flush with the surface.
    Line,
    /// Milled groove filled with the pointer material.
    Groove,
}

/// Radial indicator from `r0` to `r1` along the knob's zero direction.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pointer {
    pub kind: PointerKind,
    pub r0: f32,
    pub r1: f32,
    /// Full width as a fraction of the radius.
    pub width: f32,
    /// Groove depth as a fraction of the radius (ignored for `Line`).
    pub depth: f32,
    pub material: Material,
}

/// A knob as a height field over its base circle.
#[derive(Clone, Debug, PartialEq)]
pub struct KnobModel {
    /// Rings from the axis outwards. Beyond the last `r1` the height is zero.
    pub profile: Vec<Segment>,
    /// Optional round under-body (skirt, hub, bezel) that the angular detail does not touch.
    /// The surface is the higher of the two.
    pub under: Vec<Segment>,
    pub angular: Vec<Angular>,
    pub pointer: Option<Pointer>,
    /// Scales every profile height (1 = as authored).
    pub height: f32,
    /// Edge rounding radius as a fraction of the radius.
    pub edge: f32,
}

/// Fixed studio lighting and camera, defined in panel space.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rig {
    /// Key light azimuth in radians, measured from straight-up towards the left.
    pub key_azimuth: f32,
    /// Key light elevation above the panel, radians.
    pub key_elevation: f32,
    pub key_intensity: f32,
    /// Half-height of the key softbox as seen in reflections (tangent of the half angle).
    pub key_size: f32,
    /// Penumbra growth of the cast shadow per unit distance from the occluder.
    pub shadow_softness: f32,
    pub ambient: f32,
    /// Brightness of the studio environment reflections.
    pub environment: f32,
    /// Colour of the panel under the knob: tints wall reflections and bounce light.
    pub floor: Color,
    /// Camera tilt in radians: the viewer looks from below-front.
    pub tilt: f32,
    /// Darkness of the cast shadow on the panel, 0..1.
    pub shadow: f32,
    /// Darkness of the contact shadow / base occlusion on the panel, 0..1.
    pub occlusion: f32,
}

impl Rig {
    pub fn studio() -> Self {
        Self {
            key_azimuth: 20f32.to_radians(),
            key_elevation: 65f32.to_radians(),
            key_intensity: 1.0,
            key_size: 0.15,
            shadow_softness: 0.7,
            ambient: 1.0,
            environment: 1.0,
            floor: Color(0xffc0_c0c0),
            tilt: 14f32.to_radians(),
            shadow: 0.72,
            occlusion: 0.6,
        }
    }
}

impl Default for Rig {
    fn default() -> Self {
        Self::studio()
    }
}

/// Premultiplied sRGB sprite, `0xAARRGGBB`, row-major.
#[derive(Clone, Debug, PartialEq)]
pub struct Sprite {
    pub w: u32,
    pub h: u32,
    pub px: Vec<u32>,
    /// Where the knob's base centre sits inside the sprite, device pixels.
    pub origin: (f32, f32),
}

impl KnobModel {
    /// Stable-within-process identity of the model for cache keys.
    pub(crate) fn fingerprint(&self) -> u64 {
        hash_debug(self)
    }
    /// `Some(pitch)` when the body looks the same after turning by `pitch` radians for
    /// every angular detail it has (0 = any angle), `None` when each angle is a new shape.
    pub(crate) fn symmetry(&self) -> Option<f32> {
        let mut count = 0u32;
        for a in &self.angular {
            match a.kind {
                AngularKind::Knurl if count == 0 || count == a.count => count = a.count,
                _ => return None,
            }
        }
        Some(if count == 0 { 0.0 } else { 2.0 * std::f32::consts::PI / count as f32 })
    }
}

impl Rig {
    pub(crate) fn fingerprint(&self) -> u64 {
        hash_debug(self)
    }
}

fn hash_debug<T: std::fmt::Debug>(v: &T) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    format!("{v:?}").hash(&mut h);
    h.finish()
}
