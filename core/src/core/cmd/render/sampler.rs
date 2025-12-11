use serde::{Deserialize, Serialize};

use crate::core::render::enums::{
    AddressMode, BorderColor, CompareFunction, FilterMode, MipmapFilterMode,
};
use crate::core::render::resources::{SamplerId, SamplerResource};
use crate::core::state::EngineState;

// MARK: - Create Sampler

/// Arguments for creating a sampler resource
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdSamplerCreateArgs {
    pub sampler_id: SamplerId,
    pub window_id: u32,
    pub address_mode_u: AddressMode,
    pub address_mode_v: AddressMode,
    pub address_mode_w: AddressMode,
    pub mag_filter: FilterMode,
    pub min_filter: FilterMode,
    pub mipmap_filter: MipmapFilterMode,
    pub lod_min_clamp: f32,
    pub lod_max_clamp: f32,
    pub compare: Option<CompareFunction>,
    pub anisotropy_clamp: u16,
    pub border_color: Option<BorderColor>,
    pub label: Option<String>,
}

impl Default for CmdSamplerCreateArgs {
    fn default() -> Self {
        Self {
            sampler_id: 0,
            window_id: 0,
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            mipmap_filter: MipmapFilterMode::Linear,
            lod_min_clamp: 0.0,
            lod_max_clamp: 32.0,
            compare: None,
            anisotropy_clamp: 1,
            border_color: None,
            label: None,
        }
    }
}

/// Result for sampler creation command
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultSamplerCreate {
    pub success: bool,
    pub message: String,
}

impl Default for CmdResultSamplerCreate {
    fn default() -> Self {
        Self {
            success: false,
            message: String::new(),
        }
    }
}

/// Create a new sampler resource
pub fn engine_cmd_sampler_create(
    engine: &mut EngineState,
    args: &CmdSamplerCreateArgs,
) -> CmdResultSamplerCreate {
    // Validate window exists
    let window_state = match engine.windows.get_mut(&args.window_id) {
        Some(ws) => ws,
        None => {
            return CmdResultSamplerCreate {
                success: false,
                message: format!("Window with id {} not found", args.window_id),
            };
        }
    };

    let render_state = &mut window_state.render_state;

    // Check if sampler already exists
    if render_state
        .resources
        .samplers
        .contains_key(&args.sampler_id)
    {
        return CmdResultSamplerCreate {
            success: false,
            message: format!("Sampler with id {} already exists", args.sampler_id),
        };
    }

    // Get device
    let device = match &engine.device {
        Some(d) => d,
        None => {
            return CmdResultSamplerCreate {
                success: false,
                message: "GPU device not initialized".into(),
            };
        }
    };

    // Convert parameters
    let address_mode_u = args.address_mode_u.to_wgpu();
    let address_mode_v = args.address_mode_v.to_wgpu();
    let address_mode_w = args.address_mode_w.to_wgpu();
    let mag_filter = args.mag_filter.to_wgpu();
    let min_filter = args.min_filter.to_wgpu();
    let mipmap_filter = args.mipmap_filter.to_wgpu();
    let compare = args.compare.map(|c| c.to_wgpu());
    let border_color = args.border_color.and_then(|bc| bc.to_wgpu());

    // Create sampler descriptor
    let sampler_desc = wgpu::SamplerDescriptor {
        label: args.label.as_deref(),
        address_mode_u,
        address_mode_v,
        address_mode_w,
        mag_filter,
        min_filter,
        mipmap_filter,
        lod_min_clamp: args.lod_min_clamp,
        lod_max_clamp: args.lod_max_clamp,
        compare,
        anisotropy_clamp: args.anisotropy_clamp,
        border_color,
    };

    // Create sampler
    let sampler = device.create_sampler(&sampler_desc);

    // Create sampler resource
    let sampler_resource = SamplerResource { sampler };

    // Insert sampler into resources
    render_state
        .resources
        .samplers
        .insert(args.sampler_id, sampler_resource);

    CmdResultSamplerCreate {
        success: true,
        message: format!("Sampler {} created successfully", args.sampler_id),
    }
}

// MARK: - Update Sampler

/// Arguments for updating a sampler resource
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdSamplerUpdateArgs {
    pub sampler_id: SamplerId,
    pub window_id: u32,
    pub address_mode_u: AddressMode,
    pub address_mode_v: AddressMode,
    pub address_mode_w: AddressMode,
    pub mag_filter: FilterMode,
    pub min_filter: FilterMode,
    pub mipmap_filter: MipmapFilterMode,
    pub lod_min_clamp: f32,
    pub lod_max_clamp: f32,
    pub compare: Option<CompareFunction>,
    pub anisotropy_clamp: u16,
    pub border_color: Option<BorderColor>,
    pub label: Option<String>,
}

impl Default for CmdSamplerUpdateArgs {
    fn default() -> Self {
        Self {
            sampler_id: 0,
            window_id: 0,
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            mipmap_filter: MipmapFilterMode::Linear,
            lod_min_clamp: 0.0,
            lod_max_clamp: 32.0,
            compare: None,
            anisotropy_clamp: 1,
            border_color: None,
            label: None,
        }
    }
}

/// Result for sampler update command
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultSamplerUpdate {
    pub success: bool,
    pub message: String,
}

impl Default for CmdResultSamplerUpdate {
    fn default() -> Self {
        Self {
            success: false,
            message: String::new(),
        }
    }
}

/// Update an existing sampler resource (recreates the sampler with new parameters)
pub fn engine_cmd_sampler_update(
    engine: &mut EngineState,
    args: &CmdSamplerUpdateArgs,
) -> CmdResultSamplerUpdate {
    // Validate window exists
    let window_state = match engine.windows.get_mut(&args.window_id) {
        Some(ws) => ws,
        None => {
            return CmdResultSamplerUpdate {
                success: false,
                message: format!("Window with id {} not found", args.window_id),
            };
        }
    };

    let render_state = &mut window_state.render_state;

    // Check if sampler exists
    if !render_state
        .resources
        .samplers
        .contains_key(&args.sampler_id)
    {
        return CmdResultSamplerUpdate {
            success: false,
            message: format!("Sampler with id {} not found", args.sampler_id),
        };
    }

    // Get device
    let device = match &engine.device {
        Some(d) => d,
        None => {
            return CmdResultSamplerUpdate {
                success: false,
                message: "GPU device not initialized".into(),
            };
        }
    };

    // Convert parameters
    let address_mode_u = args.address_mode_u.to_wgpu();
    let address_mode_v = args.address_mode_v.to_wgpu();
    let address_mode_w = args.address_mode_w.to_wgpu();
    let mag_filter = args.mag_filter.to_wgpu();
    let min_filter = args.min_filter.to_wgpu();
    let mipmap_filter = args.mipmap_filter.to_wgpu();
    let compare = args.compare.map(|c| c.to_wgpu());
    let border_color = args.border_color.and_then(|bc| bc.to_wgpu());

    // Create new sampler descriptor
    let sampler_desc = wgpu::SamplerDescriptor {
        label: args.label.as_deref(),
        address_mode_u,
        address_mode_v,
        address_mode_w,
        mag_filter,
        min_filter,
        mipmap_filter,
        lod_min_clamp: args.lod_min_clamp,
        lod_max_clamp: args.lod_max_clamp,
        compare,
        anisotropy_clamp: args.anisotropy_clamp,
        border_color,
    };

    // Create new sampler
    let sampler = device.create_sampler(&sampler_desc);

    // Update sampler resource
    let sampler_resource = SamplerResource { sampler };

    render_state
        .resources
        .samplers
        .insert(args.sampler_id, sampler_resource);

    CmdResultSamplerUpdate {
        success: true,
        message: format!("Sampler {} updated successfully", args.sampler_id),
    }
}

// MARK: - Dispose Sampler

/// Arguments for disposing a sampler resource
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdSamplerDisposeArgs {
    pub sampler_id: SamplerId,
    pub window_id: u32,
}

impl Default for CmdSamplerDisposeArgs {
    fn default() -> Self {
        Self {
            sampler_id: 0,
            window_id: 0,
        }
    }
}

/// Result for sampler disposal command
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultSamplerDispose {
    pub success: bool,
    pub message: String,
}

impl Default for CmdResultSamplerDispose {
    fn default() -> Self {
        Self {
            success: false,
            message: String::new(),
        }
    }
}

/// Dispose an existing sampler resource
pub fn engine_cmd_sampler_dispose(
    engine: &mut EngineState,
    args: &CmdSamplerDisposeArgs,
) -> CmdResultSamplerDispose {
    // Validate window exists
    let window_state = match engine.windows.get_mut(&args.window_id) {
        Some(ws) => ws,
        None => {
            return CmdResultSamplerDispose {
                success: false,
                message: format!("Window with id {} not found", args.window_id),
            };
        }
    };

    let render_state = &mut window_state.render_state;

    // Prevent disposal of fallback sampler
    if args.sampler_id == crate::core::render::resources::FALLBACK_SAMPLER_ID {
        return CmdResultSamplerDispose {
            success: false,
            message: "Cannot dispose fallback sampler".into(),
        };
    }

    // Replace sampler with fallback in all materials using it
    let materials_affected: Vec<_> = render_state
        .resources
        .materials
        .iter()
        .filter(|(_, m)| m.samplers.contains(&args.sampler_id))
        .map(|(id, _)| *id)
        .collect();

    for material_id in &materials_affected {
        if let Some(material) = render_state.resources.materials.get_mut(material_id) {
            for sampler_id in material.samplers.iter_mut() {
                if *sampler_id == args.sampler_id {
                    *sampler_id = render_state.fallback_sampler_id;
                }
            }
        }
    }

    // Invalidate bind group 2 for affected materials
    if !materials_affected.is_empty() {
        render_state
            .binding_manager
            .invalidate_bind_group_2_for_materials(&materials_affected);
    }

    // Remove sampler
    match render_state.resources.samplers.remove(&args.sampler_id) {
        Some(_) => CmdResultSamplerDispose {
            success: true,
            message: if materials_affected.is_empty() {
                format!("Sampler {} disposed successfully", args.sampler_id)
            } else {
                format!(
                    "Sampler {} disposed ({} materials switched to fallback)",
                    args.sampler_id,
                    materials_affected.len()
                )
            },
        },
        None => CmdResultSamplerDispose {
            success: false,
            message: format!("Sampler with id {} not found", args.sampler_id),
        },
    }
}
