use serde::{Deserialize, Serialize};
use winit::event_loop::ActiveEventLoop;

use crate::core::result::EngineResult;
use crate::core::state::EngineState;

pub mod events;
pub mod win;

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type", content = "content", rename_all = "kebab-case")]
pub enum EngineCmd {
    CmdWindowCreate(win::CmdWindowCreateArgs),
    CmdWindowClose(win::CmdWindowCloseArgs),
    CmdWindowSetTitle(win::CmdWindowSetTitleArgs),
    CmdWindowSetPosition(win::CmdWindowSetPositionArgs),
    CmdWindowGetPosition(win::CmdWindowGetPositionArgs),
    CmdWindowSetSize(win::CmdWindowSetSizeArgs),
    CmdWindowGetSize(win::CmdWindowGetSizeArgs),
    CmdWindowGetOuterSize(win::CmdWindowGetOuterSizeArgs),
    CmdWindowGetSurfaceSize(win::CmdWindowGetSurfaceSizeArgs),
    CmdWindowSetState(win::CmdWindowSetStateArgs),
    CmdWindowGetState(win::CmdWindowGetStateArgs),
}

/// Engine event types sent from native to JavaScript
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", content = "content", rename_all = "kebab-case")]
pub enum EngineEvent {
    Window(events::WindowEvent),
    Pointer(events::PointerEvent),
    Keyboard(events::KeyboardEvent),
    Gamepad(events::GamepadEvent),
    System(events::SystemEvent),
    // MARK: Command answers
    WindowCreate(win::CmdResultWindowCreate),
    WindowClose(win::CmdResultWindowClose),
    WindowSetTitle(win::CmdResultWindowSetTitle),
    WindowSetPosition(win::CmdResultWindowSetPosition),
    WindowGetPosition(win::CmdResultWindowGetPosition),
    WindowSetSize(win::CmdResultWindowSetSize),
    WindowGetSize(win::CmdResultWindowGetSize),
    WindowGetOuterSize(win::CmdResultWindowGetOuterSize),
    WindowGetSurfaceSize(win::CmdResultWindowGetSurfaceSize),
    WindowSetState(win::CmdResultWindowSetState),
    WindowGetState(win::CmdResultWindowGetState),
}

#[derive(Debug, Deserialize, Clone)]
pub struct EngineCmdEnvelope {
    pub id: u64,
    #[serde(flatten)]
    pub cmd: EngineCmd,
}

#[derive(Debug, Serialize, Clone)]
pub struct EngineEventEnvelope {
    pub id: u64,
    #[serde(flatten)]
    pub event: EngineEvent,
}

pub type EngineBatchCmds = Vec<EngineCmdEnvelope>;

pub type EngineBatchEvents = Vec<EngineEventEnvelope>;

pub fn engine_process_batch(
    engine: &mut EngineState,
    event_loop: &ActiveEventLoop,
    batch: EngineBatchCmds,
) -> EngineResult {
    for pack in batch {
        match pack.cmd {
            EngineCmd::CmdWindowCreate(args) => {
                let result = win::engine_cmd_window_create(engine, event_loop, &args);
                engine.event_queue.push(EngineEventEnvelope {
                    id: pack.id,
                    event: EngineEvent::WindowCreate(result),
                });
            }
            EngineCmd::CmdWindowClose(args) => {
                let result = win::engine_cmd_window_close(engine, &args);
                engine.event_queue.push(EngineEventEnvelope {
                    id: pack.id,
                    event: EngineEvent::WindowClose(result),
                });
            }
            EngineCmd::CmdWindowSetTitle(args) => {
                let result = win::engine_cmd_window_set_title(engine, &args);
                engine.event_queue.push(EngineEventEnvelope {
                    id: pack.id,
                    event: EngineEvent::WindowSetTitle(result),
                });
            }
            EngineCmd::CmdWindowSetPosition(args) => {
                let result = win::engine_cmd_window_set_position(engine, &args);
                engine.event_queue.push(EngineEventEnvelope {
                    id: pack.id,
                    event: EngineEvent::WindowSetPosition(result),
                });
            }
            EngineCmd::CmdWindowGetPosition(args) => {
                let result = win::engine_cmd_window_get_position(engine, &args);
                engine.event_queue.push(EngineEventEnvelope {
                    id: pack.id,
                    event: EngineEvent::WindowGetPosition(result),
                });
            }
            EngineCmd::CmdWindowSetSize(args) => {
                let result = win::engine_cmd_window_set_size(engine, &args);
                engine.event_queue.push(EngineEventEnvelope {
                    id: pack.id,
                    event: EngineEvent::WindowSetSize(result),
                });
            }
            EngineCmd::CmdWindowGetSize(args) => {
                let result = win::engine_cmd_window_get_size(engine, &args);
                engine.event_queue.push(EngineEventEnvelope {
                    id: pack.id,
                    event: EngineEvent::WindowGetSize(result),
                });
            }
            EngineCmd::CmdWindowGetOuterSize(args) => {
                let result = win::engine_cmd_window_get_outer_size(engine, &args);
                engine.event_queue.push(EngineEventEnvelope {
                    id: pack.id,
                    event: EngineEvent::WindowGetOuterSize(result),
                });
            }
            EngineCmd::CmdWindowGetSurfaceSize(args) => {
                let result = win::engine_cmd_window_get_surface_size(engine, &args);
                engine.event_queue.push(EngineEventEnvelope {
                    id: pack.id,
                    event: EngineEvent::WindowGetSurfaceSize(result),
                });
            }
            EngineCmd::CmdWindowSetState(args) => {
                let result = win::engine_cmd_window_set_state(engine, &args);
                engine.event_queue.push(EngineEventEnvelope {
                    id: pack.id,
                    event: EngineEvent::WindowSetState(result),
                });
            }
            EngineCmd::CmdWindowGetState(args) => {
                let result = win::engine_cmd_window_get_state(engine, &args);
                engine.event_queue.push(EngineEventEnvelope {
                    id: pack.id,
                    event: EngineEvent::WindowGetState(result),
                });
            }
        }
    }

    EngineResult::Success
}
