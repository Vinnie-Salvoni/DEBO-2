//! Appearance settings independent of expression and animation state.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    pub const fn new(r: u8, g: u8, b: u8) -> Self { Self { r, g, b } }

    /// Convert to the renderer's 0xRRGGBB pixel format.
    pub const fn packed(self) -> u32 {
        ((self.r as u32) << 16) | ((self.g as u32) << 8) | self.b as u32
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EyeColorPreset {
    pub name: &'static str,
    pub color: Rgb,
}

pub const EYE_COLOR_PRESETS: [EyeColorPreset; 6] = [
    EyeColorPreset { name: "cyan", color: Rgb::new(0x5B, 0xE7, 0xF2) },
    EyeColorPreset { name: "blue", color: Rgb::new(0x55, 0x9B, 0xFF) },
    EyeColorPreset { name: "purple", color: Rgb::new(0xB8, 0x7C, 0xFF) },
    EyeColorPreset { name: "green", color: Rgb::new(0x72, 0xF0, 0xA0) },
    EyeColorPreset { name: "amber", color: Rgb::new(0xFF, 0xC2, 0x63) },
    EyeColorPreset { name: "pink", color: Rgb::new(0xFF, 0x83, 0xC8) },
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Appearance {
    pub eye_color: Rgb,
}

impl Default for Appearance {
    fn default() -> Self { Self { eye_color: EYE_COLOR_PRESETS[0].color } }
}

impl Appearance {
    pub fn cycle_eye_color(&mut self) -> EyeColorPreset {
        let current = EYE_COLOR_PRESETS.iter().position(|preset| preset.color == self.eye_color);
        let next = current.map_or(0, |index| (index + 1) % EYE_COLOR_PRESETS.len());
        let preset = EYE_COLOR_PRESETS[next];
        self.eye_color = preset.color;
        preset
    }

    pub fn set_eye_color(&mut self, color: Rgb) {
        self.eye_color = color;
    }

    pub fn named_eye_color(&mut self, name: &str) -> bool {
        let Some(preset) = EYE_COLOR_PRESETS.iter().find(|preset| preset.name.eq_ignore_ascii_case(name)) else {
            return false;
        };
        self.eye_color = preset.color;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_the_original_cyan() {
        assert_eq!(Appearance::default().eye_color.packed(), 0x5BE7F2);
    }

    #[test]
    fn named_colours_are_case_insensitive_and_unknown_names_do_nothing() {
        let mut appearance = Appearance::default();
        assert!(appearance.named_eye_color("PuRpLe"));
        assert_eq!(appearance.eye_color, EYE_COLOR_PRESETS[2].color);
        assert!(!appearance.named_eye_color("ultraviolet"));
        assert_eq!(appearance.eye_color, EYE_COLOR_PRESETS[2].color);
    }

    #[test]
    fn custom_rgb_colour_round_trips_to_packed_pixel() {
        let colour = Rgb::new(12, 34, 56);
        assert_eq!(colour.packed(), 0x0C2238);
    }

    #[test]
    fn palette_cycles_and_wraps() {
        let mut appearance = Appearance::default();
        assert_eq!(appearance.cycle_eye_color().name, "blue");
        for _ in 0..4 { appearance.cycle_eye_color(); }
        assert_eq!(appearance.cycle_eye_color().name, "cyan");
    }
}
