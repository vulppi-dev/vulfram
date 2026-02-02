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
    pub filter_enabled: bool,
    pub filter_exposure: f32,
    pub filter_gamma: f32,
    pub filter_saturation: f32,
    pub filter_contrast: f32,
    pub filter_vignette: f32,
    pub filter_grain: f32,
    pub filter_chromatic_aberration: f32,
    pub filter_blur: f32,
    pub filter_sharpen: f32,
    pub filter_tonemap_mode: u32,
    pub outline_enabled: bool,
    pub outline_strength: f32,
    pub outline_threshold: f32,
    pub outline_width: f32,
    pub outline_quality: f32,
    pub filter_posterize_steps: f32,
    pub cell_shading: bool,
    pub ssao_enabled: bool,
    pub ssao_strength: f32,
    pub ssao_radius: f32,
    pub ssao_bias: f32,
    pub ssao_power: f32,
    pub ssao_blur_radius: f32,
    pub ssao_blur_depth_threshold: f32,
    pub bloom_enabled: bool,
    pub bloom_threshold: f32,
    pub bloom_knee: f32,
    pub bloom_intensity: f32,
    pub bloom_scatter: f32,
}

impl Default for PostProcessConfig {
    fn default() -> Self {
        Self {
            filter_enabled: true,
            filter_exposure: 1.0,
            filter_gamma: 2.2,
            filter_saturation: 1.0,
            filter_contrast: 1.0,
            filter_vignette: 0.0,
            filter_grain: 0.0,
            filter_chromatic_aberration: 0.0,
            filter_blur: 0.0,
            filter_sharpen: 0.0,
            filter_tonemap_mode: 1,
            outline_enabled: false,
            outline_strength: 0.0,
            outline_threshold: 0.2,
            outline_width: 1.0,
            outline_quality: 1.0,
            filter_posterize_steps: 0.0,
            cell_shading: false,
            ssao_enabled: false,
            ssao_strength: 1.0,
            ssao_radius: 0.75,
            ssao_bias: 0.025,
            ssao_power: 1.5,
            ssao_blur_radius: 2.0,
            ssao_blur_depth_threshold: 0.02,
            bloom_enabled: false,
            bloom_threshold: 1.0,
            bloom_knee: 0.5,
            bloom_intensity: 0.8,
            bloom_scatter: 0.7,
        }
    }
}
