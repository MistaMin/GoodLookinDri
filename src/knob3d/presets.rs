//! Built-in knob models.

use super::model::{Angular, AngularKind, KnobModel, Material, Pointer, PointerKind, Segment, Shape};
use Shape::{Dome, Fillet, Flat, Line};

/// Codes accepted by [`preset`].
pub const PRESETS: [&str; 11] = ["satin", "bakelite", "spun", "gloss", "strip", "collet", "chicken", "wing", "ivory", "chrome", "phenolic"];

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
        _ => return None,
    })
}

fn seg(shape: Shape, r0: f32, r1: f32, h0: f32, h1: f32, m: Material) -> Segment {
    Segment::new(shape, r0, r1, h0, h1, m)
}
fn line(r0: f32, r1: f32, width: f32, m: Material) -> Option<Pointer> {
    Some(Pointer { kind: PointerKind::Line, r0, r1, width, depth: 0.0, material: m })
}
fn white() -> Material {
    Material::plastic(0.93, 0.92, 0.88, 0.45).with_coat(0.9)
}
fn ink() -> Material {
    Material::plastic(0.03, 0.03, 0.033, 0.9)
}
fn model(profile: Vec<Segment>, under: Vec<Segment>, angular: Vec<Angular>, pointer: Option<Pointer>, edge: f32) -> KnobModel {
    KnobModel { profile, under, angular, pointer, height: 1.0, edge }
}

/// Tall satin-aluminium cylinder on a thin dark base ring.
fn satin() -> KnobModel {
    let alu = Material::metal(0.95, 0.945, 0.935, 0.36).with_grain(0.025);
    let wall = Material::metal(0.91, 0.91, 0.91, 0.40).with_grain(0.03);
    let dark = Material::metal(0.16, 0.16, 0.17, 0.30);
    let base = Material::metal(0.26, 0.26, 0.27, 0.28);
    model(
        vec![
            seg(Dome, 0.0, 0.85, 1.148, 1.14, alu),
            seg(Flat, 0.85, 0.868, 1.118, 1.118, dark),
            seg(Dome, 0.868, 0.905, 1.135, 1.08, wall),
            seg(Line, 0.905, 0.93, 1.08, 0.05, wall),
            seg(Flat, 0.93, 0.965, 0.05, 0.05, base),
            seg(Dome, 0.965, 0.98, 0.05, 0.02, base),
        ],
        vec![],
        vec![],
        Some(Pointer { kind: PointerKind::Groove, r0: 0.60, r1: 0.908, width: 0.075, depth: 0.02, material: ink() }),
        0.012,
    )
}

/// Black scalloped phenolic knob with a dished top, a round foot and a white line.
fn bakelite() -> KnobModel {
    let black = Material::plastic(0.05, 0.048, 0.046, 0.7).with_coat(0.86).with_grain(0.05);
    model(
        vec![
            seg(Dome, 0.0, 0.30, 0.88, 0.855, black),
            seg(Line, 0.30, 0.325, 0.84, 0.84, black),
            seg(Dome, 0.325, 0.62, 0.85, 0.75, black),
            seg(Dome, 0.62, 0.87, 0.75, 0.40, black),
            seg(Line, 0.87, 0.90, 0.40, 0.10, black),
        ],
        vec![seg(Flat, 0.0, 0.83, 0.10, 0.10, black), seg(Dome, 0.83, 0.89, 0.10, 0.03, black)],
        vec![Angular { kind: AngularKind::Scallops, count: 7, depth: 0.13, r_in: 0.60, r_out: 1.0 }],
        line(0.10, 0.80, 0.065, white()),
        0.022,
    )
}

/// Spun-aluminium cap on a knurled black skirt.
fn spun() -> KnobModel {
    let cap = Material::metal(0.90, 0.905, 0.91, 0.07).with_aniso(0.55).with_grain(0.22);
    let rim = Material::metal(0.92, 0.92, 0.93, 0.10);
    let skirt = Material::plastic(0.085, 0.085, 0.09, 0.36);
    model(
        vec![
            seg(Dome, 0.0, 0.60, 1.02, 1.00, cap),
            seg(Line, 0.60, 0.64, 1.00, 0.95, rim),
            seg(Line, 0.64, 0.66, 0.95, 0.66, rim),
            seg(Line, 0.66, 0.76, 0.66, 0.60, skirt),
            seg(Line, 0.76, 0.96, 0.60, 0.10, skirt),
            seg(Dome, 0.96, 1.0, 0.10, 0.04, skirt),
        ],
        vec![],
        vec![Angular { kind: AngularKind::Knurl, count: 40, depth: 0.045, r_in: 0.76, r_out: 1.0 }],
        Some(Pointer { kind: PointerKind::Groove, r0: 0.18, r1: 0.60, width: 0.06, depth: 0.012, material: ink() }),
        0.012,
    )
}

/// Glossy cap in the accent colour with a darker wall of the same hue.
fn gloss() -> KnobModel {
    let cap = Material::plastic(0.5, 0.5, 0.5, 0.55).with_accent(1.0).with_coat(1.0).with_grain(0.03);
    let wall = Material::plastic(0.02, 0.02, 0.02, 0.5).with_accent(0.55).with_coat(0.3);
    let foot = Material::plastic(0.03, 0.03, 0.03, 0.6);
    model(
        vec![
            seg(Dome, 0.0, 0.74, 1.00, 0.95, cap),
            seg(Dome, 0.74, 0.86, 0.95, 0.84, cap),
            seg(Line, 0.86, 0.90, 0.84, 0.07, wall),
            seg(Flat, 0.90, 0.95, 0.06, 0.06, foot),
            seg(Dome, 0.95, 0.97, 0.06, 0.02, foot),
        ],
        vec![],
        vec![],
        line(0.34, 0.84, 0.07, white()),
        0.02,
    )
}

/// Matte tapered cylinder with a glossy accent cap disc and a white line down the side.
fn strip() -> KnobModel {
    let cap = Material::plastic(0.5, 0.5, 0.5, 0.5).with_accent(1.0).with_coat(1.0);
    let body = Material::plastic(0.30, 0.30, 0.31, 0.92);
    model(
        vec![
            seg(Dome, 0.0, 0.56, 1.20, 1.18, cap),
            seg(Line, 0.56, 0.58, 1.18, 1.16, body),
            seg(Dome, 0.58, 0.70, 1.16, 1.12, body),
            seg(Line, 0.70, 0.85, 1.12, 0.04, body),
            seg(Dome, 0.85, 0.87, 0.04, 0.01, body),
        ],
        vec![],
        vec![],
        line(0.40, 0.84, 0.07, Material::plastic(0.93, 0.92, 0.88, 0.7)),
        0.016,
    )
}

/// Grey anodised knurled cap inside a wide polished bezel ring.
fn collet() -> KnobModel {
    let cap = Material::metal(0.50, 0.51, 0.53, 0.42).with_grain(0.04);
    let bezel = Material::metal(0.95, 0.95, 0.96, 0.05);
    model(
        vec![seg(Dome, 0.0, 0.50, 1.00, 0.985, cap), seg(Dome, 0.50, 0.56, 0.985, 0.93, cap), seg(Line, 0.56, 0.60, 0.93, 0.26, cap)],
        vec![seg(Flat, 0.0, 0.66, 0.28, 0.28, bezel), seg(Dome, 0.66, 0.97, 0.28, 0.03, bezel)],
        vec![Angular { kind: AngularKind::Knurl, count: 44, depth: 0.05, r_in: 0.545, r_out: 1.0 }],
        line(0.14, 0.52, 0.06, Material::plastic(0.93, 0.92, 0.88, 0.6)),
        0.012,
    )
}

/// Black phenolic chicken-head pointer: the beak is the pointer.
fn chicken() -> KnobModel {
    let black = Material::plastic(0.05, 0.048, 0.046, 0.7).with_coat(0.86).with_grain(0.04);
    model(
        vec![seg(Dome, 0.0, 0.62, 0.82, 0.76, black), seg(Dome, 0.62, 1.0, 0.76, 0.14, black)],
        vec![seg(Flat, 0.0, 0.58, 0.18, 0.18, black), seg(Dome, 0.58, 0.66, 0.18, 0.03, black)],
        vec![Angular { kind: AngularKind::Bar { tip_width: 0.10, tail_width: 0.27, tail: 0.62 }, count: 1, depth: 0.0, r_in: 0.0, r_out: 1.0 }],
        line(0.22, 0.93, 0.045, white()),
        0.02,
    )
}

/// Die-cast winged pointer knob with an accent insert disc on the hub.
fn wing() -> KnobModel {
    let cast = Material::metal(0.60, 0.61, 0.62, 0.36).with_grain(0.05);
    let disc = Material::plastic(0.5, 0.5, 0.5, 0.5).with_accent(1.0).with_coat(1.0);
    model(
        vec![seg(Flat, 0.0, 0.78, 0.60, 0.60, cast), seg(Dome, 0.78, 1.0, 0.60, 0.08, cast)],
        vec![
            seg(Dome, 0.0, 0.36, 0.93, 0.92, disc),
            seg(Line, 0.36, 0.385, 0.90, 0.90, cast),
            seg(Dome, 0.385, 0.52, 0.93, 0.84, cast),
            seg(Line, 0.52, 0.56, 0.84, 0.24, cast),
            seg(Flat, 0.56, 0.68, 0.24, 0.24, cast),
            seg(Dome, 0.68, 0.74, 0.24, 0.03, cast),
        ],
        vec![Angular { kind: AngularKind::Bar { tip_width: 0.15, tail_width: 0.15, tail: 1.0 }, count: 1, depth: 0.0, r_in: 0.0, r_out: 1.0 }],
        line(0.60, 0.95, 0.05, Material::plastic(0.93, 0.92, 0.88, 0.6)),
        0.018,
    )
}

/// Cream fluted vintage knob on a flared skirt, dark pointer line.
fn ivory() -> KnobModel {
    let cream = Material::plastic(0.76, 0.69, 0.52, 0.5).with_coat(0.62).with_grain(0.03);
    let skirt = Material::plastic(0.73, 0.65, 0.47, 0.5).with_coat(0.62).with_grain(0.03);
    model(
        vec![seg(Dome, 0.0, 0.56, 0.96, 0.92, cream), seg(Dome, 0.56, 0.74, 0.92, 0.80, cream), seg(Line, 0.74, 0.79, 0.80, 0.12, cream)],
        vec![seg(Flat, 0.0, 0.80, 0.16, 0.16, skirt), seg(Dome, 0.80, 0.92, 0.16, 0.03, skirt)],
        vec![Angular { kind: AngularKind::Flutes, count: 24, depth: 0.03, r_in: 0.70, r_out: 1.0 }],
        line(0.16, 0.70, 0.05, Material::plastic(0.10, 0.07, 0.05, 0.6)),
        0.025,
    )
}

/// Chrome skirted knob with a small cap.
fn chrome() -> KnobModel {
    let cr = Material::metal(0.95, 0.95, 0.96, 0.04);
    model(
        vec![
            seg(Dome, 0.0, 0.40, 1.00, 0.97, cr),
            seg(Dome, 0.40, 0.50, 0.97, 0.88, cr),
            seg(Line, 0.50, 0.54, 0.88, 0.52, cr),
            seg(Fillet, 0.54, 0.93, 0.52, 0.10, cr),
            seg(Dome, 0.93, 0.99, 0.10, 0.03, cr),
        ],
        vec![],
        vec![],
        Some(Pointer { kind: PointerKind::Groove, r0: 0.58, r1: 0.92, width: 0.06, depth: 0.012, material: ink() }),
        0.022,
    )
}

/// Plain black skirted pointer knob with a white line.
fn phenolic() -> KnobModel {
    let black = Material::plastic(0.10, 0.10, 0.105, 0.32).with_coat(0.55).with_grain(0.03);
    model(
        vec![
            seg(Dome, 0.0, 0.42, 0.84, 0.81, black),
            seg(Dome, 0.42, 0.52, 0.81, 0.74, black),
            seg(Line, 0.52, 0.60, 0.74, 0.46, black),
            seg(Dome, 0.60, 0.97, 0.46, 0.04, black),
        ],
        vec![],
        vec![],
        line(0.12, 0.92, 0.05, white()),
        0.022,
    )
}
