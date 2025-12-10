use serde::{Deserialize, Serialize};

use crate::core::buffers::UploadType;
use crate::core::image::ImageDecoder;
use crate::core::render::enums::{TextureFormat, TextureUsage};
use crate::core::render::resources::{TextureId, TextureParams, TextureResource};
use crate::core::state::EngineState;

// MARK: - Create Texture

/// Arguments for creating a texture resource
#[derive(Debug, Deserialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdTextureCreateArgs {
    pub texture_id: TextureId,
    pub window_id: u32,
    pub buffer_id: u64,
    pub width: u32,
    pub height: u32,
    pub format: TextureFormat,
    pub usage: Vec<TextureUsage>,
    pub mip_level_count: u32,
    pub label: Option<String>,
}

impl Default for CmdTextureCreateArgs {
    fn default() -> Self {
        Self {
            texture_id: 0,
            window_id: 0,
            buffer_id: 0,
            width: 0,
            height: 0,
            format: TextureFormat::Rgba8Unorm,
            usage: vec![TextureUsage::TextureBinding, TextureUsage::CopyDst],
            mip_level_count: 1,
            label: None,
        }
    }
}

/// Result for texture creation command
#[derive(Debug, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultTextureCreate {
    pub success: bool,
    pub message: String,
}

impl Default for CmdResultTextureCreate {
    fn default() -> Self {
        Self {
            success: false,
            message: String::new(),
        }
    }
}

/// Create a new texture resource from uploaded buffer
pub fn engine_cmd_texture_create(
    engine: &mut EngineState,
    args: &CmdTextureCreateArgs,
) -> CmdResultTextureCreate {
    // Validate window exists
    let window_state = match engine.windows.get_mut(&args.window_id) {
        Some(ws) => ws,
        None => {
            return CmdResultTextureCreate {
                success: false,
                message: format!("Window with id {} not found", args.window_id),
            };
        }
    };

    let render_state = &mut window_state.render_state;

    // Check if texture already exists
    if render_state
        .resources
        .textures
        .contains_key(&args.texture_id)
    {
        return CmdResultTextureCreate {
            success: false,
            message: format!("Texture with id {} already exists", args.texture_id),
        };
    }

    // Get device and queue
    let device = match &engine.device {
        Some(d) => d,
        None => {
            return CmdResultTextureCreate {
                success: false,
                message: "GPU device not initialized".into(),
            };
        }
    };

    let queue = match &engine.queue {
        Some(q) => q,
        None => {
            return CmdResultTextureCreate {
                success: false,
                message: "GPU queue not initialized".into(),
            };
        }
    };

    // Get texture data from upload buffer
    let buffer = match engine.buffers.get(&args.buffer_id) {
        Some(buffer) => buffer,
        None => {
            return CmdResultTextureCreate {
                success: false,
                message: format!("Upload buffer with id {} not found", args.buffer_id),
            };
        }
    };

    // Decode image if UploadType is ImageData
    let (texture_data, actual_width, actual_height) = if buffer.upload_type == UploadType::ImageData
    {
        match ImageDecoder::try_decode(&buffer.data) {
            Some(decoded) => (decoded.data, decoded.width, decoded.height),
            None => {
                return CmdResultTextureCreate {
                    success: false,
                    message: "Failed to decode image data".into(),
                };
            }
        }
    } else {
        // Use raw data as-is
        (buffer.data.clone(), args.width, args.height)
    };

    // Convert texture format
    let format = args.format.to_wgpu();

    // Combine texture usage flags
    let usage = TextureUsage::combine(&args.usage);

    // Create texture params
    let params = TextureParams {
        width: actual_width,
        height: actual_height,
        format,
        usage,
        mip_level_count: args.mip_level_count,
    };

    // Create texture
    let texture_size = wgpu::Extent3d {
        width: actual_width,
        height: actual_height,
        depth_or_array_layers: 1,
    };

    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: args.label.as_deref(),
        size: texture_size,
        mip_level_count: args.mip_level_count,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage,
        view_formats: &[],
    });

    // Write texture data to GPU
    queue.write_texture(
        texture.as_image_copy(),
        &texture_data,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(4 * actual_width),
            rows_per_image: Some(actual_height),
        },
        texture_size,
    );

    // Create texture view
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

    // Create texture resource
    let texture_resource = TextureResource {
        texture_id: args.texture_id,
        texture,
        view,
        params,
    };

    // Insert texture resource
    render_state
        .resources
        .textures
        .insert(args.texture_id, texture_resource);

    // Remove upload buffer after use
    engine.buffers.remove(&args.buffer_id);

    CmdResultTextureCreate {
        success: true,
        message: "Texture resource created successfully".into(),
    }
}

// MARK: - Update Texture

/// Arguments for updating a texture resource
#[derive(Debug, Deserialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdTextureUpdateArgs {
    pub texture_id: TextureId,
    pub window_id: u32,
    pub buffer_id: u64,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub mip_level: u32,
}

impl Default for CmdTextureUpdateArgs {
    fn default() -> Self {
        Self {
            texture_id: 0,
            window_id: 0,
            buffer_id: 0,
            x: 0,
            y: 0,
            width: 0,
            height: 0,
            mip_level: 0,
        }
    }
}

/// Result for texture update command
#[derive(Debug, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultTextureUpdate {
    pub success: bool,
    pub message: String,
}

impl Default for CmdResultTextureUpdate {
    fn default() -> Self {
        Self {
            success: false,
            message: String::new(),
        }
    }
}

/// Update an existing texture resource with new data
pub fn engine_cmd_texture_update(
    engine: &mut EngineState,
    args: &CmdTextureUpdateArgs,
) -> CmdResultTextureUpdate {
    // Validate window exists
    let window_state = match engine.windows.get_mut(&args.window_id) {
        Some(ws) => ws,
        None => {
            return CmdResultTextureUpdate {
                success: false,
                message: format!("Window with id {} not found", args.window_id),
            };
        }
    };

    let render_state = &mut window_state.render_state;

    // Get texture resource
    let texture_resource = match render_state.resources.textures.get(&args.texture_id) {
        Some(t) => t,
        None => {
            return CmdResultTextureUpdate {
                success: false,
                message: format!("Texture with id {} not found", args.texture_id),
            };
        }
    };

    // Get queue
    let queue = match &engine.queue {
        Some(q) => q,
        None => {
            return CmdResultTextureUpdate {
                success: false,
                message: "GPU queue not initialized".into(),
            };
        }
    };

    // Get texture data from upload buffer
    let buffer = match engine.buffers.get(&args.buffer_id) {
        Some(buffer) => buffer,
        None => {
            return CmdResultTextureUpdate {
                success: false,
                message: format!("Upload buffer with id {} not found", args.buffer_id),
            };
        }
    };

    // Decode image if UploadType is ImageData
    let (texture_data, data_width, data_height) = if buffer.upload_type == UploadType::ImageData {
        match ImageDecoder::try_decode(&buffer.data) {
            Some(decoded) => (decoded.data, decoded.width, decoded.height),
            None => {
                return CmdResultTextureUpdate {
                    success: false,
                    message: "Failed to decode image data".into(),
                };
            }
        }
    } else {
        // Use raw data as-is with provided dimensions
        (buffer.data.clone(), args.width, args.height)
    };

    // If decoded dimensions don't match args, use decoded dimensions
    let actual_width = if buffer.upload_type == UploadType::ImageData {
        data_width
    } else {
        args.width
    };
    let actual_height = if buffer.upload_type == UploadType::ImageData {
        data_height
    } else {
        args.height
    };

    // Validate update region
    if args.x + actual_width > texture_resource.params.width
        || args.y + actual_height > texture_resource.params.height
    {
        return CmdResultTextureUpdate {
            success: false,
            message: "Update region exceeds texture bounds".into(),
        };
    }

    if args.mip_level >= texture_resource.params.mip_level_count {
        return CmdResultTextureUpdate {
            success: false,
            message: format!(
                "Mip level {} exceeds texture mip level count {}",
                args.mip_level, texture_resource.params.mip_level_count
            ),
        };
    }

    // Write texture data to GPU
    let texture_size = wgpu::Extent3d {
        width: actual_width,
        height: actual_height,
        depth_or_array_layers: 1,
    };

    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &texture_resource.texture,
            mip_level: args.mip_level,
            origin: wgpu::Origin3d {
                x: args.x,
                y: args.y,
                z: 0,
            },
            aspect: wgpu::TextureAspect::All,
        },
        &texture_data,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(4 * actual_width),
            rows_per_image: Some(actual_height),
        },
        texture_size,
    );

    // Remove upload buffer after use
    engine.buffers.remove(&args.buffer_id);

    CmdResultTextureUpdate {
        success: true,
        message: "Texture resource updated successfully".into(),
    }
}

// MARK: - Dispose Texture

/// Arguments for disposing a texture resource
#[derive(Debug, Deserialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdTextureDisposeArgs {
    pub texture_id: TextureId,
    pub window_id: u32,
}

impl Default for CmdTextureDisposeArgs {
    fn default() -> Self {
        Self {
            texture_id: 0,
            window_id: 0,
        }
    }
}

/// Result for texture dispose command
#[derive(Debug, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultTextureDispose {
    pub success: bool,
    pub message: String,
}

impl Default for CmdResultTextureDispose {
    fn default() -> Self {
        Self {
            success: false,
            message: String::new(),
        }
    }
}

/// Dispose a texture resource
pub fn engine_cmd_texture_dispose(
    engine: &mut EngineState,
    args: &CmdTextureDisposeArgs,
) -> CmdResultTextureDispose {
    // Validate window exists
    let window_state = match engine.windows.get_mut(&args.window_id) {
        Some(ws) => ws,
        None => {
            return CmdResultTextureDispose {
                success: false,
                message: format!("Window with id {} not found", args.window_id),
            };
        }
    };

    let render_state = &mut window_state.render_state;

    // Check if texture is in use by any materials
    let in_use = render_state
        .resources
        .materials
        .values()
        .any(|m| m.textures.contains(&args.texture_id));

    if in_use {
        return CmdResultTextureDispose {
            success: false,
            message: format!(
                "Texture {} is still in use by one or more materials",
                args.texture_id
            ),
        };
    }

    // Remove texture resource
    match render_state.resources.textures.remove(&args.texture_id) {
        Some(_) => CmdResultTextureDispose {
            success: true,
            message: "Texture resource disposed successfully".into(),
        },
        None => CmdResultTextureDispose {
            success: false,
            message: format!("Texture with id {} not found", args.texture_id),
        },
    }
}
