use glam::Vec4;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdTextureCreateFromBufferArgs {
    pub window_id: u32,
    pub texture_id: u32,
    pub label: Option<String>,
    pub buffer_id: u64,
    #[serde(default)]
    pub srgb: Option<bool>,
    #[serde(default)]
    pub mode: TextureCreateMode,
    #[serde(default)]
    pub atlas_options: Option<ForwardAtlasOptions>,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultTextureCreateFromBuffer {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ForwardAtlasOptions {
    pub tile_px: u32,
    pub layers: u32,
}

impl Default for ForwardAtlasOptions {
    fn default() -> Self {
        Self {
            tile_px: 256,
            layers: 1,
        }
    }
}

#[derive(Debug, Deserialize_repr, Serialize_repr, Clone, Copy)]
#[repr(u32)]
pub enum TextureCreateMode {
    Standalone = 0,
    ForwardAtlas = 1,
}

impl Default for TextureCreateMode {
    fn default() -> Self {
        TextureCreateMode::Standalone
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdTextureCreateSolidColorArgs {
    pub window_id: u32,
    pub texture_id: u32,
    pub label: Option<String>,
    pub color: Vec4,
    #[serde(default)]
    pub srgb: Option<bool>,
    #[serde(default)]
    pub mode: TextureCreateMode,
    #[serde(default)]
    pub atlas_options: Option<ForwardAtlasOptions>,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultTextureCreateSolidColor {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdTextureDisposeArgs {
    pub window_id: u32,
    pub texture_id: u32,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultTextureDispose {
    pub success: bool,
    pub message: String,
}
