pub mod binding;
pub mod collector;
pub mod library;
pub mod light;
pub mod prepare;
pub mod scene;
pub mod lifecycle;
pub mod init;

use std::collections::HashMap;
use crate::core::resources::{
    VertexAllocatorSystem, FrameComponent,
};
use crate::core::render::cache::RenderCache;
use crate::core::render::gizmos::GizmoSystem;
use crate::core::resources::shadow::ShadowManager;

pub use self::binding::BindingSystem;
pub use self::scene::RenderScene;
pub use self::library::{ResourceLibrary, SamplerSet};
pub use self::light::{LightCullingSystem, FrustumPlane};
pub use self::collector::{DrawCollector, DrawItem};

pub struct RenderState {
    pub scene: RenderScene,
    pub bindings: Option<BindingSystem>,
    pub library: Option<ResourceLibrary>,
    pub vertex: Option<VertexAllocatorSystem>,
    pub light_system: Option<LightCullingSystem>,
    pub gizmos: GizmoSystem,
    pub shadow: Option<ShadowManager>,
    pub forward_atlas: Option<crate::core::resources::ForwardAtlasSystem>,
    pub cache: RenderCache,
    pub forward_depth_target: Option<crate::core::resources::RenderTarget>,
    
    /// Per-frame collector for draw calls, reused to avoid allocations.
    pub collector: DrawCollector,
}

impl RenderState {
    pub fn on_resize(&mut self, _width: u32, _height: u32) {
        // Depth target is now managed per-frame or lazily by passes
        self.forward_depth_target = None;
    }
}
