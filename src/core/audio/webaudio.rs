use std::collections::{HashMap, HashSet};
use std::sync::mpsc::{Receiver, Sender, channel};

use glam::Vec3;
use js_sys::{Function, Reflect, Uint8Array};
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::spawn_local;
use web_sys::{
    AudioBuffer, AudioBufferSourceNode, AudioContext, GainNode, PannerNode, PannerOptions,
};

use crate::core::audio::{
    AudioListenerState, AudioPlayMode, AudioProxy, AudioReadyEvent, AudioSourceParams,
};

struct DecodeResult {
    resource_id: u32,
    buffer: Result<AudioBuffer, String>,
}

struct WebAudioLayer {
    resource_id: u32,
    intensity: f32,
    mode: AudioPlayMode,
    paused: bool,
    cursor: f64,
    duration: f64,
    rate: f32,
    start_time: f64,
    node: Option<AudioBufferSourceNode>,
    gain: Option<GainNode>,
}

struct WebAudioSource {
    params: AudioSourceParams,
    panner: PannerNode,
    layers: HashMap<u32, WebAudioLayer>,
}

pub struct WebAudioProxy {
    context: Option<AudioContext>,
    buffers: HashMap<u32, AudioBuffer>,
    sources: HashMap<u32, WebAudioSource>,
    pending: HashSet<u32>,
    canceled: HashSet<u32>,
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
            canceled: HashSet::new(),
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
        let context =
            AudioContext::new().map_err(|_| "Failed to create AudioContext".to_string())?;
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
    ) -> Result<(AudioBufferSourceNode, GainNode), String> {
        let source = context
            .create_buffer_source()
            .map_err(|_| "Failed to create AudioBufferSourceNode".to_string())?;
        let gain = context
            .create_gain()
            .map_err(|_| "Failed to create GainNode".to_string())?;
        source.set_buffer(Some(buffer));
        let loop_enabled = matches!(mode, AudioPlayMode::Loop);
        source.set_loop(loop_enabled);
        let rate = params.pitch;
        source.playback_rate().set_value(rate);
        source
            .connect_with_audio_node(&gain)
            .map_err(|_| "Failed to connect gain".to_string())?;
        gain.connect_with_audio_node(panner)
            .map_err(|_| "Failed to connect panner".to_string())?;
        gain.gain()
            .set_value(params.gain * intensity.clamp(0.0, 1.0));
        Ok((source, gain))
    }

    fn start_node_with_offset(
        node: &AudioBufferSourceNode,
        when: f64,
        offset: f64,
    ) -> Result<(), String> {
        let func = Reflect::get(node, &JsValue::from_str("start"))
            .map_err(|_| "Failed to access start".to_string())?;
        let func = func
            .dyn_into::<Function>()
            .map_err(|_| "start is not a function".to_string())?;
        if offset > 0.0 {
            func.call2(
                &JsValue::from(node),
                &JsValue::from_f64(when),
                &JsValue::from_f64(offset),
            )
            .map_err(|_| "Failed to start audio".to_string())?;
        } else {
            func.call1(&JsValue::from(node), &JsValue::from_f64(when))
                .map_err(|_| "Failed to start audio".to_string())?;
        }
        Ok(())
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

    fn buffer_create_from_bytes(&mut self, resource_id: u32, bytes: Vec<u8>) -> Result<(), String> {
        self.ensure_initialized()?;
        if self.pending.contains(&resource_id) {
            return Err(format!("Audio {resource_id} already pending"));
        }
        self.canceled.remove(&resource_id);
        let context = self
            .context
            .as_ref()
            .ok_or_else(|| "AudioContext not initialized".to_string())?
            .clone();
        let sender = self.sender.clone();
        let buffer = Uint8Array::from(bytes.as_slice()).buffer();
        self.pending.insert(resource_id);
        spawn_local(async move {
            let promise = context.decode_audio_data(&buffer);
            let result = wasm_bindgen_futures::JsFuture::from(promise).await;
            let decode = match result {
                Ok(audio) => audio
                    .dyn_into::<AudioBuffer>()
                    .map_err(|_| "Failed to decode audio".to_string()),
                Err(_) => Err("Failed to decode audio".to_string()),
            };
            let _ = sender.send(DecodeResult {
                resource_id,
                buffer: decode,
            });
        });
        Ok(())
    }

    fn source_create(&mut self, source_id: u32, params: AudioSourceParams) -> Result<(), String> {
        self.ensure_initialized()?;
        let context = self
            .context
            .as_ref()
            .ok_or_else(|| "AudioContext not initialized".to_string())?;
        let options = PannerOptions::new();
        let panner = context
            .create_panner_with_options(&options)
            .map_err(|_| "Failed to create PannerNode".to_string())?;
        Self::update_panner(&panner, params);
        panner
            .connect_with_audio_node(&context.destination())
            .map_err(|_| "Failed to connect destination".to_string())?;
        self.sources.insert(
            source_id,
            WebAudioSource {
                params,
                panner,
                layers: HashMap::new(),
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
        for layer in source.layers.values_mut() {
            let intensity = layer.intensity.clamp(0.0, 1.0);
            if let Some(gain) = layer.gain.as_ref() {
                gain.gain().set_value(params.gain * intensity);
            }
            layer.rate = params.pitch;
            if let Some(node) = layer.node.as_ref() {
                node.playback_rate().set_value(params.pitch);
            }
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
            .get(&resource_id)
            .ok_or_else(|| format!("Audio buffer {} not ready", resource_id))?;
        let duration = buffer.duration();
        let mut cursor = 0.0;
        let rate = source.params.pitch;
        if let Some(mut layer) = source.layers.remove(&timeline_id) {
            if !layer.paused {
                if let Some(node) = layer.node.take() {
                    let _ = node.stop();
                }
            }
            cursor = layer.cursor;
            rate = layer.rate;
        }
        if cursor >= duration {
            cursor = 0.0;
        }
        let (node, gain) = Self::build_source_node(
            context,
            buffer,
            source.params,
            mode,
            intensity,
            &source.panner,
        )?;
        let when =
            context.current_time() + delay_ms.map(|delay| delay as f64 / 1000.0).unwrap_or(0.0);
        Self::start_node_with_offset(&node, when, cursor)?;
        let start_time = when;
        source.layers.insert(
            timeline_id,
            WebAudioLayer {
                resource_id,
                intensity,
                mode,
                paused: false,
                cursor,
                duration,
                rate,
                start_time,
                node: Some(node),
                gain: Some(gain),
            },
        );
        Ok(())
    }

    fn source_pause(&mut self, source_id: u32, timeline_id: Option<u32>) -> Result<(), String> {
        let context = match self.context.as_ref() {
            Some(context) => context,
            None => return Ok(()),
        };
        let source = self
            .sources
            .get_mut(&source_id)
            .ok_or_else(|| format!("Audio source {source_id} not found"))?;
        if let Some(timeline_id) = timeline_id {
            if let Some(layer) = source.layers.get_mut(&timeline_id) {
                if let Some(node) = layer.node.take() {
                    if !layer.paused {
                        let elapsed = context.current_time() - layer.start_time;
                        let delta = (elapsed * layer.rate.abs() as f64).max(0.0);
                        layer.cursor = (layer.cursor + delta).min(layer.duration);
                        if matches!(layer.mode, AudioPlayMode::Loop) && layer.duration > 0.0 {
                            layer.cursor %= layer.duration;
                        }
                    }
                    let _ = node.stop();
                }
                if !layer.paused {
                    layer.paused = true;
                }
            }
            return Ok(());
        }
        for layer in source.layers.values_mut() {
            if let Some(node) = layer.node.take() {
                if !layer.paused {
                    let elapsed = context.current_time() - layer.start_time;
                    let delta = (elapsed * layer.rate.abs() as f64).max(0.0);
                    layer.cursor = (layer.cursor + delta).min(layer.duration);
                    if matches!(layer.mode, AudioPlayMode::Loop) && layer.duration > 0.0 {
                        layer.cursor %= layer.duration;
                    }
                }
                let _ = node.stop();
            }
            if !layer.paused {
                layer.paused = true;
            }
        }
        Ok(())
    }

    fn source_stop(&mut self, source_id: u32, timeline_id: Option<u32>) -> Result<(), String> {
        let source = self
            .sources
            .get_mut(&source_id)
            .ok_or_else(|| format!("Audio source {source_id} not found"))?;
        if let Some(timeline_id) = timeline_id {
            if let Some(layer) = source.layers.remove(&timeline_id) {
                if let Some(node) = layer.node {
                    let _ = node.stop();
                }
            }
            return Ok(());
        }
        for (_timeline, layer) in source.layers.drain() {
            if let Some(node) = layer.node {
                let _ = node.stop();
            }
        }
        Ok(())
    }

    fn buffer_dispose(&mut self, resource_id: u32) -> Result<(), String> {
        self.buffers.remove(&resource_id);
        self.pending.remove(&resource_id);
        self.canceled.insert(resource_id);
        for source in self.sources.values_mut() {
            let mut to_stop = Vec::new();
            for (timeline_id, layer) in source.layers.iter() {
                if layer.resource_id == resource_id {
                    to_stop.push(*timeline_id);
                }
            }
            for timeline_id in to_stop {
                if let Some(layer) = source.layers.remove(&timeline_id) {
                    if let Some(node) = layer.node {
                        let _ = node.stop();
                    }
                }
            }
        }
        Ok(())
    }

    fn source_dispose(&mut self, source_id: u32) -> Result<(), String> {
        if let Some(mut source) = self.sources.remove(&source_id) {
            for (_timeline, layer) in source.layers.drain() {
                if let Some(node) = layer.node {
                    let _ = node.stop();
                }
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
            match result.buffer {
                Ok(buffer) => {
                    self.buffers.insert(result.resource_id, buffer);
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
