use crate::core::render::graph::LogicalId;
use crate::core::cmd::EngineEvent;

use super::events::UiEvent;
use super::layout::{resolve_gap, resolve_layout, resolve_padding, resolve_size};
use super::tree::{UiEventKind, UiNodeType, UiStyle, UiTreeState};
use super::types::UiValue;

pub fn build_ui_from_tree(
    ctx: &egui::Context,
    event_queue: &mut Vec<EngineEvent>,
    context_id: &LogicalId,
    window_id: u32,
    tree: &mut UiTreeState,
) {
    egui::CentralPanel::default().show(ctx, |ui| {
        render_children(ui, event_queue, context_id, window_id, tree, &LogicalId::Str("root".into()));
    });
}

fn render_children(
    ui: &mut egui::Ui,
    event_queue: &mut Vec<EngineEvent>,
    context_id: &LogicalId,
    window_id: u32,
    tree: &mut UiTreeState,
    node_id: &LogicalId,
) {
    let children = tree
        .nodes
        .get(node_id)
        .map(|node| node.children.clone())
        .unwrap_or_default();
    for child_id in children {
        render_node(ui, event_queue, context_id, window_id, tree, &child_id);
    }
}

fn render_node(
    ui: &mut egui::Ui,
    event_queue: &mut Vec<EngineEvent>,
    context_id: &LogicalId,
    window_id: u32,
    tree: &mut UiTreeState,
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
                node_id,
                node.style.as_ref(),
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
                render_children(ui, event_queue, context_id, window_id, tree, node_id);
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
            let (width, height, has_size) = resolve_size(ui, node.style.as_ref());
            let size = if has_size {
                egui::vec2(width, height)
            } else {
                egui::vec2(64.0, 64.0)
            };
            if let Some(texture_id) = texture_id {
                let tex = egui::load::SizedTexture::new(
                    egui::TextureId::User(texture_id as u64),
                    size,
                );
                ui.add(egui::Image::new(tex));
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
            render_grid(ui, event_queue, context_id, window_id, tree, node_id, style);
        } else {
            ui.with_layout(layout, |ui| {
                render_children(ui, event_queue, context_id, window_id, tree, node_id);
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
                render_node(ui, event_queue, context_id, window_id, tree, &child_id);
                index += 1;
                if index % columns == 0 {
                    ui.end_row();
                }
            }
        });
}

fn emit_ui_event(
    event_queue: &mut Vec<EngineEvent>,
    window_id: u32,
    context_id: &LogicalId,
    label: String,
    kind: UiEventKind,
    node_id: Option<LogicalId>,
    value: Option<UiValue>,
) {
    event_queue.push(EngineEvent::Ui(UiEvent {
        window_id: LogicalId::Int(window_id as i64),
        context_id: context_id.clone(),
        label,
        kind,
        node_id,
        value,
    }));
}

fn ui_value_string(value: &UiValue) -> Option<String> {
    match value {
        UiValue::String(value) => Some(value.clone()),
        _ => None,
    }
}

fn ui_value_bool(value: &UiValue) -> Option<bool> {
    match value {
        UiValue::Bool(value) => Some(*value),
        _ => None,
    }
}

fn ui_value_float(value: &UiValue) -> Option<f32> {
    match value {
        UiValue::Float(value) => Some(*value as f32),
        UiValue::Int(value) => Some(*value as f32),
        _ => None,
    }
}

fn ui_value_u32(value: &UiValue) -> Option<u32> {
    match value {
        UiValue::Int(value) => u32::try_from(*value).ok(),
        UiValue::Float(value) => {
            if *value >= 0.0 && *value <= u32::MAX as f64 {
                Some(*value as u32)
            } else {
                None
            }
        }
        _ => None,
    }
}

fn update_node_prop(tree: &mut UiTreeState, node_id: &LogicalId, key: &str, value: UiValue) {
    if let Some(node) = tree.nodes.get_mut(node_id) {
        match node.props.as_mut() {
            Some(props) => {
                props.insert(key.to_string(), value);
            }
            None => {
                let mut props = std::collections::HashMap::new();
                props.insert(key.to_string(), value);
                node.props = Some(props);
            }
        }
    }
}
