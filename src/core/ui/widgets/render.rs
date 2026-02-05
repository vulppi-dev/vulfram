use crate::core::cmd::EngineEvent;
use crate::core::render::graph::LogicalId;

use super::dock::render_dock;
use super::events::{emit_ui_event, handle_focus_events, handle_viewport_events};
use super::values::{
    ui_value_bool, ui_value_float, ui_value_string, ui_value_u32, update_node_prop,
};
use crate::core::ui::layout::{resolve_gap, resolve_layout, resolve_padding, resolve_size};
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
    node_id: &LogicalId,
) {
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
            &child_id,
        );
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
    node_id: &LogicalId,
) {
    let Some(node) = tree.nodes.get(node_id).cloned() else {
        return;
    };

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
                node_id,
                node.style.as_ref(),
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
            ui.label(value);
        }
        UiNodeType::Button => {
            let label = node
                .props
                .as_ref()
                .and_then(|props| props.get("label"))
                .and_then(ui_value_string)
                .unwrap_or_else(|| "Button".into());
            let response = ui.button(label);
            if response.clicked() {
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
            let response = ui.text_edit_singleline(&mut value);
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
        UiNodeType::Checkbox => {
            let mut value = node
                .props
                .as_ref()
                .and_then(|props| props.get("value"))
                .and_then(ui_value_bool)
                .unwrap_or(false);
            let response = ui.checkbox(&mut value, "");
            if response.changed() {
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
            if response.changed() {
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
            egui::ScrollArea::vertical().show(ui, |ui| {
                render_children(
                    ui,
                    event_queue,
                    context_id,
                    window_id,
                    tree,
                    focused_node,
                    viewport_requests,
                    node_id,
                );
            });
        }
        UiNodeType::Separator => {
            ui.separator();
        }
        UiNodeType::Spacer => {
            let (width, height, has_size) = resolve_size(ui, node.style.as_ref());
            if has_size {
                ui.allocate_space(egui::vec2(width, height));
            } else {
                ui.add_space(4.0);
            }
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
            let (width, height, has_size) = resolve_size(ui, node.style.as_ref());
            let size = if has_size {
                egui::vec2(width, height)
            } else {
                egui::vec2(64.0, 64.0)
            };
            if let Some(texture_id) = texture_id {
                let tex =
                    egui::load::SizedTexture::new(egui::TextureId::User(texture_id as u64), size);
                let response = ui.add(egui::Image::new(tex).sense(egui::Sense::click_and_drag()));
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
                        &node,
                        node_id,
                    );
                }
            } else {
                ui.label("Image: missing textureId");
            }
        }
        UiNodeType::Select => {
            ui.label("Unsupported widget");
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
    node_id: &LogicalId,
    style: Option<&UiStyle>,
) {
    let layout_value = style
        .and_then(|style| style.get("layout"))
        .and_then(ui_value_string)
        .unwrap_or_else(|| "col".into());
    let gap = resolve_gap(style);
    let padding = resolve_padding(style);
    let (width, height, has_size) = resolve_size(ui, style);
    let wrap = style
        .and_then(|style| style.get("wrap"))
        .and_then(ui_value_bool)
        .unwrap_or(false);

    let (layout, is_grid) = resolve_layout(&layout_value, style, wrap);

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
                    node_id,
                );
            });
        }

        *ui.spacing_mut() = previous_spacing;
    };

    if padding != egui::Margin::ZERO {
        let frame = egui::Frame::none().inner_margin(padding);
        if has_size {
            frame.show(ui, |ui| {
                ui.allocate_ui_with_layout(egui::vec2(width, height), layout, render_children_fn);
            });
        } else {
            frame.show(ui, render_children_fn);
        }
        return;
    }

    if has_size {
        ui.allocate_ui_with_layout(egui::vec2(width, height), layout, render_children_fn);
    } else {
        render_children_fn(ui);
    }
}

fn render_grid(
    ui: &mut egui::Ui,
    event_queue: &mut Vec<EngineEvent>,
    context_id: &LogicalId,
    window_id: u32,
    tree: &mut UiTreeState,
    focused_node: &mut Option<LogicalId>,
    viewport_requests: &mut Vec<crate::core::ui::state::ViewportRequest>,
    node_id: &LogicalId,
    style: Option<&UiStyle>,
) {
    let columns = style
        .and_then(|style| style.get("columns"))
        .and_then(ui_value_u32)
        .unwrap_or(2)
        .max(1) as usize;
    let gap = resolve_gap(style);
    let mut index = 0usize;
    egui::Grid::new(node_id.to_string())
        .num_columns(columns)
        .spacing(gap)
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
                    &child_id,
                );
                index += 1;
                if index % columns == 0 {
                    ui.end_row();
                }
            }
        });
}
