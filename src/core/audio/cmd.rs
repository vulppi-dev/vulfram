use glam::{Quat, Vec3};
use serde::{Deserialize, Serialize};

use crate::core::buffers::state::UploadType;
use crate::core::state::EngineState;
use crate::core::audio::{AudioListenerState, AudioSourceParams};

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdAudioListenerUpdateArgs {
    pub position: Vec3,
    pub velocity: Vec3,
    pub forward: Vec3,
    pub up: Vec3,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdAudioListenerCreateArgs {
    pub window_id: u32,
    pub model_id: u32,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultAudioListenerCreate {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdAudioListenerDisposeArgs {
    pub window_id: u32,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultAudioListenerDispose {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultAudioListenerUpdate {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdAudioResourceCreateArgs {
    pub resource_id: u32,
    pub buffer_id: u64,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultAudioResourceCreate {
    pub success: bool,
    pub message: String,
    pub pending: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdAudioSourceCreateArgs {
    pub window_id: u32,
    pub source_id: u32,
    pub resource_id: u32,
    pub model_id: u32,
    pub position: Vec3,
    pub velocity: Vec3,
    pub orientation: Quat,
    pub gain: f32,
    pub pitch: f32,
    pub spatial: AudioSpatialParamsDto,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AudioSpatialParamsDto {
    pub min_distance: f32,
    pub max_distance: f32,
    pub rolloff: f32,
    pub cone_inner: f32,
    pub cone_outer: f32,
    pub cone_outer_gain: f32,
}

impl Default for AudioSpatialParamsDto {
    fn default() -> Self {
        Self {
            min_distance: 1.0,
            max_distance: 100.0,
            rolloff: 1.0,
            cone_inner: 360.0,
            cone_outer: 360.0,
            cone_outer_gain: 0.0,
        }
    }
}

impl From<AudioSpatialParamsDto> for crate::core::audio::AudioSpatialParams {
    fn from(value: AudioSpatialParamsDto) -> Self {
        Self {
            min_distance: value.min_distance,
            max_distance: value.max_distance,
            rolloff: value.rolloff,
            cone_inner: value.cone_inner,
            cone_outer: value.cone_outer,
            cone_outer_gain: value.cone_outer_gain,
        }
    }
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultAudioSourceCreate {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdAudioSourceUpdateArgs {
    pub source_id: u32,
    pub position: Vec3,
    pub velocity: Vec3,
    pub orientation: Quat,
    pub gain: f32,
    pub pitch: f32,
    pub spatial: AudioSpatialParamsDto,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultAudioSourceUpdate {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdAudioSourcePlayArgs {
    pub source_id: u32,
    pub intensity: f32,
    pub delay_ms: Option<u32>,
    pub mode: AudioPlayModeDto,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultAudioSourcePlay {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdAudioSourcePauseArgs {
    pub source_id: u32,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultAudioSourcePause {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdAudioSourceStopArgs {
    pub source_id: u32,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultAudioSourceStop {
    pub success: bool,
    pub message: String,
}
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdAudioSourceDisposeArgs {
    pub source_id: u32,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultAudioSourceDispose {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub enum AudioPlayModeDto {
    Once,
    Loop,
    Reverse,
    LoopReverse,
    PingPong,
}

impl From<AudioPlayModeDto> for crate::core::audio::AudioPlayMode {
    fn from(value: AudioPlayModeDto) -> Self {
        match value {
            AudioPlayModeDto::Once => Self::Once,
            AudioPlayModeDto::Loop => Self::Loop,
            AudioPlayModeDto::Reverse => Self::Reverse,
            AudioPlayModeDto::LoopReverse => Self::LoopReverse,
            AudioPlayModeDto::PingPong => Self::PingPong,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdAudioResourceDisposeArgs {
    pub resource_id: u32,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultAudioResourceDispose {
    pub success: bool,
    pub message: String,
}

pub fn engine_cmd_audio_listener_update(
    engine: &mut EngineState,
    args: &CmdAudioListenerUpdateArgs,
) -> CmdResultAudioListenerUpdate {
    let state = AudioListenerState {
        position: args.position,
        velocity: args.velocity,
        forward: args.forward,
        up: args.up,
    };
    match engine.audio.listener_update(state) {
        Ok(()) => CmdResultAudioListenerUpdate {
            success: true,
            message: "Listener updated".into(),
        },
        Err(message) => CmdResultAudioListenerUpdate {
            success: false,
            message,
        },
    }
}

pub fn engine_cmd_audio_listener_create(
    engine: &mut EngineState,
    args: &CmdAudioListenerCreateArgs,
) -> CmdResultAudioListenerCreate {
    if !engine.window.states.contains_key(&args.window_id) {
        return CmdResultAudioListenerCreate {
            success: false,
            message: format!("Window {} not found", args.window_id),
        };
    }
    engine.audio_listener_binding = Some(crate::core::audio::AudioListenerBinding {
        window_id: args.window_id,
        model_id: args.model_id,
    });
    CmdResultAudioListenerCreate {
        success: true,
        message: "Listener bound to model".into(),
    }
}

pub fn engine_cmd_audio_listener_dispose(
    engine: &mut EngineState,
    args: &CmdAudioListenerDisposeArgs,
) -> CmdResultAudioListenerDispose {
    let should_clear = match engine.audio_listener_binding {
        Some(binding) => binding.window_id == args.window_id,
        None => false,
    };
    if should_clear {
        engine.audio_listener_binding = None;
        CmdResultAudioListenerDispose {
            success: true,
            message: "Listener disposed".into(),
        }
    } else {
        CmdResultAudioListenerDispose {
            success: false,
            message: "Listener not found".into(),
        }
    }
}

pub fn process_audio_listener_binding(engine: &mut EngineState) {
    let binding = match engine.audio_listener_binding {
        Some(binding) => binding,
        None => return,
    };
    let window_state = match engine.window.states.get(&binding.window_id) {
        Some(state) => state,
        None => return,
    };
    let record = match window_state.render_state.scene.models.get(&binding.model_id) {
        Some(record) => record,
        None => return,
    };
    let (_, rotation, translation) = record.data.transform.to_scale_rotation_translation();
    let forward = (rotation * Vec3::NEG_Z).normalize_or_zero();
    let up = (rotation * Vec3::Y).normalize_or_zero();
    let state = AudioListenerState {
        position: translation,
        velocity: Vec3::ZERO,
        forward,
        up,
    };
    let _ = engine.audio.listener_update(state);
}

pub fn engine_cmd_audio_buffer_create_from_buffer(
    engine: &mut EngineState,
    args: &CmdAudioResourceCreateArgs,
) -> CmdResultAudioResourceCreate {
    let buffer = match engine.buffers.remove_upload(args.buffer_id) {
        Some(b) => b,
        None => {
            return CmdResultAudioResourceCreate {
                success: false,
                message: format!("Buffer with id {} not found", args.buffer_id),
                pending: false,
            };
        }
    };

    if buffer.upload_type != UploadType::BinaryAsset {
        return CmdResultAudioResourceCreate {
            success: false,
            message: format!(
                "Invalid buffer type. Expected BinaryAsset, got {:?}",
                buffer.upload_type
            ),
            pending: false,
        };
    }

    match engine.audio.buffer_create_from_bytes(args.resource_id, buffer.data) {
        Ok(()) => CmdResultAudioResourceCreate {
            success: true,
            message: "Audio buffer queued".into(),
            pending: true,
        },
        Err(message) => CmdResultAudioResourceCreate {
            success: false,
            message,
            pending: false,
        },
    }
}

pub fn engine_cmd_audio_source_create(
    engine: &mut EngineState,
    args: &CmdAudioSourceCreateArgs,
) -> CmdResultAudioSourceCreate {
    if !engine.window.states.contains_key(&args.window_id) {
        return CmdResultAudioSourceCreate {
            success: false,
            message: format!("Window {} not found", args.window_id),
        };
    }
    let params = AudioSourceParams {
        position: args.position,
        velocity: args.velocity,
        orientation: args.orientation,
        gain: args.gain,
        pitch: args.pitch,
        spatial: args.spatial.clone().into(),
    };

    engine.audio_source_params.insert(args.source_id, params);
    engine.audio_source_bindings.insert(
        args.source_id,
        crate::core::audio::AudioListenerBinding {
            window_id: args.window_id,
            model_id: args.model_id,
        },
    );
    match engine
        .audio
        .source_create(args.source_id, args.resource_id, params)
    {
        Ok(()) => CmdResultAudioSourceCreate {
            success: true,
            message: "Source created".into(),
        },
        Err(message) => CmdResultAudioSourceCreate {
            success: false,
            message,
        },
    }
}

pub fn engine_cmd_audio_source_update(
    engine: &mut EngineState,
    args: &CmdAudioSourceUpdateArgs,
) -> CmdResultAudioSourceUpdate {
    let params = AudioSourceParams {
        position: args.position,
        velocity: args.velocity,
        orientation: args.orientation,
        gain: args.gain,
        pitch: args.pitch,
        spatial: args.spatial.clone().into(),
    };
    engine.audio_source_params.insert(args.source_id, params);
    match engine.audio.source_update(args.source_id, params) {
        Ok(()) => CmdResultAudioSourceUpdate {
            success: true,
            message: "Source updated".into(),
        },
        Err(message) => CmdResultAudioSourceUpdate {
            success: false,
            message,
        },
    }
}

pub fn engine_cmd_audio_source_play(
    engine: &mut EngineState,
    args: &CmdAudioSourcePlayArgs,
) -> CmdResultAudioSourcePlay {
    let intensity = args.intensity.clamp(0.0, 1.0);
    match engine
        .audio
        .source_play(args.source_id, args.mode.clone().into(), args.delay_ms, intensity)
    {
        Ok(()) => CmdResultAudioSourcePlay {
            success: true,
            message: "Source playing".into(),
        },
        Err(message) => CmdResultAudioSourcePlay {
            success: false,
            message,
        },
    }
}

pub fn engine_cmd_audio_source_pause(
    engine: &mut EngineState,
    args: &CmdAudioSourcePauseArgs,
) -> CmdResultAudioSourcePause {
    match engine.audio.source_pause(args.source_id) {
        Ok(()) => CmdResultAudioSourcePause {
            success: true,
            message: "Source paused".into(),
        },
        Err(message) => CmdResultAudioSourcePause {
            success: false,
            message,
        },
    }
}

pub fn engine_cmd_audio_source_stop(
    engine: &mut EngineState,
    args: &CmdAudioSourceStopArgs,
) -> CmdResultAudioSourceStop {
    match engine.audio.source_stop(args.source_id) {
        Ok(()) => CmdResultAudioSourceStop {
            success: true,
            message: "Source stopped".into(),
        },
        Err(message) => CmdResultAudioSourceStop {
            success: false,
            message,
        },
    }
}

pub fn engine_cmd_audio_source_dispose(
    engine: &mut EngineState,
    args: &CmdAudioSourceDisposeArgs,
) -> CmdResultAudioSourceDispose {
    engine.audio_source_bindings.remove(&args.source_id);
    engine.audio_source_params.remove(&args.source_id);
    match engine.audio.source_dispose(args.source_id) {
        Ok(()) => CmdResultAudioSourceDispose {
            success: true,
            message: "Source disposed".into(),
        },
        Err(message) => CmdResultAudioSourceDispose {
            success: false,
            message,
        },
    }
}

pub fn engine_cmd_audio_resource_dispose(
    engine: &mut EngineState,
    args: &CmdAudioResourceDisposeArgs,
) -> CmdResultAudioResourceDispose {
    match engine.audio.buffer_dispose(args.resource_id) {
        Ok(()) => CmdResultAudioResourceDispose {
            success: true,
            message: "Resource disposed".into(),
        },
        Err(message) => CmdResultAudioResourceDispose {
            success: false,
            message,
        },
    }
}

pub fn process_audio_source_bindings(engine: &mut EngineState) {
    let listener_binding = engine.audio_listener_binding;
    let Some(listener_binding) = listener_binding else {
        return;
    };
    let window_state = match engine.window.states.get(&listener_binding.window_id) {
        Some(state) => state,
        None => return,
    };
    let listener_record = match window_state.render_state.scene.models.get(&listener_binding.model_id) {
        Some(record) => record,
        None => return,
    };
    let (_, listener_rotation, listener_translation) =
        listener_record.data.transform.to_scale_rotation_translation();
    for (source_id, binding) in engine.audio_source_bindings.iter() {
        if binding.window_id != listener_binding.window_id {
            continue;
        }
        let record = match window_state.render_state.scene.models.get(&binding.model_id) {
            Some(record) => record,
            None => continue,
        };
        let (_, rotation, translation) = record.data.transform.to_scale_rotation_translation();
        let mut params = match engine.audio_source_params.get(source_id) {
            Some(params) => *params,
            None => continue,
        };
        params.position = translation;
        params.orientation = rotation;
        if binding.model_id == listener_binding.model_id {
            params.position = listener_translation;
            params.orientation = listener_rotation;
            params.spatial.min_distance = 0.0;
            params.spatial.max_distance = 0.01;
            params.spatial.rolloff = 0.0;
        }
        let _ = engine.audio.source_update(*source_id, params);
    }
}
