use super::types::*;
use super::utils::*;
use crate::core::buffers::state::UploadType;
use crate::core::image::{ImageBuffer, ImagePixels};
use crate::core::resources::texture::{
    ForwardAtlasDesc, ForwardAtlasEntry, TextureDecodeJob, TextureRecord,
};
use crate::core::state::EngineState;
use crate::core::system::SystemEvent;
use glam::{UVec2, Vec4};

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
                pending: false,
            };
        }
    };

    if window_state
        .render_state
        .scene
        .textures
        .contains_key(&args.texture_id)
        || window_state
            .render_state
            .scene
            .forward_atlas_entries
            .contains_key(&args.texture_id)
        || engine.texture_async.is_pending(args.texture_id)
    {
        return CmdResultTextureCreateFromBuffer {
            success: false,
            message: format!("Texture with id {} already exists or pending", args.texture_id),
            pending: false,
        };
    }

    let buffer = match engine.buffers.remove_upload(args.buffer_id) {
        Some(b) => b,
        None => {
            return CmdResultTextureCreateFromBuffer {
                success: false,
                message: format!("Buffer with id {} not found", args.buffer_id),
                pending: false,
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
            pending: false,
        };
    }

    let job = TextureDecodeJob {
        window_id: args.window_id,
        texture_id: args.texture_id,
        label: args.label.clone(),
        srgb: args.srgb,
        mode: args.mode,
        atlas_options: args.atlas_options.clone(),
        bytes: buffer.data,
    };

    if let Err(message) = engine.texture_async.enqueue(job) {
        return CmdResultTextureCreateFromBuffer {
            success: false,
            message,
            pending: false,
        };
    }

    CmdResultTextureCreateFromBuffer {
        success: true,
        message: "Texture decode queued".into(),
        pending: true,
    }
}

fn create_texture_from_image(
    engine: &mut EngineState,
    args: &CmdTextureCreateFromBufferArgs,
    image: ImageBuffer,
) -> CmdResultTextureCreateFromBuffer {
    let window_state = match engine.window.states.get_mut(&args.window_id) {
        Some(ws) => ws,
        None => {
            return CmdResultTextureCreateFromBuffer {
                success: false,
                message: format!("Window {} not found", args.window_id),
                pending: false,
            };
        }
    };

    if window_state
        .render_state
        .scene
        .textures
        .contains_key(&args.texture_id)
        || window_state
            .render_state
            .scene
            .forward_atlas_entries
            .contains_key(&args.texture_id)
    {
        return CmdResultTextureCreateFromBuffer {
            success: false,
            message: format!("Texture with id {} already exists", args.texture_id),
            pending: false,
        };
    }

    let device = match engine.device.as_ref() {
        Some(d) => d,
        None => {
            return CmdResultTextureCreateFromBuffer {
                success: false,
                message: "Device not initialized".into(),
                pending: false,
            };
        }
    };

    let queue = match engine.queue.as_ref() {
        Some(q) => q,
        None => {
            return CmdResultTextureCreateFromBuffer {
                success: false,
                message: "Queue not initialized".into(),
                pending: false,
            };
        }
    };

    enum PixelUpload {
        Rgba8(Vec<u8>),
        Rgba16F(Vec<u16>),
    }

    let (format, bytes_per_row, rows_per_image, pixel_upload) = match image.pixels {
        ImagePixels::Rgba8(data) => {
            let format = if args.srgb.unwrap_or(true) {
                wgpu::TextureFormat::Rgba8UnormSrgb
            } else {
                wgpu::TextureFormat::Rgba8Unorm
            };
            (
                format,
                Some(4 * image.width),
                Some(image.height),
                PixelUpload::Rgba8(data),
            )
        }
        ImagePixels::Rgba16F(data) => {
            if matches!(args.mode, TextureCreateMode::ForwardAtlas) {
                return CmdResultTextureCreateFromBuffer {
                    success: false,
                    message: "Float textures are not supported in forward atlas".into(),
                    pending: false,
                };
            }
            (
                wgpu::TextureFormat::Rgba16Float,
                Some(8 * image.width),
                Some(image.height),
                PixelUpload::Rgba16F(data),
            )
        }
    };

    let size = wgpu::Extent3d {
        width: image.width,
        height: image.height,
        depth_or_array_layers: 1,
    };

    match &args.mode {
        TextureCreateMode::Standalone => {
            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: args.label.as_deref().or(Some("Texture From Buffer")),
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });

            let pixel_data: &[u8] = match &pixel_upload {
                PixelUpload::Rgba8(data) => data.as_slice(),
                PixelUpload::Rgba16F(data) => bytemuck::cast_slice(data),
            };

            queue.write_texture(
                texture.as_image_copy(),
                pixel_data,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row,
                    rows_per_image,
                },
                size,
            );

            let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

            window_state.render_state.scene.textures.insert(
                args.texture_id,
                TextureRecord {
                    label: args.label.clone(),
                    _texture: texture,
                    view,
                    _size: size,
                    _format: format,
                },
            );
        }
        TextureCreateMode::ForwardAtlas => {
            let options = args.atlas_options.clone().unwrap_or_default();
            let atlas_desc = ForwardAtlasDesc {
                label: Some("Forward Atlas"),
                format,
                usage: wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::COPY_DST
                    | wgpu::TextureUsages::COPY_SRC,
                tile_px: options.tile_px,
                layers: options.layers,
            };
            let (handle, transform, relocation_transforms) = {
                let atlas = match ensure_forward_atlas(
                    &mut window_state.render_state,
                    device,
                    queue,
                    &atlas_desc,
                ) {
                    Ok(atlas) => atlas,
                    Err(message) => {
                        return CmdResultTextureCreateFromBuffer {
                            success: false,
                            message,
                            pending: false,
                        };
                    }
                };
                let tiles_x = (image.width + options.tile_px - 1) / options.tile_px;
                let tiles_y = (image.height + options.tile_px - 1) / options.tile_px;
                let (handle, relocations) = match atlas.alloc(tiles_x, tiles_y) {
                    Some(result) => result,
                    None => {
                        return CmdResultTextureCreateFromBuffer {
                            success: false,
                            message: "Forward atlas allocation failed".into(),
                            pending: false,
                        };
                    }
                };
                let (x, y, _, _, layer) = match atlas.get_copy_rect(handle) {
                    Some(rect) => rect,
                    None => {
                        return CmdResultTextureCreateFromBuffer {
                            success: false,
                            message: "Forward atlas allocation invalid".into(),
                            pending: false,
                        };
                    }
                };
                let pixel_data: &[u8] = match &pixel_upload {
                    PixelUpload::Rgba8(data) => data.as_slice(),
                    PixelUpload::Rgba16F(data) => bytemuck::cast_slice(data),
                };

                queue.write_texture(
                    wgpu::TexelCopyTextureInfo {
                        texture: atlas.texture(),
                        mip_level: 0,
                        origin: wgpu::Origin3d { x, y, z: layer },
                        aspect: wgpu::TextureAspect::All,
                    },
                    pixel_data,
                    wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row,
                        rows_per_image,
                    },
                    wgpu::Extent3d {
                        width: image.width,
                        height: image.height,
                        depth_or_array_layers: 1,
                    },
                );
                let transform = match atlas.get_uv_transform(handle) {
                    Some(t) => t,
                    None => {
                        return CmdResultTextureCreateFromBuffer {
                            success: false,
                            message: "Forward atlas transform missing".into(),
                            pending: false,
                        };
                    }
                };
                let relocation_transforms: Vec<_> = relocations
                    .iter()
                    .map(|r| (r.handle, atlas.get_uv_transform(r.handle)))
                    .collect();
                (handle, transform, relocation_transforms)
            };

            for (handle, transform) in relocation_transforms {
                let mut affected_ids = Vec::new();
                if let Some(transform) = transform {
                    for (tex_id, entry) in window_state
                        .render_state
                        .scene
                        .forward_atlas_entries
                        .iter_mut()
                    {
                        if entry.handle == handle {
                            entry.uv_scale_bias =
                                Vec4::new(transform.0, transform.1, transform.2, transform.3);
                            entry.layer = transform.4;
                            affected_ids.push(*tex_id);
                        }
                    }
                }
                for tex_id in affected_ids {
                    mark_materials_dirty(&mut window_state.render_state.scene, tex_id);
                }
            }

            window_state
                .render_state
                .scene
                .forward_atlas_entries
                .insert(
                    args.texture_id,
                    ForwardAtlasEntry {
                        label: args.label.clone(),
                        handle,
                        _size: UVec2::new(image.width, image.height),
                        uv_scale_bias: Vec4::new(
                            transform.0,
                            transform.1,
                            transform.2,
                            transform.3,
                        ),
                        layer: transform.4,
                        _format: format,
                    },
                );
        }
    }

    mark_materials_dirty(&mut window_state.render_state.scene, args.texture_id);
    window_state.is_dirty = true;

    CmdResultTextureCreateFromBuffer {
        success: true,
        message: "Texture created successfully".into(),
        pending: false,
    }
}

pub fn process_async_texture_results(engine: &mut EngineState) {
    let results = engine.texture_async.drain_results();
    for result in results {
        let args = CmdTextureCreateFromBufferArgs {
            window_id: result.window_id,
            texture_id: result.texture_id,
            label: result.label.clone(),
            buffer_id: 0,
            srgb: result.srgb,
            mode: result.mode,
            atlas_options: result.atlas_options.clone(),
        };

        let response = match result.image {
            Some(image) => create_texture_from_image(engine, &args, image),
            None => CmdResultTextureCreateFromBuffer {
                success: false,
                message: result.message.clone(),
                pending: false,
            },
        };

        engine.event_queue.push(crate::core::cmd::EngineEvent::System(
            SystemEvent::TextureReady {
                window_id: result.window_id,
                texture_id: result.texture_id,
                success: response.success,
                message: response.message,
            },
        ));
    }
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
        || window_state
            .render_state
            .scene
            .forward_atlas_entries
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
        width: 1,
        height: 1,
        depth_or_array_layers: 1,
    };

    let r = (args.color.x.clamp(0.0, 1.0) * 255.0) as u8;
    let g = (args.color.y.clamp(0.0, 1.0) * 255.0) as u8;
    let b = (args.color.z.clamp(0.0, 1.0) * 255.0) as u8;
    let a = (args.color.w.clamp(0.0, 1.0) * 255.0) as u8;
    let data = [r, g, b, a];

    match &args.mode {
        TextureCreateMode::Standalone => {
            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: args.label.as_deref().or(Some("Solid Color Texture")),
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
                &data,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(4),
                    rows_per_image: Some(1),
                },
                size,
            );

            let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

            window_state.render_state.scene.textures.insert(
                args.texture_id,
                TextureRecord {
                    label: args.label.clone(),
                    _texture: texture,
                    view,
                    _size: size,
                    _format: format,
                },
            );
        }
        TextureCreateMode::ForwardAtlas => {
            let options = args.atlas_options.clone().unwrap_or_default();
            let atlas_desc = ForwardAtlasDesc {
                label: Some("Forward Atlas"),
                format,
                usage: wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::COPY_DST
                    | wgpu::TextureUsages::COPY_SRC,
                tile_px: options.tile_px,
                layers: options.layers,
            };
            let (handle, transform, relocation_transforms) = {
                let atlas = match ensure_forward_atlas(
                    &mut window_state.render_state,
                    device,
                    queue,
                    &atlas_desc,
                ) {
                    Ok(atlas) => atlas,
                    Err(message) => {
                        return CmdResultTextureCreateSolidColor {
                            success: false,
                            message,
                        };
                    }
                };
                let tiles_x = (size.width + options.tile_px - 1) / options.tile_px;
                let tiles_y = (size.height + options.tile_px - 1) / options.tile_px;
                let (handle, relocations) = match atlas.alloc(tiles_x, tiles_y) {
                    Some(result) => result,
                    None => {
                        return CmdResultTextureCreateSolidColor {
                            success: false,
                            message: "Forward atlas allocation failed".into(),
                        };
                    }
                };
                let (x, y, _, _, layer) = match atlas.get_copy_rect(handle) {
                    Some(rect) => rect,
                    None => {
                        return CmdResultTextureCreateSolidColor {
                            success: false,
                            message: "Forward atlas allocation invalid".into(),
                        };
                    }
                };
                queue.write_texture(
                    wgpu::TexelCopyTextureInfo {
                        texture: atlas.texture(),
                        mip_level: 0,
                        origin: wgpu::Origin3d { x, y, z: layer },
                        aspect: wgpu::TextureAspect::All,
                    },
                    &data,
                    wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(4 * size.width),
                        rows_per_image: Some(size.height),
                    },
                    wgpu::Extent3d {
                        width: size.width,
                        height: size.height,
                        depth_or_array_layers: 1,
                    },
                );
                let transform = match atlas.get_uv_transform(handle) {
                    Some(t) => t,
                    None => {
                        return CmdResultTextureCreateSolidColor {
                            success: false,
                            message: "Forward atlas transform missing".into(),
                        };
                    }
                };
                let relocation_transforms: Vec<_> = relocations
                    .iter()
                    .map(|r| (r.handle, atlas.get_uv_transform(r.handle)))
                    .collect();
                (handle, transform, relocation_transforms)
            };

            for (handle, transform) in relocation_transforms {
                let mut affected_ids = Vec::new();
                if let Some(transform) = transform {
                    for (tex_id, entry) in window_state
                        .render_state
                        .scene
                        .forward_atlas_entries
                        .iter_mut()
                    {
                        if entry.handle == handle {
                            entry.uv_scale_bias =
                                Vec4::new(transform.0, transform.1, transform.2, transform.3);
                            entry.layer = transform.4;
                            affected_ids.push(*tex_id);
                        }
                    }
                }
                for tex_id in affected_ids {
                    mark_materials_dirty(&mut window_state.render_state.scene, tex_id);
                }
            }

            window_state
                .render_state
                .scene
                .forward_atlas_entries
                .insert(
                    args.texture_id,
                    ForwardAtlasEntry {
                        label: args.label.clone(),
                        handle,
                        _size: UVec2::new(size.width, size.height),
                        uv_scale_bias: Vec4::new(
                            transform.0,
                            transform.1,
                            transform.2,
                            transform.3,
                        ),
                        layer: transform.4,
                        _format: format,
                    },
                );
        }
    }

    mark_materials_dirty(&mut window_state.render_state.scene, args.texture_id);
    window_state.is_dirty = true;

    CmdResultTextureCreateSolidColor {
        success: true,
        message: "Texture created successfully".into(),
    }
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
        mark_materials_dirty(&mut window_state.render_state.scene, args.texture_id);
        window_state.is_dirty = true;
        return CmdResultTextureDispose {
            success: true,
            message: "Texture disposed successfully".into(),
        };
    }

    if let Some(entry) = window_state
        .render_state
        .scene
        .forward_atlas_entries
        .remove(&args.texture_id)
    {
        if let Some(atlas) = window_state.render_state.forward_atlas.as_mut() {
            let _ = atlas.free(entry.handle);
        }
        mark_materials_dirty(&mut window_state.render_state.scene, args.texture_id);
        window_state.is_dirty = true;
        return CmdResultTextureDispose {
            success: true,
            message: "Texture disposed successfully".into(),
        };
    }

    CmdResultTextureDispose {
        success: false,
        message: format!("Texture with id {} not found", args.texture_id),
    }
}
