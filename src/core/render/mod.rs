pub mod state;

use crate::core::state::EngineState;
use std::collections::HashMap;
use std::sync::{OnceLock, RwLock};

use self::state::RenderState;

/// Render states for all windows (window_id -> RenderState)
static RENDER_STATES: OnceLock<RwLock<HashMap<u32, RenderState>>> = OnceLock::new();

fn ensure_render_state(window_id: u32) {
    let states = RENDER_STATES.get_or_init(|| RwLock::new(HashMap::new()));
    let read_guard = states.read().unwrap();

    if !read_guard.contains_key(&window_id) {
        drop(read_guard);
        let mut write_guard = states.write().unwrap();
        write_guard
            .entry(window_id)
            .or_insert_with(RenderState::new);
    }
}

pub fn render_frames(engine_state: &mut EngineState) {
    // Get device and queue
    let device = match &engine_state.device {
        Some(device) => device,
        None => return,
    };

    let queue = match &engine_state.queue {
        Some(queue) => queue,
        None => return,
    };

    // Render all windows
    for (window_id, window_state) in engine_state.windows.iter() {
        // Ensure render state exists for this window
        ensure_render_state(*window_id);

        // Get render states lock once for reading
        let states = RENDER_STATES.get().unwrap();
        let read_guard = states.read().unwrap();
        let render_state = read_guard.get(window_id).unwrap();

        // Get the surface texture
        let surface_texture = match window_state.surface.get_current_texture() {
            Ok(texture) => texture,
            Err(e) => {
                log::error!("Failed to get surface texture: {:?}", e);
                continue;
            }
        };

        // Create a texture view
        let view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // Create a command encoder
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        // Create a render pass with cached clear color
        {
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(render_state.clear_color),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
        }

        // Submit the commands
        queue.submit(std::iter::once(encoder.finish()));

        // Present the frame
        surface_texture.present();
    }
}

/// Clean up render state for a closed window
pub fn cleanup_window_render_state(window_id: u32) {
    if let Some(states) = RENDER_STATES.get() {
        let mut write_guard = states.write().unwrap();
        write_guard.remove(&window_id);
    }
}
