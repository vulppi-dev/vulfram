use std::collections::HashMap;

use egui::epaint::{ImageData, TextureId, textures::TextureOptions};
use std::collections::HashSet;

pub struct ManagedTexture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub bind_group: wgpu::BindGroup,
    pub options: TextureOptions,
}

pub struct ExternalTexture {
    pub _view: wgpu::TextureView,
    pub bind_group: wgpu::BindGroup,
    pub options: TextureOptions,
}

pub struct TextureManager {
    pub textures: HashMap<TextureId, ManagedTexture>,
    pub external_textures: HashMap<TextureId, ExternalTexture>,
    pub samplers: HashMap<TextureOptions, wgpu::Sampler>,
}

impl TextureManager {
    pub fn new() -> Self {
        Self {
            textures: HashMap::new(),
            external_textures: HashMap::new(),
            samplers: HashMap::new(),
        }
    }

    pub fn update_texture(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        layout: &wgpu::BindGroupLayout,
        id: TextureId,
        image_delta: &egui::epaint::ImageDelta,
    ) {
        let width = image_delta.image.width() as u32;
        let height = image_delta.image.height() as u32;
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let pixels: Vec<egui::Color32> = match &image_delta.image {
            ImageData::Color(image) => image.pixels.clone(),
            ImageData::Font(image) => image.srgba_pixels(None).collect(),
        };

        let mut data = Vec::with_capacity(pixels.len() * 4);
        for color in pixels {
            data.extend_from_slice(&color.to_array());
        }
        let data_bytes: &[u8] = data.as_slice();

        let (texture, view, mut bind_group, needs_bind_group, origin) = if let Some(pos) = image_delta.pos {
            let existing = self
                .textures
                .remove(&id)
                .expect("Tried to update a texture that has not been allocated yet.");
            let origin = wgpu::Origin3d {
                x: pos[0] as u32,
                y: pos[1] as u32,
                z: 0,
            };
            (
                existing.texture,
                existing.view,
                Some(existing.bind_group),
                existing.options != image_delta.options,
                origin,
            )
        } else {
            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("egui_texture"),
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[wgpu::TextureFormat::Rgba8Unorm],
            });
            let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
            (texture, view, None, true, wgpu::Origin3d::ZERO)
        };

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin,
                aspect: wgpu::TextureAspect::All,
            },
            data_bytes,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            size,
        );

        if needs_bind_group {
            let sampler = self
                .samplers
                .entry(image_delta.options)
                .or_insert_with(|| create_sampler(device, image_delta.options));
            bind_group = Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("egui_texture_bind_group"),
                layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(sampler),
                    },
                ],
            }));
        }

        let bind_group = bind_group.expect("Missing texture bind group");

        self.textures.insert(
            id,
            ManagedTexture {
                texture,
                view,
                bind_group,
                options: image_delta.options,
            },
        );
    }

    pub fn free_texture(&mut self, id: &TextureId) {
        self.textures.remove(id);
    }

    pub fn register_external_texture(
        &mut self,
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        id: TextureId,
        view: &wgpu::TextureView,
        options: TextureOptions,
    ) {
        let needs_bind_group = match self.external_textures.get(&id) {
            Some(existing) => existing.options != options,
            None => true,
        };
        if !needs_bind_group {
            return;
        }
        let sampler = self
            .samplers
            .entry(options)
            .or_insert_with(|| create_sampler(device, options));
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("egui_external_texture_bind_group"),
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(sampler),
                },
            ],
        });
        self.external_textures.insert(
            id,
            ExternalTexture {
                _view: view.clone(),
                bind_group,
                options,
            },
        );
    }

    pub fn prune_external_textures(&mut self, used: &HashSet<TextureId>) {
        self.external_textures
            .retain(|id, _| used.contains(id));
    }

    pub fn texture_bind_group(&self, id: &TextureId) -> Option<&wgpu::BindGroup> {
        self.textures
            .get(id)
            .map(|tex| &tex.bind_group)
            .or_else(|| self.external_textures.get(id).map(|tex| &tex.bind_group))
    }
}

fn create_sampler(device: &wgpu::Device, options: TextureOptions) -> wgpu::Sampler {
    let (min_filter, mag_filter) = match options.magnification {
        egui::epaint::textures::TextureFilter::Nearest => (wgpu::FilterMode::Nearest, wgpu::FilterMode::Nearest),
        egui::epaint::textures::TextureFilter::Linear => (wgpu::FilterMode::Linear, wgpu::FilterMode::Linear),
    };
    let address_mode = match options.wrap_mode {
        egui::epaint::textures::TextureWrapMode::ClampToEdge => wgpu::AddressMode::ClampToEdge,
        egui::epaint::textures::TextureWrapMode::Repeat => wgpu::AddressMode::Repeat,
        egui::epaint::textures::TextureWrapMode::MirroredRepeat => wgpu::AddressMode::MirrorRepeat,
    };

    device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("egui_sampler"),
        address_mode_u: address_mode,
        address_mode_v: address_mode,
        address_mode_w: address_mode,
        mag_filter: mag_filter,
        min_filter: min_filter,
        ..Default::default()
    })
}
