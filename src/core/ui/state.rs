use std::collections::HashMap;

use crate::core::render::graph::LogicalId;

use super::tree::UiTreeState;
use super::types::{UiRectPx, UiRenderTarget, UiThemeSource};
use crate::core::resources::RenderTarget;

#[derive(Debug, Clone)]
pub struct UiThemeRecord {
    pub theme_id: LogicalId,
    pub version: u32,
    pub source: UiThemeSource,
}

#[derive(Debug, Clone)]
pub struct UiContextRecord {
    pub window_id: u32,
    pub _context_id: LogicalId,
    pub theme_id: LogicalId,
    pub target: UiRenderTarget,
    pub screen_rect: UiRectPx,
    pub z_index: i32,
    pub tree: UiTreeState,
    pub render_target: Option<RenderTarget>,
    pub egui_ctx: egui::Context,
    pub debug_map_logged: bool,
    pub debug_draw_logged: bool,
}

#[derive(Debug, Default)]
pub struct UiState {
    pub themes: HashMap<LogicalId, UiThemeRecord>,
    pub contexts: HashMap<LogicalId, UiContextRecord>,
    pub focus_by_window: HashMap<u32, LogicalId>,
    pub capture_by_window: HashMap<u32, LogicalId>,
    pub pending_events: HashMap<LogicalId, Vec<egui::Event>>,
}
