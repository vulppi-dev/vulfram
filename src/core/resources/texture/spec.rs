use glam::{UVec2, Vec4};

use crate::core::resources::texture::ForwardAtlasHandle;

#[derive(Debug)]
pub struct TextureRecord {
    pub _size: wgpu::Extent3d,
    pub _format: wgpu::TextureFormat,
    pub _texture: wgpu::Texture,
    pub view: wgpu::TextureView,
}

#[derive(Debug)]
pub struct ForwardAtlasEntry {
    pub handle: ForwardAtlasHandle,
    pub _size: UVec2,
    pub uv_scale_bias: Vec4,
    pub layer: u32,
    pub _format: wgpu::TextureFormat,
}
