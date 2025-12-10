use super::binding::BindingManager;
use super::components::Components;
use super::pipeline::PipelineCache;
use super::resources::Resources;

pub struct RenderState {
    pub components: Components,
    pub resources: Resources,
    pub clear_color: wgpu::Color,

    /// Binding manager for component-shader-resource combinations
    pub binding_manager: BindingManager,

    /// Pipeline cache for shader-material combinations
    pub pipeline_cache: PipelineCache,
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
            binding_manager: BindingManager::new(),
            pipeline_cache: PipelineCache::new(),
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
        // Clear caches first
        self.binding_manager.clear();
        self.pipeline_cache.clear();

        // Drop components (includes render targets)
        self.components.drop_all();

        // Drop resources (includes shaders with their buffers)
        self.resources.drop_all();
    }
}
