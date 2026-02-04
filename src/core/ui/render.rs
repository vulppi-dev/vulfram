use crate::core::cmd::EngineEvent;
use crate::core::render::state::RenderScene;
use crate::core::resources::{TextureRecord, ensure_render_target};

use super::build::build_ui_from_tree;
use super::egui_renderer::{ScreenDescriptor, UiEguiRenderer};
use super::state::UiState;

pub fn ensure_ui_render_targets(device: &wgpu::Device, ui_state: &mut UiState) {
    let target_format = ui_state.output_format;
    for (ctx_id, context) in ui_state.contexts.iter_mut() {
        let width = context.screen_rect.w.max(1.0).round() as u32;
        let height = context.screen_rect.h.max(1.0).round() as u32;
        let had_target = context.render_target.is_some();
        ensure_render_target(
            device,
            &mut context.render_target,
            width,
            height,
            target_format,
        );
        if !had_target && context.render_target.is_some() {
            log::info!(
                "Ui context {:?} render target created: {}x{}, format {:?}",
                ctx_id,
                width,
                height,
                target_format
            );
        }
    }
}

pub fn render_ui_for_window(
    ui_state: &mut UiState,
    ui_renderer: &mut Option<UiEguiRenderer>,
    event_queue: &mut Vec<EngineEvent>,
    window_id: u32,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    encoder: &mut wgpu::CommandEncoder,
) {
    if ui_state.contexts.is_empty() {
        return;
    }

    let target_format = ui_state.output_format;
    if let Some(existing) = ui_renderer.as_ref() {
        if existing.output_format() != target_format {
            *ui_renderer = None;
        }
    }
    let renderer = ui_renderer.get_or_insert_with(|| UiEguiRenderer::new(device, target_format));

    for (context_id, context) in ui_state.contexts.iter_mut() {
        if context.window_id != window_id {
            continue;
        }
        let target = match context.render_target.as_ref() {
            Some(target) => target,
            None => continue,
        };
        let width = context.screen_rect.w.max(1.0).round() as u32;
        let height = context.screen_rect.h.max(1.0).round() as u32;
        let target_view = &target.view;
        let clear_color = wgpu::Color::TRANSPARENT;

        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: [width, height],
            pixels_per_point: 1.0,
        };

        let events = ui_state
            .pending_events
            .remove(context_id)
            .unwrap_or_default();

        let raw_input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(width as f32, height as f32),
            )),
            events,
            ..Default::default()
        };

        context
            .egui_ctx
            .set_pixels_per_point(screen_descriptor.pixels_per_point);
        let output = context.egui_ctx.run(raw_input, |ctx| {
            build_ui_from_tree(ctx, event_queue, context_id, window_id, &mut context.tree);
        });

        let clipped_primitives = context
            .egui_ctx
            .tessellate(output.shapes, output.pixels_per_point);

        renderer.update_textures(device, queue, &output.textures_delta);
        let draw_calls =
            renderer.update_buffers(device, queue, &clipped_primitives, &screen_descriptor);
        renderer.render(
            encoder,
            target_view,
            &draw_calls,
            &screen_descriptor,
            clear_color,
        );

        if !context.debug_draw_logged {
            log::info!(
                "Ui context {:?} draw calls: {} (nodes: {})",
                context_id,
                draw_calls.len(),
                context.tree.nodes.len()
            );
            context.debug_draw_logged = true;
        }
    }
}

pub fn map_ui_targets_for_window(
    ui_state: &mut UiState,
    render_scene: &mut RenderScene,
    window_id: u32,
) {
    for (context_id, context) in ui_state.contexts.iter_mut() {
        if context.window_id != window_id {
            continue;
        }
        map_ui_target_to_texture(render_scene, context_id, context);
    }
}

fn map_ui_target_to_texture(
    render_scene: &mut RenderScene,
    context_id: &crate::core::render::graph::LogicalId,
    context: &mut super::state::UiContextRecord,
) {
    let target_id = match &context.target {
        super::types::UiRenderTarget::TextureId(id) => id,
    };

    let texture_id = match target_id {
        crate::core::render::graph::LogicalId::Int(value) => {
            if *value < 0 || *value > u32::MAX as i64 {
                if !context.debug_map_logged {
                    log::warn!("Ui target {:?} out of u32 range", target_id);
                    context.debug_map_logged = true;
                }
                return;
            }
            *value as u32
        }
        crate::core::render::graph::LogicalId::Str(_) => {
            if !context.debug_map_logged {
                log::warn!(
                    "Ui target {:?} must be an int to map to texture id",
                    target_id
                );
                context.debug_map_logged = true;
            }
            return;
        }
    };

    let Some(target) = context.render_target.as_ref() else {
        // Não emitir warning, pois o target pode não ter sido criado ainda no primeiro frame
        return;
    };

    if let Some(existing) = render_scene.textures.get(&texture_id) {
        if existing.label.as_deref() != Some("ui_target") {
            if !context.debug_map_logged {
                log::warn!(
                    "Texture id {} already in use; skipping ui target mapping for context {:?}",
                    texture_id,
                    context_id
                );
                context.debug_map_logged = true;
            }
            return;
        }
    }

    let size = target._texture.size();
    render_scene.textures.insert(
        texture_id,
        TextureRecord {
            label: Some("ui_target".into()),
            _size: size,
            _format: target.format,
            _texture: target._texture.clone(),
            view: target.view.clone(),
        },
    );

    invalidate_material_bind_groups(render_scene, texture_id);

    if !context.debug_map_logged {
        log::info!(
            "Ui context {:?} mapped to texture {} ({}x{}, format {:?})",
            context_id,
            texture_id,
            size.width,
            size.height,
            target.format
        );
        context.debug_map_logged = true;
    }
}

fn invalidate_material_bind_groups(render_scene: &mut RenderScene, texture_id: u32) {
    for record in render_scene.materials_standard.values_mut() {
        if record.texture_ids.iter().any(|id| *id == texture_id) {
            record.bind_group = None;
        }
    }
    for record in render_scene.materials_pbr.values_mut() {
        if record.texture_ids.iter().any(|id| *id == texture_id) {
            record.bind_group = None;
        }
    }
}
