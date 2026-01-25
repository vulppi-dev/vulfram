pub mod binding;
pub mod collector;
pub mod init;
pub mod library;
pub mod lifecycle;
pub mod light;
pub mod prepare;
pub mod scene;
pub mod skinning;

use crate::core::render::cache::RenderCache;
use crate::core::render::gizmos::GizmoSystem;
use crate::core::resources::VertexAllocatorSystem;
use crate::core::resources::shadow::ShadowManager;

pub use self::binding::BindingSystem;
pub use self::collector::{DrawCollector, DrawItem};
pub use self::library::ResourceLibrary;
#[cfg(any(not(feature = "wasm"), target_arch = "wasm32"))]
pub use self::library::SamplerSet;
pub use self::light::{FrustumPlane, LightCullingSystem};
pub use self::scene::RenderScene;
pub use self::skinning::SkinningSystem;

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
    pub skinning: SkinningSystem,

    /// Per-frame collector for draw calls, reused to avoid allocations.
    pub collector: DrawCollector,
}

impl RenderState {
    #[cfg(any(not(feature = "wasm"), target_arch = "wasm32"))]
    pub fn on_resize(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        // Depth target is now managed per-frame or lazily by passes
        self.forward_depth_target = None;

        let mut any_camera_dirty = false;
        for record in self.scene.cameras.values_mut() {
            let (target_width, target_height) = record
                .view_position
                .as_ref()
                .map(|vp| vp.resolve_size(width, height))
                .unwrap_or((width, height));

            let needs_target = match record.render_target.as_ref() {
                Some(target) => {
                    let size = target._texture.size();
                    size.width != target_width || size.height != target_height
                }
                None => true,
            };

            if needs_target {
                let size = wgpu::Extent3d {
                    width: target_width,
                    height: target_height,
                    depth_or_array_layers: 1,
                };
                let target = crate::core::resources::RenderTarget::new(
                    device,
                    size,
                    wgpu::TextureFormat::Rgba16Float,
                );
                record.set_render_target(target);
            }

            record.data.update(
                None,
                None,
                None,
                None,
                (target_width, target_height),
                record.ortho_scale,
            );
            record.mark_dirty();
            any_camera_dirty = true;
        }

        if any_camera_dirty {
            if let Some(shadow) = self.shadow.as_mut() {
                shadow.mark_dirty();
            }
        }
    }
}
