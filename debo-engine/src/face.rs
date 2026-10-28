//! DEBO face geometry. Expressions are parameters, not image assets.
use crate::{animation::smoothstep, appearance::Appearance, renderer::Renderer, BACKGROUND, LOGICAL_HEIGHT, LOGICAL_WIDTH};

const EXPRESSION_TRANSITION_SECONDS: f32 = 0.24;
const TRANSITION_SQUASH: f32 = 0.34;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Expression {
    Neutral,
    Curious,
    Happy,
    Surprised,
    Sleepy,
}

impl Expression {
    pub const ALL: [Expression; 5] = [
        Self::Neutral,
        Self::Curious,
        Self::Happy,
        Self::Surprised,
        Self::Sleepy,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::Neutral => "neutral",
            Self::Curious => "curious",
            Self::Happy => "happy",
            Self::Surprised => "surprised",
            Self::Sleepy => "sleepy",
        }
    }

    pub fn next(self) -> Self {
        let i = Self::ALL.iter().position(|e| *e == self).unwrap_or(0);
        Self::ALL[(i + 1) % Self::ALL.len()]
    }
}

#[derive(Clone, Debug)]
pub struct Face {
    pub expression: Expression,
    pub gaze_x: f32,
    pub gaze_y: f32,
    pub blink: f32,
    pub elapsed: f32,
    pub appearance: Appearance,
    transition_elapsed: f32,
    transition_active: bool,
}

impl Default for Face {
    fn default() -> Self {
        Self {
            expression: Expression::Neutral,
            gaze_x: 0.0,
            gaze_y: 0.0,
            blink: 0.0,
            elapsed: 0.0,
            appearance: Appearance::default(),
            transition_elapsed: EXPRESSION_TRANSITION_SECONDS,
            transition_active: false,
        }
    }
}

impl Face {
    /// Request an expression. A new request interrupts any current transition.
    /// Re-requesting the current expression does not restart the animation.
    pub fn set_expression(&mut self, expression: Expression) {
        if expression == self.expression {
            return;
        }
        self.expression = expression;
        self.transition_elapsed = 0.0;
        self.transition_active = true;
    }

    pub fn gaze(&mut self, x: f32, y: f32) {
        self.gaze_x = x.clamp(-1.0, 1.0);
        self.gaze_y = y.clamp(-1.0, 1.0);
    }

    /// Set an RGB eye colour without changing expression or animation state.
    pub fn set_eye_color(&mut self, color: crate::appearance::Rgb) {
        self.appearance.set_eye_color(color);
    }

    /// Set a built-in colour by name. Returns false and leaves the colour unchanged
    /// when the name is not in the supported palette.
    pub fn set_named_eye_color(&mut self, name: &str) -> bool {
        self.appearance.named_eye_color(name)
    }

    /// Advance to the next built-in eye colour. Returns its name and colour.
    pub fn cycle_eye_color(&mut self) -> crate::appearance::EyeColorPreset {
        self.appearance.cycle_eye_color()
    }

    pub fn trigger_blink(&mut self) {
        self.blink = 1.0;
    }

    pub fn transition_progress(&self) -> f32 {
        if !self.transition_active {
            return 1.0;
        }
        (self.transition_elapsed / EXPRESSION_TRANSITION_SECONDS).clamp(0.0, 1.0)
    }

    fn transition_squash(&self) -> f32 {
        if !self.transition_active {
            return 1.0;
        }
        let progress = self.transition_progress();
        if progress < 0.5 {
            let t = smoothstep(progress * 2.0);
            1.0 + (TRANSITION_SQUASH - 1.0) * t
        } else {
            let t = smoothstep((progress - 0.5) * 2.0);
            TRANSITION_SQUASH + (1.0 - TRANSITION_SQUASH) * t
        }
    }

    pub fn tick(&mut self, dt: f32) {
        let dt = dt.clamp(0.0, 0.1);
        self.elapsed += dt;

        if self.transition_active {
            self.transition_elapsed += dt;
            if self.transition_elapsed >= EXPRESSION_TRANSITION_SECONDS {
                self.transition_elapsed = EXPRESSION_TRANSITION_SECONDS;
                self.transition_active = false;
            }
        }

        if self.blink > 0.0 {
            self.blink = (self.blink - dt * 5.5).max(0.0);
        }

        // A subtle automatic blink about every 4.7 seconds, with no eyelid overlay.
        let phase = self.elapsed % 4.7;
        if phase < dt {
            self.trigger_blink();
        }
    }

    pub fn render(&self, r: &mut Renderer) {
        r.clear(BACKGROUND);
        let eye_color = self.appearance.eye_color.packed();
        let (base_w, base_h, gap, y) = match self.expression {
            Expression::Neutral => (22.0, 29.0, 31.0, 47.0),
            Expression::Curious => (21.0, 29.0, 32.0, 45.0),
            Expression::Happy => (24.0, 20.0, 31.0, 47.0),
            Expression::Surprised => (19.0, 25.0, 33.0, 46.0),
            Expression::Sleepy => (23.0, 13.0, 31.0, 49.0),
        };

        let cx = LOGICAL_WIDTH as f32 / 2.0;
        let blink_scale = if self.blink > 0.0 {
            (1.0 - self.blink).max(0.035)
        } else {
            1.0
        };
        // Expression changes use a brief vertical squeeze, then spring back to shape.
        // This keeps the renderer crisp and avoids cross-fading two sets of ghosted eyes.
        let vertical_scale = blink_scale * self.transition_squash();
        let (lw, lh) = if self.expression == Expression::Curious {
            (base_w * 0.85, base_h * 0.9)
        } else {
            (base_w, base_h)
        };
        let (rw, rh) = if self.expression == Expression::Curious {
            (base_w * 1.15, base_h * 1.12)
        } else {
            (base_w, base_h)
        };
        let left = (
            cx - gap - lw / 2.0 + self.gaze_x * 3.0,
            y + self.gaze_y * 2.0,
        );
        let right = (
            cx + gap - rw / 2.0 + self.gaze_x * 3.0,
            y + self.gaze_y * 2.0,
        );

        if self.expression == Expression::Happy {
            // Cozmo-like squeezed-up happy eyes: curved strokes, no visible eyelids.
            let lift = 9.0 * vertical_scale;
            let stroke_radius = 3.0 * vertical_scale.max(0.25);
            r.quadratic_stroke(
                (left.0 - 1.0, left.1 + 5.0 * vertical_scale),
                (left.0 + lw / 2.0, left.1 - lift),
                (left.0 + lw + 1.0, left.1 + 5.0 * vertical_scale),
                stroke_radius,
                eye_color,
            );
            r.quadratic_stroke(
                (right.0 - 1.0, right.1 + 5.0 * vertical_scale),
                (right.0 + rw / 2.0, right.1 - lift),
                (right.0 + rw + 1.0, right.1 + 5.0 * vertical_scale),
                stroke_radius,
                eye_color,
            );
        } else {
            let mut left_h = lh * vertical_scale;
            let mut right_h = rh * vertical_scale;
            if self.expression == Expression::Surprised {
                left_h *= 1.35;
                right_h *= 1.35;
            }
            r.fill_ellipse(left.0 + lw / 2.0, left.1, lw / 2.0, left_h / 2.0, eye_color);
            r.fill_ellipse(right.0 + rw / 2.0, right.1, rw / 2.0, right_h / 2.0, eye_color);
        }

        // Small low-contrast status dots make it easy to verify the renderer is live.
        let dot_color = 0x1A3845;
        r.fill_ellipse(8.0, LOGICAL_HEIGHT as f32 - 7.0, 1.5, 1.5, dot_color);
        r.fill_ellipse(LOGICAL_WIDTH as f32 - 8.0, LOGICAL_HEIGHT as f32 - 7.0, 1.5, 1.5, dot_color);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_face_is_neutral_and_not_transitioning() {
        let face = Face::default();
        assert_eq!(face.expression, Expression::Neutral);
        assert!(!face.transition_active);
        assert_eq!(face.transition_progress(), 1.0);
    }

    #[test]
    fn expression_change_starts_a_transition() {
        let mut face = Face::default();
        face.set_expression(Expression::Happy);
        assert_eq!(face.expression, Expression::Happy);
        assert!(face.transition_active);
        assert_eq!(face.transition_progress(), 0.0);
    }

    #[test]
    fn transition_finishes_after_its_duration() {
        let mut face = Face::default();
        face.set_expression(Expression::Curious);
        face.tick(0.1);
        face.tick(0.1);
        face.tick(0.1);
        assert!(!face.transition_active);
        assert_eq!(face.transition_progress(), 1.0);
        assert!((face.transition_squash() - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn setting_same_expression_does_not_restart_transition() {
        let mut face = Face::default();
        face.set_expression(Expression::Happy);
        face.tick(0.08);
        let progress = face.transition_progress();
        face.set_expression(Expression::Happy);
        assert_eq!(face.transition_progress(), progress);
    }

    #[test]
    fn new_expression_interrupts_current_transition() {
        let mut face = Face::default();
        face.set_expression(Expression::Happy);
        face.tick(0.08);
        face.set_expression(Expression::Sleepy);
        assert_eq!(face.expression, Expression::Sleepy);
        assert_eq!(face.transition_progress(), 0.0);
    }

    #[test]
    fn eye_colour_is_independent_of_expression_changes() {
        let mut face = Face::default();
        assert!(face.set_named_eye_color("purple"));
        face.set_expression(Expression::Happy);
        assert_eq!(face.appearance.eye_color, crate::appearance::EYE_COLOR_PRESETS[2].color);
        assert!(face.transition_active);
    }

    #[test]
    fn custom_eye_colour_does_not_change_expression() {
        let mut face = Face::default();
        face.set_eye_color(crate::appearance::Rgb::new(1, 2, 3));
        assert_eq!(face.expression, Expression::Neutral);
        assert_eq!(face.appearance.eye_color.packed(), 0x010203);
    }

    #[test]
    fn gaze_is_clamped_to_supported_range() {
        let mut face = Face::default();
        face.gaze(4.0, -3.0);
        assert_eq!(face.gaze_x, 1.0);
        assert_eq!(face.gaze_y, -1.0);
    }
}
