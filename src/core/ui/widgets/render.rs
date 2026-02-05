use std::collections::HashMap;

use crate::core::cmd::EngineEvent;
use crate::core::render::graph::LogicalId;

use super::dock::render_dock;
use super::events::{emit_ui_event, handle_focus_events, handle_viewport_events};
use super::values::{
    ui_value_bool, ui_value_float, ui_value_string, ui_value_u32, update_node_prop,
};
use crate::core::ui::layout::{
    apply_style_overrides, build_style_cache, resolve_layout_from_cache, resolve_size_from_cache,
    UiStyleCacheEntry,
};
use crate::core::ui::tree::{UiEventKind, UiNodeType, UiStyle, UiTreeState};
use crate::core::ui::types::UiValue;

pub fn render_children(
    ui: &mut egui::Ui,
    event_queue: &mut Vec<EngineEvent>,
    context_id: &LogicalId,
    window_id: u32,
    tree: &mut UiTreeState,
    focused_node: &mut Option<LogicalId>,
    viewport_requests: &mut Vec<crate::core::ui::state::ViewportRequest>,
    style_cache: &mut HashMap<LogicalId, UiStyleCacheEntry>,
    ordered_children_cache: &mut HashMap<LogicalId, Vec<LogicalId>>,
    animated_overrides: &HashMap<LogicalId, UiStyle>,
    node_rects: &mut HashMap<LogicalId, egui::Rect>,
    debug_enabled: bool,
    sizing_pass: bool,
    node_id: &LogicalId,
) {
    let sizing_pass = sizing_pass || ui.is_sizing_pass();
    let mut debug_enabled = debug_enabled;
    if node_id == &LogicalId::Str("root".into()) {
        if let Some(root) = tree.nodes.get(node_id) {
            if let Some(style) = root.style.as_ref() {
                if let Some(debug) = style.get("debug").and_then(ui_value_bool) {
                    debug_enabled = debug;
                }
            }
        }
    }
    let ordered = get_ordered_children(
        tree,
        style_cache,
        ordered_children_cache,
        animated_overrides,
        node_id,
    );

    for child_id in ordered {
        render_node(
            ui,
            event_queue,
            context_id,
            window_id,
            tree,
            focused_node,
            viewport_requests,
            style_cache,
            ordered_children_cache,
            animated_overrides,
            node_rects,
            debug_enabled,
            sizing_pass,
            &child_id,
        );
    }

    if debug_enabled && node_id == &LogicalId::Str("root".into()) {
        draw_debug_overlay(ui, node_rects, tree.nodes.len());
    }
}

fn render_node(
    ui: &mut egui::Ui,
    event_queue: &mut Vec<EngineEvent>,
    context_id: &LogicalId,
    window_id: u32,
    tree: &mut UiTreeState,
    focused_node: &mut Option<LogicalId>,
    viewport_requests: &mut Vec<crate::core::ui::state::ViewportRequest>,
    style_cache: &mut HashMap<LogicalId, UiStyleCacheEntry>,
    ordered_children_cache: &mut HashMap<LogicalId, Vec<LogicalId>>,
    animated_overrides: &HashMap<LogicalId, UiStyle>,
    node_rects: &mut HashMap<LogicalId, egui::Rect>,
    debug_enabled: bool,
    sizing_pass: bool,
    node_id: &LogicalId,
) {
    let Some(node) = tree.nodes.get(node_id).cloned() else {
        return;
    };

    let entry = style_entry(tree, style_cache, animated_overrides, node_id);
    if !entry.display {
        return;
    }

    let mut entry_for_render = entry.clone();
    entry_for_render.translate_y = 0.0;
    let mut render_with = |ui: &mut egui::Ui, sizing_pass: bool| {
        let entry = entry_for_render.clone();
        render_node_contents(
            ui,
            event_queue,
            context_id,
            window_id,
            tree,
            focused_node,
            viewport_requests,
            style_cache,
            ordered_children_cache,
            animated_overrides,
            node_rects,
            debug_enabled,
            sizing_pass,
            node_id,
            &node,
            entry,
        );
    };

    if entry.translate_y != 0.0 && !sizing_pass {
        let base_rect = ui.available_rect_before_wrap();
        ui.scope_builder(
            egui::UiBuilder::new()
                .max_rect(base_rect)
                .sizing_pass(),
            |ui| {
                render_with(ui, true);
            },
        );

        let area_id = egui::Id::new(format!("ui_translate_{}", node_id));
        egui::Area::new(area_id)
            .fixed_pos(base_rect.min + egui::vec2(0.0, entry.translate_y))
            .order(egui::Order::Foreground)
            .show(ui.ctx(), |ui| {
                ui.set_min_size(base_rect.size());
                render_with(ui, false);
            });
        return;
    }

    let visible = entry.visible;
    let opacity = entry.opacity;
    let mut builder = egui::UiBuilder::new();
    if !visible {
        builder = builder.invisible();
    }
    if opacity <= 0.0 {
        builder = builder.disabled();
    }
    ui.scope_builder(builder, |ui| {
        if opacity < 1.0 {
            ui.set_opacity(opacity);
        }
        render_with(ui, sizing_pass);
    });
}

fn render_node_contents(
    ui: &mut egui::Ui,
    event_queue: &mut Vec<EngineEvent>,
    context_id: &LogicalId,
    window_id: u32,
    tree: &mut UiTreeState,
    focused_node: &mut Option<LogicalId>,
    viewport_requests: &mut Vec<crate::core::ui::state::ViewportRequest>,
    style_cache: &mut HashMap<LogicalId, UiStyleCacheEntry>,
    ordered_children_cache: &mut HashMap<LogicalId, Vec<LogicalId>>,
    animated_overrides: &HashMap<LogicalId, UiStyle>,
    node_rects: &mut HashMap<LogicalId, egui::Rect>,
    debug_enabled: bool,
    sizing_pass: bool,
    node_id: &LogicalId,
    node: &crate::core::ui::tree::UiNode,
    entry: UiStyleCacheEntry,
) {
        match node.node_type {
            UiNodeType::Container => {
                render_container(
                    ui,
                    event_queue,
                    context_id,
                    window_id,
                    tree,
                    focused_node,
                    viewport_requests,
                    style_cache,
                    ordered_children_cache,
                    animated_overrides,
                    node_rects,
                    debug_enabled,
                    sizing_pass,
                    node_id,
                    node.style.as_ref(),
                    entry,
                );
            }
            UiNodeType::Dock => {
                render_dock(
                    ui,
                    event_queue,
                    context_id,
                    window_id,
                    tree,
                    render_node,
                    focused_node,
                    viewport_requests,
                    style_cache,
                    ordered_children_cache,
                    animated_overrides,
                    node_rects,
                    debug_enabled,
                    sizing_pass,
                    node_id,
                    node.props.as_ref(),
                    node.listeners.as_ref(),
                );
            }
            UiNodeType::Text => {
                let value = node
                    .props
                    .as_ref()
                    .and_then(|props| props.get("value"))
                    .and_then(ui_value_string)
                    .unwrap_or_default();
                if let Some(text_style) = entry.text_style.as_ref() {
                    let style_name = egui::TextStyle::Name(text_style.clone().into());
                    let response = ui
                        .scope(|ui| {
                            ui.style_mut().override_text_style = Some(style_name);
                            ui.label(text_with_style(value, entry))
                        })
                        .inner;
                    record_rect(node_rects, node_id, response.rect);
                } else {
                    let response = ui.label(text_with_style(value, entry));
                    record_rect(node_rects, node_id, response.rect);
                }
            }
            UiNodeType::Button => {
                let label = node
                    .props
                    .as_ref()
                    .and_then(|props| props.get("label"))
                    .and_then(ui_value_string)
                    .unwrap_or_else(|| "Button".into());
                let response = ui.add(egui::Button::new(text_with_style(label, entry)));
                record_rect(node_rects, node_id, response.rect);
                if !sizing_pass && response.clicked() {
                    if let Some(listeners) = node.listeners.as_ref() {
                        if let Some(label) = listeners.on_click.clone() {
                            emit_ui_event(
                                event_queue,
                                window_id,
                                context_id,
                                label,
                                UiEventKind::Click,
                                Some(node_id.clone()),
                                None,
                            );
                        }
                    }
                }
            }
            UiNodeType::Input => {
                let mut value = node
                    .props
                    .as_ref()
                    .and_then(|props| props.get("value"))
                    .and_then(ui_value_string)
                    .unwrap_or_default();
                let mut text_edit = egui::TextEdit::singleline(&mut value);
                if let Some(size) = entry.font_size {
                    text_edit = text_edit.font(egui::FontSelection::FontId(egui::FontId::new(
                        size,
                        egui::FontFamily::Proportional,
                    )));
                }
                let response = ui.add(text_edit);
                record_rect(node_rects, node_id, response.rect);
                if !sizing_pass {
                    handle_focus_events(
                        event_queue,
                        context_id,
                        window_id,
                        node_id,
                        &node.listeners,
                        focused_node,
                        response.has_focus(),
                    );
                    if response.changed() {
                        update_node_prop(tree, node_id, "value", UiValue::String(value.clone()));
                        if let Some(listeners) = node.listeners.as_ref() {
                            if let Some(label) = listeners.on_change.clone() {
                                emit_ui_event(
                                    event_queue,
                                    window_id,
                                    context_id,
                                    label,
                                    UiEventKind::Change,
                                    Some(node_id.clone()),
                                    Some(UiValue::String(value.clone())),
                                );
                            }
                        }
                    }
                    if response.lost_focus() {
                        if let Some(listeners) = node.listeners.as_ref() {
                            if let Some(label) = listeners.on_change_commit.clone() {
                                emit_ui_event(
                                    event_queue,
                                    window_id,
                                    context_id,
                                    label,
                                    UiEventKind::Change,
                                    Some(node_id.clone()),
                                    Some(UiValue::String(value.clone())),
                                );
                            }
                            if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                                if let Some(label) = listeners.on_submit.clone() {
                                    emit_ui_event(
                                        event_queue,
                                        window_id,
                                        context_id,
                                        label,
                                        UiEventKind::Submit,
                                        Some(node_id.clone()),
                                        Some(UiValue::String(value.clone())),
                                    );
                                }
                            }
                        }
                    }
                }
            }
            UiNodeType::Checkbox => {
                let mut value = node
                    .props
                    .as_ref()
                    .and_then(|props| props.get("value"))
                    .and_then(ui_value_bool)
                    .unwrap_or(false);
                let response = ui.checkbox(&mut value, "");
                record_rect(node_rects, node_id, response.rect);
                if !sizing_pass && response.changed() {
                    update_node_prop(tree, node_id, "value", UiValue::Bool(value));
                    if let Some(listeners) = node.listeners.as_ref() {
                        if let Some(label) = listeners.on_change.clone() {
                            emit_ui_event(
                                event_queue,
                                window_id,
                                context_id,
                                label,
                                UiEventKind::Change,
                                Some(node_id.clone()),
                                Some(UiValue::Bool(value)),
                            );
                        }
                    }
                }
            }
            UiNodeType::Slider => {
                let mut value = node
                    .props
                    .as_ref()
                    .and_then(|props| props.get("value"))
                    .and_then(ui_value_float)
                    .unwrap_or(0.0);
                let min = node
                    .props
                    .as_ref()
                    .and_then(|props| props.get("min"))
                    .and_then(ui_value_float)
                    .unwrap_or(0.0);
                let max = node
                    .props
                    .as_ref()
                    .and_then(|props| props.get("max"))
                    .and_then(ui_value_float)
                    .unwrap_or(1.0);
                let response = ui.add(egui::Slider::new(&mut value, min..=max));
                record_rect(node_rects, node_id, response.rect);
                if !sizing_pass && response.changed() {
                    update_node_prop(tree, node_id, "value", UiValue::Float(value as f64));
                    if let Some(listeners) = node.listeners.as_ref() {
                        if let Some(label) = listeners.on_change.clone() {
                            emit_ui_event(
                                event_queue,
                                window_id,
                                context_id,
                                label,
                                UiEventKind::Change,
                                Some(node_id.clone()),
                                Some(UiValue::Float(value as f64)),
                            );
                        }
                    }
                }
            }
            UiNodeType::Scroll => {
                let scroll_x = entry.scroll_x;
                let scroll_y = entry.scroll_y;
                let mut scroll_area = egui::ScrollArea::new([scroll_x, scroll_y])
                    .id_salt(node_id.to_string())
                    .scroll_bar_visibility(entry.scroll_bar_visibility);
                let (width, height, has_size) = resolve_size_from_cache(ui, &entry);
                if has_size {
                    scroll_area = scroll_area.max_width(width).max_height(height);
                }
                let scroll_offset_x = node
                    .props
                    .as_ref()
                    .and_then(|props| props.get("scrollX"))
                    .and_then(ui_value_float)
                    .unwrap_or(0.0);
                let scroll_offset_y = node
                    .props
                    .as_ref()
                    .and_then(|props| props.get("scrollY"))
                    .and_then(ui_value_float)
                    .unwrap_or(0.0);
                let output = scroll_area
                    .scroll_offset(egui::vec2(scroll_offset_x, scroll_offset_y))
                    .show(ui, |ui| {
                        render_children(
                            ui,
                            event_queue,
                            context_id,
                            window_id,
                            tree,
                            focused_node,
                            viewport_requests,
                            style_cache,
                            ordered_children_cache,
                            animated_overrides,
                            node_rects,
                            debug_enabled,
                            sizing_pass,
                            node_id,
                        );
                    });
                record_rect(node_rects, node_id, output.inner_rect);
                if !sizing_pass {
                    update_node_prop(
                        tree,
                        node_id,
                        "scrollX",
                        UiValue::Float(output.state.offset.x as f64),
                    );
                    update_node_prop(
                        tree,
                        node_id,
                        "scrollY",
                        UiValue::Float(output.state.offset.y as f64),
                    );
                }
            }
            UiNodeType::Separator => {
                let response = ui.separator();
                record_rect(node_rects, node_id, response.rect);
            }
            UiNodeType::Spacer => {
                let (width, height, has_size) = resolve_size_from_cache(ui, &entry);
                let size = if has_size {
                    egui::vec2(width, height)
                } else {
                    egui::vec2(0.0, 4.0)
                };
                let (_id, rect) = ui.allocate_space(size);
                record_rect(node_rects, node_id, rect);
            }
            UiNodeType::Image => {
                let texture_id = node
                    .props
                    .as_ref()
                    .and_then(|props| props.get("textureId"))
                    .and_then(ui_value_u32);
                let camera_id = node
                    .props
                    .as_ref()
                    .and_then(|props| props.get("cameraId"))
                    .and_then(ui_value_u32);
                let (width, height, has_size) = resolve_size_from_cache(ui, &entry);
                let size = if has_size {
                    egui::vec2(width, height)
                } else {
                    egui::vec2(64.0, 64.0)
                };
                if let Some(texture_id) = texture_id {
                    let tex =
                        egui::load::SizedTexture::new(egui::TextureId::User(texture_id as u64), size);
                    let response =
                        ui.add(egui::Image::new(tex).sense(egui::Sense::click_and_drag()));
                    record_rect(node_rects, node_id, response.rect);
                    if !sizing_pass {
                        if let Some(camera_id) = camera_id {
                            viewport_requests.push(crate::core::ui::state::ViewportRequest {
                                camera_id,
                                size_points: response.rect.size(),
                            });

                            handle_viewport_events(
                                event_queue,
                                window_id,
                                context_id,
                                &response,
                                camera_id,
                                node,
                                node_id,
                            );
                        }
                    }
                } else {
                    let response = ui.label("Image: missing textureId");
                    record_rect(node_rects, node_id, response.rect);
                }
            }
            UiNodeType::Select => {
                let response = ui.label("Unsupported widget");
                record_rect(node_rects, node_id, response.rect);
            }
        }
}

fn render_container(
    ui: &mut egui::Ui,
    event_queue: &mut Vec<EngineEvent>,
    context_id: &LogicalId,
    window_id: u32,
    tree: &mut UiTreeState,
    focused_node: &mut Option<LogicalId>,
    viewport_requests: &mut Vec<crate::core::ui::state::ViewportRequest>,
    style_cache: &mut HashMap<LogicalId, UiStyleCacheEntry>,
    ordered_children_cache: &mut HashMap<LogicalId, Vec<LogicalId>>,
    animated_overrides: &HashMap<LogicalId, UiStyle>,
    node_rects: &mut HashMap<LogicalId, egui::Rect>,
    debug_enabled: bool,
    sizing_pass: bool,
    node_id: &LogicalId,
    style: Option<&UiStyle>,
    entry: UiStyleCacheEntry,
) {
    let gap = entry.gap;
    let padding = entry.padding;
    let (width, height, has_size) = resolve_size_from_cache(ui, &entry);
    let (layout, is_grid) = resolve_layout_from_cache(&entry);

    let mut render_children_fn = |ui: &mut egui::Ui| {
        let previous_spacing = ui.spacing().clone();
        ui.spacing_mut().item_spacing = gap;

        if is_grid {
            render_grid(
                ui,
                event_queue,
                context_id,
                window_id,
                tree,
                focused_node,
                viewport_requests,
                style_cache,
                ordered_children_cache,
                animated_overrides,
                node_rects,
                debug_enabled,
                sizing_pass,
                node_id,
                style,
            );
        } else {
            ui.with_layout(layout, |ui| {
                render_children(
                    ui,
                    event_queue,
                    context_id,
                    window_id,
                    tree,
                    focused_node,
                    viewport_requests,
                    style_cache,
                    ordered_children_cache,
                    animated_overrides,
                    node_rects,
                    debug_enabled,
                    sizing_pass,
                    node_id,
                );
            });
        }

        *ui.spacing_mut() = previous_spacing;
    };

    if padding != egui::Margin::ZERO {
        let frame = egui::Frame::none().inner_margin(padding);
        let response = if has_size {
            frame
                .show(ui, |ui| {
                    ui.allocate_ui_with_layout(
                        egui::vec2(width, height),
                        layout,
                        render_children_fn,
                    )
                })
                .response
        } else {
            frame.show(ui, render_children_fn).response
        };
        record_rect(node_rects, node_id, response.rect);
        return;
    }

    if has_size {
        let response =
            ui.allocate_ui_with_layout(egui::vec2(width, height), layout, render_children_fn)
                .response;
        record_rect(node_rects, node_id, response.rect);
        return;
    }

    render_children_fn(ui);
    record_rect(node_rects, node_id, ui.min_rect());
}

fn render_grid(
    ui: &mut egui::Ui,
    event_queue: &mut Vec<EngineEvent>,
    context_id: &LogicalId,
    window_id: u32,
    tree: &mut UiTreeState,
    focused_node: &mut Option<LogicalId>,
    viewport_requests: &mut Vec<crate::core::ui::state::ViewportRequest>,
    style_cache: &mut HashMap<LogicalId, UiStyleCacheEntry>,
    ordered_children_cache: &mut HashMap<LogicalId, Vec<LogicalId>>,
    animated_overrides: &HashMap<LogicalId, UiStyle>,
    node_rects: &mut HashMap<LogicalId, egui::Rect>,
    debug_enabled: bool,
    sizing_pass: bool,
    node_id: &LogicalId,
    style: Option<&UiStyle>,
) {
    let columns = style
        .and_then(|style| style.get("columns"))
        .and_then(ui_value_u32)
        .unwrap_or(2)
        .max(1) as usize;
    let gap = style
        .and_then(|style| style.get("gap"))
        .and_then(ui_value_float)
        .unwrap_or(4.0);
    let gap_x = style
        .and_then(|style| style.get("gapX"))
        .and_then(ui_value_float)
        .unwrap_or(gap);
    let gap_y = style
        .and_then(|style| style.get("gapY"))
        .and_then(ui_value_float)
        .unwrap_or(gap);
    let mut index = 0usize;
    egui::Grid::new(node_id.to_string())
        .num_columns(columns)
        .spacing(egui::vec2(gap_x, gap_y))
        .show(ui, |ui| {
            let children = tree
                .nodes
                .get(node_id)
                .map(|node| node.children.clone())
                .unwrap_or_default();
            for child_id in children {
                render_node(
                    ui,
                    event_queue,
                    context_id,
                    window_id,
                    tree,
                    focused_node,
                    viewport_requests,
                    style_cache,
                    ordered_children_cache,
                    animated_overrides,
                    node_rects,
                    debug_enabled,
                    sizing_pass,
                    &child_id,
                );
                index += 1;
                if index % columns == 0 {
                    ui.end_row();
                }
            }
        });
}

fn style_entry(
    tree: &mut UiTreeState,
    style_cache: &mut HashMap<LogicalId, UiStyleCacheEntry>,
    animated_overrides: &HashMap<LogicalId, UiStyle>,
    node_id: &LogicalId,
) -> UiStyleCacheEntry {
    let style = tree
        .nodes
        .get(node_id)
        .and_then(|node| node.style.as_ref());
    let needs_refresh = tree.dirty_nodes.remove(node_id) || !style_cache.contains_key(node_id);
    if needs_refresh {
        let entry = build_style_cache(style);
        style_cache.insert(node_id.clone(), entry);
    }
    let mut entry = style_cache
        .get(node_id)
        .cloned()
        .unwrap_or_else(|| build_style_cache(style));
    if let Some(overrides) = animated_overrides.get(node_id) {
        apply_style_overrides(&mut entry, overrides);
    }
    entry
}

fn get_ordered_children(
    tree: &mut UiTreeState,
    style_cache: &mut HashMap<LogicalId, UiStyleCacheEntry>,
    ordered_children_cache: &mut HashMap<LogicalId, Vec<LogicalId>>,
    animated_overrides: &HashMap<LogicalId, UiStyle>,
    node_id: &LogicalId,
) -> Vec<LogicalId> {
    let needs_refresh = tree.dirty_structure.remove(node_id)
        || !ordered_children_cache.contains_key(node_id);
    if needs_refresh {
        let children = tree
            .nodes
            .get(node_id)
            .map(|node| node.children.clone())
            .unwrap_or_default();

        let mut ordered: Vec<(usize, LogicalId, i32)> = children
            .into_iter()
            .enumerate()
            .map(|(index, child_id)| {
                let z_index = style_entry(tree, style_cache, animated_overrides, &child_id).z_index;
                (index, child_id, z_index)
            })
            .collect();

        ordered.sort_by(|a, b| a.2.cmp(&b.2).then(a.0.cmp(&b.0)));
        let ordered_ids: Vec<LogicalId> = ordered.into_iter().map(|(_, id, _)| id).collect();
        ordered_children_cache.insert(node_id.clone(), ordered_ids);
    }

    ordered_children_cache
        .get(node_id)
        .cloned()
        .unwrap_or_default()
}

fn text_with_style(text: String, entry: UiStyleCacheEntry) -> egui::RichText {
    if let Some(size) = entry.font_size {
        egui::RichText::new(text).size(size)
    } else {
        egui::RichText::new(text)
    }
}

fn record_rect(node_rects: &mut HashMap<LogicalId, egui::Rect>, node_id: &LogicalId, rect: egui::Rect) {
    node_rects.insert(node_id.clone(), rect);
}

fn draw_debug_overlay(ui: &egui::Ui, node_rects: &HashMap<LogicalId, egui::Rect>, node_count: usize) {
    let painter = ui.ctx().layer_painter(egui::LayerId::new(
        egui::Order::Foreground,
        egui::Id::new("ui_debug_overlay"),
    ));
    for (node_id, rect) in node_rects {
        painter.rect_stroke(*rect, 0.0, (1.0, egui::Color32::from_rgb(0, 255, 255)));
        painter.text(
            rect.left_top(),
            egui::Align2::LEFT_TOP,
            node_id.to_string(),
            egui::FontId::monospace(10.0),
            egui::Color32::from_rgba_premultiplied(0, 255, 255, 200),
        );
    }

    painter.text(
        egui::pos2(6.0, 6.0),
        egui::Align2::LEFT_TOP,
        format!("UI Debug: nodes={}", node_count),
        egui::FontId::proportional(12.0),
        egui::Color32::from_rgba_premultiplied(255, 255, 255, 200),
    );
}
