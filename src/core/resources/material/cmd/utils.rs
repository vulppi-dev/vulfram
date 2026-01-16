use super::types::{MaterialSampler, PbrOptions, StandardOptions};
use crate::core::resources::{
    MaterialPbrParams, MaterialPbrRecord, MaterialStandardParams, MaterialStandardRecord,
    PBR_INPUTS_PER_MATERIAL, PBR_INVALID_SLOT, PBR_TEXTURE_SLOTS, STANDARD_INPUTS_PER_MATERIAL,
    STANDARD_INVALID_SLOT, STANDARD_TEXTURE_SLOTS,
};
use glam::Vec4;

pub(crate) fn pack_standard_material(
    material_id: u32,
    opts: &StandardOptions,
    record: &mut MaterialStandardRecord,
) {
    let inputs_offset = material_id.saturating_mul(STANDARD_INPUTS_PER_MATERIAL);

    record.data = MaterialStandardParams::default();
    record.data.inputs_offset_count = glam::UVec2::new(inputs_offset, STANDARD_INPUTS_PER_MATERIAL);
    let mut flags = opts.flags;
    if opts.spec_color.is_some() || opts.spec_power.is_some() || opts.spec_tex_id.is_some() {
        flags |= 1;
    }
    record.data.surface_flags = glam::UVec2::new(opts.surface_type as u32, flags);

    let mut texture_slots = [glam::UVec4::splat(STANDARD_INVALID_SLOT); 2];
    let mut sampler_indices = [glam::UVec4::ZERO; 2];
    let mut tex_sources = [glam::UVec4::splat(2); 2];
    let atlas_layers = [glam::UVec4::ZERO; 2];
    let atlas_scale_bias = [glam::Vec4::new(1.0, 1.0, 0.0, 0.0); STANDARD_TEXTURE_SLOTS];
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
            assign_slot(&mut tex_sources, 0, 0);
            assign_sampler(
                &mut sampler_indices,
                0,
                opts.base_sampler.unwrap_or(MaterialSampler::LinearClamp) as u32,
            );
        }
    }
    if let Some(tex_id) = opts.spec_tex_id {
        let slot = 1;
        if slot < STANDARD_TEXTURE_SLOTS {
            record.texture_ids[slot] = tex_id;
            assign_slot(&mut texture_slots, 1, slot as u32);
            assign_slot(&mut tex_sources, 1, 0);
            assign_sampler(
                &mut sampler_indices,
                1,
                opts.spec_sampler.unwrap_or(MaterialSampler::LinearClamp) as u32,
            );
        }
    }
    if let Some(tex_id) = opts.normal_tex_id {
        let slot = 2;
        if slot < STANDARD_TEXTURE_SLOTS {
            record.texture_ids[slot] = tex_id;
            assign_slot(&mut texture_slots, 2, slot as u32);
            assign_slot(&mut tex_sources, 2, 0);
            assign_sampler(
                &mut sampler_indices,
                2,
                opts.normal_sampler.unwrap_or(MaterialSampler::LinearClamp) as u32,
            );
        }
    }
    if let Some(tex_id) = opts.toon_ramp_tex_id {
        let slot = 3;
        if slot < STANDARD_TEXTURE_SLOTS {
            record.texture_ids[slot] = tex_id;
            assign_slot(&mut texture_slots, 3, slot as u32);
            assign_slot(&mut tex_sources, 3, 0);
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
    record.data.tex_sources = tex_sources;
    record.data.atlas_layers = atlas_layers;
    record.data.atlas_scale_bias = atlas_scale_bias;

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

pub(crate) fn pack_pbr_material(
    material_id: u32,
    opts: &PbrOptions,
    record: &mut MaterialPbrRecord,
) {
    let inputs_offset = material_id.saturating_mul(PBR_INPUTS_PER_MATERIAL);

    record.data = MaterialPbrParams::default();
    record.data.inputs_offset_count = glam::UVec2::new(inputs_offset, PBR_INPUTS_PER_MATERIAL);
    record.data.surface_flags = glam::UVec2::new(opts.surface_type as u32, opts.flags);

    let mut texture_slots = [glam::UVec4::splat(PBR_INVALID_SLOT); 2];
    let mut sampler_indices = [glam::UVec4::ZERO; 2];
    let mut tex_sources = [glam::UVec4::splat(2); 2];
    let atlas_layers = [glam::UVec4::ZERO; 2];
    let atlas_scale_bias = [glam::Vec4::new(1.0, 1.0, 0.0, 0.0); PBR_TEXTURE_SLOTS];
    record.texture_ids = [PBR_INVALID_SLOT; PBR_TEXTURE_SLOTS];

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
        if slot < PBR_TEXTURE_SLOTS {
            record.texture_ids[slot] = tex_id;
            assign_slot(&mut texture_slots, 0, slot as u32);
            assign_slot(&mut tex_sources, 0, 0);
            assign_sampler(
                &mut sampler_indices,
                0,
                opts.base_sampler.unwrap_or(MaterialSampler::LinearClamp) as u32,
            );
        }
    }
    if let Some(tex_id) = opts.normal_tex_id {
        let slot = 1;
        if slot < PBR_TEXTURE_SLOTS {
            record.texture_ids[slot] = tex_id;
            assign_slot(&mut texture_slots, 1, slot as u32);
            assign_slot(&mut tex_sources, 1, 0);
            assign_sampler(
                &mut sampler_indices,
                1,
                opts.normal_sampler.unwrap_or(MaterialSampler::LinearClamp) as u32,
            );
        }
    }
    if let Some(tex_id) = opts.metallic_roughness_tex_id {
        let slot = 2;
        if slot < PBR_TEXTURE_SLOTS {
            record.texture_ids[slot] = tex_id;
            assign_slot(&mut texture_slots, 2, slot as u32);
            assign_slot(&mut tex_sources, 2, 0);
            assign_sampler(
                &mut sampler_indices,
                2,
                opts.metallic_roughness_sampler
                    .unwrap_or(MaterialSampler::LinearClamp) as u32,
            );
        }
    }
    if let Some(tex_id) = opts.emissive_tex_id {
        let slot = 3;
        if slot < PBR_TEXTURE_SLOTS {
            record.texture_ids[slot] = tex_id;
            assign_slot(&mut texture_slots, 3, slot as u32);
            assign_slot(&mut tex_sources, 3, 0);
            assign_sampler(
                &mut sampler_indices,
                3,
                opts.emissive_sampler
                    .unwrap_or(MaterialSampler::LinearClamp) as u32,
            );
        }
    }
    if let Some(tex_id) = opts.ao_tex_id {
        let slot = 4;
        if slot < PBR_TEXTURE_SLOTS {
            record.texture_ids[slot] = tex_id;
            assign_slot(&mut texture_slots, 4, slot as u32);
            assign_slot(&mut tex_sources, 4, 0);
            assign_sampler(
                &mut sampler_indices,
                4,
                opts.ao_sampler.unwrap_or(MaterialSampler::LinearClamp) as u32,
            );
        }
    }

    record.data.texture_slots = texture_slots;
    record.data.sampler_indices = sampler_indices;
    record.data.tex_sources = tex_sources;
    record.data.atlas_layers = atlas_layers;
    record.data.atlas_scale_bias = atlas_scale_bias;

    record.surface_type = opts.surface_type;
    if record.inputs.len() != PBR_INPUTS_PER_MATERIAL as usize {
        record.inputs = vec![Vec4::ZERO; PBR_INPUTS_PER_MATERIAL as usize];
    }
    record.inputs[0] = opts.base_color;
    record.inputs[1] = opts.emissive_color;
    record.inputs[2] = Vec4::new(opts.metallic, opts.roughness, opts.ao, 0.0);
    record.inputs[3] = Vec4::new(opts.normal_scale, 0.0, 0.0, 0.0);
}
