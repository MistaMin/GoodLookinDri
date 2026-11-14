//! Contact sheet for the 3D knobs: every preset at two sizes and three angles, over a light
//! brushed panel (left half) and a dark blue panel (right half). Writes a binary PPM.
//!
//! `cargo run --release --example knob3d_sheet -- out.ppm`

use std::f32::consts::PI;
use std::time::Instant;

use goodlookindri::color::Color;
use goodlookindri::knob3d::{preset, render, KnobBake, KnobCache, Rig, Sprite, PRESETS};

const HALF: usize = 960;
const ROW: usize = 215;
const ANGLES: [f32; 3] = [-0.8 * PI, -0.2 * PI, 0.55 * PI];

fn hash(x: i32, y: i32) -> f32 {
    let mut h = (x as u32).wrapping_mul(0x9e37_79b1) ^ (y as u32).wrapping_mul(0x85eb_ca6b);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2c1b_3c6d);
    h ^= h >> 12;
    (h & 0xffff) as f32 / 65535.0
}

fn panel(x: usize, y: usize) -> [f32; 3] {
    let (xi, yi) = (x as i32, y as i32);
    if x < HALF {
        // brushed aluminium: long horizontal streaks plus a soft vertical gradient
        let streak = 0.5 * hash(xi / 90, yi) + 0.3 * hash((xi + 40) / 23, yi + 1000) + 0.2 * hash(xi, yi);
        let g = 0.80 + 0.07 * (streak - 0.5) - 0.05 * (y as f32 / (ROW * PRESETS.len()) as f32);
        [g, g, g * 1.005]
    } else {
        let n = 0.012 * (hash(xi, yi) - 0.5);
        let v = 1.0 - 0.12 * (y as f32 / (ROW * PRESETS.len()) as f32);
        [(0.09 + n) * v, (0.20 + n) * v, (0.40 + n) * v]
    }
}

fn blit(img: &mut [u8], w: usize, h: usize, s: &Sprite, cx: i32, cy: i32) {
    let (x0, y0) = (cx - s.origin.0.round() as i32, cy - s.origin.1.round() as i32);
    for sy in 0..s.h as i32 {
        for sx in 0..s.w as i32 {
            let (x, y) = (x0 + sx, y0 + sy);
            if x < 0 || y < 0 || x >= w as i32 || y >= h as i32 {
                continue;
            }
            let p = s.px[(sy * s.w as i32 + sx) as usize];
            let ia = 255 - (p >> 24);
            let o = (y as usize * w + x as usize) * 3;
            for (c, shift) in [16u32, 8, 0].iter().enumerate() {
                let d = img[o + c] as u32;
                img[o + c] = ((p >> shift & 0xff) + (d * ia + 127) / 255).min(255) as u8;
            }
        }
    }
}

fn dots(img: &mut [u8], w: usize, cx: i32, cy: i32, r: f32, ink: [u8; 3]) {
    for i in 0..11 {
        let a = -0.8 * PI + 1.6 * PI * i as f32 / 10.0;
        let (px, py) = (cx as f32 + a.sin() * r * 1.32, cy as f32 - a.cos() * r * 1.32);
        let dr = (r * 0.045).max(1.2);
        for y in (py - dr - 1.0) as i32..=(py + dr + 1.0) as i32 {
            for x in (px - dr - 1.0) as i32..=(px + dr + 1.0) as i32 {
                let d = ((x as f32 + 0.5 - px).powi(2) + (y as f32 + 0.5 - py).powi(2)).sqrt();
                let k = (dr + 0.5 - d).clamp(0.0, 1.0);
                let o = (y as usize * w + x as usize) * 3;
                for c in 0..3 {
                    img[o + c] = (img[o + c] as f32 * (1.0 - k) + ink[c] as f32 * k) as u8;
                }
            }
        }
    }
}

fn main() {
    let out = std::env::args().nth(1).expect("usage: knob3d_sheet <out.ppm>");
    let (w, h) = (HALF * 2, ROW * PRESETS.len());
    let mut img = vec![0u8; w * h * 3];
    for y in 0..h {
        for x in 0..w {
            let p = panel(x, y);
            for c in 0..3 {
                img[(y * w + x) * 3 + c] = (p[c].clamp(0.0, 1.0) * 255.0 + hash(x as i32 + c as i32 * 7, y as i32 + 31) - 0.5).round() as u8;
            }
        }
    }
    let floors = [Color(0xffcc_cccc), Color(0xff17_3366)];
    let cache = KnobCache::global();
    println!("{:<9} {:>9} {:>9} {:>9}   (radius 72; bake and frame only for symmetric bodies)", "preset", "direct ms", "bake ms", "frame ms");
    for (row, code) in PRESETS.iter().enumerate() {
        let model = preset(code).unwrap();
        let accent = Color(match *code {
            "gloss" => 0xff2e_8f6a,
            "strip" => 0xff2f_6fd0,
            "wing" => 0xffc8_3228,
            _ => 0xffe0_a020u32,
        });
        let rig = Rig { floor: floors[0], ..Rig::studio() };
        let ms = |t: Instant| t.elapsed().as_secs_f64() * 1000.0;
        let t = Instant::now();
        let direct = render(&model, &rig, 72.0, ANGLES[1], accent);
        let direct_ms = ms(t);
        let (mut bake_ms, mut frame_ms) = (f64::NAN, f64::NAN);
        if let Some(bake) = KnobBake::new(&model, &rig, 72.0, accent) {
            let t = Instant::now();
            bake.warm();
            bake_ms = ms(t);
            for i in 0..16 {
                bake.frame(i as f32 * 0.0123); // touch every knurl phase once
            }
            let t = Instant::now();
            let n = 16;
            let mut sum = 0u32;
            for i in 0..n {
                sum = sum.wrapping_add(bake.frame(i as f32 * 0.0123).px[direct.px.len() / 2]);
            }
            frame_ms = ms(t) / n as f64;
            assert!(sum != 1);
        }
        println!("{code:<9} {direct_ms:>9.1} {bake_ms:>9.1} {frame_ms:>9.2}");
        let cy = (row * ROW + ROW / 2 - 8) as i32;
        for (half, floor) in floors.iter().enumerate() {
            for (si, radius) in [72.0f32, 30.0].iter().enumerate() {
                for (ai, angle) in ANGLES.iter().enumerate() {
                    let s = cache.get(&model, &rig, *radius, *angle, accent, *floor);
                    let cx = if si == 0 { 120 + ai as i32 * 220 } else { 700 + ai as i32 * 95 } + (half * HALF) as i32;
                    let ink = if half == 0 { [40, 40, 42] } else { [235, 235, 230] };
                    dots(&mut img, w, cx, cy, *radius, ink);
                    blit(&mut img, w, h, &s, cx, cy);
                }
            }
        }
    }
    let (n, bytes) = cache.stats();
    println!("cache: {n} sprites, {:.1} MiB", bytes as f64 / (1 << 20) as f64);
    let mut data = format!("P6\n{w} {h}\n255\n").into_bytes();
    data.extend_from_slice(&img);
    std::fs::write(&out, data).expect("write ppm");
    println!("wrote {out}");
}
