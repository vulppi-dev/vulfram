use std::collections::{HashMap, HashSet};
use std::io::Cursor;
use std::sync::mpsc::{Receiver, Sender, channel};

use glam::{Mat3, Quat, Vec3};
use kira::manager::{AudioManager, AudioManagerSettings, backend::DefaultBackend};
use kira::sound::PlaybackRate;
use kira::sound::static_sound::{StaticSoundData, StaticSoundHandle};
use kira::spatial::emitter::{EmitterHandle, EmitterSettings};
use kira::spatial::listener::{ListenerHandle, ListenerSettings};
use kira::spatial::scene::{SpatialSceneHandle, SpatialSceneSettings};
use kira::tween::Tween;
use kira::{StartTime, Volume};
use mint::{Quaternion, Vector3};

use crate::core::audio::{
    AudioListenerState, AudioPlayMode, AudioProxy, AudioReadyEvent, AudioSourceParams,
};

struct DecodeResult {
    resource_id: u32,
    data: Result<StaticSoundData, String>,
}

struct KiraLayer {
    resource_id: u32,
    handle: StaticSoundHandle,
}

struct KiraSource {
    params: AudioSourceParams,
    emitter: EmitterHandle,
    handles: HashMap<u32, KiraLayer>,
}

pub struct KiraAudioProxy {
    manager: Option<AudioManager<DefaultBackend>>,
    scene: Option<SpatialSceneHandle>,
    listener: Option<ListenerHandle>,
    buffers: HashMap<u32, StaticSoundData>,
    sources: HashMap<u32, KiraSource>,
    pending: HashSet<u32>,
    canceled: HashSet<u32>,
    sender: Sender<DecodeResult>,
    receiver: Receiver<DecodeResult>,
}

impl Default for KiraAudioProxy {
    fn default() -> Self {
        let (sender, receiver) = channel();
        Self {
            manager: None,
            scene: None,
            listener: None,
            buffers: HashMap::new(),
            sources: HashMap::new(),
            pending: HashSet::new(),
            canceled: HashSet::new(),
            sender,
            receiver,
        }
    }
}

impl KiraAudioProxy {
    fn ensure_initialized(&mut self) -> Result<(), String> {
        if self.manager.is_some() {
            return Ok(());
        }
        let mut manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())
            .map_err(|err| err.to_string())?;
        let mut scene = manager
            .add_spatial_scene(SpatialSceneSettings::default())
            .map_err(|err| format!("Audio scene error: {err}"))?;
        let listener = scene
            .add_listener(
                Self::to_mint_vec3(Vec3::ZERO),
                Self::to_mint_quat(Quat::IDENTITY),
                ListenerSettings::default(),
            )
            .map_err(|err| format!("Audio listener error: {err}"))?;
        self.manager = Some(manager);
        self.scene = Some(scene);
        self.listener = Some(listener);
        Ok(())
    }

    fn listener_orientation(forward: Vec3, up: Vec3) -> Quat {
        let f = forward.normalize_or_zero();
        let mut u = up.normalize_or_zero();
        let mut r = f.cross(u).normalize_or_zero();
        if r.length_squared() == 0.0 {
            r = Vec3::X;
        }
        u = r.cross(f).normalize_or_zero();
        let basis = Mat3::from_cols(r, u, -f);
        Quat::from_mat3(&basis)
    }

    fn to_mint_vec3(value: Vec3) -> Vector3<f32> {
        Vector3 {
            x: value.x,
            y: value.y,
            z: value.z,
        }
    }

    fn to_mint_quat(value: Quat) -> Quaternion<f32> {
        let v = value.xyz();
        Quaternion {
            s: value.w,
            v: Vector3 {
                x: v.x,
                y: v.y,
                z: v.z,
            },
        }
    }

    fn build_sound_data(
        base: &StaticSoundData,
        emitter: &EmitterHandle,
        params: AudioSourceParams,
        mode: AudioPlayMode,
        delay_ms: Option<u32>,
        intensity: f32,
    ) -> StaticSoundData {
        let mut data = base.clone();
        let delay = delay_ms
            .map(|ms| StartTime::Delayed(std::time::Duration::from_millis(ms as u64)))
            .unwrap_or(StartTime::Immediate);
        let intensity = intensity.clamp(0.0, 1.0);
        if matches!(mode, AudioPlayMode::Loop) {
            data = data.loop_region(..);
        }
        data.output_destination(emitter)
            .volume(Volume::Amplitude((params.gain * intensity) as f64))
            .playback_rate(PlaybackRate::Factor(params.pitch as f64))
            .start_time(delay)
    }

    fn update_handle(handle: &mut StaticSoundHandle, params: AudioSourceParams) {
        let tween = Tween::default();
        handle.set_volume(Volume::Amplitude(params.gain as f64), tween);
        handle.set_playback_rate(PlaybackRate::Factor(params.pitch as f64), tween);
    }
}

impl AudioProxy for KiraAudioProxy {
    fn init(&mut self) -> Result<(), String> {
        self.ensure_initialized()
    }

    fn listener_update(&mut self, state: AudioListenerState) -> Result<(), String> {
        self.ensure_initialized()?;
        let listener = self
            .listener
            .as_mut()
            .ok_or_else(|| "Audio listener not initialized".to_string())?;
        let tween = Tween::default();
        listener.set_position(Self::to_mint_vec3(state.position), tween);
        let orientation = Self::listener_orientation(state.forward, state.up);
        listener.set_orientation(Self::to_mint_quat(orientation), tween);
        Ok(())
    }

    fn buffer_create_from_bytes(&mut self, resource_id: u32, bytes: Vec<u8>) -> Result<(), String> {
        self.ensure_initialized()?;
        if self.pending.contains(&resource_id) {
            return Err(format!("Audio {resource_id} already pending"));
        }
        self.canceled.remove(&resource_id);
        self.pending.insert(resource_id);
        let sender = self.sender.clone();
        std::thread::spawn(move || {
            let result =
                StaticSoundData::from_cursor(Cursor::new(bytes)).map_err(|err| err.to_string());
            let _ = sender.send(DecodeResult {
                resource_id,
                data: result,
            });
        });
        Ok(())
    }

    fn source_create(&mut self, source_id: u32, params: AudioSourceParams) -> Result<(), String> {
        self.ensure_initialized()?;
        let scene = self
            .scene
            .as_mut()
            .ok_or_else(|| "Audio scene not initialized".to_string())?;
        let settings = EmitterSettings::new()
            .distances((params.spatial.min_distance, params.spatial.max_distance));
        let emitter = scene
            .add_emitter(Self::to_mint_vec3(params.position), settings)
            .map_err(|err| format!("Audio emitter error: {err}"))?;
        self.sources.insert(
            source_id,
            KiraSource {
                params,
                emitter,
                handles: HashMap::new(),
            },
        );
        Ok(())
    }

    fn source_update(&mut self, source_id: u32, params: AudioSourceParams) -> Result<(), String> {
        let source = self
            .sources
            .get_mut(&source_id)
            .ok_or_else(|| format!("Audio source {source_id} not found"))?;
        source.params = params;
        source
            .emitter
            .set_position(Self::to_mint_vec3(params.position), Tween::default());
        for layer in source.handles.values_mut() {
            Self::update_handle(&mut layer.handle, params);
        }
        Ok(())
    }

    fn source_play(
        &mut self,
        source_id: u32,
        resource_id: u32,
        timeline_id: u32,
        mode: AudioPlayMode,
        delay_ms: Option<u32>,
        intensity: f32,
    ) -> Result<(), String> {
        self.ensure_initialized()?;
        let manager = self
            .manager
            .as_mut()
            .ok_or_else(|| "Audio manager not initialized".to_string())?;
        let source = self
            .sources
            .get_mut(&source_id)
            .ok_or_else(|| format!("Audio source {source_id} not found"))?;
        let buffer = self
            .buffers
            .get(&resource_id)
            .ok_or_else(|| format!("Audio buffer {} not ready", resource_id))?;
        if let Some(mut layer) = source.handles.remove(&timeline_id) {
            layer.handle.stop(Tween::default());
        }
        let sound = Self::build_sound_data(
            buffer,
            &source.emitter,
            source.params,
            mode,
            delay_ms,
            intensity,
        );
        let handle = manager.play(sound).map_err(|err| err.to_string())?;
        source.handles.insert(
            timeline_id,
            KiraLayer {
                resource_id,
                handle,
            },
        );
        Ok(())
    }

    fn source_pause(&mut self, source_id: u32, timeline_id: Option<u32>) -> Result<(), String> {
        let source = self
            .sources
            .get_mut(&source_id)
            .ok_or_else(|| format!("Audio source {source_id} not found"))?;
        if let Some(timeline_id) = timeline_id {
            if let Some(layer) = source.handles.get_mut(&timeline_id) {
                layer.handle.pause(Tween::default());
            }
            return Ok(());
        }
        for layer in source.handles.values_mut() {
            layer.handle.pause(Tween::default());
        }
        Ok(())
    }

    fn source_stop(&mut self, source_id: u32, timeline_id: Option<u32>) -> Result<(), String> {
        let source = self
            .sources
            .get_mut(&source_id)
            .ok_or_else(|| format!("Audio source {source_id} not found"))?;
        if let Some(timeline_id) = timeline_id {
            if let Some(mut layer) = source.handles.remove(&timeline_id) {
                layer.handle.stop(Tween::default());
            }
            return Ok(());
        }
        for (_timeline, mut layer) in source.handles.drain() {
            layer.handle.stop(Tween::default());
        }
        Ok(())
    }

    fn buffer_dispose(&mut self, resource_id: u32) -> Result<(), String> {
        self.buffers.remove(&resource_id);
        self.pending.remove(&resource_id);
        self.canceled.insert(resource_id);
        for source in self.sources.values_mut() {
            let mut to_stop = Vec::new();
            for (timeline_id, layer) in source.handles.iter() {
                if layer.resource_id == resource_id {
                    to_stop.push(*timeline_id);
                }
            }
            for timeline_id in to_stop {
                if let Some(mut layer) = source.handles.remove(&timeline_id) {
                    layer.handle.stop(Tween::default());
                }
            }
        }
        Ok(())
    }

    fn source_dispose(&mut self, source_id: u32) -> Result<(), String> {
        if let Some(mut source) = self.sources.remove(&source_id) {
            for (_timeline, mut layer) in source.handles.drain() {
                layer.handle.stop(Tween::default());
            }
        }
        Ok(())
    }

    fn drain_events(&mut self) -> Vec<AudioReadyEvent> {
        let mut events = Vec::new();
        while let Ok(result) = self.receiver.try_recv() {
            self.pending.remove(&result.resource_id);
            if self.canceled.remove(&result.resource_id) {
                events.push(AudioReadyEvent {
                    resource_id: result.resource_id,
                    success: false,
                    message: "Audio decode canceled".into(),
                });
                continue;
            }
            match result.data {
                Ok(data) => {
                    self.buffers.insert(result.resource_id, data);
                    events.push(AudioReadyEvent {
                        resource_id: result.resource_id,
                        success: true,
                        message: "Audio decoded".into(),
                    });
                }
                Err(message) => {
                    events.push(AudioReadyEvent {
                        resource_id: result.resource_id,
                        success: false,
                        message,
                    });
                }
            }
        }
        events
    }
}
