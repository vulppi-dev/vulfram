use super::super::RenderState;

pub(crate) struct FallbackTextures {
    pub(crate) texture: wgpu::Texture,
    pub(crate) view: wgpu::TextureView,
    pub(crate) atlas_texture: wgpu::Texture,
    pub(crate) atlas_view: wgpu::TextureView,
    pub(crate) shadow_texture: wgpu::Texture,
    pub(crate) shadow_view: wgpu::TextureView,
}

impl RenderState {
    pub(crate) fn init_fallback_textures(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> FallbackTextures {
        let white_pixel: [u8; 4] = [255, 255, 255, 255];

        let fallback_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Fallback Texture 1x1"),
            size: wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            fallback_texture.as_image_copy(),
            &white_pixel,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4),
                rows_per_image: Some(1),
            },
            wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
        );

        let fallback_view = fallback_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let fallback_forward_atlas_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Fallback Forward Atlas 1x1"),
            size: wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            fallback_forward_atlas_texture.as_image_copy(),
            &white_pixel,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4),
                rows_per_image: Some(1),
            },
            wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
        );

        let fallback_forward_atlas_view =
            fallback_forward_atlas_texture.create_view(&wgpu::TextureViewDescriptor {
                label: Some("Fallback Forward Atlas View"),
                format: Some(wgpu::TextureFormat::Rgba8UnormSrgb),
                dimension: Some(wgpu::TextureViewDimension::D2Array),
                aspect: wgpu::TextureAspect::All,
                base_mip_level: 0,
                mip_level_count: None,
                base_array_layer: 0,
                array_layer_count: Some(1),
                usage: None,
            });

        let fallback_shadow_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Fallback Shadow Texture 1x1"),
            size: wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        let fallback_shadow_view =
            fallback_shadow_texture.create_view(&wgpu::TextureViewDescriptor {
                label: Some("Fallback Shadow View"),
                format: Some(wgpu::TextureFormat::Depth32Float),
                dimension: Some(wgpu::TextureViewDimension::D2Array),
                aspect: wgpu::TextureAspect::DepthOnly,
                base_mip_level: 0,
                mip_level_count: None,
                base_array_layer: 0,
                array_layer_count: Some(1),
                usage: None,
            });

        FallbackTextures {
            texture: fallback_texture,
            view: fallback_view,
            atlas_texture: fallback_forward_atlas_texture,
            atlas_view: fallback_forward_atlas_view,
            shadow_texture: fallback_shadow_texture,
            shadow_view: fallback_shadow_view,
        }
    }
}
