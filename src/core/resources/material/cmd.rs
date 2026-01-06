use glam::Vec4;
use serde::{Deserialize, Serialize};

use crate::core::resources::{
    MaterialStandardParams, MaterialStandardRecord, SurfaceType, STANDARD_INPUTS_PER_MATERIAL,
    STANDARD_INVALID_SLOT, STANDARD_TEXTURE_SLOTS, MATERIAL_FALLBACK_ID,
};
use crate::core::state::EngineState;

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum MaterialKind {
    Standard,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
#[serde(rename_all = "camelCase")]
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
    pub flags: u32,
    pub toon_params: Option<Vec4>,
}

impl Default for StandardOptions {
    fn default() -> Self {
        Self {
            base_color: Vec4::ONE,
            surface_type: SurfaceType::Opaque,
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
            flags: 0,
            toon_params: None,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(tag = "type", content = "content", rename_all = "camelCase")]
pub enum MaterialOptions {
    Standard(StandardOptions),
}

// MARK: - Create Material

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdMaterialCreateArgs {
    pub window_id: u32,
    pub material_id: u32,
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

pub fn engine_cmd_material_create(
    engine: &mut EngineState,
    args: &CmdMaterialCreateArgs,
) -> CmdResultMaterialCreate {
    let window_state = match engine.window.states.get_mut(&args.window_id) {
        Some(ws) => ws,
        None => {
            return CmdResultMaterialCreate {
                success: false,
                message: format!("Window {} not found", args.window_id),
            };
        }
    };

    if window_state
        .render_state
        .scene
        .materials_standard
        .contains_key(&args.material_id)
    {
        return CmdResultMaterialCreate {
            success: false,
            message: format!("Material with id {} already exists", args.material_id),
        };
    }

    if args.kind != MaterialKind::Standard {
        return CmdResultMaterialCreate {
            success: false,
            message: "Unsupported material kind".into(),
        };
    }

    let opts = match &args.options {
        Some(MaterialOptions::Standard(opts)) => opts.clone(),
        None => StandardOptions::default(),
    };
    if let Some(message) = validate_texture_ids(&window_state.render_state.scene, &opts) {
        return CmdResultMaterialCreate {
            success: false,
            message,
        };
    }

    let mut record = MaterialStandardRecord::new(MaterialStandardParams::default());
    pack_standard_material(args.material_id, &opts, &mut record);
    record.bind_group = None;
    window_state
        .render_state
        .scene
        .materials_standard
        .insert(args.material_id, record);

    window_state.is_dirty = true;

    CmdResultMaterialCreate {
        success: true,
        message: "Material created successfully".into(),
    }
}

// MARK: - Update Material

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdMaterialUpdateArgs {
    pub window_id: u32,
    pub material_id: u32,
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

pub fn engine_cmd_material_update(
    engine: &mut EngineState,
    args: &CmdMaterialUpdateArgs,
) -> CmdResultMaterialUpdate {
    let window_state = match engine.window.states.get_mut(&args.window_id) {
        Some(ws) => ws,
        None => {
            return CmdResultMaterialUpdate {
                success: false,
                message: format!("Window {} not found", args.window_id),
            };
        }
    };

    if !window_state
        .render_state
        .scene
        .materials_standard
        .contains_key(&args.material_id)
    {
        return CmdResultMaterialUpdate {
            success: false,
            message: format!("Material with id {} not found", args.material_id),
        };
    }

    if let Some(kind) = args.kind {
        if kind != MaterialKind::Standard {
            return CmdResultMaterialUpdate {
                success: false,
                message: "Unsupported material kind".into(),
            };
        }
    }

    let options = match &args.options {
        Some(MaterialOptions::Standard(opts)) => Some(opts.clone()),
        None => None,
    };

    if let Some(opts) = options {
        if let Some(message) = validate_texture_ids(&window_state.render_state.scene, &opts) {
            return CmdResultMaterialUpdate {
                success: false,
                message,
            };
        }
        if let Some(record) = window_state
            .render_state
            .scene
            .materials_standard
            .get_mut(&args.material_id)
        {
            pack_standard_material(args.material_id, &opts, record);
            record.bind_group = None;
            record.mark_dirty();
        }
    }

    window_state.is_dirty = true;

    CmdResultMaterialUpdate {
        success: true,
        message: "Material updated successfully".into(),
    }
}

fn pack_standard_material(
    material_id: u32,
    opts: &StandardOptions,
    record: &mut MaterialStandardRecord,
) {
    let inputs_offset = material_id.saturating_mul(STANDARD_INPUTS_PER_MATERIAL);

    record.data = MaterialStandardParams::default();
    record.data.inputs_offset_count =
        glam::UVec2::new(inputs_offset, STANDARD_INPUTS_PER_MATERIAL);
    let mut flags = opts.flags;
    if opts.spec_color.is_some() || opts.spec_power.is_some() {
        flags |= 1;
    }
    record.data.surface_flags = glam::UVec2::new(opts.surface_type as u32, flags);

    let mut texture_slots = [glam::UVec4::splat(STANDARD_INVALID_SLOT); 2];
    let mut sampler_indices = [glam::UVec4::ZERO; 2];
    record.texture_ids = [STANDARD_INVALID_SLOT; STANDARD_TEXTURE_SLOTS];

    let assign_slot = |slots: &mut [glam::UVec4; 2], index: usize, value: u32| {
        let vec_index = index / 4;
        let lane = index % 4;
        let mut vec = slots[vec_index];
        match lane {
            0 => vec.x = value,
            1 => vec.y = value,
            2 => vec.z = value,
            _ => vec.w = value,
        }
        slots[vec_index] = vec;
    };

    let assign_sampler = |samplers: &mut [glam::UVec4; 2], index: usize, value: u32| {
        let vec_index = index / 4;
        let lane = index % 4;
        let mut vec = samplers[vec_index];
        match lane {
            0 => vec.x = value,
            1 => vec.y = value,
            2 => vec.z = value,
            _ => vec.w = value,
        }
        samplers[vec_index] = vec;
    };

    if let Some(tex_id) = opts.base_tex_id {
        let slot = 0;
        if slot < STANDARD_TEXTURE_SLOTS {
            record.texture_ids[slot] = tex_id;
            assign_slot(&mut texture_slots, 0, slot as u32);
            assign_sampler(
                &mut sampler_indices,
                0,
                opts.base_sampler
                    .unwrap_or(MaterialSampler::LinearClamp) as u32,
            );
        }
    }
    if let Some(tex_id) = opts.spec_tex_id {
        let slot = 1;
        if slot < STANDARD_TEXTURE_SLOTS {
            record.texture_ids[slot] = tex_id;
            assign_slot(&mut texture_slots, 1, slot as u32);
            assign_sampler(
                &mut sampler_indices,
                1,
                opts.spec_sampler
                    .unwrap_or(MaterialSampler::LinearClamp) as u32,
            );
        }
    }
    if let Some(tex_id) = opts.normal_tex_id {
        let slot = 2;
        if slot < STANDARD_TEXTURE_SLOTS {
            record.texture_ids[slot] = tex_id;
            assign_slot(&mut texture_slots, 2, slot as u32);
            assign_sampler(
                &mut sampler_indices,
                2,
                opts.normal_sampler
                    .unwrap_or(MaterialSampler::LinearClamp) as u32,
            );
        }
    }
    if let Some(tex_id) = opts.toon_ramp_tex_id {
        let slot = 3;
        if slot < STANDARD_TEXTURE_SLOTS {
            record.texture_ids[slot] = tex_id;
            assign_slot(&mut texture_slots, 3, slot as u32);
            assign_sampler(
                &mut sampler_indices,
                3,
                opts.toon_ramp_sampler
                    .unwrap_or(MaterialSampler::LinearClamp) as u32,
            );
        }
    }

    record.data.texture_slots = texture_slots;
    record.data.sampler_indices = sampler_indices;

    record.surface_type = opts.surface_type;
    if record.inputs.len() != STANDARD_INPUTS_PER_MATERIAL as usize {
        record.inputs = vec![Vec4::ZERO; STANDARD_INPUTS_PER_MATERIAL as usize];
    }
    record.inputs[0] = opts.base_color;
    record.inputs[1] = opts.spec_color.unwrap_or(Vec4::ONE);
    record.inputs[2] = Vec4::new(opts.spec_power.unwrap_or(32.0), 0.0, 0.0, 0.0);
    if let Some(toon_params) = opts.toon_params {
        record.inputs[3] = toon_params;
    }
}

fn validate_texture_ids(
    scene: &crate::core::render::state::RenderScene,
    opts: &StandardOptions,
) -> Option<String> {
    let mut missing = Vec::new();
    let mut check = |label: &str, id: Option<u32>| {
        if let Some(tex_id) = id {
            if !scene.textures.contains_key(&tex_id) {
                missing.push(format!("{label}={tex_id}"));
            }
        }
    };

    check("base_tex_id", opts.base_tex_id);
    check("spec_tex_id", opts.spec_tex_id);
    check("normal_tex_id", opts.normal_tex_id);
    check("toon_ramp_tex_id", opts.toon_ramp_tex_id);

    if missing.is_empty() {
        None
    } else {
        Some(format!(
            "Texture id(s) not found for material: {}",
            missing.join(", ")
        ))
    }
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

pub fn engine_cmd_material_dispose(
    engine: &mut EngineState,
    args: &CmdMaterialDisposeArgs,
) -> CmdResultMaterialDispose {
    let window_state = match engine.window.states.get_mut(&args.window_id) {
        Some(ws) => ws,
        None => {
            return CmdResultMaterialDispose {
                success: false,
                message: format!("Window {} not found", args.window_id),
            };
        }
    };

    if args.material_id == MATERIAL_FALLBACK_ID {
        return CmdResultMaterialDispose {
            success: false,
            message: "Fallback material cannot be disposed".into(),
        };
    }

    if window_state
        .render_state
        .scene
        .materials_standard
        .remove(&args.material_id)
        .is_some()
    {
        window_state.is_dirty = true;
        CmdResultMaterialDispose {
            success: true,
            message: "Material disposed successfully".into(),
        }
    } else {
        CmdResultMaterialDispose {
            success: false,
            message: format!("Material with id {} not found", args.material_id),
        }
    }
}
