use glam::{IVec2, UVec2, Vec2};
use std::collections::HashMap;
use std::sync::Arc;
use winit::window::{Window, WindowId};

use crate::core::input::InputCacheManager;
use crate::core::render::RenderState;

use super::cache::WindowCacheManager;

/// Represents a window with its associated WGPU resources
pub struct WindowState {
    pub window: Arc<Window>,
    pub surface: wgpu::Surface<'static>,
    pub config: wgpu::SurfaceConfiguration,
    pub render_state: RenderState,
    pub inner_position: IVec2,
    pub outer_position: IVec2,
    pub inner_size: UVec2,
    pub outer_size: UVec2,
    pub(crate) is_dirty: bool,
}

/// Aggregates window state, IDs and caches
pub struct WindowManager {
    pub states: HashMap<u32, WindowState>,
    pub window_id_map: HashMap<WindowId, u32>,
    pub window_id_counter: u32,
    pub cursor_positions: HashMap<u32, Vec2>,
    pub cache: WindowCacheManager,
}

impl WindowManager {
    pub fn new() -> Self {
        Self {
            states: HashMap::new(),
            window_id_map: HashMap::new(),
            window_id_counter: 0,
            cursor_positions: HashMap::new(),
            cache: WindowCacheManager::new(),
        }
    }

    pub fn next_id(&mut self) -> u32 {
        let id = self.window_id_counter;
        self.window_id_counter += 1;
        id
    }

    pub fn map_window(&mut self, winit_id: WindowId, engine_id: u32) {
        self.window_id_map.insert(winit_id, engine_id);
    }

    pub fn resolve_window_id(&self, winit_id: &WindowId) -> Option<u32> {
        self.window_id_map.get(winit_id).copied()
    }

    pub fn insert_state(&mut self, window_id: u32, state: WindowState) {
        self.states.insert(window_id, state);
    }

    pub fn cleanup_window(&mut self, window_id: u32, input_cache: &mut InputCacheManager) -> bool {
        if let Some(mut window_state) = self.states.remove(&window_id) {
            self.window_id_map.remove(&window_state.window.id());
            self.cache.remove(window_id);
            input_cache.remove_pointer(window_id);
            self.cursor_positions.remove(&window_id);
            window_state.render_state.drop_all();
            true
        } else {
            false
        }
    }
}
