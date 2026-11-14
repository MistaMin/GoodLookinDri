//! Renders one gallery page to a PPM file, for looking at parts without a window.
//! Usage: cargo run -p goodlookindri-shell --example render_page -- <page 0..5> <out.ppm> [scale] [flat]

use goodlookindri::color::Color;
use goodlookindri::gallery;
use goodlookindri::scene::Scene;
use goodlookindri_shell::raster::Framebuffer;

fn main() {
    let a: Vec<String> = std::env::args().collect();
    if a.len() < 3 {
        eprintln!("usage: render_page <page 0..5> <out.ppm> [scale] [flat]");
        std::process::exit(2);
    }
    let page: usize = a[1].parse().unwrap_or(0);
    let scale: f32 = a.get(3).and_then(|s| s.parse().ok()).unwrap_or(2.0);
    let flat = a.get(4).map(|s| s == "flat").unwrap_or(false);
    let mut scene = Scene::new();
    gallery::paint_page(&mut scene, page, flat, true);
    let mut fb = Framebuffer::new((1280.0 * scale) as usize, (860.0 * scale) as usize);
    fb.clear(Color(0xff14181b));
    fb.replay(&scene, scale);
    std::fs::write(&a[2], fb.to_ppm()).expect("write ppm");
}
