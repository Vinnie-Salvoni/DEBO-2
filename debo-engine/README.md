# DEBO custom software graphics engine

A Rust software-rendering foundation for DEBO. The face is rasterised into a CPU-owned pixel buffer using code in this repository. There is no PixiJS, game engine, HTML Canvas renderer, pre-rendered face art, or GPU rendering pipeline.

`minifb` is used only as a thin desktop window and input layer. DEBO's renderer owns the logical framebuffer and implements pixel writes, filled rectangles, ellipses, lines, thick strokes, quadratic curves, polygon filling, restrained 2×2 coverage antialiasing, and crisp nearest-neighbour scaling.

## Current architecture

- `src/renderer.rs` — custom software rasteriser and framebuffer.
- `src/face.rs` — procedural face geometry, expression state, gaze, blinking, and expression transitions.
- `src/appearance.rs` — expression-independent appearance settings, RGB colours, and named eye-colour presets.
- `src/animation.rs` — reusable time-based tween/easing helpers.
- `src/protocol.rs` — versioned JSON command validation, independent of rendering.
- `src/main.rs` — desktop preview/input adapter plus a loopback TCP listener; presents the completed pixel buffer only.
- `tools/debo_control.py` — standard-library Python command-line controller. The face does not depend on Python being installed.
- `src/lib.rs` — engine modules and logical display dimensions (184×96).

The 184×96 canvas is a chosen logical design target inspired by compact robot displays, not a claim about any specific robot panel. The preview is enlarged 5× using nearest-neighbour scaling. Ellipse edges use restrained 2×2 sub-pixel coverage: enough to soften harsh stair-stepping without blurring the whole face.

## Current milestone

The face supports five procedural expressions: neutral, curious, happy, surprised, and sleepy. Eye colour is now a separate appearance property: it can be changed to a named preset or arbitrary RGB value without changing the current expression or animation state. Expression changes trigger a brief vertical squeeze-and-release transition rather than cross-fading two complete images, so the edges remain crisp and don't create ghosted eyes. A new expression request interrupts an existing transition cleanly. Gaze is clamped to its valid range, and manual and automatic blinks remain independent.

## Local face-control protocol (version 1)

The desktop preview opens a TCP listener on **127.0.0.1:45821**. It accepts newline-delimited JSON (one command per line), with a 4,096-byte maximum message. The listener parses and validates commands on background threads; a bounded queue passes valid commands to the main thread, so JSON parsing and socket reads do not run in the render loop. The listener binds only to loopback, not the LAN.

Every request must include `"version": 1`. Supported commands:

```json
{"version":1,"command":"set_expression","value":"happy"}
{"version":1,"command":"set_eye_color","value":"purple"}
{"version":1,"command":"gaze","x":-0.5,"y":0.25}
{"version":1,"command":"blink"}
{"version":1,"command":"centre_gaze"}
```

Valid expression names are `neutral`, `curious`, `happy`, `surprised`, and `sleepy`. Eye-colour names are `cyan`, `blue`, `purple`, `green`, `amber`, and `pink`. Gaze coordinates must each be finite numbers in `[-1, 1]`. Unknown commands, names, versions, invalid JSON, out-of-range gaze, oversized messages, and a full command queue are rejected. Responses are one JSON line, for example `{"version":1,"accepted":true,"status":"queued"}`. `queued` means accepted into the render-thread queue; it is not a guarantee that a visible frame has already been presented.

The protocol is intentionally local and unauthenticated, so do not change the bind address to a public or LAN interface without designing authentication and access controls first.

### Python controller

With the face window running, use a second terminal from the project directory:

```sh
python tools/debo_control.py expression happy
python tools/debo_control.py eye-color purple
python tools/debo_control.py gaze -0.5 0.25
python tools/debo_control.py blink
python tools/debo_control.py centre
```

The Python controller uses only the standard library; it does not need an AI provider, credentials, or internet access. Run its unit tests with:

```sh
python -m unittest discover -s tools -p 'test_*.py' -v
```

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
- `E` cycle eye colour (cyan → blue → purple → green → amber → pink)
- `Esc` close

## Tests

Unit tests cover renderer clipping and primitive drawing, crisp scaling, antialiased edges, tweening, face defaults, expression transitions, transition interruption, repeated expression requests, gaze clamping, independent appearance changes, RGB packing, named colour validation, palette cycling, protocol version and payload validation, and safe rejection of unknown commands. Python unit tests cover controller message creation and argument validation.

## Not implemented yet

Fonts/text rasterisation, weather layouts, persistent user preferences, speech, camera, Tauri packaging, and physical display drivers are not implemented yet. The versioned local command protocol and a small Python test controller are implemented, but this is not yet a conversational brain or an authenticated general-purpose IPC service. Eye-colour changes are available in the engine and preview, but voice control is not implemented. Later, the brain can map a request such as “make your eyes purple” to the same named-colour API, with validation outside the render loop.

## Development rule

Keep the logical framebuffer independent of the desktop window. Add tests for every new raster primitive or state transition before adding more elaborate effects. No AI inference, network request, or speech operation should ever run inside the rendering loop.
