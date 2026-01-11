use std::sync::Arc;

use glam::{IVec2, UVec2};
use pollster::FutureExt;
use serde::{Deserialize, Serialize};
use winit::{
    dpi::{PhysicalPosition, PhysicalSize, Position},
    event_loop::ActiveEventLoop,
    window::Window,
};

use super::{EngineWindowState, window_size_default};
use crate::core::state::EngineState;
use crate::core::window::WindowState;

// MARK: - Create Window

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdWindowCreateArgs {
    pub window_id: u32,
    pub title: String,
    #[serde(default = "window_size_default")]
    pub size: UVec2,
    pub position: IVec2,
    pub borderless: bool,
    pub resizable: bool,
    pub transparent: bool,
    pub initial_state: EngineWindowState,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultWindowCreate {
    pub success: bool,
    pub message: String,
}

pub fn engine_cmd_window_create(
    engine: &mut EngineState,
    event_loop: &ActiveEventLoop,
    args: &CmdWindowCreateArgs,
) -> CmdResultWindowCreate {
    // Ensure minimum valid size
    let window_width = args.size.x.max(100);
    let window_height = args.size.y.max(100);

    let mut win_attrs = Window::default_attributes()
        .with_title(args.title.as_str())
        .with_decorations(!args.borderless)
        .with_resizable(args.resizable)
        .with_min_inner_size(PhysicalSize::new(100, 100))
        .with_inner_size(PhysicalSize::new(window_width, window_height))
        .with_transparent(args.transparent);

    // Only set position if explicitly provided (not default 0, 0)
    // Wayland doesn't support arbitrary window positioning
    if args.position.x != 0 || args.position.y != 0 {
        win_attrs = win_attrs.with_position(Position::Physical(PhysicalPosition::new(
            args.position.x,
            args.position.y,
        )));
    }

    let window = match event_loop.create_window(win_attrs) {
        Ok(window) => Arc::new(window),
        Err(e) => {
            println!("Failed to create window: {}", e);
            return CmdResultWindowCreate {
                success: false,
                message: format!("Winit create window error: {}", e),
            };
        }
    };

    let win_id = args.window_id;
    engine.window.map_window(window.id(), win_id);

    let surface = match engine.wgpu.create_surface(window.clone()) {
        Ok(surface) => surface,
        Err(e) => {
            return CmdResultWindowCreate {
                success: false,
                message: format!("WGPU create surface error: {}", e),
            };
        }
    };

    // Get or create adapter and device
    let (adapter, is_new_device) = if engine.device.is_none() {
        // First window - create new adapter and device
        let adapter =
            match pollster::block_on(engine.wgpu.request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })) {
                Ok(adapter) => adapter,
                Err(_) => {
                    return CmdResultWindowCreate {
                        success: false,
                        message: "WGPU adapter request error".into(),
                    };
                }
            };

        let (device, queue) = match adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: wgpu::MemoryHints::default(),
                ..Default::default()
            })
            .block_on()
        {
            Ok((device, queue)) => (device, queue),
            Err(e) => {
                return CmdResultWindowCreate {
                    success: false,
                    message: format!("WGPU device request error: {}", e),
                };
            }
        };

        engine.caps = Some(surface.get_capabilities(&adapter));
        engine.device = Some(device);
        engine.queue = Some(queue);
        (adapter, true)
    } else {
        // Subsequent windows - validate surface compatibility with existing adapter
        let adapter = match pollster::block_on(engine.wgpu.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            },
        )) {
            Ok(adapter) => adapter,
            Err(_) => {
                return CmdResultWindowCreate {
                        success: false,
                        message: "Surface is not compatible with existing WGPU adapter. Cannot create window.".into(),
                    };
            }
        };
        (adapter, false)
    };

    // Get surface capabilities
    let caps = if is_new_device {
        engine.caps.as_ref().unwrap()
    } else {
        // For subsequent windows, get fresh capabilities and store them
        let new_caps = surface.get_capabilities(&adapter);
        engine.caps = Some(new_caps);
        engine.caps.as_ref().unwrap()
    };

    let format = caps
        .formats
        .iter()
        .copied()
        .find(|f| f.is_srgb())
        .unwrap_or(caps.formats[0]);

    // Use Opaque alpha mode to prevent window transparency
    // This ensures the clear color is rendered correctly
    let alpha_mode = wgpu::CompositeAlphaMode::Auto;

    let config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        width: window_width,
        height: window_height,
        present_mode: if caps.present_modes.contains(&wgpu::PresentMode::Mailbox) {
            wgpu::PresentMode::Mailbox
        } else {
            wgpu::PresentMode::Fifo
        },
        format,
        alpha_mode,
        view_formats: vec![],
        desired_maximum_frame_latency: 2,
    };

    // Configure the surface with the device
    surface.configure(engine.device.as_ref().unwrap(), &config);

    // Get initial window positions and sizes
    let inner_position = window.inner_position().unwrap_or_default();
    let outer_position = window.outer_position().unwrap_or_default();
    let inner_size = window.inner_size();
    let outer_size = window.outer_size();

    // Create render state and initialize blit resources
    let mut render_state = crate::core::render::RenderState::new(format);
    if let Some(device) = &engine.device {
        if let Some(queue) = &engine.queue {
            render_state.init(device, queue, format);

            // Initialize size-dependent resources (like depth buffer)
            render_state.on_resize(window_width, window_height);
        }
    }

    engine.window.insert_state(
        win_id,
        WindowState {
            window,
            surface,
            config: config.clone(),
            inner_position: IVec2::new(inner_position.x, inner_position.y),
            outer_position: IVec2::new(outer_position.x, outer_position.y),
            inner_size: UVec2::new(inner_size.width, inner_size.height),
            outer_size: UVec2::new(outer_size.width, outer_size.height),
            render_state,
            is_dirty: true,
        },
    );

    // Initialize window cache
    let cache = engine.window.cache.get_or_create(win_id);
    cache.inner_position = IVec2::new(inner_position.x, inner_position.y);
    cache.outer_position = IVec2::new(outer_position.x, outer_position.y);
    cache.inner_size = UVec2::new(inner_size.width, inner_size.height);
    cache.outer_size = UVec2::new(outer_size.width, outer_size.height);
    cache.scale_factor = 1.0; // Will be updated on first scale factor change event
    cache.focused = false;
    cache.occluded = false;
    cache.dark_mode = false;

    CmdResultWindowCreate {
        success: true,
        message: "Window created successfully".into(),
    }
}

// MARK: - Close Window

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdWindowCloseArgs {
    pub window_id: u32,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultWindowClose {
    success: bool,
    message: String,
}

pub fn engine_cmd_window_close(
    engine: &mut EngineState,
    args: &CmdWindowCloseArgs,
) -> CmdResultWindowClose {
    // Check if window exists
    if !engine.window.states.contains_key(&args.window_id) {
        return CmdResultWindowClose {
            success: false,
            message: format!("Window with id {} not found", args.window_id),
        };
    }

    // Cleanup window and all associated resources
    if engine.cleanup_window(args.window_id) {
        CmdResultWindowClose {
            success: true,
            message: "Window closed successfully".into(),
        }
    } else {
        CmdResultWindowClose {
            success: false,
            message: "Failed to close window".into(),
        }
    }
}
