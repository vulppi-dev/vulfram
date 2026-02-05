use crate::core::render::graph::LogicalId;

use super::tree::UiTreeState;
use super::widgets::render_children;
use crate::core::ui::layout::UiStyleCacheEntry;
use std::collections::HashMap;

pub fn build_ui_from_tree(
    ctx: &egui::Context,
    event_queue: &mut Vec<crate::core::cmd::EngineEvent>,
    context_id: &LogicalId,
    window_id: u32,
    tree: &mut UiTreeState,
    focused_node: &mut Option<LogicalId>,
    viewport_requests: &mut Vec<crate::core::ui::state::ViewportRequest>,
    style_cache: &mut HashMap<LogicalId, UiStyleCacheEntry>,
    ordered_children_cache: &mut HashMap<LogicalId, Vec<LogicalId>>,
    animated_overrides: &HashMap<LogicalId, crate::core::ui::tree::UiStyle>,
    node_rects: &mut HashMap<LogicalId, egui::Rect>,
    debug_enabled: bool,
) {
    egui::CentralPanel::default().show(ctx, |ui| {
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
            false,
            &LogicalId::Str("root".into()),
        );
    });
}
