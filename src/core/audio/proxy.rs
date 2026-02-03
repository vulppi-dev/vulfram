use glam::{Quat, Vec3};

#[derive(Debug, Clone, Copy, Default)]
#[allow(dead_code)]
pub struct AudioListenerState {
    pub position: Vec3,
    pub velocity: Vec3,
    pub forward: Vec3,
    pub up: Vec3,
}

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub struct AudioSpatialParams {
    pub min_distance: f32,
    pub max_distance: f32,
    pub rolloff: f32,
    pub cone_inner: f32,
    pub cone_outer: f32,
    pub cone_outer_gain: f32,
}

impl Default for AudioSpatialParams {
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

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub struct AudioSourceParams {
    pub position: Vec3,
    pub velocity: Vec3,
    pub orientation: Quat,
    pub gain: f32,
    pub pitch: f32,
    pub spatial: AudioSpatialParams,
}

impl Default for AudioSourceParams {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            velocity: Vec3::ZERO,
            orientation: Quat::IDENTITY,
            gain: 1.0,
            pitch: 1.0,
            spatial: AudioSpatialParams::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioPlayMode {
    Once,
    Loop,
}

#[derive(Debug, Clone, Copy)]
pub struct AudioListenerBinding {
    pub window_id: u32,
    pub model_id: u32,
}

#[derive(Debug, Clone)]
pub struct AudioReadyEvent {
    pub resource_id: u32,
    pub success: bool,
    pub message: String,
}

pub trait AudioProxy: Send {
    fn init(&mut self) -> Result<(), String>;
    fn listener_update(&mut self, state: AudioListenerState) -> Result<(), String>;

    fn buffer_create_from_bytes(&mut self, resource_id: u32, bytes: Vec<u8>) -> Result<(), String>;

    fn source_create(&mut self, source_id: u32, params: AudioSourceParams) -> Result<(), String>;

    fn source_update(&mut self, source_id: u32, params: AudioSourceParams) -> Result<(), String>;
    fn source_play(
        &mut self,
        source_id: u32,
        resource_id: u32,
        timeline_id: u32,
        mode: AudioPlayMode,
        delay_ms: Option<u32>,
        intensity: f32,
    ) -> Result<(), String>;
    fn source_pause(&mut self, source_id: u32, timeline_id: Option<u32>) -> Result<(), String>;
    fn source_stop(&mut self, source_id: u32, timeline_id: Option<u32>) -> Result<(), String>;

    fn buffer_dispose(&mut self, resource_id: u32) -> Result<(), String>;
    fn source_dispose(&mut self, source_id: u32) -> Result<(), String>;

    fn drain_events(&mut self) -> Vec<AudioReadyEvent>;
}
