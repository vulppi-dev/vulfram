use serde_repr::{Deserialize_repr, Serialize_repr};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize_repr, Serialize_repr)]
#[repr(u32)]
pub enum UploadType {
    Raw = 0,
    ShaderSource,
    GeometryData,
    VertexData,
    IndexData,
    ImageData,
    BinaryAsset,
}

impl UploadType {
    pub fn from_u32(value: u32) -> Option<Self> {
        match value {
            0 => Some(UploadType::Raw),
            1 => Some(UploadType::ShaderSource),
            2 => Some(UploadType::GeometryData),
            3 => Some(UploadType::VertexData),
            4 => Some(UploadType::IndexData),
            5 => Some(UploadType::ImageData),
            6 => Some(UploadType::BinaryAsset),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct UploadBuffer {
    pub upload_type: UploadType,
    pub data: Vec<u8>,
}

#[derive(Debug, Default)]
pub struct BufferStorage {
    pub uploads: HashMap<u64, UploadBuffer>,
}

impl BufferStorage {
    pub fn new() -> Self {
        Self {
            uploads: HashMap::new(),
        }
    }

    pub fn insert_upload(&mut self, id: u64, buffer: UploadBuffer) -> bool {
        self.uploads.insert(id, buffer).is_none()
    }

    pub fn remove_upload(&mut self, id: u64) -> Option<UploadBuffer> {
        self.uploads.remove(&id)
    }
}
