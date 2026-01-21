use std::time::{Duration, Instant};

use crate::core::platform::{EventLoop, EventLoopExtPumpEvents, EventLoopProxy};
use crate::core::singleton::EngineCustomEvents;
use crate::core::state::EngineState;
use crate::core::window::{CmdResultWindowCreate, CmdWindowCreateArgs};

use super::PlatformProxy;

pub mod handler;

pub struct DesktopProxy {
    event_loop: EventLoop<EngineCustomEvents>,
    proxy: EventLoopProxy<EngineCustomEvents>,
}

impl DesktopProxy {
    pub fn new() -> Self {
        let event_loop = EventLoop::<EngineCustomEvents>::with_user_event()
            .build()
            .unwrap();
        let proxy = event_loop.create_proxy();
        Self { event_loop, proxy }
    }
}

impl PlatformProxy for DesktopProxy {
    fn event_loop_proxy(&self) -> &EventLoopProxy<EngineCustomEvents> {
        &self.proxy
    }

    fn handle_window_create(
        &mut self,
        _state: &mut EngineState,
        cmd_id: u64,
        args: &CmdWindowCreateArgs,
    ) -> Result<(), CmdResultWindowCreate> {
        let _ = self
            .proxy
            .send_event(EngineCustomEvents::CreateWindow(cmd_id, args.clone()));
        Ok(())
    }

    fn process_gamepads(&mut self, state: &mut EngineState) -> u64 {
        let start = Instant::now();
        let mut gilrs_events = Vec::new();
        if let Some(gilrs) = &mut state.gamepad.gilrs {
            while let Some(event) = gilrs.next_event() {
                gilrs_events.push(event);
            }
        }

        for event in gilrs_events {
            crate::core::gamepad::process_gilrs_event(state, event);
        }

        start.elapsed().as_nanos() as u64
    }

    fn pump_events(&mut self, state: &mut EngineState) -> u64 {
        let pump_start = Instant::now();
        self.event_loop
            .pump_app_events(Some(Duration::from_millis(16)), state);
        let total_pump_time = pump_start.elapsed().as_nanos() as u64;
        total_pump_time.saturating_sub(state.profiling.custom_events_ns)
    }

    fn render(&mut self, state: &mut EngineState) -> u64 {
        let start = Instant::now();
        for window_state in state.window.states.values_mut() {
            window_state.window.request_redraw();
        }
        start.elapsed().as_nanos() as u64
    }
}
