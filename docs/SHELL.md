# goodlookindri-shell

Window, input and a software rasteriser for GoodLookinDri scenes. A tool that is not a plugin
(a manager, an installer, a standalone utility) uses it to show a `Scene` on screen.

```rust
use goodlookindri_shell::{run, App, Config, Event};

struct MyApp;
impl App for MyApp {
    fn event(&mut self, e: Event) -> bool { false }              // true = repaint
    fn paint(&mut self, scene: &mut Scene, w: f32, h: f32) { /* draw with goodlookindri */ }
}
std::process::exit(run(Config { title: "Tool".into(), width: 560, height: 300, smoke_ms: None }, Box::new(MyApp)));
```

`render_offscreen` draws one frame without a window, for tests and machines with no display.

## Systems

| Lane | Systems | How |
| --- | --- | --- |
| Windows | 7 SP1 to 11, x64 and x86 | `*-win7-windows-msvc` targets, GDI. Arm64 from Windows 10 on Arm. |
| macOS modern | 10.12 to current, Intel and Apple silicon | stable Rust, Cocoa, one universal binary |
| macOS legacy | 10.6.8 to 10.11, Intel | pinned nightly, custom target, see `legacy/README.md` |

No GPU API is used. The window blits a software-rendered buffer (GDI `StretchDIBits`,
CoreGraphics `CGImage`).

## What is there

- Win32 and Cocoa windows, 16 ms timer, mouse move/down/up, system DPI and Retina scale.
- Replay of every `Cmd`: gradients, ellipses, round rects, arcs, polygons, strokes, clips.
  One pixel of anti-aliasing from signed distance fields.
- Text through a 5x7 placeholder bitmap face.
- `--smoke` mode in the sample: opens the window, draws, exits 0 only if the frame is not blank.

## Known limits (Phase 0)

- Text is a placeholder face. Outline font, measurement and wrapping come next.
- The rasteriser is simple and not fast on large, busy scenes. Dirty regions and cached plates
  are in the 0.3 milestone.
- Keyboard input, wheel, per-monitor DPI on Windows 8.1+, and window-resize behaviour on
  Windows 7 are not done or not tested.
- Nothing has run on a real Windows 7 or macOS 10.6 to 10.11 machine yet.
