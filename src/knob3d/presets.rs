//! Built-in knob models.

use super::model::{Angular, KnobModel, Material, Part, Plan, Pointer, Segment, Shape};
use Shape::{Dome, Fillet, Flat, Line};

/// Codes accepted by [`preset`].
pub const PRESETS: [&str; 15] =
    ["satin", "bakelite", "spun", "gloss", "strip", "collet", "chicken", "wing", "ivory", "chrome", "phenolic", "wood", "brass", "dome", "dual"];

/// Look up a built-in model by code, see [`PRESETS`].
pub fn preset(code: &str) -> Option<KnobModel> {
    Some(match code {
        "satin" => satin(),
        "bakelite" => bakelite(),
        "spun" => spun(),
        "gloss" => gloss(),
        "strip" => strip(),
        "collet" => collet(),
        "chicken" => chicken(),
        "wing" => wing(),
        "ivory" => ivory(),
        "chrome" => chrome(),
        "phenolic" => phenolic(),
        "wood" => wood(),
        "brass" => brass(),
        "dome" => dome(),
        "dual" => dual(),
        _ => return None,
    })
}

fn seg(shape: Shape, r0: f32, r1: f32, h0: f32, h1: f32, m: Material) -> Segment {
    Segment::new(shape, r0, r1, h0, h1, m)
}
fn white() -> Material {
    Material::plastic(0.93, 0.92, 0.88, 0.45).with_coat(0.5)
}

/// Satin-aluminium cylinder: nearly flat creamy top, a crisp dark line where the top
/// meets the polished wall, thin dark base ring.
fn satin() -> KnobModel {
    // half paint, half metal: keeps the soft form shading of a blasted face
    let top = Material { metallic: 0.5, ..Material::plastic(0.95, 0.94, 0.91, 0.5) }.with_grain(0.012);
    let gap = Material::plastic(0.03, 0.03, 0.035, 0.5);
    let wall = Material::metal(0.93, 0.93, 0.94, 0.26);
    let base = Material::metal(0.22, 0.22, 0.23, 0.30);
    KnobModel::new(vec![
        seg(Dome, 0.0, 0.80, 0.86, 0.855, top),
        seg(Dome, 0.80, 0.845, 0.855, 0.835, top),
        seg(Flat, 0.845, 0.862, 0.81, 0.81, gap),
        seg(Dome, 0.862, 0.91, 0.825, 0.76, wall),
        seg(Line, 0.91, 0.935, 0.76, 0.06, wall),
        seg(Flat, 0.935, 0.965, 0.05, 0.05, base),
        seg(Dome, 0.965, 0.985, 0.05, 0.02, base),
    ])
    .with_edge(0.008)
    .with_pointer(Pointer::groove(0.56, 0.94, 0.08, 0.016, Material::ink()))
}

/// Black scalloped program-equaliser knob: eight-lobed grip with a flat top ring and a
/// recessed centre, on a wide flared skirt that carries the white index line.
fn bakelite() -> KnobModel {
    let black = Material::black_gloss().with_grain(0.04);
    KnobModel::new(vec![
        seg(Dome, 0.0, 0.31, 0.885, 0.875, black),
        seg(Line, 0.31, 0.345, 0.875, 0.96, black),
        seg(Flat, 0.345, 0.52, 0.96, 0.96, black),
        seg(Dome, 0.52, 0.66, 0.96, 0.82, black),
        seg(Line, 0.66, 0.70, 0.82, 0.30, black),
    ])
    .with_under(vec![seg(Flat, 0.0, 0.60, 0.40, 0.40, black), seg(Line, 0.60, 0.965, 0.40, 0.09, black), seg(Dome, 0.965, 1.0, 0.09, 0.02, black)])
    .with_angular(Angular { r_out: 0.9, ..Angular::scallops(8, 0.13, 0.50) })
    .with_edge(0.02)
    .with_pointer(Pointer::line(0.74, 0.985, 0.05, white()))
}

/// Spun-aluminium cap on a knurled black skirt.
fn spun() -> KnobModel {
    let cap = Material::spun_aluminium();
    let rim = Material::metal(0.92, 0.92, 0.93, 0.10);
    let skirt = Material::plastic(0.085, 0.085, 0.09, 0.36);
    KnobModel::new(vec![
        seg(Dome, 0.0, 0.60, 1.02, 1.00, cap),
        seg(Line, 0.60, 0.64, 1.00, 0.95, rim),
        seg(Line, 0.64, 0.66, 0.95, 0.66, rim),
        seg(Line, 0.66, 0.76, 0.66, 0.60, skirt),
        seg(Line, 0.76, 0.96, 0.60, 0.10, skirt),
        seg(Dome, 0.96, 1.0, 0.10, 0.04, skirt),
    ])
    .with_angular(Angular::knurl(40, 0.045, 0.76))
    .with_edge(0.012)
    .with_pointer(Pointer::groove(0.18, 0.60, 0.06, 0.012, Material::ink()))
}

/// Glossy cap in the accent colour with a darker wall of the same hue.
fn gloss() -> KnobModel {
    let cap = Material::plastic(0.5, 0.5, 0.5, 0.5).with_accent(1.0).with_coat(1.0).with_grain(0.02);
    let wall = Material::plastic(0.5, 0.5, 0.5, 0.5).with_accent_shade(1.0, 0.7).with_coat(0.8);
    let foot = Material::plastic(0.03, 0.03, 0.03, 0.6);
    KnobModel::new(vec![
        seg(Dome, 0.0, 0.74, 1.00, 0.95, cap),
        seg(Dome, 0.74, 0.86, 0.95, 0.84, cap),
        seg(Line, 0.86, 0.90, 0.84, 0.07, wall),
        seg(Flat, 0.90, 0.95, 0.06, 0.06, foot),
        seg(Dome, 0.95, 0.97, 0.06, 0.02, foot),
    ])
    .with_edge(0.02)
    .with_pointer(Pointer::line(0.34, 0.84, 0.07, white()))
}

/// Console-strip knob: straight tapered body, flat cap disc in the accent colour and a
/// white line that runs over the cap edge and down the wall.
fn strip() -> KnobModel {
    let cap = Material::plastic(0.5, 0.5, 0.5, 0.45).with_accent(1.0).with_coat(0.55);
    let body = Material::plastic(0.20, 0.20, 0.21, 0.62);
    KnobModel::new(vec![
        seg(Dome, 0.0, 0.50, 1.172, 1.155, cap),
        seg(Flat, 0.50, 0.52, 1.14, 1.14, body),
        seg(Flat, 0.52, 0.57, 1.16, 1.16, body),
        seg(Line, 0.57, 0.60, 1.16, 1.12, body),
        seg(Line, 0.60, 0.86, 1.12, 0.05, body),
        seg(Dome, 0.86, 0.885, 0.05, 0.01, body),
    ])
    .with_edge(0.008)
    .with_pointer(Pointer::line(0.22, 0.86, 0.075, Material::white_paint()))
}

/// Grey ribbed cap with a flat insert inside a wide polished bezel that carries the index.
fn collet() -> KnobModel {
    let cap = Material::plastic(0.42, 0.43, 0.45, 0.55);
    let face = Material::plastic(0.56, 0.57, 0.59, 0.5).with_coat(0.3);
    let bezel = Material::metal(0.95, 0.95, 0.96, 0.24).with_aniso(0.10);
    KnobModel::new(vec![
        seg(Flat, 0.0, 0.40, 0.97, 0.97, face),
        seg(Line, 0.40, 0.425, 0.97, 1.0, cap),
        seg(Dome, 0.425, 0.56, 1.0, 0.94, cap),
        seg(Line, 0.56, 0.60, 0.94, 0.26, cap),
    ])
    .with_under(vec![seg(Flat, 0.0, 0.64, 0.30, 0.30, bezel), seg(Line, 0.64, 0.955, 0.30, 0.07, bezel), seg(Dome, 0.955, 0.985, 0.07, 0.02, bezel)])
    .with_angular(Angular::flutes(18, 0.07, 0.50))
    .with_edge(0.012)
    .with_pointer(Pointer::line(0.66, 0.95, 0.06, Material::ink()))
}

/// Black chicken-head pointer: round skirt, tall tapering beak with a flat top, bevelled
/// flanks and a white inlay line.
fn chicken() -> KnobModel {
    let black = Material::black_gloss().with_grain(0.03);
    let beak = Part::new(
        vec![seg(Flat, 0.0, 0.945, 0.88, 0.88, black), seg(Line, 0.945, 0.985, 0.88, 0.70, black), seg(Line, 0.985, 1.0, 0.70, 0.12, black)],
        Plan::Bar { tip: 1.0, tail: 0.66, tip_width: 0.085, tail_width: 0.29, rounding: 0.10 },
    );
    KnobModel::new(vec![seg(Flat, 0.0, 0.56, 0.30, 0.30, black), seg(Line, 0.56, 0.64, 0.30, 0.20, black), seg(Line, 0.64, 0.665, 0.20, 0.03, black)])
        .with_part(beak)
        .with_edge(0.016)
        .with_pointer(Pointer::line(0.30, 0.975, 0.04, white()))
}

/// Die-cast winged pointer knob: round hub with a recessed accent insert and a fin that
/// carries the index line.
fn wing() -> KnobModel {
    let cast = Material::die_cast();
    let disc = Material::plastic(0.5, 0.5, 0.5, 0.45).with_accent(1.0).with_coat(0.9);
    let fin = Part::new(
        vec![seg(Flat, 0.0, 0.95, 0.84, 0.84, cast), seg(Line, 0.95, 0.985, 0.84, 0.76, cast), seg(Line, 0.985, 1.0, 0.76, 0.05, cast)],
        Plan::Bar { tip: 1.0, tail: 0.60, tip_width: 0.15, tail_width: 0.27, rounding: 0.05 },
    );
    KnobModel::new(vec![
        seg(Flat, 0.0, 0.47, 0.98, 0.98, cast),
        seg(Line, 0.47, 0.51, 0.98, 0.94, cast),
        seg(Line, 0.51, 0.54, 0.94, 0.16, cast),
        seg(Flat, 0.54, 0.61, 0.14, 0.14, cast),
        seg(Line, 0.61, 0.635, 0.14, 0.03, cast),
    ])
    .with_part(fin)
    .with_insert(0.39, 0.03, disc)
    .with_edge(0.014)
    .with_pointer(Pointer::line(0.62, 0.975, 0.05, Material::white_paint()))
}

/// Cream fluted vintage knob on a flared skirt, dark pointer line.
fn ivory() -> KnobModel {
    let cream = Material::cream();
    KnobModel::new(vec![seg(Dome, 0.0, 0.56, 0.96, 0.93, cream), seg(Dome, 0.56, 0.74, 0.93, 0.80, cream), seg(Line, 0.74, 0.79, 0.80, 0.12, cream)])
        .with_under(vec![seg(Flat, 0.0, 0.80, 0.16, 0.16, cream), seg(Dome, 0.80, 0.92, 0.16, 0.03, cream)])
        .with_angular(Angular::flutes(22, 0.022, 0.70))
        .with_edge(0.025)
        .with_pointer(Pointer::line(0.16, 0.70, 0.05, Material::plastic(0.10, 0.07, 0.05, 0.6)))
}

/// Chrome skirted knob with a small cap.
fn chrome() -> KnobModel {
    let cr = Material::chrome();
    KnobModel::new(vec![
        seg(Dome, 0.0, 0.40, 1.00, 0.97, cr),
        seg(Dome, 0.40, 0.50, 0.97, 0.88, cr),
        seg(Line, 0.50, 0.54, 0.88, 0.52, cr),
        seg(Fillet, 0.54, 0.93, 0.52, 0.10, cr),
        seg(Dome, 0.93, 0.99, 0.10, 0.03, cr),
    ])
    .with_edge(0.022)
    .with_pointer(Pointer::groove(0.58, 0.92, 0.06, 0.012, Material::ink()))
}

/// Plain black skirted pointer knob: white line on the cap and again on the skirt.
fn phenolic() -> KnobModel {
    let black = Material::plastic(0.085, 0.085, 0.09, 0.32).with_coat(0.55).with_grain(0.03);
    KnobModel::new(vec![
        seg(Dome, 0.0, 0.42, 0.84, 0.82, black),
        seg(Dome, 0.42, 0.52, 0.82, 0.74, black),
        seg(Line, 0.52, 0.58, 0.74, 0.46, black),
        seg(Line, 0.58, 0.93, 0.46, 0.10, black),
        seg(Dome, 0.93, 0.975, 0.10, 0.02, black),
    ])
    .with_edge(0.02)
    .with_pointer(Pointer::line(0.10, 0.46, 0.05, white()))
    .with_mark(Pointer::line(0.64, 0.93, 0.05, white()))
}

/// Lacquered walnut knob with a pale inlaid pointer line.
fn wood() -> KnobModel {
    let walnut = Material::walnut();
    let foot = Material::brass().with_roughness(0.25);
    KnobModel::new(vec![
        seg(Dome, 0.0, 0.66, 0.92, 0.86, walnut),
        seg(Dome, 0.66, 0.84, 0.86, 0.72, walnut),
        seg(Line, 0.84, 0.90, 0.72, 0.10, walnut),
        seg(Flat, 0.90, 0.945, 0.08, 0.08, foot),
        seg(Dome, 0.945, 0.97, 0.08, 0.02, foot),
    ])
    .with_edge(0.02)
    .with_pointer(Pointer::line(0.36, 0.86, 0.06, Material::plastic(0.90, 0.84, 0.68, 0.5).with_coat(0.85)))
}

/// Knurled brass knob with a turned face.
fn brass() -> KnobModel {
    let face = Material::brass().with_roughness(0.12).with_aniso(0.4).with_rings(0.6);
    let side = Material::brass();
    KnobModel::new(vec![
        seg(Dome, 0.0, 0.70, 0.86, 0.85, face),
        seg(Line, 0.70, 0.76, 0.85, 0.80, side),
        seg(Line, 0.76, 0.80, 0.80, 0.80, side),
        seg(Line, 0.80, 0.90, 0.80, 0.14, side),
        seg(Flat, 0.90, 0.945, 0.12, 0.12, side),
        seg(Dome, 0.945, 0.97, 0.12, 0.02, side),
    ])
    .with_angular(Angular { r_out: 0.9, ..Angular::knurl(44, 0.04, 0.80) })
    .with_edge(0.012)
    .with_pointer(Pointer::groove(0.22, 0.70, 0.055, 0.012, Material::ink()))
}

/// Smooth domed plastic knob with a dot pointer.
fn dome() -> KnobModel {
    let body = Material::plastic(0.10, 0.10, 0.11, 0.45).with_coat(0.7).with_grain(0.02);
    KnobModel::new(vec![seg(Dome, 0.0, 0.90, 0.80, 0.34, body), seg(Line, 0.90, 0.94, 0.34, 0.03, body)])
        .with_edge(0.03)
        .with_pointer(Pointer::dot(0.62, 0.15, 0.012, Material::white_paint()))
}

/// Two-tier concentric knob: wide knurled lower ring, smaller satin cap on top.
fn dual() -> KnobModel {
    let lower = Material::anodised(0.16, 0.16, 0.17).with_roughness(0.38);
    let upper = Material::satin_aluminium();
    KnobModel::two_tier(0.40, 0.56, 0.98, lower, upper)
        .with_angular(Angular::knurl(48, 0.035, 0.90))
        .with_edge(0.012)
        .with_pointer(Pointer::groove(0.14, 0.56, 0.06, 0.012, Material::ink()))
        .with_mark(Pointer::line(0.66, 0.93, 0.045, Material::white_paint()))
}
