#[derive(Debug)]
pub struct TextureRecord {
    pub _size: wgpu::Extent3d,
    pub _format: wgpu::TextureFormat,
    pub _texture: wgpu::Texture,
    pub view: wgpu::TextureView,
}
