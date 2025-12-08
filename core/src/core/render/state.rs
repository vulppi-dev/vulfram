use super::components::Components;
use super::resources::Resources;

pub struct RenderState {
    pub components: Components,
    pub resources: Resources,
    pub clear_color: wgpu::Color,
}

impl Default for RenderState {
    fn default() -> Self {
        Self {
            components: Components::new(),
            resources: Resources::new(),
            clear_color: wgpu::Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            },
        }
    }
}

impl RenderState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Explicitly drop all render state resources
    /// This ensures proper cleanup of GPU resources
    pub fn drop_all(&mut self) {
        self.components.drop_all();
        self.resources.drop_all();
    }
}
