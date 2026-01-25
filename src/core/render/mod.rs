pub mod cache;
pub mod gizmos;
mod passes;
pub mod state;

use crate::core::state::EngineState;
pub use state::RenderState;

#[cfg(feature = "wasm")]
use js_sys::Date;

#[cfg(feature = "wasm")]
fn now_ns() -> u64 {
    (Date::now() * 1_000_000.0) as u64
}

pub fn render_frames(engine_state: &mut EngineState) {
    engine_state.profiling.render_total_ns = 0;
    engine_state.profiling.render_shadow_ns = 0;
    engine_state.profiling.render_windows_ns = 0;
    engine_state.profiling.gpu_shadow_ns = 0;
    engine_state.profiling.gpu_light_cull_ns = 0;
    engine_state.profiling.gpu_forward_ns = 0;
    engine_state.profiling.gpu_compose_ns = 0;
    engine_state.profiling.gpu_total_ns = 0;

    let device = match &engine_state.device {
        Some(device) => device,
        None => return,
    };

    let queue = match &engine_state.queue {
        Some(queue) => queue,
        None => return,
    };

    if let Some(gpu_profiler) = engine_state.gpu_profiler.as_mut() {
        gpu_profiler.ensure_capacity(device, queue, engine_state.window.states.len());
    }

    let time = engine_state.time as f32 / 1000.0;
    let delta_time = engine_state.delta_time as f32 / 1000.0;
    let frame_index = engine_state.frame_index as u32;
    let frame_spec = crate::core::resources::FrameComponent::new(time, delta_time, frame_index);
    let mut gpu_written = false;

    #[cfg(not(feature = "wasm"))]
    let total_start = std::time::Instant::now();
    #[cfg(feature = "wasm")]
    let total_start = now_ns();

    // 1. Update Shadows (Global for all windows - using first window's state as proxy)
    if let Some((_, window_state)) = engine_state.window.states.iter_mut().next() {
        #[cfg(not(feature = "wasm"))]
        let shadow_start = std::time::Instant::now();
        #[cfg(feature = "wasm")]
        let shadow_start = now_ns();
        // Ensure data is ready but WITHOUT shadow atlas binding to avoid conflicts
        window_state
            .render_state
            .prepare_render(device, frame_spec, false);

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Shadow Update Encoder"),
        });

        if let Some(gpu_profiler) = engine_state.gpu_profiler.as_ref() {
            if gpu_profiler.query_count() >= 2 {
                encoder.write_timestamp(gpu_profiler.query_set(), 0);
                gpu_written = true;
            }
        }

        passes::pass_shadow_update(
            &mut window_state.render_state,
            device,
            queue,
            &mut encoder,
            engine_state.frame_index,
        );

        if let Some(gpu_profiler) = engine_state.gpu_profiler.as_ref() {
            if gpu_profiler.query_count() >= 2 {
                encoder.write_timestamp(gpu_profiler.query_set(), 1);
                gpu_written = true;
            }
        }

        if let Some(shadow) = &mut window_state.render_state.shadow {
            shadow.sync_table();
        }

        queue.submit(Some(encoder.finish()));
        #[cfg(not(feature = "wasm"))]
        {
            engine_state.profiling.render_shadow_ns =
                shadow_start.elapsed().as_nanos() as u64;
        }
        #[cfg(feature = "wasm")]
        {
            engine_state.profiling.render_shadow_ns = now_ns().saturating_sub(shadow_start);
        }
    }

    // 2. Render all windows
    let mut windows_ns: u64 = 0;
    for (window_index, (_window_id, window_state)) in engine_state.window.states.iter_mut().enumerate() {
        #[cfg(not(feature = "wasm"))]
        let window_start = std::time::Instant::now();
        #[cfg(feature = "wasm")]
        let window_start = now_ns();
        let surface_texture = match window_state.surface.get_current_texture() {
            Ok(texture) => texture,
            Err(e) => {
                log::error!("Failed to get surface texture: {:?}", e);
                continue;
            }
        };

        let render_state = &mut window_state.render_state;
        render_state.prepare_render(device, frame_spec, true);

        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        let gpu_base = engine_state.gpu_profiler.as_ref().and_then(|gpu_profiler| {
            let base = 2 + (window_index as u32) * 6;
            if gpu_profiler.query_count() >= base + 6 {
                Some(base)
            } else {
                None
            }
        });

        if let (Some(gpu_profiler), Some(base)) = (engine_state.gpu_profiler.as_ref(), gpu_base) {
            encoder.write_timestamp(gpu_profiler.query_set(), base);
            gpu_written = true;
        }

        passes::pass_light_cull(render_state, device, &mut encoder, engine_state.frame_index);

        if let (Some(gpu_profiler), Some(base)) = (engine_state.gpu_profiler.as_ref(), gpu_base) {
            encoder.write_timestamp(gpu_profiler.query_set(), base + 1);
            gpu_written = true;
        }

        if let (Some(gpu_profiler), Some(base)) = (engine_state.gpu_profiler.as_ref(), gpu_base) {
            encoder.write_timestamp(gpu_profiler.query_set(), base + 2);
            gpu_written = true;
        }
        passes::pass_forward(
            render_state,
            device,
            queue,
            &mut encoder,
            engine_state.frame_index,
        );
        if let (Some(gpu_profiler), Some(base)) = (engine_state.gpu_profiler.as_ref(), gpu_base) {
            encoder.write_timestamp(gpu_profiler.query_set(), base + 3);
            gpu_written = true;
        }

        if let (Some(gpu_profiler), Some(base)) = (engine_state.gpu_profiler.as_ref(), gpu_base) {
            encoder.write_timestamp(gpu_profiler.query_set(), base + 4);
            gpu_written = true;
        }
        passes::pass_compose(
            render_state,
            device,
            queue,
            &mut encoder,
            &surface_texture,
            &window_state.config,
            engine_state.frame_index,
        );
        if let (Some(gpu_profiler), Some(base)) = (engine_state.gpu_profiler.as_ref(), gpu_base) {
            encoder.write_timestamp(gpu_profiler.query_set(), base + 5);
            gpu_written = true;
        }

        queue.submit(Some(encoder.finish()));
        surface_texture.present();
        #[cfg(not(feature = "wasm"))]
        {
            let now = std::time::Instant::now();
            let delta_ns = window_state
                .last_present_instant
                .map(|prev| now.duration_since(prev).as_nanos() as u64)
                .unwrap_or(0);
            window_state.last_present_instant = Some(now);
            window_state.last_frame_delta_ns = delta_ns;
            window_state.fps_instant = if delta_ns > 0 {
                1_000_000_000.0 / delta_ns as f64
            } else {
                0.0
            };
        }
        #[cfg(feature = "wasm")]
        {
            let now = now_ns();
            let delta_ns = if window_state.last_present_ns > 0 {
                now.saturating_sub(window_state.last_present_ns)
            } else {
                0
            };
            window_state.last_present_ns = now;
            window_state.last_frame_delta_ns = delta_ns;
            window_state.fps_instant = if delta_ns > 0 {
                1_000_000_000.0 / delta_ns as f64
            } else {
                0.0
            };
        }
        #[cfg(not(feature = "wasm"))]
        {
            windows_ns = windows_ns.saturating_add(
                window_start.elapsed().as_nanos() as u64,
            );
        }
        #[cfg(feature = "wasm")]
        {
            windows_ns =
                windows_ns.saturating_add(now_ns().saturating_sub(window_start));
        }
    }

    if gpu_written {
        if let Some(gpu_profiler) = engine_state.gpu_profiler.as_ref() {
            if gpu_profiler.query_count() > 0 {
                let mut resolve_encoder =
                    device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label: Some("GpuProfiler Resolve Encoder"),
                    });
            resolve_encoder.resolve_query_set(
                gpu_profiler.query_set(),
                0..gpu_profiler.query_count(),
                gpu_profiler.resolve_buffer(),
                0,
            );
            resolve_encoder.copy_buffer_to_buffer(
                gpu_profiler.resolve_buffer(),
                0,
                gpu_profiler.readback_buffer(),
                0,
                gpu_profiler.buffer_size(),
            );
            queue.submit(Some(resolve_encoder.finish()));
                gpu_profiler.readback_and_update(device, &mut engine_state.profiling);
            }
        }
    }
    engine_state.profiling.render_windows_ns = windows_ns;
    #[cfg(not(feature = "wasm"))]
    {
        engine_state.profiling.render_total_ns = total_start.elapsed().as_nanos() as u64;
    }
    #[cfg(feature = "wasm")]
    {
        engine_state.profiling.render_total_ns = now_ns().saturating_sub(total_start);
    }
}
