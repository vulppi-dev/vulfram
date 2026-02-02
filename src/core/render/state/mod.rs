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
use crate::core::resources::EnvironmentConfig;
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
pub use crate::core::render::graph::RenderGraphState;

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
    pub forward_msaa_target: Option<crate::core::resources::RenderTarget>,
    pub forward_emissive_msaa_target: Option<crate::core::resources::RenderTarget>,
    pub post_uniform_buffer: Option<wgpu::Buffer>,
    pub ssao_uniform_buffer: Option<wgpu::Buffer>,
    pub ssao_blur_uniform_buffer: Option<wgpu::Buffer>,
    pub bloom_uniform_buffer: Option<wgpu::Buffer>,
    pub skybox_uniform_buffer: Option<wgpu::Buffer>,
    pub environment: EnvironmentConfig,
    pub environment_is_configured: bool,
    pub skinning: SkinningSystem,
    pub render_graph: RenderGraphState,

    /// Per-frame collector for draw calls, reused to avoid allocations.
    pub collector: DrawCollector,
}

impl RenderState {
    pub fn msaa_sample_count(&self) -> u32 {
        if self.environment.msaa.enabled && self.environment.msaa.sample_count >= 2 {
            self.environment.msaa.sample_count
        } else {
            1
        }
    }

    #[cfg(any(not(feature = "wasm"), target_arch = "wasm32"))]
    pub fn on_resize(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        // Depth target is now managed per-frame or lazily by passes
        self.forward_depth_target = None;
        self.forward_msaa_target = None;
        self.forward_emissive_msaa_target = None;

        let mut any_camera_dirty = false;
        for record in self.scene.cameras.values_mut() {
            let (target_width, target_height) = record
                .view_position
                .as_ref()
                .map(|vp| vp.resolve_size(width, height))
                .unwrap_or((width, height));

            crate::core::resources::ensure_render_target(
                device,
                &mut record.render_target,
                target_width,
                target_height,
                wgpu::TextureFormat::Rgba16Float,
            );
            crate::core::resources::ensure_render_target(
                device,
                &mut record.emissive_target,
                target_width,
                target_height,
                wgpu::TextureFormat::Rgba16Float,
            );
            crate::core::resources::ensure_render_target(
                device,
                &mut record.post_target,
                target_width,
                target_height,
                wgpu::TextureFormat::Rgba16Float,
            );
            crate::core::resources::ensure_render_target(
                device,
                &mut record.outline_target,
                target_width,
                target_height,
                wgpu::TextureFormat::Rgba8Unorm,
            );
            crate::core::resources::ensure_render_target(
                device,
                &mut record.ssao_target,
                target_width,
                target_height,
                wgpu::TextureFormat::Rgba16Float,
            );
            crate::core::resources::ensure_render_target(
                device,
                &mut record.ssao_blur_target,
                target_width,
                target_height,
                wgpu::TextureFormat::Rgba16Float,
            );
            crate::core::resources::ensure_render_target(
                device,
                &mut record.bloom_target,
                target_width,
                target_height,
                wgpu::TextureFormat::Rgba16Float,
            );
            for (level, target) in record.bloom_chain.iter_mut().enumerate() {
                let level_width = crate::core::render::bloom_chain_size(target_width, level);
                let level_height = crate::core::render::bloom_chain_size(target_height, level);
                crate::core::resources::ensure_render_target(
                    device,
                    target,
                    level_width,
                    level_height,
                    wgpu::TextureFormat::Rgba16Float,
                );
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
