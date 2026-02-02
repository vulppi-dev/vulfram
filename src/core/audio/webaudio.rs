use std::collections::{HashMap, HashSet};
use std::sync::mpsc::{Receiver, Sender, channel};

use glam::Vec3;
use js_sys::Uint8Array;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use web_sys::{
    AudioBuffer, AudioBufferSourceNode, AudioContext, GainNode, PannerNode, PannerOptions,
};

use crate::core::audio::{
    AudioListenerState, AudioPlayMode, AudioProxy, AudioReadyEvent, AudioSourceParams,
};

struct DecodeResult {
    audio_id: u32,
    buffer: Result<AudioBuffer, String>,
}

struct WebAudioSource {
    audio_id: u32,
    params: AudioSourceParams,
    panner: PannerNode,
    gain: GainNode,
    current: Option<AudioBufferSourceNode>,
}

pub struct WebAudioProxy {
    context: Option<AudioContext>,
    buffers: HashMap<u32, AudioBuffer>,
    sources: HashMap<u32, WebAudioSource>,
    pending: HashSet<u32>,
    sender: Sender<DecodeResult>,
    receiver: Receiver<DecodeResult>,
}

// WebAudio is single-threaded in wasm; we never move this across threads.
unsafe impl Send for WebAudioProxy {}

impl Default for WebAudioProxy {
    fn default() -> Self {
        let (sender, receiver) = channel();
        Self {
            context: None,
            buffers: HashMap::new(),
            sources: HashMap::new(),
            pending: HashSet::new(),
            sender,
            receiver,
        }
    }
}

impl WebAudioProxy {
    fn ensure_initialized(&mut self) -> Result<(), String> {
        if self.context.is_some() {
            return Ok(());
        }
        let context = AudioContext::new().map_err(|_| "Failed to create AudioContext".to_string())?;
        self.context = Some(context);
        Ok(())
    }

    fn update_panner(panner: &PannerNode, params: AudioSourceParams) {
        panner.set_position(params.position.x, params.position.y, params.position.z);
        let forward = params.orientation * Vec3::NEG_Z;
        panner.set_orientation(forward.x, forward.y, forward.z);
        panner.set_ref_distance(params.spatial.min_distance as f64);
        panner.set_max_distance(params.spatial.max_distance as f64);
        panner.set_rolloff_factor(params.spatial.rolloff as f64);
        panner.set_cone_inner_angle(params.spatial.cone_inner as f64);
        panner.set_cone_outer_angle(params.spatial.cone_outer as f64);
        panner.set_cone_outer_gain(params.spatial.cone_outer_gain as f64);
    }

    fn apply_listener(context: &AudioContext, state: AudioListenerState) {
        let listener = context.listener();
        listener.set_position(state.position.x, state.position.y, state.position.z);
        listener.set_forward(state.forward.x, state.forward.y, state.forward.z);
        listener.set_up(state.up.x, state.up.y, state.up.z);
    }

    fn build_source_node(
        context: &AudioContext,
        buffer: &AudioBuffer,
        params: AudioSourceParams,
        mode: AudioPlayMode,
        intensity: f32,
        panner: &PannerNode,
        gain: &GainNode,
    ) -> Result<AudioBufferSourceNode, String> {
        let source = context
            .create_buffer_source()
            .map_err(|_| "Failed to create AudioBufferSourceNode".to_string())?;
        source.set_buffer(Some(buffer));
        let loop_enabled = matches!(
            mode,
            AudioPlayMode::Loop | AudioPlayMode::LoopReverse | AudioPlayMode::PingPong
        );
        source.set_loop(loop_enabled);
        let rate = match mode {
            AudioPlayMode::Reverse | AudioPlayMode::LoopReverse => -params.pitch.abs(),
            _ => params.pitch,
        };
        source.playback_rate().set_value(rate);
        source.connect_with_audio_node(panner).map_err(|_| "Failed to connect panner".to_string())?;
        panner.connect_with_audio_node(gain).map_err(|_| "Failed to connect gain".to_string())?;
        gain.connect_with_audio_node(&context.destination())
            .map_err(|_| "Failed to connect destination".to_string())?;
        gain.gain().set_value(params.gain * intensity.clamp(0.0, 1.0));
        Ok(source)
    }
}

impl AudioProxy for WebAudioProxy {
    fn init(&mut self) -> Result<(), String> {
        self.ensure_initialized()
    }

    fn listener_update(&mut self, state: AudioListenerState) -> Result<(), String> {
        self.ensure_initialized()?;
        let context = self
            .context
            .as_ref()
            .ok_or_else(|| "AudioContext not initialized".to_string())?;
        Self::apply_listener(context, state);
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
        let context = self
            .context
            .as_ref()
            .ok_or_else(|| "AudioContext not initialized".to_string())?
            .clone();
        let sender = self.sender.clone();
        let buffer = Uint8Array::from(bytes.as_slice()).buffer();
        self.pending.insert(audio_id);
        spawn_local(async move {
            let promise = context.decode_audio_data(&buffer);
            let result = wasm_bindgen_futures::JsFuture::from(promise).await;
            let decode = match result {
                Ok(audio) => audio
                    .dyn_into::<AudioBuffer>()
                    .map_err(|_| "Failed to decode audio".to_string()),
                Err(_) => Err("Failed to decode audio".to_string()),
            };
            let _ = sender.send(DecodeResult { audio_id, buffer: decode });
        });
        Ok(())
    }

    fn source_create(
        &mut self,
        source_id: u32,
        audio_id: u32,
        params: AudioSourceParams,
    ) -> Result<(), String> {
        self.ensure_initialized()?;
        let context = self
            .context
            .as_ref()
            .ok_or_else(|| "AudioContext not initialized".to_string())?;
        let options = PannerOptions::new();
        let panner = context
            .create_panner_with_options(&options)
            .map_err(|_| "Failed to create PannerNode".to_string())?;
        let gain = context
            .create_gain()
            .map_err(|_| "Failed to create GainNode".to_string())?;
        Self::update_panner(&panner, params);
        gain.gain().set_value(params.gain);
        self.sources.insert(
            source_id,
            WebAudioSource {
                audio_id,
                params,
                panner,
                gain,
                current: None,
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
        Self::update_panner(&source.panner, params);
        source.gain.gain().set_value(params.gain);
        if let Some(node) = source.current.as_ref() {
            node.playback_rate().set_value(params.pitch);
        }
        Ok(())
    }

    fn source_play(
        &mut self,
        source_id: u32,
        mode: AudioPlayMode,
        delay_ms: Option<u32>,
        intensity: f32,
    ) -> Result<(), String> {
        self.ensure_initialized()?;
        let context = self
            .context
            .as_ref()
            .ok_or_else(|| "AudioContext not initialized".to_string())?;
        let source = self
            .sources
            .get_mut(&source_id)
            .ok_or_else(|| format!("Audio source {source_id} not found"))?;
        let buffer = self
            .buffers
            .get(&source.audio_id)
            .ok_or_else(|| format!("Audio buffer {} not ready", source.audio_id))?;
        if let Some(node) = source.current.take() {
            let _ = node.stop();
        }
        let node = Self::build_source_node(
            context,
            buffer,
            source.params,
            mode,
            intensity,
            &source.panner,
            &source.gain,
        )?;
        if let Some(delay) = delay_ms {
            let when = context.current_time() + (delay as f64 / 1000.0);
            node.start_with_when(when)
                .map_err(|_| "Failed to start audio".to_string())?;
        } else {
            node.start().map_err(|_| "Failed to start audio".to_string())?;
        }
        source.current = Some(node);
        Ok(())
    }

    fn source_pause(&mut self, source_id: u32) -> Result<(), String> {
        let source = self
            .sources
            .get_mut(&source_id)
            .ok_or_else(|| format!("Audio source {source_id} not found"))?;
        if let Some(node) = source.current.take() {
            let _ = node.stop();
        }
        Ok(())
    }

    fn source_stop(&mut self, source_id: u32) -> Result<(), String> {
        self.source_pause(source_id)
    }

    fn buffer_dispose(&mut self, audio_id: u32) -> Result<(), String> {
        self.buffers.remove(&audio_id);
        self.pending.remove(&audio_id);
        Ok(())
    }

    fn source_dispose(&mut self, source_id: u32) -> Result<(), String> {
        if let Some(mut source) = self.sources.remove(&source_id) {
            if let Some(node) = source.current.take() {
                let _ = node.stop();
            }
        }
        Ok(())
    }

    fn drain_events(&mut self) -> Vec<AudioReadyEvent> {
        let mut events = Vec::new();
        while let Ok(result) = self.receiver.try_recv() {
            self.pending.remove(&result.audio_id);
            match result.buffer {
                Ok(buffer) => {
                    self.buffers.insert(result.audio_id, buffer);
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
