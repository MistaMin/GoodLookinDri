# Dependencies and licensing

GoodLookinDri is MIT licensed, copyright 2026 Marcos Deida. See the root [LICENSE](../LICENSE).

The crate depends only on the Rust standard library. `cargo test` uses the built-in test harness. Nothing is downloaded at build time besides the Rust toolchain you already installed.

Documentation images in `docs/images` are renders of the shared control catalogue, produced by the GoodLookinUI gallery. They are part of this repository and covered by the same MIT license. They are reference pictures, not runtime assets. The library does not load them.

GoodLookinDri does not vendor a plugin framework, a window system, a font, or an FFT library. The spectrum transform is implemented in `src/spectrum.rs`.

If you replay `Scene` through another graphics library, that library keeps its own license. Linking GoodLookinDri does not relicense your plugin.
