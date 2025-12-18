use crate::core::render::RenderState;

pub fn pass_compose(
    render_state: &mut RenderState,
    queue: &wgpu::Queue,
    encoder: &mut wgpu::CommandEncoder,
    surface_texture: &wgpu::SurfaceTexture,
    config: &wgpu::SurfaceConfiguration,
) {
    let view = surface_texture
        .texture
        .create_view(&wgpu::TextureViewDescriptor::default());
}
