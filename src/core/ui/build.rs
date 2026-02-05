use crate::core::render::graph::LogicalId;

use super::tree::UiTreeState;
use super::widgets::render_children;

pub fn build_ui_from_tree(
    ctx: &egui::Context,
    event_queue: &mut Vec<crate::core::cmd::EngineEvent>,
    context_id: &LogicalId,
    window_id: u32,
    tree: &mut UiTreeState,
    focused_node: &mut Option<LogicalId>,
    viewport_requests: &mut Vec<crate::core::ui::state::ViewportRequest>,
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
            &LogicalId::Str("root".into()),
        );
    });
}
