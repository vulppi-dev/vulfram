use crate::core::platforms::PlatformProxy;
use serde::{Deserialize, Serialize};

use crate::core::VulframResult;
use crate::core::gamepad::events::GamepadEvent;
use crate::core::input::events::{KeyboardEvent, PointerEvent};
use crate::core::state::EngineState;
use crate::core::system::SystemEvent;
use crate::core::window::WindowEvent;

pub use crate::core::buffers as buf;
pub use crate::core::render::gizmos as gizmo;
pub use crate::core::resources as res;
pub use crate::core::system as sys;
pub use crate::core::window as win;

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(tag = "type", content = "content", rename_all = "kebab-case")]
pub enum EngineCmd {
    CmdNotificationSend(sys::CmdNotificationSendArgs),
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
    CmdUploadBufferDiscardAll(buf::CmdUploadBufferDiscardAllArgs),
    CmdCameraCreate(res::CmdCameraCreateArgs),
    CmdCameraUpdate(res::CmdCameraUpdateArgs),
    CmdCameraDispose(res::CmdCameraDisposeArgs),
    CmdModelCreate(res::CmdModelCreateArgs),
    CmdModelUpdate(res::CmdModelUpdateArgs),
    CmdPoseUpdate(res::CmdPoseUpdateArgs),
    CmdModelDispose(res::CmdModelDisposeArgs),
    CmdLightCreate(res::CmdLightCreateArgs),
    CmdLightUpdate(res::CmdLightUpdateArgs),
    CmdLightDispose(res::CmdLightDisposeArgs),
    CmdMaterialCreate(res::CmdMaterialCreateArgs),
    CmdMaterialUpdate(res::CmdMaterialUpdateArgs),
    CmdMaterialDispose(res::CmdMaterialDisposeArgs),
    CmdTextureCreateFromBuffer(res::CmdTextureCreateFromBufferArgs),
    CmdTextureCreateSolidColor(res::CmdTextureCreateSolidColorArgs),
    CmdTextureDispose(res::CmdTextureDisposeArgs),
    CmdGeometryCreate(res::CmdGeometryCreateArgs),
    CmdGeometryUpdate(res::CmdGeometryUpdateArgs),
    CmdGeometryDispose(res::CmdGeometryDisposeArgs),
    CmdPrimitiveGeometryCreate(res::CmdPrimitiveGeometryCreateArgs),
    CmdShadowConfigure(res::shadow::CmdShadowConfigureArgs),
    CmdModelList(res::CmdModelListArgs),
    CmdMaterialList(res::CmdMaterialListArgs),
    CmdTextureList(res::CmdTextureListArgs),
    CmdGeometryList(res::CmdGeometryListArgs),
    CmdLightList(res::CmdLightListArgs),
    CmdCameraList(res::CmdCameraListArgs),
    CmdGizmoDrawLine(gizmo::CmdGizmoDrawLineArgs),
    CmdGizmoDrawAabb(gizmo::CmdGizmoDrawAabbArgs),
}

/// Spontaneous engine events (input, window changes, system events)
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type", content = "content", rename_all = "kebab-case")]
pub enum EngineEvent {
    Window(WindowEvent),
    Pointer(PointerEvent),
    Keyboard(KeyboardEvent),
    Gamepad(GamepadEvent),
    System(SystemEvent),
}

/// Command responses (answers to commands sent by user)
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type", content = "content", rename_all = "kebab-case")]
pub enum CommandResponse {
    NotificationSend(sys::CmdResultNotificationSend),
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
    UploadBufferDiscardAll(buf::CmdResultUploadBufferDiscardAll),
    CameraCreate(res::CmdResultCameraCreate),
    CameraUpdate(res::CmdResultCameraUpdate),
    CameraDispose(res::CmdResultCameraDispose),
    ModelCreate(res::CmdResultModelCreate),
    ModelUpdate(res::CmdResultModelUpdate),
    PoseUpdate(res::CmdResultPoseUpdate),
    ModelDispose(res::CmdResultModelDispose),
    LightCreate(res::CmdResultLightCreate),
    LightUpdate(res::CmdResultLightUpdate),
    LightDispose(res::CmdResultLightDispose),
    MaterialCreate(res::CmdResultMaterialCreate),
    MaterialUpdate(res::CmdResultMaterialUpdate),
    MaterialDispose(res::CmdResultMaterialDispose),
    TextureCreateFromBuffer(res::CmdResultTextureCreateFromBuffer),
    TextureCreateSolidColor(res::CmdResultTextureCreateSolidColor),
    TextureDispose(res::CmdResultTextureDispose),
    GeometryCreate(res::CmdResultGeometryCreate),
    GeometryUpdate(res::CmdResultGeometryUpdate),
    GeometryDispose(res::CmdResultGeometryDispose),
    PrimitiveGeometryCreate(res::CmdResultPrimitiveGeometryCreate),
    ShadowConfigure(res::shadow::CmdResultShadowConfigure),
    ModelList(res::CmdResultModelList),
    MaterialList(res::CmdResultMaterialList),
    TextureList(res::CmdResultTextureList),
    GeometryList(res::CmdResultGeometryList),
    LightList(res::CmdResultLightList),
    CameraList(res::CmdResultCameraList),
    GizmoDrawLine(gizmo::CmdResultGizmoDraw),
    GizmoDrawAabb(gizmo::CmdResultGizmoDraw),
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EngineCmdEnvelope {
    pub id: u64,
    #[serde(flatten)]
    pub cmd: EngineCmd,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
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
    platform: &mut dyn PlatformProxy,
    batch: EngineBatchCmds,
) -> VulframResult {
    for pack in batch {
        match pack.cmd {
            EngineCmd::CmdNotificationSend(args) => {
                let result =
                    sys::engine_cmd_notification_send(engine, platform.event_loop_proxy(), &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::NotificationSend(result),
                });
            }
            EngineCmd::CmdWindowCreate(args) => {
                match platform.handle_window_create(engine, pack.id, &args) {
                    Ok(()) => {}
                    Err(result) => {
                        engine.response_queue.push(CommandResponseEnvelope {
                            id: pack.id,
                            response: CommandResponse::WindowCreate(result),
                        });
                    }
                }
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
            EngineCmd::CmdUploadBufferDiscardAll(args) => {
                let result = buf::engine_cmd_upload_buffer_discard_all(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::UploadBufferDiscardAll(result),
                });
            }
            EngineCmd::CmdCameraCreate(args) => {
                let result = res::engine_cmd_camera_create(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::CameraCreate(result),
                });
            }
            EngineCmd::CmdCameraUpdate(args) => {
                let result = res::engine_cmd_camera_update(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::CameraUpdate(result),
                });
            }
            EngineCmd::CmdCameraDispose(args) => {
                let result = res::engine_cmd_camera_dispose(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::CameraDispose(result),
                });
            }
            EngineCmd::CmdModelCreate(args) => {
                let result = res::engine_cmd_model_create(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::ModelCreate(result),
                });
            }
            EngineCmd::CmdModelUpdate(args) => {
                let result = res::engine_cmd_model_update(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::ModelUpdate(result),
                });
            }
            EngineCmd::CmdPoseUpdate(args) => {
                let result = res::engine_cmd_pose_update(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::PoseUpdate(result),
                });
            }
            EngineCmd::CmdModelDispose(args) => {
                let result = res::engine_cmd_model_dispose(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::ModelDispose(result),
                });
            }
            EngineCmd::CmdLightCreate(args) => {
                let result = res::engine_cmd_light_create(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::LightCreate(result),
                });
            }
            EngineCmd::CmdLightUpdate(args) => {
                let result = res::engine_cmd_light_update(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::LightUpdate(result),
                });
            }
            EngineCmd::CmdLightDispose(args) => {
                let result = res::engine_cmd_light_dispose(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::LightDispose(result),
                });
            }
            EngineCmd::CmdMaterialCreate(args) => {
                let result = res::engine_cmd_material_create(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::MaterialCreate(result),
                });
            }
            EngineCmd::CmdMaterialUpdate(args) => {
                let result = res::engine_cmd_material_update(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::MaterialUpdate(result),
                });
            }
            EngineCmd::CmdMaterialDispose(args) => {
                let result = res::engine_cmd_material_dispose(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::MaterialDispose(result),
                });
            }
            EngineCmd::CmdTextureCreateFromBuffer(args) => {
                let result = res::engine_cmd_texture_create_from_buffer(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::TextureCreateFromBuffer(result),
                });
            }
            EngineCmd::CmdTextureCreateSolidColor(args) => {
                let result = res::engine_cmd_texture_create_solid_color(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::TextureCreateSolidColor(result),
                });
            }
            EngineCmd::CmdTextureDispose(args) => {
                let result = res::engine_cmd_texture_dispose(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::TextureDispose(result),
                });
            }
            EngineCmd::CmdGeometryCreate(args) => {
                let result = res::engine_cmd_geometry_create(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::GeometryCreate(result),
                });
            }
            EngineCmd::CmdGeometryUpdate(args) => {
                let result = res::engine_cmd_geometry_update(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::GeometryUpdate(result),
                });
            }
            EngineCmd::CmdGeometryDispose(args) => {
                let result = res::engine_cmd_geometry_dispose(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::GeometryDispose(result),
                });
            }
            EngineCmd::CmdPrimitiveGeometryCreate(args) => {
                let result = res::engine_cmd_primitive_geometry_create(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::PrimitiveGeometryCreate(result),
                });
            }
            EngineCmd::CmdShadowConfigure(args) => {
                let result = res::shadow::engine_cmd_shadow_configure(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::ShadowConfigure(result),
                });
            }
            EngineCmd::CmdModelList(args) => {
                let result = res::engine_cmd_model_list(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::ModelList(result),
                });
            }
            EngineCmd::CmdMaterialList(args) => {
                let result = res::engine_cmd_material_list(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::MaterialList(result),
                });
            }
            EngineCmd::CmdTextureList(args) => {
                let result = res::engine_cmd_texture_list(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::TextureList(result),
                });
            }
            EngineCmd::CmdGeometryList(args) => {
                let result = res::engine_cmd_geometry_list(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::GeometryList(result),
                });
            }
            EngineCmd::CmdLightList(args) => {
                let result = res::engine_cmd_light_list(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::LightList(result),
                });
            }
            EngineCmd::CmdCameraList(args) => {
                let result = res::engine_cmd_camera_list(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::CameraList(result),
                });
            }
            EngineCmd::CmdGizmoDrawLine(args) => {
                for window_state in engine.window.states.values_mut() {
                    window_state
                        .render_state
                        .gizmos
                        .add_line(args.start, args.end, args.color);
                    window_state.is_dirty = true;
                }
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::GizmoDrawLine(gizmo::CmdResultGizmoDraw {
                        status: 0,
                    }),
                });
            }
            EngineCmd::CmdGizmoDrawAabb(args) => {
                for window_state in engine.window.states.values_mut() {
                    window_state
                        .render_state
                        .gizmos
                        .add_aabb(args.min, args.max, args.color);
                    window_state.is_dirty = true;
                }
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::GizmoDrawAabb(gizmo::CmdResultGizmoDraw {
                        status: 0,
                    }),
                });
            }
        }
    }

    VulframResult::Success
}
