use glam::{UVec2, Vec4};
use serde::{Deserialize, Serialize};

use crate::core::buffers::state::UploadType;
use crate::core::image::ImageDecoder;
use crate::core::resources::texture::TextureRecord;
use crate::core::state::EngineState;

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdTextureCreateFromBufferArgs {
    pub window_id: u32,
    pub texture_id: u32,
    pub buffer_id: u64,
    #[serde(default)]
    pub srgb: Option<bool>,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultTextureCreateFromBuffer {
    pub success: bool,
    pub message: String,
}

pub fn engine_cmd_texture_create_from_buffer(
    engine: &mut EngineState,
    args: &CmdTextureCreateFromBufferArgs,
) -> CmdResultTextureCreateFromBuffer {
    let window_state = match engine.window.states.get_mut(&args.window_id) {
        Some(ws) => ws,
        None => {
            return CmdResultTextureCreateFromBuffer {
                success: false,
                message: format!("Window {} not found", args.window_id),
            };
        }
    };

    if window_state
        .render_state
        .scene
        .textures
        .contains_key(&args.texture_id)
    {
        return CmdResultTextureCreateFromBuffer {
            success: false,
            message: format!("Texture with id {} already exists", args.texture_id),
        };
    }

    let device = match engine.device.as_ref() {
        Some(d) => d,
        None => {
            return CmdResultTextureCreateFromBuffer {
                success: false,
                message: "Device not initialized".into(),
            };
        }
    };

    let queue = match engine.queue.as_ref() {
        Some(q) => q,
        None => {
            return CmdResultTextureCreateFromBuffer {
                success: false,
                message: "Queue not initialized".into(),
            };
        }
    };

    let buffer = match engine.buffers.remove_upload(args.buffer_id) {
        Some(b) => b,
        None => {
            return CmdResultTextureCreateFromBuffer {
                success: false,
                message: format!("Buffer with id {} not found", args.buffer_id),
            };
        }
    };

    if buffer.upload_type != UploadType::ImageData {
        return CmdResultTextureCreateFromBuffer {
            success: false,
            message: format!(
                "Invalid buffer type. Expected ImageData, got {:?}",
                buffer.upload_type
            ),
        };
    }

    let image = match ImageDecoder::try_decode(&buffer.data) {
        Some(img) => img,
        None => {
            return CmdResultTextureCreateFromBuffer {
                success: false,
                message: "Failed to decode image. Supported formats: PNG, JPEG, WebP, AVIF".into(),
            };
        }
    };

    let format = if args.srgb.unwrap_or(true) {
        wgpu::TextureFormat::Rgba8UnormSrgb
    } else {
        wgpu::TextureFormat::Rgba8Unorm
    };

    let size = wgpu::Extent3d {
        width: image.width,
        height: image.height,
        depth_or_array_layers: 1,
    };

    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Texture From Buffer"),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });

    queue.write_texture(
        texture.as_image_copy(),
        &image.data,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(4 * image.width),
            rows_per_image: Some(image.height),
        },
        size,
    );

    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

    window_state.render_state.scene.textures.insert(
        args.texture_id,
        TextureRecord {
            _texture: texture,
            view,
            _size: size,
            _format: format,
        },
    );
    window_state.is_dirty = true;

    CmdResultTextureCreateFromBuffer {
        success: true,
        message: "Texture created successfully".into(),
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdTextureCreateSolidColorArgs {
    pub window_id: u32,
    pub texture_id: u32,
    pub color: Vec4,
    pub size: UVec2,
    #[serde(default)]
    pub srgb: Option<bool>,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultTextureCreateSolidColor {
    pub success: bool,
    pub message: String,
}

pub fn engine_cmd_texture_create_solid_color(
    engine: &mut EngineState,
    args: &CmdTextureCreateSolidColorArgs,
) -> CmdResultTextureCreateSolidColor {
    let window_state = match engine.window.states.get_mut(&args.window_id) {
        Some(ws) => ws,
        None => {
            return CmdResultTextureCreateSolidColor {
                success: false,
                message: format!("Window {} not found", args.window_id),
            };
        }
    };

    if window_state
        .render_state
        .scene
        .textures
        .contains_key(&args.texture_id)
    {
        return CmdResultTextureCreateSolidColor {
            success: false,
            message: format!("Texture with id {} already exists", args.texture_id),
        };
    }

    let device = match engine.device.as_ref() {
        Some(d) => d,
        None => {
            return CmdResultTextureCreateSolidColor {
                success: false,
                message: "Device not initialized".into(),
            };
        }
    };

    let queue = match engine.queue.as_ref() {
        Some(q) => q,
        None => {
            return CmdResultTextureCreateSolidColor {
                success: false,
                message: "Queue not initialized".into(),
            };
        }
    };

    let format = if args.srgb.unwrap_or(true) {
        wgpu::TextureFormat::Rgba8UnormSrgb
    } else {
        wgpu::TextureFormat::Rgba8Unorm
    };

    let size = wgpu::Extent3d {
        width: args.size.x.max(1),
        height: args.size.y.max(1),
        depth_or_array_layers: 1,
    };

    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Texture Solid Color"),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });

    let color = args.color.clamp(Vec4::ZERO, Vec4::ONE);
    let rgba = [
        (color.x * 255.0) as u8,
        (color.y * 255.0) as u8,
        (color.z * 255.0) as u8,
        (color.w * 255.0) as u8,
    ];

    let pixel_count = (size.width * size.height) as usize;
    let mut data = Vec::with_capacity(pixel_count * 4);
    for _ in 0..pixel_count {
        data.extend_from_slice(&rgba);
    }
    queue.write_texture(
        texture.as_image_copy(),
        &data,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(4 * size.width),
            rows_per_image: Some(size.height),
        },
        size,
    );

    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

    window_state.render_state.scene.textures.insert(
        args.texture_id,
        TextureRecord {
            _texture: texture,
            view,
            _size: size,
            _format: format,
        },
    );
    window_state.is_dirty = true;

    CmdResultTextureCreateSolidColor {
        success: true,
        message: "Texture created successfully".into(),
    }
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdTextureDisposeArgs {
    pub window_id: u32,
    pub texture_id: u32,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultTextureDispose {
    pub success: bool,
    pub message: String,
}

pub fn engine_cmd_texture_dispose(
    engine: &mut EngineState,
    args: &CmdTextureDisposeArgs,
) -> CmdResultTextureDispose {
    let window_state = match engine.window.states.get_mut(&args.window_id) {
        Some(ws) => ws,
        None => {
            return CmdResultTextureDispose {
                success: false,
                message: format!("Window {} not found", args.window_id),
            };
        }
    };

    if window_state
        .render_state
        .scene
        .textures
        .remove(&args.texture_id)
        .is_some()
    {
        window_state.is_dirty = true;
        CmdResultTextureDispose {
            success: true,
            message: "Texture disposed successfully".into(),
        }
    } else {
        CmdResultTextureDispose {
            success: false,
            message: format!("Texture with id {} not found", args.texture_id),
        }
    }
}
