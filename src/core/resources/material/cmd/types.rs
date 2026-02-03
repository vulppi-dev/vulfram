use crate::core::resources::SurfaceType;
use glam::Vec4;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum MaterialKind {
    Standard,
    Pbr,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
#[serde(rename_all = "kebab-case")]
#[repr(u32)]
pub enum MaterialSampler {
    PointClamp = 0,
    LinearClamp = 1,
    PointRepeat = 2,
    LinearRepeat = 3,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StandardOptions {
    pub base_color: Vec4,
    pub surface_type: SurfaceType,
    #[serde(default)]
    pub emissive_color: Vec4,
    pub spec_color: Option<Vec4>,
    pub spec_power: Option<f32>,
    pub base_tex_id: Option<u32>,
    pub base_sampler: Option<MaterialSampler>,
    pub spec_tex_id: Option<u32>,
    pub spec_sampler: Option<MaterialSampler>,
    pub normal_tex_id: Option<u32>,
    pub normal_sampler: Option<MaterialSampler>,
    pub toon_ramp_tex_id: Option<u32>,
    pub toon_ramp_sampler: Option<MaterialSampler>,
    pub emissive_tex_id: Option<u32>,
    pub emissive_sampler: Option<MaterialSampler>,
    pub flags: u32,
    pub toon_params: Option<Vec4>,
}

impl Default for StandardOptions {
    fn default() -> Self {
        Self {
            base_color: Vec4::ONE,
            surface_type: SurfaceType::Opaque,
            emissive_color: Vec4::ZERO,
            spec_color: None,
            spec_power: None,
            base_tex_id: None,
            base_sampler: None,
            spec_tex_id: None,
            spec_sampler: None,
            normal_tex_id: None,
            normal_sampler: None,
            toon_ramp_tex_id: None,
            toon_ramp_sampler: None,
            emissive_tex_id: None,
            emissive_sampler: None,
            flags: 0,
            toon_params: None,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PbrOptions {
    pub base_color: Vec4,
    pub surface_type: SurfaceType,
    pub emissive_color: Vec4,
    pub metallic: f32,
    pub roughness: f32,
    pub ao: f32,
    pub normal_scale: f32,
    pub base_tex_id: Option<u32>,
    pub base_sampler: Option<MaterialSampler>,
    pub normal_tex_id: Option<u32>,
    pub normal_sampler: Option<MaterialSampler>,
    pub metallic_roughness_tex_id: Option<u32>,
    pub metallic_roughness_sampler: Option<MaterialSampler>,
    pub emissive_tex_id: Option<u32>,
    pub emissive_sampler: Option<MaterialSampler>,
    pub ao_tex_id: Option<u32>,
    pub ao_sampler: Option<MaterialSampler>,
    pub flags: u32,
}

impl Default for PbrOptions {
    fn default() -> Self {
        Self {
            base_color: Vec4::ONE,
            surface_type: SurfaceType::Opaque,
            emissive_color: Vec4::ZERO,
            metallic: 0.0,
            roughness: 0.5,
            ao: 1.0,
            normal_scale: 1.0,
            base_tex_id: None,
            base_sampler: None,
            normal_tex_id: None,
            normal_sampler: None,
            metallic_roughness_tex_id: None,
            metallic_roughness_sampler: None,
            emissive_tex_id: None,
            emissive_sampler: None,
            ao_tex_id: None,
            ao_sampler: None,
            flags: 0,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(tag = "type", content = "content", rename_all = "kebab-case")]
pub enum MaterialOptions {
    Standard(StandardOptions),
    Pbr(PbrOptions),
}

// MARK: - Create Material

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdMaterialCreateArgs {
    pub window_id: u32,
    pub material_id: u32,
    pub label: Option<String>,
    pub kind: MaterialKind,
    #[serde(default)]
    pub options: Option<MaterialOptions>,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultMaterialCreate {
    pub success: bool,
    pub message: String,
}

// MARK: - Update Material

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdMaterialUpdateArgs {
    pub window_id: u32,
    pub material_id: u32,
    pub label: Option<String>,
    pub kind: Option<MaterialKind>,
    #[serde(default)]
    pub options: Option<MaterialOptions>,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultMaterialUpdate {
    pub success: bool,
    pub message: String,
}

// MARK: - Dispose Material

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdMaterialDisposeArgs {
    pub window_id: u32,
    pub material_id: u32,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultMaterialDispose {
    pub success: bool,
    pub message: String,
}
