use minifb::{Key, KeyRepeat, Window, WindowOptions};
use std::time::Instant;
use debo_engine::{face::{Expression, Face}, renderer::Renderer, LOGICAL_HEIGHT, LOGICAL_WIDTH};

const SCALE: usize = 5;
const TITLE: &str = "DEBO — custom software face engine";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let width = LOGICAL_WIDTH * SCALE; let height = LOGICAL_HEIGHT * SCALE;
    let mut window = Window::new(TITLE, width, height, WindowOptions { resize: false, ..WindowOptions::default() })?;
    window.limit_update_rate(Some(std::time::Duration::from_micros(16_667)));
    let mut renderer = Renderer::new(LOGICAL_WIDTH, LOGICAL_HEIGHT, 0x080D16);
    let mut output = vec![0_u32; width * height];
    let mut face = Face::default();
    let mut last = Instant::now();

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let now = Instant::now(); let dt = now.duration_since(last).as_secs_f32(); last = now;
        if window.is_key_pressed(Key::Key1, KeyRepeat::No) { face.set_expression(Expression::Neutral); }
        if window.is_key_pressed(Key::Key2, KeyRepeat::No) { face.set_expression(Expression::Curious); }
        if window.is_key_pressed(Key::Key3, KeyRepeat::No) { face.set_expression(Expression::Happy); }
        if window.is_key_pressed(Key::Key4, KeyRepeat::No) { face.set_expression(Expression::Surprised); }
        if window.is_key_pressed(Key::Key5, KeyRepeat::No) { face.set_expression(Expression::Sleepy); }
        if window.is_key_pressed(Key::Space, KeyRepeat::No) { face.trigger_blink(); }
        if window.is_key_pressed(Key::Left, KeyRepeat::No) { face.gaze(-1.0, face.gaze_y); }
        if window.is_key_pressed(Key::Right, KeyRepeat::No) { face.gaze(1.0, face.gaze_y); }
        if window.is_key_pressed(Key::Up, KeyRepeat::No) { face.gaze(face.gaze_x, -1.0); }
        if window.is_key_pressed(Key::Down, KeyRepeat::No) { face.gaze(face.gaze_x, 1.0); }
        if window.is_key_pressed(Key::C, KeyRepeat::No) { face.gaze(0.0, 0.0); }
        if window.is_key_pressed(Key::Tab, KeyRepeat::No) { face.set_expression(face.expression.next()); }
        face.tick(dt);
        face.render(&mut renderer);
        output.copy_from_slice(&renderer.upscale(SCALE));
        window.update_with_buffer(&output, width, height)?;
    }
    Ok(())
}
