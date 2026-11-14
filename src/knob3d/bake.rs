//! Angle-independent bake of a knob and the cheap per-angle frame on top of it.

use std::sync::OnceLock;

use super::field::Field;
use super::model::{KnobModel, Rig, Sprite};
use super::render::{render_field, repaint_pointer};
use super::shade::Lights;
use crate::color::Color;

/// Knurl phases kept per bake: the ridges snap to the nearest quarter pitch.
const PHASES: usize = 4;

struct Layer {
    field: Field,
    base: Sprite,
    hits: Vec<[f32; 2]>,
}

/// Everything about a rotationally symmetric knob that does not depend on its angle: the
/// lit body, its shadow and occlusion. [`KnobBake::frame`] then only re-shades the pixels
/// that see the pointer.
pub struct KnobBake {
    model: KnobModel,
    lights: Lights,
    radius: f32,
    accent: Color,
    pitch: f32,
    layers: [OnceLock<Layer>; PHASES],
}

impl KnobBake {
    /// `None` when the body is not rotationally symmetric (flutes, scallops, bars); such
    /// knobs need [`super::render`] for every angle.
    pub fn new(model: &KnobModel, rig: &Rig, radius_px: f32, accent: Color) -> Option<KnobBake> {
        let pitch = model.symmetry()?;
        Some(KnobBake { model: model.clone(), lights: Lights::new(rig), radius: radius_px, accent, pitch, layers: Default::default() })
    }

    fn layer(&self, angle: f32) -> &Layer {
        let (idx, phase) = if self.pitch > 0.0 {
            let j = ((angle / self.pitch).rem_euclid(1.0) * PHASES as f32).round() as usize % PHASES;
            (j, j as f32 * self.pitch / PHASES as f32)
        } else {
            (0, 0.0)
        };
        self.layers[idx].get_or_init(|| {
            let field = Field::body(&self.model, self.radius, phase, self.accent);
            let (base, hits) = render_field(&field, &self.lights, true);
            Layer { field, base, hits }
        })
    }

    /// Do the angle-independent work now (otherwise it happens on the first frame).
    pub fn warm(&self) {
        self.layer(0.0);
    }

    /// The knob at `angle`.
    pub fn frame(&self, angle: f32) -> Sprite {
        let layer = self.layer(angle);
        let mut sprite = layer.base.clone();
        let field = layer.field.with_pointer(self.model.pointer, angle, self.accent);
        repaint_pointer(&field, &self.lights, &mut sprite, &layer.hits);
        sprite
    }

    /// Approximate heap size in bytes.
    pub(crate) fn bytes(&self) -> usize {
        self.layers.iter().filter_map(|l| l.get()).map(|l| l.base.px.len() * 12 + 40_000).sum()
    }
}
