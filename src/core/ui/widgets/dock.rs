use crate::core::cmd::EngineEvent;
use crate::core::render::graph::LogicalId;

use super::events::emit_ui_event;
use super::values::{ui_value_string, ui_value_u32, update_node_prop};
use crate::core::ui::tree::{UiEventKind, UiListeners, UiProps, UiTreeState};
use crate::core::ui::types::UiValue;

pub(super) fn render_dock(
    ui: &mut egui::Ui,
    event_queue: &mut Vec<EngineEvent>,
    context_id: &LogicalId,
    window_id: u32,
    tree: &mut UiTreeState,
    render_node: impl Fn(
        &mut egui::Ui,
        &mut Vec<EngineEvent>,
        &LogicalId,
        u32,
        &mut UiTreeState,
        &mut Option<LogicalId>,
        &mut Vec<crate::core::ui::state::ViewportRequest>,
        &LogicalId,
    ),
    focused_node: &mut Option<LogicalId>,
    viewport_requests: &mut Vec<crate::core::ui::state::ViewportRequest>,
    node_id: &LogicalId,
    props: Option<&UiProps>,
    listeners: Option<&UiListeners>,
) {
    let children = tree
        .nodes
        .get(node_id)
        .map(|node| node.children.clone())
        .unwrap_or_default();
    if children.is_empty() {
        return;
    }

    let mut active_index = props
        .and_then(|props| props.get("activeIndex"))
        .and_then(ui_value_u32)
        .unwrap_or(0) as usize;
    if active_index >= children.len() {
        active_index = 0;
    }

    ui.vertical(|ui| {
        ui.horizontal(|ui| {
            for (index, child_id) in children.iter().enumerate() {
                let label = tree
                    .nodes
                    .get(child_id)
                    .and_then(|node| {
                        node.props
                            .as_ref()
                            .and_then(|props| props.get("title"))
                            .and_then(ui_value_string)
                            .or_else(|| node.variant.clone())
                    })
                    .unwrap_or_else(|| child_id.to_string());

                let response = ui.selectable_label(index == active_index, label);
                if response.clicked() && index != active_index {
                    update_node_prop(
                        tree,
                        node_id,
                        "activeIndex",
                        UiValue::Int(index as i64),
                    );
                    if let Some(listeners) = listeners {
                        if let Some(label) = listeners.on_change.clone() {
                            emit_ui_event(
                                event_queue,
                                window_id,
                                context_id,
                                label,
                                UiEventKind::Change,
                                Some(node_id.clone()),
                                Some(UiValue::Int(index as i64)),
                            );
                        }
                    }
                    active_index = index;
                }
            }
        });

        ui.separator();

        if let Some(active_id) = children.get(active_index) {
            render_node(
                ui,
                event_queue,
                context_id,
                window_id,
                tree,
                focused_node,
                viewport_requests,
                active_id,
            );
        }
    });
}
