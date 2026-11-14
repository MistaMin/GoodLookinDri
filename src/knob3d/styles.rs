//! One lit model for each physical knob style of the catalogue.
//!
//! Every model follows the description of its style in [`crate::design::KNOB_STYLES`] and
//! the character of its classic drawing (colours, proportions, skirt, cap, flutes and
//! pointer type), built as a solid object. The accent colour appears only where a real
//! knob carries colour: a cap disc, an insert, a pointer line, or the moulding compound of
//! the few styles that are moulded in colour. Display and neon styles have no model.

use super::model::{Angular, KnobModel, Material, Part, Plan, Pointer, Segment, Shape};
use Shape::{Dome, Fillet, Flat, Line};

/// Codes of the styles that have a lit model, in catalogue order.
pub const STYLE_CODES: [&str; 35] = [
    "Brit", "N", "A", "FS", "Mtl", "Wd", "Bk", "Pie", "Snk", "Slv", "Rd", "Ptr", "Dm", "Sq", "Fat", "Pag", "Lvr", "Tab", "Wh", "Knl", "Skt",
    "Rib", "Chr", "Key", "Brz", "Chk", "Ivr", "Mpl", "Dbl", "Mini", "Tl", "Mic", "Ebn", "Wnt", "Rck",
];

/// Lit model of a catalogue style, by code or legacy alias. `None` for the display and
/// neon styles, which keep their vector drawing.
pub fn style_model(code: &str) -> Option<KnobModel> {
    Some(match code {
        "Brit" | "console" => brit(),
        "N" => n(),
        "A" => a(),
        "FS" => fs(),
        "Mtl" | "metal" => mtl(),
        "Wd" => wd(),
        "Bk" | "bakelite" => bk(),
        "Pie" => pie(),
        "Snk" => snk(),
        "Slv" | "ivory" => slv(),
        "Rd" => rd(),
        "Ptr" => ptr(),
        "Dm" => dm(),
        "Sq" => sq(),
        "Fat" => fat(),
        "Pag" => pag(),
        "Lvr" => lvr(),
        "Tab" => tab(),
        "Wh" => wh(),
        "Knl" => knl(),
        "Skt" => skt(),
        "Rib" => rib(),
        "Chr" => chr(),
        "Key" => key(),
        "Brz" => brz(),
        "Chk" => chk(),
        "Ivr" => ivr(),
        "Mpl" => mpl(),
        "Dbl" => dbl(),
        "Mini" => mini(),
        "Tl" => tl(),
        "Mic" => mic(),
        "Ebn" => ebn(),
        "Wnt" => wnt(),
        "Rck" => rck(),
        _ => return None,
    })
}

fn seg(shape: Shape, r0: f32, r1: f32, h0: f32, h1: f32, m: Material) -> Segment {
    Segment::new(shape, r0, r1, h0, h1, m)
}
/// Matte paint in the accent colour, for cap discs.
fn accent_matte() -> Material {
    Material::plastic(0.5, 0.5, 0.5, 0.55).with_accent(1.0).with_coat(0.15)
}
/// Glossy plastic in the accent colour, for moulded caps.
fn accent_gloss() -> Material {
    Material::plastic(0.5, 0.5, 0.5, 0.45).with_accent(1.0).with_coat(0.8)
}
/// Paint in the accent colour for pointer lines.
fn accent_paint() -> Material {
    Material::plastic(0.5, 0.5, 0.5, 0.6).with_accent(1.0)
}
fn white() -> Material {
    Material::white_paint()
}
fn polished() -> Material {
    Material::metal(0.93, 0.93, 0.94, 0.12)
}

/// Black fluted grip under a large flat cap disc in the accent colour; the white index
/// is milled into the cap.
fn brit() -> KnobModel {
    let grip = Material::black_matte().with_grain(0.03);
    let cap = accent_matte();
    KnobModel::new(vec![
        seg(Dome, 0.0, 0.56, 1.03, 1.015, cap),
        seg(Line, 0.56, 0.585, 1.015, 0.99, cap),
        seg(Flat, 0.585, 0.61, 0.955, 0.955, grip),
        seg(Dome, 0.61, 0.70, 0.99, 0.93, grip),
        seg(Line, 0.70, 0.90, 0.93, 0.10, grip),
        seg(Dome, 0.90, 0.93, 0.10, 0.02, grip),
    ])
    .with_angular(Angular::flutes(24, 0.03, 0.64))
    .with_edge(0.012)
    .with_pointer(Pointer::groove(0.14, 0.56, 0.07, 0.014, white()))
}

/// Grey fluted grip with a flat accent cap, standing in a rounded chrome collar that
/// repeats the index.
fn n() -> KnobModel {
    let cr = Material::metal(0.95, 0.95, 0.96, 0.20);
    let grip = Material::plastic(0.40, 0.41, 0.43, 0.55);
    let cap = accent_matte();
    KnobModel::new(vec![
        seg(Flat, 0.0, 0.43, 1.0, 1.0, cap),
        seg(Flat, 0.43, 0.455, 0.965, 0.965, grip),
        seg(Dome, 0.455, 0.56, 0.99, 0.92, grip),
        seg(Line, 0.56, 0.66, 0.92, 0.28, grip),
    ])
    .with_under(vec![seg(Flat, 0.0, 0.72, 0.30, 0.30, cr), seg(Dome, 0.72, 0.965, 0.30, 0.06, cr), seg(Line, 0.965, 0.985, 0.06, 0.02, cr)])
    .with_angular(Angular::flutes(16, 0.045, 0.50))
    .with_edge(0.012)
    .with_pointer(Pointer::groove(0.10, 0.43, 0.06, 0.012, Material::ink()))
    .with_mark(Pointer::line(0.74, 0.95, 0.05, Material::ink()))
}

/// Flared silver crown with a pointer fin and a large flat matte cap in the accent colour.
fn a() -> KnobModel {
    let cast = Material::die_cast();
    let fin = Part::new(
        vec![seg(Flat, 0.0, 0.94, 0.66, 0.66, cast), seg(Line, 0.94, 0.98, 0.66, 0.58, cast), seg(Line, 0.98, 1.0, 0.58, 0.04, cast)],
        Plan::Bar { tip: 1.0, tail: 0.50, tip_width: 0.14, tail_width: 0.30, rounding: 0.06 },
    );
    KnobModel::new(vec![
        seg(Flat, 0.0, 0.58, 0.98, 0.98, cast),
        seg(Line, 0.58, 0.62, 0.98, 0.93, cast),
        seg(Fillet, 0.62, 0.78, 0.93, 0.12, cast),
        seg(Dome, 0.78, 0.82, 0.12, 0.02, cast),
    ])
    .with_part(fin)
    .with_insert(0.52, 0.025, accent_matte())
    .with_edge(0.014)
    .with_pointer(Pointer::line(0.72, 0.975, 0.05, white()))
}

/// Wide pale-grey skirt with a glossy cap in the accent colour; the black index runs
/// from the cap out over the skirt.
fn fs() -> KnobModel {
    let skirt = Material::plastic(0.80, 0.81, 0.82, 0.5).with_coat(0.25);
    let cap = accent_gloss();
    KnobModel::new(vec![
        seg(Dome, 0.0, 0.50, 0.80, 0.75, cap),
        seg(Dome, 0.50, 0.60, 0.75, 0.62, cap),
        seg(Line, 0.60, 0.62, 0.62, 0.38, cap),
        seg(Line, 0.62, 0.67, 0.38, 0.43, skirt),
        seg(Flat, 0.67, 0.85, 0.43, 0.43, skirt),
        seg(Dome, 0.85, 0.97, 0.43, 0.08, skirt),
        seg(Line, 0.97, 0.985, 0.08, 0.02, skirt),
    ])
    .with_edge(0.02)
    .with_pointer(Pointer::line(0.12, 0.95, 0.06, Material::ink()))
}

/// Solid aluminium: knurled satin wall, polished chamfer and a turned top.
fn mtl() -> KnobModel {
    let top = Material::spun_aluminium();
    let wall = Material::satin_aluminium();
    KnobModel::new(vec![
        seg(Dome, 0.0, 0.74, 1.0, 0.99, top),
        seg(Line, 0.74, 0.80, 0.99, 0.93, polished()),
        seg(Line, 0.80, 0.84, 0.93, 0.91, wall),
        seg(Line, 0.84, 0.90, 0.91, 0.10, wall),
        seg(Flat, 0.90, 0.94, 0.08, 0.08, wall),
        seg(Dome, 0.94, 0.96, 0.08, 0.02, wall),
    ])
    .with_angular(Angular { r_out: 0.90, ..Angular::knurl(52, 0.03, 0.80) })
    .with_edge(0.012)
    .with_pointer(Pointer::groove(0.20, 0.74, 0.055, 0.012, Material::ink()))
}

/// Turned mid-brown wooden knob with a lathe groove and a pale inlaid index.
fn wd() -> KnobModel {
    let wood = Material::wood(0.62, 0.40, 0.22).with_coat(0.2).with_roughness(0.7);
    KnobModel::new(vec![
        seg(Dome, 0.0, 0.70, 0.70, 0.68, wood),
        seg(Flat, 0.70, 0.72, 0.665, 0.665, wood),
        seg(Dome, 0.72, 0.88, 0.68, 0.58, wood),
        seg(Line, 0.88, 0.935, 0.58, 0.10, wood),
        seg(Dome, 0.935, 0.955, 0.10, 0.02, wood),
    ])
    .with_edge(0.02)
    .with_pointer(Pointer::line(0.24, 0.88, 0.06, Material::plastic(0.92, 0.86, 0.70, 0.5).with_coat(0.85)))
}

/// Black bakelite: a domed hub drawn out into a teardrop pointer, over a ribbed base.
fn bk() -> KnobModel {
    let black = Material::black_gloss().with_grain(0.04);
    let nose = Part::new(
        vec![seg(Flat, 0.0, 0.90, 0.62, 0.62, black), seg(Dome, 0.90, 0.99, 0.62, 0.34, black), seg(Line, 0.99, 1.0, 0.34, 0.24, black)],
        Plan::Bar { tip: 0.90, tail: 0.0, tip_width: 0.075, tail_width: 0.34, rounding: 0.06 },
    );
    KnobModel::new(vec![seg(Flat, 0.0, 0.80, 0.26, 0.26, black), seg(Dome, 0.80, 0.96, 0.26, 0.05, black), seg(Line, 0.96, 0.98, 0.05, 0.02, black)])
        .with_under(vec![seg(Dome, 0.0, 0.38, 0.88, 0.82, black), seg(Dome, 0.38, 0.50, 0.82, 0.60, black), seg(Line, 0.50, 0.53, 0.60, 0.20, black)])
        .with_angular(Angular::flutes(28, 0.03, 0.62))
        .with_part(nose)
        .with_edge(0.02)
        .with_pointer(Pointer::line(0.34, 0.86, 0.045, white()))
}

/// White pie-crust skirt with rounded lobes under a glossy black cap.
fn pie() -> KnobModel {
    let crust = Material::plastic(0.90, 0.90, 0.87, 0.45).with_coat(0.4);
    let black = Material::black_gloss();
    KnobModel::new(vec![seg(Flat, 0.0, 0.76, 0.30, 0.30, crust), seg(Dome, 0.76, 0.97, 0.30, 0.06, crust), seg(Line, 0.97, 0.99, 0.06, 0.02, crust)])
        .with_under(vec![seg(Dome, 0.0, 0.50, 0.84, 0.78, black), seg(Dome, 0.50, 0.62, 0.78, 0.62, black), seg(Line, 0.62, 0.65, 0.62, 0.28, black)])
        .with_angular(Angular::scallops(18, 0.08, 0.72))
        .with_edge(0.02)
        .with_pointer(Pointer::line(0.12, 0.60, 0.06, accent_paint()))
        .with_mark(Pointer::line(0.70, 0.90, 0.05, Material::ink()))
}

/// Plain matte black cylinder with a fine grip and a pointer line in the accent colour.
fn snk() -> KnobModel {
    let body = Material::rubber(0.11, 0.11, 0.115);
    KnobModel::new(vec![
        seg(Dome, 0.0, 0.66, 1.0, 0.985, body),
        seg(Dome, 0.66, 0.75, 0.985, 0.93, body),
        seg(Line, 0.75, 0.79, 0.93, 0.07, body),
        seg(Flat, 0.79, 0.83, 0.06, 0.06, body),
        seg(Dome, 0.83, 0.85, 0.06, 0.02, body),
    ])
    .with_angular(Angular { r_out: 0.79, ..Angular::knurl(40, 0.02, 0.73) })
    .with_edge(0.016)
    .with_pointer(Pointer::line(0.24, 0.72, 0.06, accent_paint()))
}

/// Silver-grey cylinder on a dark ring inside a satin silver skirt, black index.
fn slv() -> KnobModel {
    let top = Material { metallic: 0.5, ..Material::plastic(0.88, 0.89, 0.90, 0.5) }.with_grain(0.012);
    let ring = Material::plastic(0.06, 0.065, 0.07, 0.5);
    let skirt = Material::satin_aluminium();
    KnobModel::new(vec![
        seg(Dome, 0.0, 0.52, 0.92, 0.91, top),
        seg(Dome, 0.52, 0.58, 0.91, 0.86, top),
        seg(Line, 0.58, 0.61, 0.86, 0.20, polished()),
    ])
    .with_under(vec![
        seg(Flat, 0.0, 0.79, 0.20, 0.20, ring),
        seg(Line, 0.79, 0.82, 0.20, 0.28, skirt),
        seg(Flat, 0.82, 0.92, 0.28, 0.28, skirt),
        seg(Dome, 0.92, 0.99, 0.28, 0.03, skirt),
    ])
    .with_edge(0.012)
    .with_pointer(Pointer::groove(0.14, 0.52, 0.06, 0.012, Material::ink()))
    .with_mark(Pointer::dot(0.70, 0.07, 0.0, white()))
}

/// Round black body with a ribbed shoulder, a glossy cap disc in the accent colour set
/// in the top and a white index on the shoulder.
fn rd() -> KnobModel {
    let black = Material::black_gloss().with_grain(0.03);
    KnobModel::new(vec![
        seg(Dome, 0.0, 0.49, 0.925, 0.905, accent_gloss()),
        seg(Line, 0.49, 0.51, 0.905, 0.885, accent_gloss()),
        seg(Flat, 0.51, 0.56, 0.90, 0.90, black),
        seg(Dome, 0.56, 0.84, 0.90, 0.66, black),
        seg(Line, 0.84, 0.90, 0.66, 0.08, black),
        seg(Dome, 0.90, 0.93, 0.08, 0.02, black),
    ])
    .with_angular(Angular::knurl(36, 0.025, 0.62))
    .with_edge(0.02)
    .with_pointer(Pointer::line(0.58, 0.88, 0.06, white()))
}

/// Flat-topped silver pointer: a wide tapered bar on a low round hub, dark index.
fn ptr() -> KnobModel {
    let alu = Material::satin_aluminium();
    let bar = Part::new(
        vec![seg(Flat, 0.0, 0.93, 0.82, 0.82, alu), seg(Line, 0.93, 0.98, 0.82, 0.72, polished()), seg(Line, 0.98, 1.0, 0.72, 0.10, alu)],
        Plan::Bar { tip: 1.0, tail: 0.60, tip_width: 0.15, tail_width: 0.36, rounding: 0.06 },
    );
    KnobModel::new(vec![seg(Flat, 0.0, 0.54, 0.30, 0.30, alu), seg(Line, 0.54, 0.61, 0.30, 0.22, alu), seg(Line, 0.61, 0.64, 0.22, 0.03, alu)])
        .with_part(bar)
        .with_edge(0.014)
        .with_pointer(Pointer::groove(0.25, 0.95, 0.045, 0.012, Material::ink()))
}

/// Chunky dome moulded in the item colour on a black base ring, with a white dimple.
fn dm() -> KnobModel {
    let body = Material::plastic(0.5, 0.5, 0.5, 0.45).with_accent_shade(1.0, 0.85).with_coat(0.7).with_grain(0.02);
    let base = Material::black_matte();
    KnobModel::new(vec![
        seg(Dome, 0.0, 0.80, 0.88, 0.50, body),
        seg(Line, 0.80, 0.84, 0.50, 0.12, body),
        seg(Flat, 0.84, 0.92, 0.10, 0.10, base),
        seg(Dome, 0.92, 0.95, 0.10, 0.02, base),
    ])
    .with_edge(0.03)
    .with_pointer(Pointer::dot(0.54, 0.14, 0.012, white()))
}

/// Square keycap with rounded corners, sloped sides, a dished top and a legend line.
fn sq() -> KnobModel {
    let cap = Material::plastic(0.15, 0.16, 0.17, 0.68).with_coat(0.08);
    KnobModel::new(vec![
        seg(Flat, 0.0, 0.20, 0.50, 0.50, cap),
        seg(Line, 0.20, 0.45, 0.50, 0.53, cap),
        seg(Line, 0.45, 0.66, 0.53, 0.58, cap),
        seg(Flat, 0.66, 0.72, 0.58, 0.58, cap),
        seg(Dome, 0.72, 0.80, 0.58, 0.50, cap),
        seg(Line, 0.80, 0.97, 0.50, 0.04, cap),
    ])
    .with_plan(Plan::Polygon { sides: 4, rounding: 0.35 })
    .with_edge(0.02)
    .with_pointer(Pointer::line(0.10, 0.56, 0.065, white()))
}

/// Large black skirt sweeping out from a small crown; a thick white index runs from the
/// crown to the edge.
fn fat() -> KnobModel {
    let black = Material::plastic(0.07, 0.07, 0.075, 0.4).with_coat(0.5).with_grain(0.03);
    KnobModel::new(vec![
        seg(Dome, 0.0, 0.34, 0.80, 0.78, black),
        seg(Dome, 0.34, 0.42, 0.78, 0.70, black),
        seg(Line, 0.42, 0.50, 0.70, 0.50, black),
        seg(Fillet, 0.50, 0.94, 0.50, 0.10, black),
        seg(Dome, 0.94, 1.0, 0.10, 0.02, black),
    ])
    .with_edge(0.02)
    .with_pointer(Pointer::line(0.08, 0.97, 0.10, white()))
}

/// Black octagonal bakelite: ribbed sloping faces around a glossy centre dome.
fn pag() -> KnobModel {
    let black = Material::black_gloss().with_grain(0.04);
    KnobModel::new(vec![seg(Flat, 0.0, 0.58, 0.34, 0.34, black), seg(Line, 0.58, 0.90, 0.34, 0.22, black), seg(Line, 0.90, 0.97, 0.22, 0.03, black)])
        .with_under(vec![seg(Dome, 0.0, 0.46, 0.74, 0.36, Material::black_gloss())])
        .with_plan(Plan::Polygon { sides: 8, rounding: 0.12 })
        .with_angular(Angular { r_out: 0.90, ..Angular::flutes(32, 0.02, 0.56) })
        .with_edge(0.016)
        .with_pointer(Pointer::line(0.54, 0.88, 0.055, accent_paint()))
}

/// Bar lever on a round hub, tipped with the accent colour.
fn lvr() -> KnobModel {
    let black = Material::black_gloss().with_grain(0.03);
    let bar = Part::new(
        vec![seg(Flat, 0.0, 0.92, 0.40, 0.40, black), seg(Dome, 0.92, 1.0, 0.40, 0.10, black)],
        Plan::Bar { tip: 1.0, tail: 0.30, tip_width: 0.11, tail_width: 0.15, rounding: 0.08 },
    );
    KnobModel::new(vec![seg(Dome, 0.0, 0.28, 0.54, 0.51, black), seg(Dome, 0.28, 0.36, 0.51, 0.42, black), seg(Line, 0.36, 0.39, 0.42, 0.03, black)])
        .with_part(bar)
        .with_edge(0.016)
        .with_pointer(Pointer::line(0.70, 0.95, 0.11, accent_paint()))
}

/// Small matte black body with a short tab that carries the white index.
fn tab() -> KnobModel {
    let black = Material::black_matte().with_grain(0.03);
    let tab = Part::new(
        vec![seg(Flat, 0.0, 0.90, 0.34, 0.34, black), seg(Line, 0.90, 1.0, 0.34, 0.12, black)],
        Plan::Bar { tip: 0.98, tail: 0.0, tip_width: 0.09, tail_width: 0.12, rounding: 0.05 },
    );
    KnobModel::new(vec![seg(Dome, 0.0, 0.48, 0.70, 0.64, black), seg(Dome, 0.48, 0.58, 0.64, 0.50, black), seg(Line, 0.58, 0.62, 0.50, 0.03, black)])
        .with_part(tab)
        .with_edge(0.016)
        .with_pointer(Pointer::line(0.30, 0.94, 0.05, white()))
}

/// Wide black knurled skirt around a white cap with a black index.
fn wh() -> KnobModel {
    let cap = Material::plastic(0.93, 0.93, 0.91, 0.45).with_coat(0.5);
    let skirt = Material::plastic(0.075, 0.075, 0.08, 0.38);
    KnobModel::new(vec![
        seg(Dome, 0.0, 0.60, 0.80, 0.775, cap),
        seg(Line, 0.60, 0.63, 0.775, 0.74, cap),
        seg(Flat, 0.63, 0.70, 0.72, 0.72, skirt),
        seg(Line, 0.70, 0.96, 0.72, 0.10, skirt),
        seg(Dome, 0.96, 1.0, 0.10, 0.02, skirt),
    ])
    .with_angular(Angular::knurl(44, 0.035, 0.70))
    .with_edge(0.012)
    .with_pointer(Pointer::line(0.12, 0.58, 0.06, Material::ink()))
}

/// Dark anodised cylinder with a coarse straight knurl and a turned aluminium cap.
fn knl() -> KnobModel {
    let cap = Material::spun_aluminium();
    let body = Material::anodised(0.13, 0.13, 0.14);
    KnobModel::new(vec![
        seg(Dome, 0.0, 0.66, 1.0, 0.99, cap),
        seg(Line, 0.66, 0.70, 0.99, 0.95, polished()),
        seg(Line, 0.70, 0.84, 0.95, 0.90, body),
        seg(Line, 0.84, 0.88, 0.90, 0.08, body),
        seg(Flat, 0.88, 0.92, 0.07, 0.07, body),
        seg(Dome, 0.92, 0.94, 0.07, 0.02, body),
    ])
    .with_angular(Angular { r_out: 0.88, ..Angular::knurl(30, 0.04, 0.74) })
    .with_edge(0.012)
    .with_pointer(Pointer::groove(0.18, 0.66, 0.06, 0.012, accent_paint()))
}

/// Tall narrow crown on a concave black skirt; the bright index runs down the side.
fn skt() -> KnobModel {
    let black = Material::black_gloss().with_grain(0.03);
    KnobModel::new(vec![
        seg(Dome, 0.0, 0.40, 1.0, 0.97, black),
        seg(Dome, 0.40, 0.48, 0.97, 0.90, black),
        seg(Line, 0.48, 0.52, 0.90, 0.55, black),
        seg(Fillet, 0.52, 0.92, 0.55, 0.12, black),
        seg(Line, 0.92, 0.96, 0.12, 0.02, black),
    ])
    .with_angular(Angular { r_out: 0.92, ..Angular::knurl(48, 0.015, 0.62) })
    .with_edge(0.02)
    .with_pointer(Pointer::line(0.20, 0.94, 0.06, accent_paint()))
}

/// Large knob moulded in the item colour with broad ribs that run up onto the crown.
fn rib() -> KnobModel {
    let body = Material::plastic(0.5, 0.5, 0.5, 0.5).with_accent_shade(1.0, 0.8).with_coat(0.35);
    KnobModel::new(vec![
        seg(Dome, 0.0, 0.50, 0.80, 0.74, body),
        seg(Dome, 0.50, 0.84, 0.74, 0.50, body),
        seg(Line, 0.84, 0.94, 0.50, 0.06, body),
        seg(Dome, 0.94, 0.97, 0.06, 0.02, body),
    ])
    .with_angular(Angular::flutes(26, 0.045, 0.50))
    .with_edge(0.02)
    .with_pointer(Pointer::line(0.10, 0.50, 0.06, white()))
}

/// Glossy dark cap on a flared chrome skirt, white index.
fn chr() -> KnobModel {
    let cr = Material::metal(0.95, 0.95, 0.96, 0.10);
    let cap = Material::black_gloss();
    KnobModel::new(vec![seg(Dome, 0.0, 0.48, 0.95, 0.86, cap), seg(Dome, 0.48, 0.58, 0.86, 0.72, cap), seg(Line, 0.58, 0.61, 0.72, 0.36, cap)])
        .with_under(vec![seg(Flat, 0.0, 0.60, 0.40, 0.40, cr), seg(Fillet, 0.60, 0.94, 0.40, 0.08, cr), seg(Dome, 0.94, 0.99, 0.08, 0.02, cr)])
        .with_edge(0.02)
        .with_pointer(Pointer::line(0.14, 0.56, 0.06, white()))
}

/// Round keycap with a dished top and a legend line in the accent colour.
fn key() -> KnobModel {
    let cap = Material::plastic(0.10, 0.10, 0.11, 0.62).with_coat(0.12);
    KnobModel::new(vec![
        seg(Flat, 0.0, 0.20, 0.46, 0.46, cap),
        seg(Line, 0.20, 0.45, 0.46, 0.49, cap),
        seg(Line, 0.45, 0.68, 0.49, 0.55, cap),
        seg(Flat, 0.68, 0.74, 0.55, 0.55, cap),
        seg(Dome, 0.74, 0.84, 0.55, 0.46, cap),
        seg(Line, 0.84, 0.92, 0.46, 0.04, cap),
    ])
    .with_edge(0.02)
    .with_pointer(Pointer::line(0.12, 0.60, 0.07, accent_paint()))
}

/// Polished brass: domed turned face inside a lathe groove, knurled wall, black index.
fn brz() -> KnobModel {
    let face = Material::brass().with_roughness(0.14).with_aniso(0.3).with_rings(0.4);
    let side = Material::brass();
    KnobModel::new(vec![
        seg(Dome, 0.0, 0.64, 0.78, 0.72, face),
        seg(Line, 0.64, 0.68, 0.72, 0.70, side),
        seg(Flat, 0.68, 0.72, 0.66, 0.66, side),
        seg(Dome, 0.72, 0.80, 0.70, 0.64, side),
        seg(Line, 0.80, 0.92, 0.64, 0.10, side),
        seg(Dome, 0.92, 0.95, 0.10, 0.02, side),
    ])
    .with_angular(Angular::knurl(36, 0.04, 0.76))
    .with_edge(0.012)
    .with_pointer(Pointer::groove(0.15, 0.62, 0.06, 0.012, Material::ink()))
}

/// Black chicken-head pointer: slim tall beak with a rounded tail over a round skirt.
fn chk() -> KnobModel {
    let black = Material::black_gloss().with_grain(0.03);
    let beak = Part::new(
        vec![seg(Flat, 0.0, 0.94, 0.94, 0.94, black), seg(Line, 0.94, 0.985, 0.94, 0.74, black), seg(Line, 0.985, 1.0, 0.74, 0.12, black)],
        Plan::Bar { tip: 1.0, tail: 0.62, tip_width: 0.07, tail_width: 0.26, rounding: 0.14 },
    );
    KnobModel::new(vec![seg(Flat, 0.0, 0.58, 0.26, 0.26, black), seg(Dome, 0.58, 0.68, 0.26, 0.14, black), seg(Line, 0.68, 0.70, 0.14, 0.02, black)])
        .with_part(beak)
        .with_edge(0.016)
        .with_pointer(Pointer::line(0.20, 0.97, 0.035, white()))
}

/// Cream bakelite: deeply fluted grip with a domed crown on a rounded skirt, dark index.
fn ivr() -> KnobModel {
    let cream = Material::cream();
    KnobModel::new(vec![seg(Dome, 0.0, 0.50, 0.92, 0.88, cream), seg(Dome, 0.50, 0.70, 0.88, 0.74, cream), seg(Line, 0.70, 0.76, 0.74, 0.14, cream)])
        .with_under(vec![seg(Flat, 0.0, 0.78, 0.18, 0.18, cream), seg(Dome, 0.78, 0.94, 0.18, 0.03, cream)])
        .with_angular(Angular::flutes(16, 0.075, 0.50))
        .with_edge(0.025)
        .with_pointer(Pointer::line(0.14, 0.64, 0.055, Material::plastic(0.16, 0.10, 0.06, 0.6)))
}

/// Pale maple cylinder with a chamfered flat top and a dark inlaid index.
fn mpl() -> KnobModel {
    let maple = Material::maple().with_coat(0.3);
    KnobModel::new(vec![
        seg(Dome, 0.0, 0.72, 0.74, 0.72, maple),
        seg(Line, 0.72, 0.80, 0.72, 0.64, maple),
        seg(Line, 0.80, 0.86, 0.64, 0.08, maple),
        seg(Dome, 0.86, 0.88, 0.08, 0.02, maple),
    ])
    .with_edge(0.02)
    .with_pointer(Pointer::line(0.28, 0.78, 0.06, Material::plastic(0.30, 0.17, 0.09, 0.5).with_coat(0.5)))
}

/// Stacked pair: knurled silver outer ring with a black inner knob on top.
fn dbl() -> KnobModel {
    KnobModel::two_tier(0.34, 0.52, 0.92, Material::satin_aluminium(), Material::black_gloss())
        .with_angular(Angular::knurl(40, 0.03, 0.86))
        .with_edge(0.012)
        .with_pointer(Pointer::line(0.12, 0.50, 0.06, white()))
        .with_mark(Pointer::line(0.62, 0.90, 0.05, Material::ink()))
}

/// Small trimmer: short black body on a flange, with a white-filled slot.
fn mini() -> KnobModel {
    let black = Material::black_matte();
    KnobModel::new(vec![
        seg(Flat, 0.0, 0.40, 0.50, 0.50, black),
        seg(Dome, 0.40, 0.48, 0.50, 0.42, black),
        seg(Line, 0.48, 0.52, 0.42, 0.12, black),
        seg(Flat, 0.52, 0.60, 0.10, 0.10, black),
        seg(Dome, 0.60, 0.63, 0.10, 0.02, black),
    ])
    .with_edge(0.014)
    .with_pointer(Pointer::groove(0.0, 0.44, 0.09, 0.03, white()))
}

/// Tall slim black knob with a straight knurl and a white index over the top edge.
fn tl() -> KnobModel {
    let black = Material::plastic(0.08, 0.08, 0.085, 0.4).with_coat(0.2);
    KnobModel::new(vec![
        seg(Dome, 0.0, 0.52, 1.45, 1.43, black),
        seg(Dome, 0.52, 0.60, 1.43, 1.36, black),
        seg(Line, 0.60, 0.70, 1.36, 0.10, black),
        seg(Dome, 0.70, 0.73, 0.10, 0.02, black),
    ])
    .with_angular(Angular { r_out: 0.70, ..Angular::knurl(32, 0.03, 0.58) })
    .with_edge(0.012)
    .with_pointer(Pointer::line(0.15, 0.69, 0.06, white()))
}

/// Translucent crown in the item colour drawn out into a pointer nose, white indicator.
fn mic() -> KnobModel {
    let body = Material::translucent(0.5, 0.5, 0.5).with_accent(1.0);
    let nose = Part::new(
        vec![seg(Flat, 0.0, 0.85, 0.56, 0.56, body), seg(Dome, 0.85, 1.0, 0.56, 0.10, body)],
        Plan::Bar { tip: 0.98, tail: 0.0, tip_width: 0.10, tail_width: 0.40, rounding: 0.08 },
    );
    KnobModel::new(vec![seg(Dome, 0.0, 0.50, 0.80, 0.70, body), seg(Dome, 0.50, 0.62, 0.70, 0.50, body), seg(Line, 0.62, 0.66, 0.50, 0.03, body)])
        .with_part(nose)
        .with_edge(0.025)
        .with_pointer(Pointer::line(0.25, 0.94, 0.055, white()))
}

/// Lacquered ebony with a turned brass cap in the centre and a pale inlaid index.
fn ebn() -> KnobModel {
    let ebony = Material::wood(0.10, 0.075, 0.06).with_wood(0.5);
    let cap = Material::brass().with_roughness(0.14).with_rings(0.5);
    KnobModel::new(vec![
        seg(Dome, 0.0, 0.28, 0.93, 0.905, cap),
        seg(Line, 0.28, 0.31, 0.905, 0.86, cap),
        seg(Dome, 0.31, 0.84, 0.86, 0.58, ebony),
        seg(Line, 0.84, 0.90, 0.58, 0.06, ebony),
        seg(Dome, 0.90, 0.92, 0.06, 0.02, ebony),
    ])
    .with_edge(0.016)
    .with_pointer(Pointer::line(0.40, 0.86, 0.055, Material::plastic(0.88, 0.78, 0.52, 0.5).with_coat(0.6)))
}

/// Walnut top let into a raised chrome collar, pale index on the wood.
fn wnt() -> KnobModel {
    let walnut = Material::walnut().with_coat(0.4);
    let cr = Material::metal(0.95, 0.95, 0.96, 0.14);
    KnobModel::new(vec![
        seg(Dome, 0.0, 0.62, 0.78, 0.76, walnut),
        seg(Flat, 0.62, 0.64, 0.74, 0.74, walnut),
        seg(Line, 0.64, 0.68, 0.76, 0.80, cr),
        seg(Flat, 0.68, 0.80, 0.80, 0.80, cr),
        seg(Dome, 0.80, 0.92, 0.80, 0.60, cr),
        seg(Line, 0.92, 0.96, 0.60, 0.05, cr),
        seg(Dome, 0.96, 0.98, 0.05, 0.02, cr),
    ])
    .with_edge(0.014)
    .with_pointer(Pointer::line(0.18, 0.60, 0.06, Material::plastic(0.92, 0.86, 0.70, 0.5).with_coat(0.6)))
}

/// Small black rack knob: satin top over a fluted skirt, bright index down the side.
fn rck() -> KnobModel {
    let top = Material::plastic(0.11, 0.11, 0.115, 0.62);
    let skirt = Material::anodised(0.14, 0.14, 0.15);
    KnobModel::new(vec![
        seg(Dome, 0.0, 0.50, 0.84, 0.83, top),
        seg(Line, 0.50, 0.54, 0.83, 0.79, top),
        seg(Line, 0.54, 0.60, 0.79, 0.60, skirt),
        seg(Line, 0.60, 0.80, 0.60, 0.08, skirt),
        seg(Dome, 0.80, 0.83, 0.08, 0.02, skirt),
    ])
    .with_angular(Angular::flutes(18, 0.035, 0.56))
    .with_edge(0.014)
    .with_pointer(Pointer::line(0.12, 0.78, 0.06, accent_paint()))
}
