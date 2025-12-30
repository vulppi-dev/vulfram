mod compose;
mod forward;

pub use compose::*;
pub use forward::*;

use crate::core::resources::RenderTarget;

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

/// Aggregates all render passes used by a window
pub struct RenderPasses {
    pub forward: ForwardPass,
    pub compose: ComposePass,
    // Future: shadow: ShadowPass,
    // Future: ui: UiPass,
}

impl RenderPasses {
    pub fn new() -> Self {
        Self {
            forward: ForwardPass::new(),
            compose: ComposePass::new(),
        }
    }
}
