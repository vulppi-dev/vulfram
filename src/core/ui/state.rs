use std::collections::HashMap;

use crate::core::render::graph::LogicalId;

use super::tree::UiTreeState;
use super::types::{UiRectPx, UiRenderTarget, UiThemeConfig};
use crate::core::resources::RenderTarget;
use crate::core::ui::layout::UiStyleCacheEntry;

#[derive(Debug, Clone)]
pub struct UiThemeRecord {
    pub theme_id: LogicalId,
    pub version: u32,
    pub theme: UiThemeConfig,
}

#[derive(Debug, Clone)]
pub struct UiContextRecord {
    pub window_id: u32,
    pub _context_id: LogicalId,
    pub theme_id: Option<LogicalId>,
    pub target: UiRenderTarget,
    pub screen_rect: UiRectPx,
    pub z_index: i32,
    pub tree: UiTreeState,
    pub render_target: Option<RenderTarget>,
    pub egui_ctx: egui::Context,
    pub focused_node: Option<LogicalId>,
    pub viewport_requests: Vec<ViewportRequest>,
    pub style_cache: HashMap<LogicalId, UiStyleCacheEntry>,
    pub ordered_children_cache: HashMap<LogicalId, Vec<LogicalId>>,
    pub animations: Vec<crate::core::ui::animation::UiAnimation>,
    pub animated_overrides: HashMap<LogicalId, crate::core::ui::tree::UiStyle>,
    pub node_rects: HashMap<LogicalId, egui::Rect>,
    pub debug_enabled: bool,
    pub applied_theme_version: u32,
    pub applied_theme_id: Option<LogicalId>,
    pub applied_theme_fallback: bool,
    pub debug_map_logged: bool,
    pub debug_draw_logged: bool,
}

#[derive(Debug, Clone)]
pub struct ViewportRequest {
    pub camera_id: u32,
    pub size_points: egui::Vec2,
}

#[derive(Debug)]
pub struct UiState {
    pub themes: HashMap<LogicalId, UiThemeRecord>,
    pub contexts: HashMap<LogicalId, UiContextRecord>,
    pub focus_by_window: HashMap<u32, LogicalId>,
    pub capture_by_window: HashMap<u32, LogicalId>,
    pub pending_events: HashMap<LogicalId, Vec<egui::Event>>,
    pub output_format: wgpu::TextureFormat,
    pub fallback_theme: UiThemeConfig,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            themes: HashMap::new(),
            contexts: HashMap::new(),
            focus_by_window: HashMap::new(),
            capture_by_window: HashMap::new(),
            pending_events: HashMap::new(),
            output_format: wgpu::TextureFormat::Rgba16Float,
            fallback_theme: UiThemeConfig::default(),
        }
    }
}
