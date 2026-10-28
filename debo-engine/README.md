# DEBO custom software graphics engine

A first real Rust engine foundation for DEBO. The face is rasterised into a CPU-owned pixel buffer using code in this repository. There is no PixiJS, game engine, HTML Canvas renderer, pre-rendered face art, or GPU rendering pipeline.

`minifb` is used only as a thin desktop window and input layer. DEBO's renderer owns the logical framebuffer and implements pixel writes, filled rectangles, ellipses, lines, thick strokes, quadratic curves, polygon filling, coverage-antialiased ellipses and strokes, and nearest-neighbour scaling, plus optional bilinear scaling for callers that explicitly want softness.

## Current architecture

- `src/renderer.rs` — custom software rasteriser and framebuffer.
- `src/face.rs` — procedural face geometry, expression parameters, gaze, and blink state.
- `src/animation.rs` — time-based tween/easing helpers for future transitions.
- `src/main.rs` — desktop preview/input adapter only; it presents the completed buffer.
- `src/lib.rs` — engine module and logical display dimensions (184×96).

The 184×96 canvas is a chosen logical design target inspired by compact robot displays, not a claim about any specific robot panel. The preview is enlarged 5× with crisp nearest-neighbour scaling. Ellipse edges use restrained 2×2 sub-pixel coverage: enough to soften harsh stair-stepping, without bilinear resampling that makes the eyes look blurry. The overall aesthetic stays deliberately compact, crisp, and retro rather than becoming high-resolution vector art.

## Requirements

- Rust stable toolchain (`rustup` recommended)
- A desktop environment supported by `minifb`
- Network access on first build unless dependencies are already cached

Install Rust: https://rustup.rs/

## Build and run

From this directory:

```sh
cargo test
cargo run
```

Release build:

```sh
cargo build --release
```

## Controls

- `1` neutral
- `2` curious
- `3` happy (curved, squeezed-up eyes; no eyelid overlays)
- `4` surprised
- `5` sleepy
- `Tab` cycle expressions
- `Space` trigger a blink
- Arrow keys move gaze
- `C` centre gaze
- `Esc` close

## What is and isn't implemented

Implemented in this starter: CPU framebuffer, basic primitive rasterisation, expression geometry, gaze offsets, blink animation, a minimal desktop preview, and unit tests for renderer primitives, lightly antialiased ellipse edges, crisp scaling, and tweening.

Not implemented yet: Python/AI brain connection, versioned IPC protocol, weather layouts, fonts/text rasterisation, image loading, persistent expression-transition timelines, audio/camera, Tauri packaging, or physical display drivers. Those should be added behind the renderer API rather than moving face drawing into the window layer.

## Development notes

This is a foundation, not a finished production renderer. Rendering currently favours simplicity and auditability over optimisation. When expanding it, keep the logical framebuffer independent of the window size and add tests for every raster primitive before adding more elaborate effects.
