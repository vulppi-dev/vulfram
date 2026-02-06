use crate::core::cmd::EngineEvent;
use crate::core::render::state::RenderScene;
use crate::core::resources::{TextureRecord, ensure_render_target};

use super::build::build_ui_from_tree;
use super::egui_renderer::{ScreenDescriptor, UiEguiRenderer};
use super::state::UiState;
use super::tree::UiTreeState;
use super::types::UiRenderTarget;
use crate::core::ui::animation::update_animations;
use crate::core::ui::theme::apply_theme;

/// Garante que os render targets da UI estão criados e com o tamanho correto.
/// Chamado durante a fase de preparação da renderização.
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

/// Renderiza a UI para uma janela específica.
///
/// Este é o ponto principal de renderização da UI. Executa o loop do egui,
/// processa eventos, aplica viewport requests (que redimensionam câmeras),
/// e renderiza os widgets para o render target da UI.
///
/// Fluxo:
/// 1. Executa egui_ctx.run() com eventos de input
/// 2. Gera viewport requests (para widgets Image com cameraId)
/// 3. Aplica viewport requests imediatamente (redimensiona render targets das câmeras)
/// 4. Registra texturas externas (camera targets) no egui
/// 5. Renderiza a UI para o target
pub fn render_ui_for_window(
    ui_state: &mut UiState,
    ui_renderer: &mut Option<UiEguiRenderer>,
    event_queue: &mut Vec<EngineEvent>,
    window_id: u32,
    render_scene: &mut RenderScene,
    pixels_per_point: f32,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    encoder: &mut wgpu::CommandEncoder,
    time_seconds: f64,
    filter_target: Option<bool>, // None = all, Some(true) = TextureId only, Some(false) = Screen only
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
        // Filter by target type if specified
        if let Some(is_texture_id) = filter_target {
            let context_is_texture_id = matches!(context.target, UiRenderTarget::TextureId(_));
            if context_is_texture_id != is_texture_id {
                continue;
            }
        }
        if let Some(theme_id) = context.theme_id.as_ref() {
            if let Some(theme) = ui_state.themes.get(theme_id) {
                if context.applied_theme_id.as_ref() != Some(theme_id)
                    || theme.version != context.applied_theme_version
                {
                    apply_theme(&context.egui_ctx, &theme.theme);
                    if let Some(debug) = theme.theme.debug {
                        context.debug_enabled = debug;
                    }
                    context.applied_theme_version = theme.version;
                    context.applied_theme_id = Some(theme_id.clone());
                    context.applied_theme_fallback = false;
                }
            }
        } else if !context.applied_theme_fallback {
            apply_theme(&context.egui_ctx, &ui_state.fallback_theme);
            if let Some(debug) = ui_state.fallback_theme.debug {
                context.debug_enabled = debug;
            }
            context.applied_theme_version = 0;
            context.applied_theme_id = None;
            context.applied_theme_fallback = true;
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
            pixels_per_point,
        };

        let events = ui_state
            .pending_events
            .remove(context_id)
            .unwrap_or_default();

        let screen_w = width as f32 / pixels_per_point;
        let screen_h = height as f32 / pixels_per_point;
        let raw_input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(screen_w, screen_h),
            )),
            events,
            time: Some(time_seconds),
            ..Default::default()
        };

        context
            .egui_ctx
            .set_pixels_per_point(screen_descriptor.pixels_per_point);
        update_animations(
            &mut context.animations,
            &mut context.animated_overrides,
            &context.tree,
            event_queue,
            context_id,
            window_id,
            time_seconds,
        );
        context.viewport_requests.clear();
        context.node_rects.clear();
        let output = context.egui_ctx.run(raw_input, |ctx| {
            build_ui_from_tree(
                ctx,
                event_queue,
                context_id,
                window_id,
                &mut context.tree,
                &mut context.focused_node,
                &mut context.viewport_requests,
                &mut context.style_cache,
                &mut context.ordered_children_cache,
                &context.animated_overrides,
                &mut context.node_rects,
                context.debug_enabled,
            );
        });

        // Aplica viewport requests imediatamente para que o próximo frame use os tamanhos corretos
        apply_viewport_requests(
            render_scene,
            device,
            pixels_per_point,
            &context.viewport_requests,
        );

        let clipped_primitives = context
            .egui_ctx
            .tessellate(output.shapes, output.pixels_per_point);

        renderer.update_textures(device, queue, &output.textures_delta);

        let used_images = register_image_textures(renderer, device, render_scene, &context.tree);
        let draw_calls =
            renderer.update_buffers(device, queue, &clipped_primitives, &screen_descriptor);
        renderer.render(
            encoder,
            target_view,
            &draw_calls,
            &screen_descriptor,
            clear_color,
        );
        renderer.prune_external_textures(&used_images);

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

fn apply_viewport_requests(
    render_scene: &mut RenderScene,
    device: &wgpu::Device,
    pixels_per_point: f32,
    requests: &[crate::core::ui::state::ViewportRequest],
) {
    if requests.is_empty() {
        return;
    }

    for request in requests {
        let Some(camera) = render_scene.cameras.get_mut(&request.camera_id) else {
            log::warn!(
                "Camera {} not found for viewport request",
                request.camera_id
            );
            continue;
        };
        let width = (request.size_points.x * pixels_per_point).round().max(1.0) as u32;
        let height = (request.size_points.y * pixels_per_point).round().max(1.0) as u32;

        // Early exit se o tamanho já está correto
        let needs_resize = match camera.render_target.as_ref() {
            Some(target) => {
                let size = target._texture.size();
                size.width != width || size.height != height
            }
            None => true,
        };
        if !needs_resize {
            continue;
        }

        // Atualiza todos os render targets com o novo tamanho
        crate::core::resources::ensure_render_target(
            device,
            &mut camera.render_target,
            width,
            height,
            wgpu::TextureFormat::Rgba16Float,
        );
        crate::core::resources::ensure_render_target(
            device,
            &mut camera.emissive_target,
            width,
            height,
            wgpu::TextureFormat::Rgba16Float,
        );
        crate::core::resources::ensure_render_target(
            device,
            &mut camera.post_target,
            width,
            height,
            wgpu::TextureFormat::Rgba16Float,
        );
        crate::core::resources::ensure_render_target(
            device,
            &mut camera.outline_target,
            width,
            height,
            wgpu::TextureFormat::Rgba8Unorm,
        );
        crate::core::resources::ensure_render_target(
            device,
            &mut camera.ssao_target,
            width,
            height,
            wgpu::TextureFormat::Rgba16Float,
        );
        crate::core::resources::ensure_render_target(
            device,
            &mut camera.ssao_blur_target,
            width,
            height,
            wgpu::TextureFormat::Rgba16Float,
        );
        crate::core::resources::ensure_render_target(
            device,
            &mut camera.bloom_target,
            width,
            height,
            wgpu::TextureFormat::Rgba16Float,
        );
        for (level, target) in camera.bloom_chain.iter_mut().enumerate() {
            let level_width = crate::core::render::bloom_chain_size(width, level);
            let level_height = crate::core::render::bloom_chain_size(height, level);
            crate::core::resources::ensure_render_target(
                device,
                target,
                level_width,
                level_height,
                wgpu::TextureFormat::Rgba16Float,
            );
        }
        camera
            .data
            .update(None, None, None, None, (width, height), camera.ortho_scale);
        camera.mark_dirty();
    }
}

fn register_image_textures(
    renderer: &mut UiEguiRenderer,
    device: &wgpu::Device,
    render_scene: &RenderScene,
    tree: &UiTreeState,
) -> std::collections::HashSet<egui::TextureId> {
    let mut used = std::collections::HashSet::new();
    let texture_ids = collect_image_texture_ids(tree);
    for texture_id in texture_ids {
        let Some(texture) = render_scene.textures.get(&texture_id) else {
            log::warn!("UI image texture {} not found in render scene", texture_id);
            continue;
        };
        let egui_id = egui::TextureId::User(texture_id as u64);
        used.insert(egui_id);
        renderer.register_external_texture(
            device,
            egui_id,
            &texture.view,
            egui::epaint::textures::TextureOptions::LINEAR,
        );
    }
    used
}

fn collect_image_texture_ids(tree: &UiTreeState) -> Vec<u32> {
    let mut ids = Vec::new();
    for node in tree.nodes.values() {
        if node.node_type != crate::core::ui::tree::UiNodeType::Image {
            continue;
        }
        let Some(props) = node.props.as_ref() else {
            continue;
        };
        let Some(value) = props.get("textureId") else {
            continue;
        };
        if let Some(id) = ui_value_u32(value) {
            ids.push(id);
        }
    }
    ids
}

fn ui_value_u32(value: &crate::core::ui::types::UiValue) -> Option<u32> {
    match value {
        crate::core::ui::types::UiValue::Int(value) => u32::try_from(*value).ok(),
        crate::core::ui::types::UiValue::Float(value) => {
            if *value >= 0.0 && *value <= u32::MAX as f64 {
                Some(*value as u32)
            } else {
                None
            }
        }
        _ => None,
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
        super::types::UiRenderTarget::Screen => return,
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
