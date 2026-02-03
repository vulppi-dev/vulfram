use glam::{Quat, Vec3};
use serde::{Deserialize, Serialize};

use crate::core::audio::{AudioListenerState, AudioSourceParams};
use crate::core::buffers::state::UploadType;
use crate::core::state::EngineState;

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
    pub total_bytes: Option<u64>,
    pub offset_bytes: Option<u64>,
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
pub struct CmdAudioResourcePushArgs {
    pub resource_id: u32,
    pub buffer_id: u64,
    pub offset_bytes: u64,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultAudioResourcePush {
    pub success: bool,
    pub message: String,
    pub received_bytes: u64,
    pub total_bytes: u64,
    pub complete: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdAudioSourceCreateArgs {
    pub window_id: u32,
    pub source_id: u32,
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
    pub resource_id: u32,
    pub timeline_id: Option<u32>,
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
    pub timeline_id: Option<u32>,
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
    pub timeline_id: Option<u32>,
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
}

impl From<AudioPlayModeDto> for crate::core::audio::AudioPlayMode {
    fn from(value: AudioPlayModeDto) -> Self {
        match value {
            AudioPlayModeDto::Once => Self::Once,
            AudioPlayModeDto::Loop => Self::Loop,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AudioStreamState {
    pub total_bytes: u64,
    pub received_bytes: u64,
    pub data: Vec<u8>,
    pub ranges: Vec<(u64, u64)>,
}

impl AudioStreamState {
    pub fn new(total_bytes: u64) -> Result<Self, String> {
        let size = usize::try_from(total_bytes)
            .map_err(|_| "Audio stream size exceeds addressable memory".to_string())?;
        Ok(Self {
            total_bytes,
            received_bytes: 0,
            data: vec![0; size],
            ranges: Vec::new(),
        })
    }

    pub fn apply_chunk(&mut self, offset: u64, bytes: &[u8]) -> Result<u64, String> {
        if offset >= self.total_bytes {
            return Err("Chunk offset exceeds total size".into());
        }
        let end = (offset + bytes.len() as u64).min(self.total_bytes);
        let write_len = (end - offset) as usize;
        if write_len == 0 {
            return Ok(0);
        }
        let start_index = offset as usize;
        self.data[start_index..start_index + write_len].copy_from_slice(&bytes[..write_len]);
        let added = Self::merge_range(&mut self.ranges, offset, end);
        self.received_bytes = self.received_bytes.saturating_add(added);
        Ok(added)
    }

    fn merge_range(ranges: &mut Vec<(u64, u64)>, mut start: u64, mut end: u64) -> u64 {
        let mut added = end.saturating_sub(start);
        let mut i = 0;
        while i < ranges.len() {
            let (s, e) = ranges[i];
            if e < start {
                i += 1;
                continue;
            }
            if s > end {
                break;
            }
            let overlap_start = start.max(s);
            let overlap_end = end.min(e);
            if overlap_end > overlap_start {
                added = added.saturating_sub(overlap_end - overlap_start);
            }
            start = start.min(s);
            end = end.max(e);
            ranges.remove(i);
        }
        ranges.insert(i, (start, end));
        added
    }

    pub fn complete(&self) -> bool {
        self.received_bytes >= self.total_bytes && self.total_bytes > 0
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
    let record = match window_state
        .render_state
        .scene
        .models
        .get(&binding.model_id)
    {
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

    if let Some(total_bytes) = args.total_bytes {
        let offset = args.offset_bytes.unwrap_or(0);
        let stream = match engine.audio_streams.entry(args.resource_id) {
            std::collections::hash_map::Entry::Vacant(entry) => {
                match AudioStreamState::new(total_bytes) {
                    Ok(state) => entry.insert(state),
                    Err(message) => {
                        return CmdResultAudioResourceCreate {
                            success: false,
                            message,
                            pending: false,
                        };
                    }
                }
            }
            std::collections::hash_map::Entry::Occupied(entry) => entry.into_mut(),
        };
        if let Err(message) = stream.apply_chunk(offset, &buffer.data) {
            return CmdResultAudioResourceCreate {
                success: false,
                message,
                pending: false,
            };
        }
        let complete = stream.complete();
        engine.event_queue.push(crate::core::cmd::EngineEvent::System(
            crate::core::system::events::SystemEvent::AudioStreamProgress {
                resource_id: args.resource_id,
                received_bytes: stream.received_bytes,
                total_bytes: stream.total_bytes,
                complete,
            },
        ));
        if complete {
            let stream = engine.audio_streams.remove(&args.resource_id).unwrap();
            match engine.audio.buffer_create_from_bytes(args.resource_id, stream.data) {
                Ok(()) => CmdResultAudioResourceCreate {
                    success: true,
                    message: "Audio stream queued".into(),
                    pending: true,
                },
                Err(message) => CmdResultAudioResourceCreate {
                    success: false,
                    message,
                    pending: false,
                },
            }
        } else {
            CmdResultAudioResourceCreate {
                success: true,
                message: "Audio stream chunk queued".into(),
                pending: true,
            }
        }
    } else {
        match engine
            .audio
            .buffer_create_from_bytes(args.resource_id, buffer.data)
        {
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
}

pub fn engine_cmd_audio_resource_push(
    engine: &mut EngineState,
    args: &CmdAudioResourcePushArgs,
) -> CmdResultAudioResourcePush {
    let buffer = match engine.buffers.remove_upload(args.buffer_id) {
        Some(b) => b,
        None => {
            return CmdResultAudioResourcePush {
                success: false,
                message: format!("Buffer with id {} not found", args.buffer_id),
                received_bytes: 0,
                total_bytes: 0,
                complete: false,
            };
        }
    };
    if buffer.upload_type != UploadType::BinaryAsset {
        return CmdResultAudioResourcePush {
            success: false,
            message: format!(
                "Invalid buffer type. Expected BinaryAsset, got {:?}",
                buffer.upload_type
            ),
            received_bytes: 0,
            total_bytes: 0,
            complete: false,
        };
    }
    let (received_bytes, total_bytes, complete) = {
        let stream = match engine.audio_streams.get_mut(&args.resource_id) {
            Some(stream) => stream,
            None => {
                return CmdResultAudioResourcePush {
                    success: false,
                    message: format!("Audio stream {} not found", args.resource_id),
                    received_bytes: 0,
                    total_bytes: 0,
                    complete: false,
                };
            }
        };
        if let Err(message) = stream.apply_chunk(args.offset_bytes, &buffer.data) {
            return CmdResultAudioResourcePush {
                success: false,
                message,
                received_bytes: stream.received_bytes,
                total_bytes: stream.total_bytes,
                complete: stream.complete(),
            };
        }
        (stream.received_bytes, stream.total_bytes, stream.complete())
    };
    engine.event_queue.push(crate::core::cmd::EngineEvent::System(
        crate::core::system::events::SystemEvent::AudioStreamProgress {
            resource_id: args.resource_id,
            received_bytes,
            total_bytes,
            complete,
        },
    ));
    if complete {
        let stream = engine.audio_streams.remove(&args.resource_id).unwrap();
        if let Err(message) = engine.audio.buffer_create_from_bytes(args.resource_id, stream.data) {
            return CmdResultAudioResourcePush {
                success: false,
                message,
                received_bytes,
                total_bytes,
                complete: true,
            };
        }
    }
    CmdResultAudioResourcePush {
        success: true,
        message: if complete {
            "Audio stream complete".into()
        } else {
            "Audio stream chunk queued".into()
        },
        received_bytes,
        total_bytes,
        complete,
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
    match engine.audio.source_create(args.source_id, params) {
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
    let timeline_id = args.timeline_id.unwrap_or(0);
    match engine.audio.source_play(
        args.source_id,
        args.resource_id,
        timeline_id,
        args.mode.clone().into(),
        args.delay_ms,
        intensity,
    ) {
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
    match engine.audio.source_pause(args.source_id, args.timeline_id) {
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
    match engine.audio.source_stop(args.source_id, args.timeline_id) {
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
    engine.audio_streams.remove(&args.resource_id);
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
    let listener_record = match window_state
        .render_state
        .scene
        .models
        .get(&listener_binding.model_id)
    {
        Some(record) => record,
        None => return,
    };
    let (_, listener_rotation, listener_translation) = listener_record
        .data
        .transform
        .to_scale_rotation_translation();
    for (source_id, binding) in engine.audio_source_bindings.iter() {
        if binding.window_id != listener_binding.window_id {
            continue;
        }
        let record = match window_state
            .render_state
            .scene
            .models
            .get(&binding.model_id)
        {
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
