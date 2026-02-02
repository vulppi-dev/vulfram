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
    pub post: PostProcessConfig,
}

impl Default for EnvironmentConfig {
    fn default() -> Self {
        Self {
            msaa: MsaaConfig::default(),
            skybox: SkyboxConfig::default(),
            post: PostProcessConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PostProcessConfig {
    pub enabled: bool,
    pub exposure: f32,
    pub gamma: f32,
    pub saturation: f32,
    pub contrast: f32,
    pub vignette: f32,
    pub grain: f32,
    pub chromatic_aberration: f32,
    pub blur: f32,
    pub sharpen: f32,
    pub outline_enabled: bool,
    pub outline_strength: f32,
    pub outline_threshold: f32,
    pub outline_width: f32,
    pub outline_quality: f32,
    pub posterize_steps: f32,
    pub cell_shading: bool,
    pub ssao_enabled: bool,
    pub ssao_strength: f32,
    pub ssao_radius: f32,
    pub ssao_bias: f32,
    pub ssao_power: f32,
    pub ssao_blur_radius: f32,
    pub ssao_blur_depth_threshold: f32,
}

impl Default for PostProcessConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            exposure: 1.0,
            gamma: 2.2,
            saturation: 1.0,
            contrast: 1.0,
            vignette: 0.0,
            grain: 0.0,
            chromatic_aberration: 0.0,
            blur: 0.0,
            sharpen: 0.0,
            outline_enabled: false,
            outline_strength: 0.0,
            outline_threshold: 0.2,
            outline_width: 1.0,
            outline_quality: 1.0,
            posterize_steps: 0.0,
            cell_shading: false,
            ssao_enabled: false,
            ssao_strength: 1.0,
            ssao_radius: 0.75,
            ssao_bias: 0.025,
            ssao_power: 1.5,
            ssao_blur_radius: 2.0,
            ssao_blur_depth_threshold: 0.02,
        }
    }
}
