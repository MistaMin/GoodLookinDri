use goodlookindri::{color::Color, design::*, draw, gallery, knob3d, looks, scene::{Cmd, Point, Scene}};

fn drawing(code: &str, mode: KnobMode) -> Scene {
    let mut scene = Scene::new();
    draw::knob_named_mode(&mut scene, code, Point::new(60.0, 60.0), 18.0, 0.4, Color(0xffc96f4a), false, mode);
    scene
}

#[test]
fn prefixes_roundtrip_and_validate() {
    for style in KNOB_STYLES {
        for (prefix, mode) in [("", KnobMode::Classic), ("flat_", KnobMode::Flat), ("lit_", KnobMode::Lit)] {
            let name = format!("{prefix}{}", style.code);
            assert_eq!(parse_style(&name), (mode, style.code));
            assert_eq!(find_style(&name).unwrap().style, style.style);
            assert_eq!(is_flat_style(&name), mode == KnobMode::Flat);
            let item = Item::new("gain", "gain", "Gain", &name, "#c96f4a", 0.0, 0.0, 60.0, 60.0, 11.0);
            assert_eq!(read_design(&write_design(&[item.clone()]).unwrap()).unwrap(), vec![item]);
        }
    }
    assert_eq!(parse_style("flat"), (KnobMode::Flat, "Brit"));
    assert!(looks::valid_look(&looks::Look {
        parameter: "gain".into(), choice: "one".into(), style: "lit_Mtl".into(),
        colour: String::new(), faceplate: String::new(), plate: String::new(),
    }));
    for code in knob3d::PRESETS {
        assert!(find_style(&format!("lit_{code}")).is_some());
        assert!(drawing(code, KnobMode::Lit).cmds.iter().any(|c| matches!(c, Cmd::Image { .. })));
    }
    assert_eq!(KNOB_STYLES.len(), 47);
}

#[test]
fn lit_catalogue_and_fallback() {
    assert!(gallery::every_style_draws_mode(KnobMode::Lit));
    for style in KNOB_STYLES {
        if draw::lit_preset_for(style.code).is_none() {
            assert_eq!(drawing(style.code, KnobMode::Lit).cmds, drawing(style.code, KnobMode::Classic).cmds);
        }
    }
}

#[test]
fn lit_image_identity_and_placement() {
    let a = drawing("lit_Mtl", KnobMode::Classic);
    let b = drawing("Mtl", KnobMode::Lit);
    assert_eq!(a.cmds, b.cmds);
    let images: Vec<_> = a.cmds.iter().filter_map(|cmd| match cmd {
        Cmd::Image { img, dest, opacity } => Some((img, dest, opacity)), _ => None,
    }).collect();
    assert_eq!(images.len(), 1);
    let (img, dest, opacity) = images[0];
    let sprite = knob3d::KnobCache::global().get_preset("spun", &knob3d::Rig::studio(), 36.0, 0.4, Color(0xffc96f4a), Color(0xff808080)).unwrap();
    assert_eq!(dest.w, img.w as f32 / 2.0);
    assert_eq!(dest.h, img.h as f32 / 2.0);
    assert_eq!(dest.x + sprite.origin.0 / 2.0, 60.0);
    assert_eq!(dest.y + sprite.origin.1 / 2.0, 60.0);
    assert_eq!(*opacity, 1.0);
    assert_eq!(drawing("flat_Mtl", KnobMode::Lit).cmds, drawing("Mtl", KnobMode::Flat).cmds);
}
