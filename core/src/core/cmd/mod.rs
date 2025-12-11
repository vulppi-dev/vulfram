use serde::{Deserialize, Serialize};
use winit::event_loop::EventLoopProxy;

use crate::core::VulframResult;
use crate::core::singleton::EngineCustomEvents;
use crate::core::state::EngineState;

pub mod events;
pub mod render;
pub mod win;

#[derive(Debug, Deserialize, Serialize, Clone)]
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
    CmdWindowSetIcon(win::CmdWindowSetIconArgs),
    CmdWindowSetDecorations(win::CmdWindowSetDecorationsArgs),
    CmdWindowHasDecorations(win::CmdWindowHasDecorationsArgs),
    CmdWindowSetResizable(win::CmdWindowSetResizableArgs),
    CmdWindowIsResizable(win::CmdWindowIsResizableArgs),
    CmdWindowRequestAttention(win::CmdWindowRequestAttentionArgs),
    CmdWindowFocus(win::CmdWindowFocusArgs),
    CmdWindowSetCursorVisible(win::CmdWindowSetCursorVisibleArgs),
    CmdWindowSetCursorGrab(win::CmdWindowSetCursorGrabArgs),
    CmdWindowSetCursorIcon(win::CmdWindowSetCursorIconArgs),
    CmdShaderCreate(render::CmdShaderCreateArgs),
    CmdShaderDispose(render::CmdShaderDisposeArgs),
    CmdGeometryCreate(render::CmdGeometryCreateArgs),
    CmdGeometryDispose(render::CmdGeometryDisposeArgs),
    CmdMaterialCreate(render::CmdMaterialCreateArgs),
    CmdMaterialUpdate(render::CmdMaterialUpdateArgs),
    CmdMaterialDispose(render::CmdMaterialDisposeArgs),
    CmdTextureCreate(render::CmdTextureCreateArgs),
    CmdTextureUpdate(render::CmdTextureUpdateArgs),
    CmdTextureDispose(render::CmdTextureDisposeArgs),
    CmdSamplerCreate(render::CmdSamplerCreateArgs),
    CmdSamplerUpdate(render::CmdSamplerUpdateArgs),
    CmdSamplerDispose(render::CmdSamplerDisposeArgs),
    CmdCameraCreate(render::CmdCameraCreateArgs),
    CmdCameraUpdate(render::CmdCameraUpdateArgs),
    CmdCameraDispose(render::CmdCameraDisposeArgs),
    CmdModelCreate(render::CmdModelCreateArgs),
    CmdModelUpdate(render::CmdModelUpdateArgs),
    CmdModelDispose(render::CmdModelDisposeArgs),
}

/// Spontaneous engine events (input, window changes, system events)
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type", content = "content", rename_all = "kebab-case")]
pub enum EngineEvent {
    Window(events::WindowEvent),
    Pointer(events::PointerEvent),
    Keyboard(events::KeyboardEvent),
    Gamepad(events::GamepadEvent),
    System(events::SystemEvent),
}

/// Command responses (answers to commands sent by user)
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type", content = "content", rename_all = "kebab-case")]
pub enum CommandResponse {
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
    WindowSetIcon(win::CmdResultWindowSetIcon),
    WindowSetDecorations(win::CmdResultWindowSetDecorations),
    WindowHasDecorations(win::CmdResultWindowHasDecorations),
    WindowSetResizable(win::CmdResultWindowSetResizable),
    WindowIsResizable(win::CmdResultWindowIsResizable),
    WindowRequestAttention(win::CmdResultWindowRequestAttention),
    WindowFocus(win::CmdResultWindowFocus),
    WindowSetCursorVisible(win::CmdResultWindowSetCursorVisible),
    WindowSetCursorGrab(win::CmdResultWindowSetCursorGrab),
    WindowSetCursorIcon(win::CmdResultWindowSetCursorIcon),
    ShaderCreate(render::CmdResultShaderCreate),
    ShaderDispose(render::CmdResultShaderDispose),
    GeometryCreate(render::CmdResultGeometryCreate),
    GeometryDispose(render::CmdResultGeometryDispose),
    MaterialCreate(render::CmdResultMaterialCreate),
    MaterialUpdate(render::CmdResultMaterialUpdate),
    MaterialDispose(render::CmdResultMaterialDispose),
    TextureCreate(render::CmdResultTextureCreate),
    TextureUpdate(render::CmdResultTextureUpdate),
    TextureDispose(render::CmdResultTextureDispose),
    SamplerCreate(render::CmdResultSamplerCreate),
    SamplerUpdate(render::CmdResultSamplerUpdate),
    SamplerDispose(render::CmdResultSamplerDispose),
    CameraCreate(render::CmdResultCameraCreate),
    CameraUpdate(render::CmdResultCameraUpdate),
    CameraDispose(render::CmdResultCameraDispose),
    ModelCreate(render::CmdResultModelCreate),
    ModelUpdate(render::CmdResultModelUpdate),
    ModelDispose(render::CmdResultModelDispose),
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct EngineCmdEnvelope {
    pub id: u64,
    #[serde(flatten)]
    pub cmd: EngineCmd,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CommandResponseEnvelope {
    pub id: u64,
    #[serde(flatten)]
    pub response: CommandResponse,
}

pub type EngineBatchCmds = Vec<EngineCmdEnvelope>;

pub type EngineBatchEvents = Vec<EngineEvent>;

pub type EngineBatchResponses = Vec<CommandResponseEnvelope>;

pub fn engine_process_batch(
    engine: &mut EngineState,
    loop_proxy: &mut EventLoopProxy<EngineCustomEvents>,
    batch: EngineBatchCmds,
) -> VulframResult {
    for pack in batch {
        match pack.cmd {
            EngineCmd::CmdWindowCreate(args) => {
                let _ =
                    loop_proxy.send_event(EngineCustomEvents::CreateWindow(pack.id, args.clone()));
            }
            EngineCmd::CmdWindowClose(args) => {
                let result = win::engine_cmd_window_close(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::WindowClose(result),
                });
            }
            EngineCmd::CmdWindowSetTitle(args) => {
                let result = win::engine_cmd_window_set_title(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::WindowSetTitle(result),
                });
            }
            EngineCmd::CmdWindowSetPosition(args) => {
                let result = win::engine_cmd_window_set_position(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::WindowSetPosition(result),
                });
            }
            EngineCmd::CmdWindowGetPosition(args) => {
                let result = win::engine_cmd_window_get_position(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::WindowGetPosition(result),
                });
            }
            EngineCmd::CmdWindowSetSize(args) => {
                let result = win::engine_cmd_window_set_size(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::WindowSetSize(result),
                });
            }
            EngineCmd::CmdWindowGetSize(args) => {
                let result = win::engine_cmd_window_get_size(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::WindowGetSize(result),
                });
            }
            EngineCmd::CmdWindowGetOuterSize(args) => {
                let result = win::engine_cmd_window_get_outer_size(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::WindowGetOuterSize(result),
                });
            }
            EngineCmd::CmdWindowGetSurfaceSize(args) => {
                let result = win::engine_cmd_window_get_surface_size(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::WindowGetSurfaceSize(result),
                });
            }
            EngineCmd::CmdWindowSetState(args) => {
                let result = win::engine_cmd_window_set_state(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::WindowSetState(result),
                });
            }
            EngineCmd::CmdWindowGetState(args) => {
                let result = win::engine_cmd_window_get_state(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::WindowGetState(result),
                });
            }
            EngineCmd::CmdWindowSetIcon(args) => {
                let result = win::engine_cmd_window_set_icon(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::WindowSetIcon(result),
                });
            }
            EngineCmd::CmdWindowSetDecorations(args) => {
                let result = win::engine_cmd_window_set_decorations(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::WindowSetDecorations(result),
                });
            }
            EngineCmd::CmdWindowHasDecorations(args) => {
                let result = win::engine_cmd_window_has_decorations(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::WindowHasDecorations(result),
                });
            }
            EngineCmd::CmdWindowSetResizable(args) => {
                let result = win::engine_cmd_window_set_resizable(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::WindowSetResizable(result),
                });
            }
            EngineCmd::CmdWindowIsResizable(args) => {
                let result = win::engine_cmd_window_is_resizable(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::WindowIsResizable(result),
                });
            }
            EngineCmd::CmdWindowRequestAttention(args) => {
                let result = win::engine_cmd_window_request_attention(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::WindowRequestAttention(result),
                });
            }
            EngineCmd::CmdWindowFocus(args) => {
                let result = win::engine_cmd_window_focus(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::WindowFocus(result),
                });
            }
            EngineCmd::CmdWindowSetCursorVisible(args) => {
                let result = win::engine_cmd_window_set_cursor_visible(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::WindowSetCursorVisible(result),
                });
            }
            EngineCmd::CmdWindowSetCursorGrab(args) => {
                let result = win::engine_cmd_window_set_cursor_grab(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::WindowSetCursorGrab(result),
                });
            }
            EngineCmd::CmdWindowSetCursorIcon(args) => {
                let result = win::engine_cmd_window_set_cursor_icon(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::WindowSetCursorIcon(result),
                });
            }
            EngineCmd::CmdShaderCreate(args) => {
                eprintln!("🔍 DEBUG: Received CmdShaderCreate command");
                let result = render::engine_cmd_shader_create(engine, &args);
                eprintln!(
                    "🔍 DEBUG: Shader creation result: success={}, message={}",
                    result.success, result.message
                );
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::ShaderCreate(result),
                });
            }
            EngineCmd::CmdShaderDispose(args) => {
                let result = render::engine_cmd_shader_dispose(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::ShaderDispose(result),
                });
            }
            EngineCmd::CmdGeometryCreate(args) => {
                let result = render::engine_cmd_geometry_create(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::GeometryCreate(result),
                });
            }
            EngineCmd::CmdGeometryDispose(args) => {
                let result = render::engine_cmd_geometry_dispose(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::GeometryDispose(result),
                });
            }
            EngineCmd::CmdMaterialCreate(args) => {
                eprintln!("🔍 DEBUG: Received CmdMaterialCreate command");
                let result = render::engine_cmd_material_create(engine, &args);
                eprintln!(
                    "🔍 DEBUG: Material creation result: success={}, message={}",
                    result.success, result.message
                );
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::MaterialCreate(result),
                });
            }
            EngineCmd::CmdMaterialUpdate(args) => {
                let result = render::engine_cmd_material_update(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::MaterialUpdate(result),
                });
            }
            EngineCmd::CmdMaterialDispose(args) => {
                let result = render::engine_cmd_material_dispose(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::MaterialDispose(result),
                });
            }
            EngineCmd::CmdTextureCreate(args) => {
                let result = render::engine_cmd_texture_create(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::TextureCreate(result),
                });
            }
            EngineCmd::CmdTextureUpdate(args) => {
                let result = render::engine_cmd_texture_update(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::TextureUpdate(result),
                });
            }
            EngineCmd::CmdTextureDispose(args) => {
                let result = render::engine_cmd_texture_dispose(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::TextureDispose(result),
                });
            }
            EngineCmd::CmdSamplerCreate(args) => {
                let result = render::engine_cmd_sampler_create(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::SamplerCreate(result),
                });
            }
            EngineCmd::CmdSamplerUpdate(args) => {
                let result = render::engine_cmd_sampler_update(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::SamplerUpdate(result),
                });
            }
            EngineCmd::CmdSamplerDispose(args) => {
                let result = render::engine_cmd_sampler_dispose(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::SamplerDispose(result),
                });
            }
            EngineCmd::CmdCameraCreate(args) => {
                let result = render::engine_cmd_camera_create(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::CameraCreate(result),
                });
            }
            EngineCmd::CmdCameraUpdate(args) => {
                let result = render::engine_cmd_camera_update(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::CameraUpdate(result),
                });
            }
            EngineCmd::CmdCameraDispose(args) => {
                let result = render::engine_cmd_camera_dispose(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::CameraDispose(result),
                });
            }
            EngineCmd::CmdModelCreate(args) => {
                let result = render::engine_cmd_model_create(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::ModelCreate(result),
                });
            }
            EngineCmd::CmdModelUpdate(args) => {
                let result = render::engine_cmd_model_update(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::ModelUpdate(result),
                });
            }
            EngineCmd::CmdModelDispose(args) => {
                let result = render::engine_cmd_model_dispose(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::ModelDispose(result),
                });
            }
        }
    }

    VulframResult::Success
}
