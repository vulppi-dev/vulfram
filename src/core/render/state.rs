/// Cached render resources for a window
#[derive(Clone)]
pub struct RenderState {
    pub clear_color: wgpu::Color,
}

impl Default for RenderState {
    fn default() -> Self {
        Self {
            clear_color: wgpu::Color {
                r: 0.5,
                g: 0.0,
                b: 0.5,
                a: 0.5,
            },
        }
    }
}

impl RenderState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_clear_color(&mut self, r: f64, g: f64, b: f64, a: f64) {
        self.clear_color = wgpu::Color { r, g, b, a };
    }
}
