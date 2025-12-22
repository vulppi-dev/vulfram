use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Quat, UVec2, Vec2, Vec3, Vec4};
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use smallvec::SmallVec;

use crate::core::resources::ArenaSlice;

#[derive(Debug, Clone)]
pub struct GeometryPrimitiveSlice {
    pub vertex_count: u32,
    pub index_count: u32,
    pub indices: ArenaSlice,
    pub position: ArenaSlice,
    pub normal: Option<ArenaSlice>,
    pub tangent: Option<ArenaSlice>,
    pub colors: SmallVec<[ArenaSlice; 4]>,
    pub uvs: SmallVec<[ArenaSlice; 8]>,
    pub skins: SmallVec<[SkinSet; 4]>,
}

#[derive(Debug, Clone)]
pub struct SkinSet {
    pub joints: ArenaSlice,
    pub weights: ArenaSlice,
}
