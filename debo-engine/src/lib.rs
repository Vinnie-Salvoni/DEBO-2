//! DEBO's small, custom software graphics engine.
//!
//! The engine owns its pixel buffer and all rasterisation. `minifb` is used only
//! to present the finished buffer in a desktop window and forward keyboard input.

pub mod animation;
pub mod appearance;
pub mod face;
pub mod renderer;
pub mod protocol;

pub const LOGICAL_WIDTH: usize = 184;
pub const LOGICAL_HEIGHT: usize = 96;
pub const BACKGROUND: u32 = 0x080D16;
