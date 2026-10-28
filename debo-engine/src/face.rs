//! DEBO face geometry. Expressions are parameters, not image assets.
use crate::{renderer::Renderer, BACKGROUND, EYE_COLOR, LOGICAL_HEIGHT, LOGICAL_WIDTH};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Expression { Neutral, Curious, Happy, Surprised, Sleepy }
impl Expression {
    pub const ALL: [Expression; 5] = [Self::Neutral, Self::Curious, Self::Happy, Self::Surprised, Self::Sleepy];
    pub fn name(self) -> &'static str { match self { Self::Neutral=>"neutral", Self::Curious=>"curious", Self::Happy=>"happy", Self::Surprised=>"surprised", Self::Sleepy=>"sleepy" } }
    pub fn next(self) -> Self { let i = Self::ALL.iter().position(|e| *e == self).unwrap_or(0); Self::ALL[(i + 1) % Self::ALL.len()] }
}

#[derive(Clone, Debug)]
pub struct Face { pub expression: Expression, pub gaze_x: f32, pub gaze_y: f32, pub blink: f32, pub elapsed: f32 }
impl Default for Face { fn default() -> Self { Self { expression: Expression::Neutral, gaze_x: 0.0, gaze_y: 0.0, blink: 0.0, elapsed: 0.0 } } }
impl Face {
    pub fn set_expression(&mut self, expression: Expression) { self.expression = expression; }
    pub fn gaze(&mut self, x: f32, y: f32) { self.gaze_x = x.clamp(-1.0, 1.0); self.gaze_y = y.clamp(-1.0, 1.0); }
    pub fn trigger_blink(&mut self) { self.blink = 1.0; }
    pub fn tick(&mut self, dt: f32) {
        let dt = dt.clamp(0.0, 0.1); self.elapsed += dt;
        if self.blink > 0.0 { self.blink = (self.blink - dt * 5.5).max(0.0); }
        // A subtle automatic blink about every 4.7 seconds, with no eyelid overlay.
        let phase = self.elapsed % 4.7;
        if phase < dt { self.trigger_blink(); }
    }

    pub fn render(&self, r: &mut Renderer) {
        r.clear(BACKGROUND);
        let (base_w, base_h, gap, y) = match self.expression {
            Expression::Neutral => (22.0, 29.0, 31.0, 47.0),
            Expression::Curious => (21.0, 29.0, 32.0, 45.0),
            Expression::Happy => (24.0, 20.0, 31.0, 47.0),
            Expression::Surprised => (19.0, 25.0, 33.0, 46.0),
            Expression::Sleepy => (23.0, 13.0, 31.0, 49.0),
        };
        let cx = LOGICAL_WIDTH as f32 / 2.0;
        let blink_scale = if self.blink > 0.0 { (1.0 - self.blink).max(0.035) } else { 1.0 };
        let (lw, lh) = if self.expression == Expression::Curious { (base_w * 0.85, base_h * 0.9) } else { (base_w, base_h) };
        let (rw, rh) = if self.expression == Expression::Curious { (base_w * 1.15, base_h * 1.12) } else { (base_w, base_h) };
        let left = (cx - gap - lw / 2.0 + self.gaze_x * 3.0, y + self.gaze_y * 2.0);
        let right = (cx + gap - rw / 2.0 + self.gaze_x * 3.0, y + self.gaze_y * 2.0);
        if self.expression == Expression::Happy {
            // Cozmo-like squeezed-up happy eyes: curved strokes, no visible eyelids.
            let lift = 9.0;
            r.quadratic_stroke((left.0 - 1.0, left.1 + 5.0), (left.0 + lw / 2.0, left.1 - lift), (left.0 + lw + 1.0, left.1 + 5.0), 3.0, EYE_COLOR);
            r.quadratic_stroke((right.0 - 1.0, right.1 + 5.0), (right.0 + rw / 2.0, right.1 - lift), (right.0 + rw + 1.0, right.1 + 5.0), 3.0, EYE_COLOR);
        } else {
            let mut left_h = lh * blink_scale; let mut right_h = rh * blink_scale;
            if self.expression == Expression::Surprised { left_h *= 1.35; right_h *= 1.35; }
            r.fill_ellipse(left.0 + lw / 2.0, left.1, lw / 2.0, left_h / 2.0, EYE_COLOR);
            r.fill_ellipse(right.0 + rw / 2.0, right.1, rw / 2.0, right_h / 2.0, EYE_COLOR);
        }
        // Small low-contrast status dots make it easy to verify the renderer is live.
        let dot_color = 0x1A3845;
        r.fill_ellipse(8.0, LOGICAL_HEIGHT as f32 - 7.0, 1.5, 1.5, dot_color);
        r.fill_ellipse(LOGICAL_WIDTH as f32 - 8.0, LOGICAL_HEIGHT as f32 - 7.0, 1.5, 1.5, dot_color);
    }
}
