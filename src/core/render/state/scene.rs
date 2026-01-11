use crate::core::resources::{
    CameraRecord, ForwardAtlasEntry, LightRecord, MaterialPbrRecord, MaterialStandardRecord,
    ModelRecord, TextureRecord,
};
use std::collections::HashMap;

/// Holds the actual scene data to be rendered
#[derive(Default)]
pub struct RenderScene {
    pub cameras: HashMap<u32, CameraRecord>,
    pub models: HashMap<u32, ModelRecord>,
    pub lights: HashMap<u32, LightRecord>,
    pub materials_standard: HashMap<u32, MaterialStandardRecord>,
    pub materials_pbr: HashMap<u32, MaterialPbrRecord>,
    pub textures: HashMap<u32, TextureRecord>,
    pub forward_atlas_entries: HashMap<u32, ForwardAtlasEntry>,
}
