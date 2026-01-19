use crate::core::buffers::state::BufferStorage;
use crate::core::cmd::{EngineBatchCmds, EngineBatchEvents, EngineBatchResponses};
use crate::core::gamepad::state::GamepadState;
use crate::core::input::InputState;
use crate::core::profiling::TickProfiling;
use crate::core::window::WindowManager;

/// Main engine state holding all runtime data
pub struct EngineState {
    pub window: WindowManager,

    pub wgpu: wgpu::Instance,
    pub caps: Option<wgpu::SurfaceCapabilities>,
    pub device: Option<wgpu::Device>,
    pub queue: Option<wgpu::Queue>,

    pub buffers: BufferStorage,

    pub cmd_queue: EngineBatchCmds,
    pub event_queue: EngineBatchEvents,
    pub response_queue: EngineBatchResponses,

    pub(crate) time: u64,
    pub(crate) delta_time: u32,
    pub(crate) frame_index: u64,

    pub input: InputState,
    pub(crate) gamepad: GamepadState,

    pub(crate) profiling: TickProfiling,
}

impl EngineState {
    pub fn new() -> Self {
        #[cfg(feature = "wasm")]
        let wgpu_descriptor = wgpu::InstanceDescriptor {
            backends: wgpu::Backends::BROWSER_WEBGPU,
            backend_options: wgpu::BackendOptions::default(),
            flags: wgpu::InstanceFlags::empty(),
            memory_budget_thresholds: wgpu::MemoryBudgetThresholds::default(),
        };

        #[cfg(not(feature = "wasm"))]
        let wgpu_descriptor = wgpu::InstanceDescriptor {
            backends: if cfg!(target_os = "ios") || cfg!(target_os = "macos") {
                wgpu::Backends::METAL | wgpu::Backends::VULKAN
            } else {
                wgpu::Backends::DX12 | wgpu::Backends::VULKAN
            },
            backend_options: wgpu::BackendOptions::default(),
            flags: wgpu::InstanceFlags::empty(),
            memory_budget_thresholds: wgpu::MemoryBudgetThresholds::default(),
        };

        Self {
            window: WindowManager::new(),
            wgpu: wgpu::Instance::new(&wgpu_descriptor),
            caps: None,
            device: None,
            queue: None,
            buffers: BufferStorage::new(),
            cmd_queue: Vec::new(),
            event_queue: Vec::new(),
            response_queue: Vec::new(),
            time: 0,
            delta_time: 0,
            frame_index: 0,
            input: InputState::new(),
            gamepad: GamepadState::new(),
            profiling: TickProfiling::default(),
        }
    }

    pub fn cleanup_window(&mut self, window_id: u32) -> bool {
        self.window.cleanup_window(window_id, &mut self.input.cache)
    }
}
