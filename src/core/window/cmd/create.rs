#[cfg(all(feature = "wasm", target_arch = "wasm32"))]
use crate::core::cmd::{CommandResponse, CommandResponseEnvelope, EngineEvent};
#[cfg(any(
    not(feature = "wasm"),
    all(feature = "wasm", not(target_arch = "wasm32"))
))]
use crate::core::platform::ActiveEventLoop;
#[cfg(any(not(feature = "wasm"), all(feature = "wasm", target_arch = "wasm32")))]
use crate::core::platform::Window;
#[cfg(not(feature = "wasm"))]
use crate::core::platform::winit::dpi::{PhysicalPosition, PhysicalSize, Position};
#[cfg(all(feature = "wasm", target_arch = "wasm32"))]
use crate::core::singleton::with_engine_singleton;
#[cfg(all(feature = "wasm", target_arch = "wasm32"))]
use crate::core::window::WindowEvent;
use glam::{IVec2, UVec2};
#[cfg(not(feature = "wasm"))]
use pollster::FutureExt;
use serde::{Deserialize, Serialize};
#[cfg(any(not(feature = "wasm"), all(feature = "wasm", target_arch = "wasm32")))]
use std::sync::Arc;
#[cfg(all(feature = "wasm", target_arch = "wasm32"))]
use wasm_bindgen::JsCast;
#[cfg(all(feature = "wasm", target_arch = "wasm32"))]
use wasm_bindgen_futures::spawn_local;
#[cfg(all(feature = "wasm", target_arch = "wasm32"))]
use web_sys::HtmlCanvasElement;

use super::{EngineWindowState, window_size_default};
use crate::core::profiling::gpu::GpuProfiler;
use crate::core::state::EngineState;
#[cfg(any(not(feature = "wasm"), all(feature = "wasm", target_arch = "wasm32")))]
use crate::core::window::WindowState;

// MARK: - Create Window

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdWindowCreateArgs {
    pub window_id: u32,
    #[serde(default)]
    pub title: String,
    #[serde(default = "window_size_default")]
    pub size: UVec2,
    #[serde(default)]
    pub position: IVec2,
    #[serde(default)]
    pub canvas_id: Option<String>,
    #[serde(default)]
    pub borderless: bool,
    #[serde(default)]
    pub resizable: bool,
    #[serde(default)]
    pub transparent: bool,
    #[serde(default)]
    pub initial_state: EngineWindowState,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultWindowCreate {
    pub success: bool,
    pub message: String,
}

#[cfg(all(feature = "wasm", target_arch = "wasm32"))]
pub fn engine_cmd_window_create_async(
    args: &CmdWindowCreateArgs,
    cmd_id: u64,
) -> Result<(), CmdResultWindowCreate> {
    let canvas_id = match &args.canvas_id {
        Some(id) => id,
        None => {
            return Err(CmdResultWindowCreate {
                success: false,
                message: "canvasId is required in wasm mode".into(),
            });
        }
    };

    let window = match web_sys::window() {
        Some(window) => window,
        None => {
            return Err(CmdResultWindowCreate {
                success: false,
                message: "Web window not available".into(),
            });
        }
    };
    let document = match window.document() {
        Some(document) => document,
        None => {
            return Err(CmdResultWindowCreate {
                success: false,
                message: "Document not available".into(),
            });
        }
    };
    let element = match document.get_element_by_id(canvas_id) {
        Some(element) => element,
        None => {
            return Err(CmdResultWindowCreate {
                success: false,
                message: format!("Canvas with id '{}' not found", canvas_id),
            });
        }
    };
    let canvas: HtmlCanvasElement = match element.dyn_into() {
        Ok(canvas) => canvas,
        Err(_) => {
            return Err(CmdResultWindowCreate {
                success: false,
                message: format!("Element '{}' is not a canvas", canvas_id),
            });
        }
    };

    let window_width = canvas.client_width().max(1) as u32;
    let window_height = canvas.client_height().max(1) as u32;
    canvas.set_width(window_width);
    canvas.set_height(window_height);

    let win_id = args.window_id;
    let canvas_clone = canvas.clone();
    spawn_local(async move {
        let instance_descriptor = wgpu::InstanceDescriptor {
            backends: wgpu::Backends::BROWSER_WEBGPU,
            backend_options: wgpu::BackendOptions::default(),
            flags: wgpu::InstanceFlags::empty(),
            memory_budget_thresholds: wgpu::MemoryBudgetThresholds::default(),
        };
        let instance = wgpu::Instance::new(&instance_descriptor);
        let surface =
            match instance.create_surface(wgpu::SurfaceTarget::Canvas(canvas_clone.clone())) {
                Ok(surface) => surface,
                Err(e) => {
                    let _ = with_engine_singleton(|engine| {
                        engine.state.response_queue.push(CommandResponseEnvelope {
                            id: cmd_id,
                            response: CommandResponse::WindowCreate(CmdResultWindowCreate {
                                success: false,
                                message: format!("WGPU create surface error: {}", e),
                            }),
                        });
                    });
                    return;
                }
            };

        let adapter = match instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
        {
            Ok(adapter) => adapter,
            Err(_) => {
                let _ = with_engine_singleton(|engine| {
                    engine.state.response_queue.push(CommandResponseEnvelope {
                        id: cmd_id,
                        response: CommandResponse::WindowCreate(CmdResultWindowCreate {
                            success: false,
                            message: "WGPU adapter request error".into(),
                        }),
                    });
                });
                return;
            }
        };

        let ui_output_format = if adapter
            .get_texture_format_features(wgpu::TextureFormat::Rgba16Float)
            .flags
            .contains(wgpu::TextureFormatFeatureFlags::FILTERABLE)
        {
            wgpu::TextureFormat::Rgba16Float
        } else {
            wgpu::TextureFormat::Rgba8UnormSrgb
        };
        let _ = with_engine_singleton(|engine| {
            engine.state.ui.output_format = ui_output_format;
        });

        let adapter_features = adapter.features();
        let mut required_features = wgpu::Features::empty();
        let gpu_profiling_supported = adapter_features.contains(
            wgpu::Features::TIMESTAMP_QUERY | wgpu::Features::TIMESTAMP_QUERY_INSIDE_ENCODERS,
        );
        if gpu_profiling_supported {
            required_features |=
                wgpu::Features::TIMESTAMP_QUERY | wgpu::Features::TIMESTAMP_QUERY_INSIDE_ENCODERS;
        }

        let required_limits = wgpu::Limits::downlevel_webgl2_defaults();
        let (device, queue) = match adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features,
                required_limits,
                memory_hints: wgpu::MemoryHints::default(),
                ..Default::default()
            })
            .await
        {
            Ok((device, queue)) => (device, queue),
            Err(e) => {
                let _ = with_engine_singleton(|engine| {
                    engine.state.response_queue.push(CommandResponseEnvelope {
                        id: cmd_id,
                        response: CommandResponse::WindowCreate(CmdResultWindowCreate {
                            success: false,
                            message: format!("WGPU device request error: {}", e),
                        }),
                    });
                });
                return;
            }
        };

        let caps = surface.get_capabilities(&adapter);
        let format = caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            width: window_width,
            height: window_height,
            present_mode: wgpu::PresentMode::Fifo,
            format,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &config);

        let mut render_state = crate::core::render::RenderState::new(format);
        render_state.init(&device, &queue, format);
        render_state.on_resize(&device, window_width, window_height);

        let listeners =
            crate::core::platforms::browser::input::attach_canvas_listeners(win_id, &canvas_clone);
        let window_handle = Arc::new(Window::new(win_id, canvas_clone.clone()));

        let _ = with_engine_singleton(|engine| {
            engine.state.wgpu = instance;
            engine.state.caps = Some(caps);
            engine.state.device = Some(device);
            engine.state.queue = Some(queue);
            engine.state.window.map_window(window_handle.id(), win_id);
            engine.state.window.insert_state(
                win_id,
                WindowState {
                    window: window_handle,
                    surface,
                    config: config.clone(),
                    scale_factor: 1.0,
                    #[cfg(not(feature = "wasm"))]
                    inner_position: IVec2::ZERO,
                    #[cfg(not(feature = "wasm"))]
                    outer_position: IVec2::ZERO,
                    inner_size: UVec2::new(window_width, window_height),
                    outer_size: UVec2::new(window_width, window_height),
                    render_state,
                    is_dirty: true,
                    #[cfg(not(feature = "wasm"))]
                    last_present_instant: None,
                    #[cfg(feature = "wasm")]
                    last_present_ns: 0,
                    last_frame_delta_ns: 0,
                    fps_instant: 0.0,
                    _web_listeners: listeners,
                },
            );

            engine
                .state
                .event_queue
                .push(EngineEvent::Window(WindowEvent::OnCreate {
                    window_id: win_id,
                }));
            engine.state.response_queue.push(CommandResponseEnvelope {
                id: cmd_id,
                response: CommandResponse::WindowCreate(CmdResultWindowCreate {
                    success: true,
                    message: "Canvas window created successfully".into(),
                }),
            });
            if gpu_profiling_supported && engine.state.gpu_profiler.is_none() {
                if let (Some(device), Some(queue)) =
                    (engine.state.device.as_ref(), engine.state.queue.as_ref())
                {
                    engine.state.gpu_profiler = Some(GpuProfiler::new(
                        device,
                        queue,
                        engine.state.window.states.len(),
                    ));
                }
            }
        });
    });

    Ok(())
}

#[cfg(all(feature = "wasm", not(target_arch = "wasm32")))]
pub fn engine_cmd_window_create(
    _engine: &mut EngineState,
    _event_loop: &ActiveEventLoop,
    _args: &CmdWindowCreateArgs,
) -> CmdResultWindowCreate {
    CmdResultWindowCreate {
        success: false,
        message: "wasm feature requires the wasm32-unknown-unknown target".into(),
    }
}

#[cfg(not(feature = "wasm"))]
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
    let mut gpu_profiling_supported = false;
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

        engine.ui.output_format = if adapter
            .get_texture_format_features(wgpu::TextureFormat::Rgba16Float)
            .flags
            .contains(wgpu::TextureFormatFeatureFlags::FILTERABLE)
        {
            wgpu::TextureFormat::Rgba16Float
        } else {
            wgpu::TextureFormat::Rgba8UnormSrgb
        };

        let adapter_features = adapter.features();
        let mut required_features = wgpu::Features::empty();
        gpu_profiling_supported = adapter_features.contains(
            wgpu::Features::TIMESTAMP_QUERY | wgpu::Features::TIMESTAMP_QUERY_INSIDE_ENCODERS,
        );
        if gpu_profiling_supported {
            required_features |=
                wgpu::Features::TIMESTAMP_QUERY | wgpu::Features::TIMESTAMP_QUERY_INSIDE_ENCODERS;
        }

        let (device, queue) = match adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features,
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
        match engine.caps.as_ref() {
            Some(caps) => caps,
            None => {
                return CmdResultWindowCreate {
                    success: false,
                    message: "Surface capabilities not initialized".into(),
                };
            }
        }
    } else {
        // For subsequent windows, get fresh capabilities and store them
        let new_caps = surface.get_capabilities(&adapter);
        engine.caps = Some(new_caps);
        match engine.caps.as_ref() {
            Some(caps) => caps,
            None => {
                return CmdResultWindowCreate {
                    success: false,
                    message: "Surface capabilities not initialized".into(),
                };
            }
        }
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
    let device = match engine.device.as_ref() {
        Some(device) => device,
        None => {
            return CmdResultWindowCreate {
                success: false,
                message: "Graphics device not initialized".into(),
            };
        }
    };
    surface.configure(device, &config);

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
            render_state.on_resize(device, window_width, window_height);
        }
    }

    engine.window.insert_state(
        win_id,
        WindowState {
            window,
            surface,
            config: config.clone(),
            scale_factor: 1.0,
            inner_position: IVec2::new(inner_position.x, inner_position.y),
            outer_position: IVec2::new(outer_position.x, outer_position.y),
            inner_size: UVec2::new(inner_size.width, inner_size.height),
            outer_size: UVec2::new(outer_size.width, outer_size.height),
            render_state,
            is_dirty: true,
            last_present_instant: None,
            last_frame_delta_ns: 0,
            fps_instant: 0.0,
        },
    );

    if is_new_device && gpu_profiling_supported && engine.gpu_profiler.is_none() {
        if let (Some(device), Some(queue)) = (&engine.device, &engine.queue) {
            engine.gpu_profiler = Some(GpuProfiler::new(device, queue, engine.window.states.len()));
        }
    }

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
