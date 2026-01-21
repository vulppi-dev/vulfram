use crate::core::platform::EventLoopProxy;
use crate::core::singleton::EngineCustomEvents;
use crate::core::state::EngineState;
use crate::core::window::{CmdResultWindowCreate, CmdWindowCreateArgs};

#[cfg(feature = "wasm")]
pub mod browser;
#[cfg(not(feature = "wasm"))]
pub mod desktop;

pub trait PlatformProxy {
    fn event_loop_proxy(&self) -> &EventLoopProxy<EngineCustomEvents>;
    fn handle_window_create(
        &mut self,
        state: &mut EngineState,
        cmd_id: u64,
        args: &CmdWindowCreateArgs,
    ) -> Result<(), CmdResultWindowCreate>;
    fn process_gamepads(&mut self, state: &mut EngineState) -> u64;
    fn pump_events(&mut self, state: &mut EngineState) -> u64;
    fn render(&mut self, state: &mut EngineState) -> u64;
}

#[cfg(feature = "wasm")]
pub type DefaultPlatformProxy = browser::BrowserProxy;
#[cfg(not(feature = "wasm"))]
pub type DefaultPlatformProxy = desktop::DesktopProxy;
