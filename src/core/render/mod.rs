pub mod cache;
mod passes;
pub mod state;

use crate::core::state::EngineState;
pub use state::RenderState;

pub fn render_frames(engine_state: &mut EngineState) {
    let device = match &engine_state.device {
        Some(device) => device,
        None => return,
    };

    let queue = match &engine_state.queue {
        Some(queue) => queue,
        None => return,
    };

    // Convert time values from ms to seconds
    let time = engine_state.time as f32 / 1000.0;
    let delta_time = engine_state.delta_time as f32 / 1000.0;
    let frame_index = engine_state.frame_index as u32;

    let frame_spec = crate::core::resources::FrameSpec::new(time, delta_time, frame_index);

    // Render all windows
    for (_window_id, window_state) in engine_state.window.states.iter_mut() {
        let surface_texture = match window_state.surface.get_current_texture() {
            Ok(texture) => texture,
            Err(e) => {
                log::error!("Failed to get surface texture: {:?}", e);
                continue;
            }
        };

        let render_state = &mut window_state.render_state;

        render_state.prepare_render(device, frame_spec);

        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        passes::pass_forward(render_state, queue, &mut encoder);
        passes::pass_compose(
            render_state,
            queue,
            &mut encoder,
            &surface_texture,
            &window_state.config,
        );

        queue.submit(Some(encoder.finish()));
        surface_texture.present();
    }
}
