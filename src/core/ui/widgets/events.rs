use crate::core::cmd::EngineEvent;
use crate::core::render::graph::LogicalId;

use crate::core::ui::events::UiEvent;
use crate::core::ui::tree::{UiEventKind, UiNode, UiListeners};
use crate::core::ui::types::UiValue;

pub(super) fn emit_ui_event(
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

pub(super) fn handle_focus_events(
    event_queue: &mut Vec<EngineEvent>,
    context_id: &LogicalId,
    window_id: u32,
    node_id: &LogicalId,
    listeners: &Option<UiListeners>,
    focused_node: &mut Option<LogicalId>,
    has_focus: bool,
) {
    if has_focus {
        if focused_node.as_ref() != Some(node_id) {
            *focused_node = Some(node_id.clone());
            if let Some(listeners) = listeners.as_ref() {
                if let Some(label) = listeners.on_focus.clone() {
                    emit_ui_event(
                        event_queue,
                        window_id,
                        context_id,
                        label,
                        UiEventKind::Focus,
                        Some(node_id.clone()),
                        None,
                    );
                }
            }
        }
        return;
    }

    if focused_node.as_ref() == Some(node_id) {
        *focused_node = None;
        if let Some(listeners) = listeners.as_ref() {
            if let Some(label) = listeners.on_blur.clone() {
                emit_ui_event(
                    event_queue,
                    window_id,
                    context_id,
                    label,
                    UiEventKind::Blur,
                    Some(node_id.clone()),
                    None,
                );
            }
        }
    }
}

pub(super) fn handle_viewport_events(
    event_queue: &mut Vec<EngineEvent>,
    window_id: u32,
    context_id: &LogicalId,
    response: &egui::Response,
    camera_id: u32,
    node: &UiNode,
    node_id: &LogicalId,
) {
    let listeners = match node.listeners.as_ref() {
        Some(listeners) => listeners,
        None => return,
    };

    let hover_pos = match response.hover_pos() {
        Some(pos) => pos,
        None => return,
    };

    let rect = response.rect;
    let normalized_x = ((hover_pos.x - rect.min.x) / rect.width()).clamp(0.0, 1.0);
    let normalized_y = ((hover_pos.y - rect.min.y) / rect.height()).clamp(0.0, 1.0);

    if response.hovered() {
        if let Some(label) = listeners.on_viewport_hover.clone() {
            emit_viewport_event(
                event_queue,
                window_id,
                context_id,
                label,
                UiEventKind::ViewportHover,
                Some(node_id.clone()),
                camera_id,
                normalized_x,
                normalized_y,
                None,
                None,
                response
                    .ctx
                    .input(|i| i.pointer.button_down(egui::PointerButton::Primary)),
                response
                    .ctx
                    .input(|i| i.pointer.button_down(egui::PointerButton::Secondary)),
                response
                    .ctx
                    .input(|i| i.pointer.button_down(egui::PointerButton::Middle)),
            );
        }
    }

    if response.clicked() {
        if let Some(label) = listeners.on_viewport_click.clone() {
            emit_viewport_event(
                event_queue,
                window_id,
                context_id,
                label,
                UiEventKind::ViewportClick,
                Some(node_id.clone()),
                camera_id,
                normalized_x,
                normalized_y,
                None,
                None,
                response
                    .ctx
                    .input(|i| i.pointer.button_down(egui::PointerButton::Primary)),
                response
                    .ctx
                    .input(|i| i.pointer.button_down(egui::PointerButton::Secondary)),
                response
                    .ctx
                    .input(|i| i.pointer.button_down(egui::PointerButton::Middle)),
            );
        }
    }

    if response.dragged() {
        if let Some(label) = listeners.on_viewport_drag.clone() {
            let delta = response.drag_delta();
            let delta_x = delta.x / rect.width();
            let delta_y = delta.y / rect.height();
            emit_viewport_event(
                event_queue,
                window_id,
                context_id,
                label,
                UiEventKind::ViewportDrag,
                Some(node_id.clone()),
                camera_id,
                normalized_x,
                normalized_y,
                Some(delta_x),
                Some(delta_y),
                response
                    .ctx
                    .input(|i| i.pointer.button_down(egui::PointerButton::Primary)),
                response
                    .ctx
                    .input(|i| i.pointer.button_down(egui::PointerButton::Secondary)),
                response
                    .ctx
                    .input(|i| i.pointer.button_down(egui::PointerButton::Middle)),
            );
        }
    }

    if response.drag_stopped() {
        if let Some(label) = listeners.on_viewport_drag_end.clone() {
            emit_viewport_event(
                event_queue,
                window_id,
                context_id,
                label,
                UiEventKind::ViewportDragEnd,
                Some(node_id.clone()),
                camera_id,
                normalized_x,
                normalized_y,
                None,
                None,
                false,
                false,
                false,
            );
        }
    }
}

fn emit_viewport_event(
    event_queue: &mut Vec<EngineEvent>,
    window_id: u32,
    context_id: &LogicalId,
    label: String,
    kind: UiEventKind,
    node_id: Option<LogicalId>,
    camera_id: u32,
    normalized_x: f32,
    normalized_y: f32,
    delta_x: Option<f32>,
    delta_y: Option<f32>,
    primary: bool,
    secondary: bool,
    middle: bool,
) {
    let viewport_data = format!(
        "{{\"cameraId\":{},\"x\":{:.4},\"y\":{:.4},\"dx\":{},\"dy\":{},\"primary\":{},\"secondary\":{},\"middle\":{}}}",
        camera_id,
        normalized_x,
        normalized_y,
        delta_x
            .map(|value| format!("{:.4}", value))
            .unwrap_or("null".into()),
        delta_y
            .map(|value| format!("{:.4}", value))
            .unwrap_or("null".into()),
        primary,
        secondary,
        middle
    );

    emit_ui_event(
        event_queue,
        window_id,
        context_id,
        label,
        kind,
        node_id,
        Some(UiValue::String(viewport_data)),
    );
}
