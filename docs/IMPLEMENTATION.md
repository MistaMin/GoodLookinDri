# Implementation plan

## Product requirements

Audience: developers of audio utilities and effects. Plugins live in their own repositories. A rack and console aesthetic combines generated hardware surfaces with analysis displays. Instruments and synths are outside the current scope.

Controls and panels are generated in code. No designer-produced image assets are required. Parameter changes reach DSP immediately even while the graphic is still moving.

Development builds can carry an in-plugin editor. Testers change style, label, colour, type size, and bounds. Developers keep automation ids and stored enum values stable when the presentation changes.

Release users can scale the interface. They cannot edit typography or layout. The accepted design is a CSV file embedded in the plugin.

GoodLookinDri is the Rust implementation of that model. [GoodLookinUI](https://github.com/MistaMin/GoodLookinUI) is the C++ implementation, with a JUCE painting adapter. Design files move between them. The plugin project chooses which one to link. The toolkit does not choose a plugin framework.

Target systems are whatever the host plugin already ships: Apple Silicon and Intel macOS, Windows, and Linux. Format wrappers remain the plugin's responsibility.

## 0.1 — current foundation

Shipped in this repository:

- Design CSV, motion, settings, look table, level tap, spectrum analysis.
- Drawing commands for 47 knob styles in the hardware and flat families, 40 plates and finishes, meters, switches, retro faces, sliders, lamps, and four spectrum looks.
- Gallery page layout and a developer editor state machine with undo, a colour wheel, and atomic autosave.
- Tests for the design fixture, FFT calibration, level tap, and a non-empty command list for every knob style.

Known limits of 0.1:

- Commands approximate a few JUCE gradients. Rim strokes are a single ellipse. Gloss is a diagonal fill. Grub screws are empty, matching the C++ stubs. Text is not rasterised here.
- There is no pixel comparison against `docs/images`.
- Hit-testing exists for the colour wheel only. Knobs, faders, and switches do not yet report drags.
- No window demo is included. The first host plugin is the proof of embedding.

## 0.2 — interaction

Hit-testing and keyboard focus for knobs, faders, switches, and keys. A small standalone gallery that replays `Scene` into a software renderer and writes comparison images. Redo beside undo. Dragging a control updates the CSV through the existing autosave.

## 0.3 — host polish

Dirty-region redraw, cached static plates at the editor scale, and idle suspension when the editor is closed. Accessibility names taken from the design row. A documented replay example for one immediate-mode UI and one retained-mode canvas.

## 0.4 — qualification

Continuous tests on macOS, Windows, and Linux. Pixel thresholds against the gallery images. A second plugin, in its own repository, linking this crate without copying it. DAW smoke tests stay in those plugin repositories.

## Later

Menus, control creation, alignment, and snapping. A routing editor is a DSP feature and does not belong in this crate.
