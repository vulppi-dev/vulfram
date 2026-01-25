use glam::Vec3;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum SkyboxMode {
    None,
    Procedural,
    Cubemap,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MsaaConfig {
    pub enabled: bool,
    pub sample_count: u32,
}

impl Default for MsaaConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            sample_count: 1,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SkyboxConfig {
    pub mode: SkyboxMode,
    pub intensity: f32,
    pub rotation: f32,
    pub tint: Vec3,
    #[serde(default)]
    pub cubemap_texture_id: Option<u32>,
}

impl Default for SkyboxConfig {
    fn default() -> Self {
        Self {
            mode: SkyboxMode::None,
            intensity: 1.0,
            rotation: 0.0,
            tint: Vec3::ONE,
            cubemap_texture_id: None,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvironmentConfig {
    pub msaa: MsaaConfig,
    pub skybox: SkyboxConfig,
}

impl Default for EnvironmentConfig {
    fn default() -> Self {
        Self {
            msaa: MsaaConfig::default(),
            skybox: SkyboxConfig::default(),
        }
    }
}
