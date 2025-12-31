mod compose;
mod forward;
mod light_cull;
mod shadow;

pub use compose::*;
pub use forward::*;
pub use light_cull::*;
pub use shadow::*;

use crate::core::resources::RenderTarget;

/// Resources and state for the shadow map update pass
pub struct ShadowPass {
    pub pipeline: Option<wgpu::RenderPipeline>,
}

impl ShadowPass {
    pub fn new() -> Self {
        Self { pipeline: None }
    }
}

/// Resources and state for the forward rendering pass
pub struct ForwardPass {
    pub depth_target: Option<RenderTarget>,
}

impl ForwardPass {
    pub fn new() -> Self {
        Self { depth_target: None }
    }
}

/// Resources and state for the final composition pass
pub struct ComposePass {
    pub pipeline: Option<wgpu::RenderPipeline>,
}

impl ComposePass {
    pub fn new() -> Self {
        Self { pipeline: None }
    }
}

/// Resources and state for the light culling compute pass
pub struct LightCullPass {
    pub pipeline: Option<wgpu::ComputePipeline>,
    pub bind_group: Option<wgpu::BindGroup>,
}

impl LightCullPass {
    pub fn new() -> Self {
        Self {
            pipeline: None,
            bind_group: None,
        }
    }
}

/// Aggregates all render passes used by a window
pub struct RenderPasses {
    pub forward: ForwardPass,
    pub light_cull: LightCullPass,
    pub compose: ComposePass,
    pub shadow: ShadowPass,
    // Future: ui: UiPass,
}

impl RenderPasses {
    pub fn new() -> Self {
        Self {
            forward: ForwardPass::new(),
            light_cull: LightCullPass::new(),
            compose: ComposePass::new(),
            shadow: ShadowPass::new(),
        }
    }
}
