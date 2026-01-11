use glam::{UVec2, Vec4};

use crate::core::resources::texture::ForwardAtlasHandle;

#[derive(Debug)]
pub struct TextureRecord {
    pub label: Option<String>,
    pub size: wgpu::Extent3d,
    pub format: wgpu::TextureFormat,
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
}

#[derive(Debug)]
pub struct ForwardAtlasEntry {
    pub label: Option<String>,
    pub handle: ForwardAtlasHandle,
    pub size: UVec2,
    pub uv_scale_bias: Vec4,
    pub layer: u32,
    pub format: wgpu::TextureFormat,
}
