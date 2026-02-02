use std::collections::{HashMap, HashSet};
use std::io::Cursor;
use std::sync::mpsc::{Receiver, Sender, channel};

use glam::{Mat3, Quat, Vec3};
use kira::manager::{AudioManager, AudioManagerSettings, backend::DefaultBackend};
use kira::sound::static_sound::{StaticSoundData, StaticSoundHandle};
use kira::sound::PlaybackRate;
use kira::spatial::emitter::{EmitterHandle, EmitterSettings};
use kira::spatial::listener::{ListenerHandle, ListenerSettings};
use kira::spatial::scene::{SpatialSceneHandle, SpatialSceneSettings};
use kira::tween::Tween;
use kira::Volume;
use mint::{Quaternion, Vector3};

use crate::core::audio::{
    AudioListenerState, AudioProxy, AudioReadyEvent, AudioSourceParams,
};

struct DecodeResult {
    audio_id: u32,
    data: Result<StaticSoundData, String>,
}

struct KiraSource {
    audio_id: u32,
    looping: bool,
    params: AudioSourceParams,
    emitter: EmitterHandle,
    handle: Option<StaticSoundHandle>,
}

pub struct KiraAudioProxy {
    manager: Option<AudioManager<DefaultBackend>>,
    scene: Option<SpatialSceneHandle>,
    listener: Option<ListenerHandle>,
    buffers: HashMap<u32, StaticSoundData>,
    sources: HashMap<u32, KiraSource>,
    pending: HashSet<u32>,
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
        looping: bool,
    ) -> StaticSoundData {
        let mut data = base.clone();
        if looping {
            data = data.loop_region(..);
        }
        data.output_destination(emitter)
            .volume(Volume::Amplitude(params.gain as f64))
            .playback_rate(PlaybackRate::Factor(params.pitch as f64))
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

    fn buffer_create_from_bytes(
        &mut self,
        audio_id: u32,
        bytes: Vec<u8>,
    ) -> Result<(), String> {
        self.ensure_initialized()?;
        if self.pending.contains(&audio_id) {
            return Err(format!("Audio {audio_id} already pending"));
        }
        self.pending.insert(audio_id);
        let sender = self.sender.clone();
        std::thread::spawn(move || {
            let result = StaticSoundData::from_cursor(Cursor::new(bytes))
                .map_err(|err| err.to_string());
            let _ = sender.send(DecodeResult { audio_id, data: result });
        });
        Ok(())
    }

    fn source_create(
        &mut self,
        source_id: u32,
        audio_id: u32,
        looping: bool,
        params: AudioSourceParams,
    ) -> Result<(), String> {
        self.ensure_initialized()?;
        let scene = self
            .scene
            .as_mut()
            .ok_or_else(|| "Audio scene not initialized".to_string())?;
        let settings = EmitterSettings::new().distances((
            params.spatial.min_distance,
            params.spatial.max_distance,
        ));
        let emitter = scene
            .add_emitter(Self::to_mint_vec3(params.position), settings)
            .map_err(|err| format!("Audio emitter error: {err}"))?;
        self.sources.insert(
            source_id,
            KiraSource {
                audio_id,
                looping,
                params,
                emitter,
                handle: None,
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
        if let Some(handle) = source.handle.as_mut() {
            Self::update_handle(handle, params);
        }
        Ok(())
    }

    fn source_play(&mut self, source_id: u32) -> Result<(), String> {
        self.ensure_initialized()?;
        let manager = self
            .manager
            .as_mut()
            .ok_or_else(|| "Audio manager not initialized".to_string())?;
        let source = self
            .sources
            .get_mut(&source_id)
            .ok_or_else(|| format!("Audio source {source_id} not found"))?;
        if let Some(handle) = source.handle.as_mut() {
            handle.resume(Tween::default());
            return Ok(());
        }
        let buffer = self
            .buffers
            .get(&source.audio_id)
            .ok_or_else(|| format!("Audio buffer {} not ready", source.audio_id))?;
        let sound = Self::build_sound_data(buffer, &source.emitter, source.params, source.looping);
        let handle = manager.play(sound).map_err(|err| err.to_string())?;
        source.handle = Some(handle);
        Ok(())
    }

    fn source_pause(&mut self, source_id: u32) -> Result<(), String> {
        let source = self
            .sources
            .get_mut(&source_id)
            .ok_or_else(|| format!("Audio source {source_id} not found"))?;
        if let Some(handle) = source.handle.as_mut() {
            handle.pause(Tween::default());
            return Ok(());
        }
        Err(format!("Audio source {source_id} not playing"))
    }

    fn source_stop(&mut self, source_id: u32) -> Result<(), String> {
        let source = self
            .sources
            .get_mut(&source_id)
            .ok_or_else(|| format!("Audio source {source_id} not found"))?;
        if let Some(mut handle) = source.handle.take() {
            handle.stop(Tween::default());
        }
        Ok(())
    }

    fn buffer_dispose(&mut self, audio_id: u32) -> Result<(), String> {
        self.buffers.remove(&audio_id);
        self.pending.remove(&audio_id);
        Ok(())
    }

    fn source_dispose(&mut self, source_id: u32) -> Result<(), String> {
        if let Some(mut source) = self.sources.remove(&source_id) {
            if let Some(mut handle) = source.handle.take() {
                handle.stop(Tween::default());
            }
        }
        Ok(())
    }

    fn drain_events(&mut self) -> Vec<AudioReadyEvent> {
        let mut events = Vec::new();
        while let Ok(result) = self.receiver.try_recv() {
            self.pending.remove(&result.audio_id);
            match result.data {
                Ok(data) => {
                    self.buffers.insert(result.audio_id, data);
                    events.push(AudioReadyEvent {
                        audio_id: result.audio_id,
                        success: true,
                        message: "Audio decoded".into(),
                    });
                }
                Err(message) => {
                    events.push(AudioReadyEvent {
                        audio_id: result.audio_id,
                        success: false,
                        message,
                    });
                }
            }
        }
        events
    }
}
