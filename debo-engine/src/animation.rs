//! Time-based interpolation helpers used by the face and later UI animations.

pub fn clamp01(value: f32) -> f32 { value.clamp(0.0, 1.0) }
pub fn smoothstep(value: f32) -> f32 { let t = clamp01(value); t * t * (3.0 - 2.0 * t) }
pub fn lerp(a: f32, b: f32, t: f32) -> f32 { a + (b - a) * clamp01(t) }

#[derive(Clone, Copy, Debug)]
pub struct Tween { pub from: f32, pub to: f32, pub elapsed: f32, pub duration: f32 }
impl Tween {
    pub fn new(from: f32, to: f32, duration: f32) -> Self { Self { from, to, elapsed: 0.0, duration: duration.max(0.001) } }
    pub fn tick(&mut self, dt: f32) -> f32 { self.elapsed = (self.elapsed + dt.max(0.0)).min(self.duration); lerp(self.from, self.to, smoothstep(self.elapsed / self.duration)) }
    pub fn finished(&self) -> bool { self.elapsed >= self.duration }
}

#[cfg(test)] mod tests {
    use super::*;
    #[test] fn tween_reaches_target() { let mut t = Tween::new(0.0, 10.0, 1.0); assert!((t.tick(1.0) - 10.0).abs() < f32::EPSILON); assert!(t.finished()); }
    #[test] fn negative_delta_does_not_reverse_time() { let mut t = Tween::new(0.0, 1.0, 1.0); t.tick(-2.0); assert_eq!(t.elapsed, 0.0); }
}
