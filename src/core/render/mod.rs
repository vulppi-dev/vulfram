pub mod cache;
pub mod cmd;
pub mod state;

use crate::core::state::EngineState;

pub use state::RenderState;

pub fn render_frames(engine_state: &mut EngineState) {
    let _device = match &engine_state.device {
        Some(device) => device,
        None => return,
    };

    let _queue = match &engine_state.queue {
        Some(queue) => queue,
        None => return,
    };

    // Convert time values from ms to seconds
    let _time = engine_state.time as f32 / 1000.0;
    let _delta_time = engine_state.delta_time as f32 / 1000.0;

    // Render all windows
    for (_window_id, window_state) in engine_state.window.states.iter_mut() {
        let surface_texture = match window_state.surface.get_current_texture() {
            Ok(texture) => texture,
            Err(e) => {
                log::error!("Failed to get surface texture: {:?}", e);
                continue;
            }
        };

        let _view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let _render_state = &mut window_state.render_state;

        surface_texture.present();
    }
}
