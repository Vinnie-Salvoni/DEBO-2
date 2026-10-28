//! A deliberately small CPU rasteriser with coverage-antialiased edges.
//! Pixels, lines, ellipses, thick strokes, curves, and polygon filling are native code.

#[derive(Clone, Debug)]
pub struct Renderer {
    width: usize,
    height: usize,
    pixels: Vec<u32>,
}

impl Renderer {
    pub fn new(width: usize, height: usize, clear: u32) -> Self {
        Self { width, height, pixels: vec![clear; width * height] }
    }
    pub fn width(&self) -> usize { self.width }
    pub fn height(&self) -> usize { self.height }
    pub fn pixels(&self) -> &[u32] { &self.pixels }
    pub fn clear(&mut self, color: u32) { self.pixels.fill(color); }

    pub fn pixel(&mut self, x: i32, y: i32, color: u32) {
        if x >= 0 && y >= 0 && (x as usize) < self.width && (y as usize) < self.height {
            self.pixels[y as usize * self.width + x as usize] = color;
        }
    }

    pub fn fill_rect(&mut self, x: i32, y: i32, w: i32, h: i32, color: u32) {
        if w <= 0 || h <= 0 { return; }
        let x0 = x.max(0); let y0 = y.max(0);
        let x1 = (x + w).min(self.width as i32); let y1 = (y + h).min(self.height as i32);
        for py in y0..y1 { for px in x0..x1 { self.pixel(px, py, color); } }
    }

    /// Draw an ellipse with restrained 2x2 sub-pixel coverage sampling. This
    /// softens stair-stepping while keeping edges crisp rather than blurry.
    pub fn fill_ellipse(&mut self, cx: f32, cy: f32, rx: f32, ry: f32, color: u32) {
        if rx <= 0.0 || ry <= 0.0 { return; }
        const SAMPLES: usize = 2;
        const SAMPLE_COUNT: f32 = (SAMPLES * SAMPLES) as f32;
        let x0 = (cx - rx - 1.0).floor() as i32;
        let x1 = (cx + rx + 1.0).ceil() as i32;
        let y0 = (cy - ry - 1.0).floor() as i32;
        let y1 = (cy + ry + 1.0).ceil() as i32;
        let rx2 = rx * rx;
        let ry2 = ry * ry;
        for y in y0..=y1 {
            for x in x0..=x1 {
                let mut covered = 0_u32;
                for sy in 0..SAMPLES {
                    for sx in 0..SAMPLES {
                        let px = x as f32 + (sx as f32 + 0.5) / SAMPLES as f32;
                        let py = y as f32 + (sy as f32 + 0.5) / SAMPLES as f32;
                        let dx = px - cx;
                        let dy = py - cy;
                        if dx * dx / rx2 + dy * dy / ry2 <= 1.0 { covered += 1; }
                    }
                }
                if covered > 0 {
                    let coverage = covered as f32 / SAMPLE_COUNT;
                    self.blend_pixel(x, y, color, coverage);
                }
            }
        }
    }

    fn blend_pixel(&mut self, x: i32, y: i32, color: u32, coverage: f32) {
        if x < 0 || y < 0 || (x as usize) >= self.width || (y as usize) >= self.height { return; }
        if coverage >= 1.0 { self.pixel(x, y, color); return; }
        let index = y as usize * self.width + x as usize;
        let dst = self.pixels[index];
        let mix = |shift: u32| {
            let a = ((dst >> shift) & 0xff) as f32;
            let b = ((color >> shift) & 0xff) as f32;
            (a + (b - a) * coverage).round().clamp(0.0, 255.0) as u32
        };
        self.pixels[index] = (mix(16) << 16) | (mix(8) << 8) | mix(0);
    }

    pub fn line(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, color: u32) {
        let (mut x, mut y) = (x0, y0);
        let dx = (x1 - x0).abs(); let sx = if x0 < x1 { 1 } else { -1 };
        let dy = -(y1 - y0).abs(); let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;
        loop {
            self.pixel(x, y, color);
            if x == x1 && y == y1 { break; }
            let e2 = 2 * err;
            if e2 >= dy { err += dy; x += sx; }
            if e2 <= dx { err += dx; y += sy; }
        }
    }

    pub fn thick_line(&mut self, a: (f32, f32), b: (f32, f32), radius: f32, color: u32) {
        let dx = b.0 - a.0; let dy = b.1 - a.1;
        let steps = dx.abs().max(dy.abs()).ceil().max(1.0) as usize;
        for i in 0..=steps {
            let t = i as f32 / steps as f32;
            self.fill_ellipse(a.0 + dx * t, a.1 + dy * t, radius, radius, color);
        }
    }

    pub fn quadratic_stroke(&mut self, p0: (f32, f32), p1: (f32, f32), p2: (f32, f32), radius: f32, color: u32) {
        let rough = (p2.0 - p0.0).abs().max((p2.1 - p0.1).abs());
        let steps = (rough * 2.0).ceil().max(8.0) as usize;
        let mut prev = p0;
        for i in 1..=steps {
            let t = i as f32 / steps as f32; let u = 1.0 - t;
            let next = (u*u*p0.0 + 2.0*u*t*p1.0 + t*t*p2.0, u*u*p0.1 + 2.0*u*t*p1.1 + t*t*p2.1);
            self.thick_line(prev, next, radius, color); prev = next;
        }
    }

    pub fn fill_polygon(&mut self, points: &[(f32, f32)], color: u32) {
        if points.len() < 3 { return; }
        let min_y = points.iter().map(|p| p.1).fold(f32::INFINITY, f32::min).floor() as i32;
        let max_y = points.iter().map(|p| p.1).fold(f32::NEG_INFINITY, f32::max).ceil() as i32;
        for y in min_y.max(0)..=max_y.min(self.height as i32 - 1) {
            let scan_y = y as f32 + 0.5; let mut xs = Vec::new();
            for i in 0..points.len() {
                let a = points[i]; let b = points[(i + 1) % points.len()];
                if (a.1 <= scan_y && b.1 > scan_y) || (b.1 <= scan_y && a.1 > scan_y) {
                    xs.push(a.0 + (scan_y - a.1) * (b.0 - a.0) / (b.1 - a.1));
                }
            }
            xs.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            for pair in xs.chunks_exact(2) {
                for x in pair[0].ceil() as i32..pair[1].ceil() as i32 { self.pixel(x, y, color); }
            }
        }
    }

    /// Crisp nearest-neighbour scaling. Keeps edge pixels defined instead of blurring adjacent colours.
    pub fn upscale(&self, scale: usize) -> Vec<u32> {
        let scale = scale.max(1); let mut out = vec![0; self.width * scale * self.height * scale];
        let out_width = self.width * scale;
        for y in 0..self.height { for x in 0..self.width {
            let color = self.pixels[y * self.width + x];
            for sy in 0..scale { let start = (y * scale + sy) * out_width + x * scale; out[start..start + scale].fill(color); }
        }}
        out
    }

    /// Optional bilinear scaling for use cases that explicitly want a soft image.
    /// The desktop face preview uses `upscale` instead to preserve crisp edges.
    pub fn upscale_smooth(&self, scale: usize) -> Vec<u32> {
        let scale = scale.max(1);
        let out_width = self.width * scale;
        let out_height = self.height * scale;
        let mut out = vec![0; out_width * out_height];
        if self.width == 0 || self.height == 0 { return out; }
        for oy in 0..out_height {
            let fy = ((oy as f32 + 0.5) / scale as f32 - 0.5).clamp(0.0, (self.height - 1) as f32);
            let y0 = fy.floor() as usize;
            let y1 = (y0 + 1).min(self.height - 1);
            let ty = fy - y0 as f32;
            for ox in 0..out_width {
                let fx = ((ox as f32 + 0.5) / scale as f32 - 0.5).clamp(0.0, (self.width - 1) as f32);
                let x0 = fx.floor() as usize;
                let x1 = (x0 + 1).min(self.width - 1);
                let tx = fx - x0 as f32;
                let c00 = self.pixels[y0 * self.width + x0];
                let c10 = self.pixels[y0 * self.width + x1];
                let c01 = self.pixels[y1 * self.width + x0];
                let c11 = self.pixels[y1 * self.width + x1];
                let mut result = 0_u32;
                for shift in [0, 8, 16] {
                    let a = ((c00 >> shift) & 0xff) as f32;
                    let b = ((c10 >> shift) & 0xff) as f32;
                    let c = ((c01 >> shift) & 0xff) as f32;
                    let d = ((c11 >> shift) & 0xff) as f32;
                    let top = a + (b - a) * tx;
                    let bottom = c + (d - c) * tx;
                    let value = (top + (bottom - top) * ty).round().clamp(0.0, 255.0) as u32;
                    result |= value << shift;
                }
                out[oy * out_width + ox] = result;
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn pixel_clips_outside_buffer() { let mut r = Renderer::new(3, 2, 7); r.pixel(-1, 0, 9); r.pixel(3, 0, 9); assert_eq!(r.pixels(), &[7; 6]); }
    #[test] fn rect_fills_expected_pixels() { let mut r = Renderer::new(4, 3, 0); r.fill_rect(1, 1, 2, 1, 5); assert_eq!(r.pixels(), &[0,0,0,0, 0,5,5,0, 0,0,0,0]); }
    #[test] fn upscale_preserves_pixel_blocks() { let mut r = Renderer::new(2, 1, 0); r.pixel(1, 0, 4); assert_eq!(r.upscale(2), vec![0,0,4,4,0,0,4,4]); }
    #[test] fn ellipse_edges_use_partial_coverage() {
        let mut r = Renderer::new(12, 12, 0x000000);
        r.fill_ellipse(5.25, 5.25, 3.0, 3.0, 0xffffff);
        assert!(r.pixels().iter().any(|&p| p != 0 && p != 0xffffff));
    }
    #[test] fn smooth_upscale_has_expected_dimensions() {
        let r = Renderer::new(3, 2, 0x123456);
        assert_eq!(r.upscale_smooth(4).len(), 12 * 8);
        assert!(r.upscale_smooth(1).iter().all(|&p| p == 0x123456));
    }
}
