use super::super::RenderState;
use crate::core::resources::{
    MaterialPbrParams, MaterialStandardParams, PBR_INVALID_SLOT, PBR_TEXTURE_SLOTS,
    STANDARD_INVALID_SLOT, STANDARD_TEXTURE_SLOTS,
};

impl RenderState {
    pub(crate) fn prepare_materials(&mut self, device: &wgpu::Device) {
        let bindings = self.bindings.as_mut().unwrap();
        let library = self.library.as_ref().unwrap();

        for (id, record) in &mut self.scene.materials_standard {
            let mut atlas_changed = false;
            let set_uvec4_lane = |vecs: &mut [glam::UVec4; 2], index: usize, value: u32| {
                let vec_index = index / 4;
                let lane = index % 4;
                let mut v = vecs[vec_index];
                match lane {
                    0 => v.x = value,
                    1 => v.y = value,
                    2 => v.z = value,
                    _ => v.w = value,
                }
                vecs[vec_index] = v;
            };

            for slot in 0..STANDARD_TEXTURE_SLOTS {
                let tex_id = record.texture_ids[slot];
                let mut desired_source = 2u32;
                let mut desired_layer = 0u32;
                let mut desired_scale_bias = glam::Vec4::new(1.0, 1.0, 0.0, 0.0);

                if tex_id != STANDARD_INVALID_SLOT {
                    if let Some(entry) = self.scene.forward_atlas_entries.get(&tex_id) {
                        desired_source = 1;
                        desired_layer = entry.layer;
                        desired_scale_bias = entry.uv_scale_bias;
                    } else {
                        desired_source = 0;
                    }
                }

                let current_source = match (slot / 4, slot % 4) {
                    (0, 0) => record.data.tex_sources[0].x,
                    (0, 1) => record.data.tex_sources[0].y,
                    (0, 2) => record.data.tex_sources[0].z,
                    (0, 3) => record.data.tex_sources[0].w,
                    (1, 0) => record.data.tex_sources[1].x,
                    (1, 1) => record.data.tex_sources[1].y,
                    (1, 2) => record.data.tex_sources[1].z,
                    _ => record.data.tex_sources[1].w,
                };
                let current_layer = match (slot / 4, slot % 4) {
                    (0, 0) => record.data.atlas_layers[0].x,
                    (0, 1) => record.data.atlas_layers[0].y,
                    (0, 2) => record.data.atlas_layers[0].z,
                    (0, 3) => record.data.atlas_layers[0].w,
                    (1, 0) => record.data.atlas_layers[1].x,
                    (1, 1) => record.data.atlas_layers[1].y,
                    (1, 2) => record.data.atlas_layers[1].z,
                    _ => record.data.atlas_layers[1].w,
                };
                let current_scale_bias = record.data.atlas_scale_bias[slot];

                if current_source != desired_source {
                    set_uvec4_lane(&mut record.data.tex_sources, slot, desired_source);
                    atlas_changed = true;
                }
                if current_layer != desired_layer {
                    set_uvec4_lane(&mut record.data.atlas_layers, slot, desired_layer);
                    atlas_changed = true;
                }
                if current_scale_bias != desired_scale_bias {
                    record.data.atlas_scale_bias[slot] = desired_scale_bias;
                    atlas_changed = true;
                }
            }

            if record.is_dirty || atlas_changed {
                bindings.material_standard_pool.write(*id, &record.data);
                if record.is_dirty {
                    bindings
                        .material_standard_inputs
                        .write_slice(record.data.inputs_offset_count.x, &record.inputs);
                    record.clear_dirty();
                }
            }

            // Update Bind Group
            if record.bind_group.is_none() {
                let mut entries = Vec::new();
                entries.push(wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: bindings.instance_pool.buffer(),
                        offset: 0,
                        size: None,
                    }),
                });
                entries.push(wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: bindings.material_standard_pool.buffer(),
                        offset: 0,
                        size: Some(
                            std::num::NonZeroU64::new(
                                std::mem::size_of::<MaterialStandardParams>() as u64,
                            )
                            .unwrap(),
                        ),
                    }),
                });
                entries.push(wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: bindings.material_standard_inputs.buffer(),
                        offset: 0,
                        size: None,
                    }),
                });

                for slot in 0..STANDARD_TEXTURE_SLOTS {
                    let tex_id = record.texture_ids[slot];
                    let view = if tex_id != STANDARD_INVALID_SLOT {
                        self.scene
                            .textures
                            .get(&tex_id)
                            .map(|t| &t.view)
                            .unwrap_or(&library.fallback_view)
                    } else {
                        &library.fallback_view
                    };
                    entries.push(wgpu::BindGroupEntry {
                        binding: (3 + slot) as u32,
                        resource: wgpu::BindingResource::TextureView(view),
                    });
                }

                record.bind_group = Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("BindGroup Material Standard"),
                    layout: &library.layout_object_standard,
                    entries: &entries,
                }));
            }
        }

        for (id, record) in &mut self.scene.materials_pbr {
            let mut atlas_changed = false;
            let set_uvec4_lane = |vecs: &mut [glam::UVec4; 2], index: usize, value: u32| {
                let vec_index = index / 4;
                let lane = index % 4;
                let mut v = vecs[vec_index];
                match lane {
                    0 => v.x = value,
                    1 => v.y = value,
                    2 => v.z = value,
                    _ => v.w = value,
                }
                vecs[vec_index] = v;
            };

            for slot in 0..PBR_TEXTURE_SLOTS {
                let tex_id = record.texture_ids[slot];
                let mut desired_source = 2u32;
                let mut desired_layer = 0u32;
                let mut desired_scale_bias = glam::Vec4::new(1.0, 1.0, 0.0, 0.0);

                if tex_id != PBR_INVALID_SLOT {
                    if let Some(entry) = self.scene.forward_atlas_entries.get(&tex_id) {
                        desired_source = 1;
                        desired_layer = entry.layer;
                        desired_scale_bias = entry.uv_scale_bias;
                    } else {
                        desired_source = 0;
                    }
                }

                let current_source = match (slot / 4, slot % 4) {
                    (0, 0) => record.data.tex_sources[0].x,
                    (0, 1) => record.data.tex_sources[0].y,
                    (0, 2) => record.data.tex_sources[0].z,
                    (0, 3) => record.data.tex_sources[0].w,
                    (1, 0) => record.data.tex_sources[1].x,
                    (1, 1) => record.data.tex_sources[1].y,
                    (1, 2) => record.data.tex_sources[1].z,
                    _ => record.data.tex_sources[1].w,
                };
                let current_layer = match (slot / 4, slot % 4) {
                    (0, 0) => record.data.atlas_layers[0].x,
                    (0, 1) => record.data.atlas_layers[0].y,
                    (0, 2) => record.data.atlas_layers[0].z,
                    (0, 3) => record.data.atlas_layers[0].w,
                    (1, 0) => record.data.atlas_layers[1].x,
                    (1, 1) => record.data.atlas_layers[1].y,
                    (1, 2) => record.data.atlas_layers[1].z,
                    _ => record.data.atlas_layers[1].w,
                };
                let current_scale_bias = record.data.atlas_scale_bias[slot];

                if current_source != desired_source {
                    set_uvec4_lane(&mut record.data.tex_sources, slot, desired_source);
                    atlas_changed = true;
                }
                if current_layer != desired_layer {
                    set_uvec4_lane(&mut record.data.atlas_layers, slot, desired_layer);
                    atlas_changed = true;
                }
                if current_scale_bias != desired_scale_bias {
                    record.data.atlas_scale_bias[slot] = desired_scale_bias;
                    atlas_changed = true;
                }
            }

            if record.is_dirty || atlas_changed {
                bindings.material_pbr_pool.write(*id, &record.data);
                if record.is_dirty {
                    bindings
                        .material_pbr_inputs
                        .write_slice(record.data.inputs_offset_count.x, &record.inputs);
                    record.clear_dirty();
                }
            }

            // Update Bind Group
            if record.bind_group.is_none() {
                let mut entries = Vec::new();
                entries.push(wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: bindings.instance_pool.buffer(),
                        offset: 0,
                        size: None,
                    }),
                });
                entries.push(wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: bindings.material_pbr_pool.buffer(),
                        offset: 0,
                        size: Some(
                            std::num::NonZeroU64::new(
                                std::mem::size_of::<MaterialPbrParams>() as u64
                            )
                            .unwrap(),
                        ),
                    }),
                });
                entries.push(wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: bindings.material_pbr_inputs.buffer(),
                        offset: 0,
                        size: None,
                    }),
                });

                for slot in 0..PBR_TEXTURE_SLOTS {
                    let tex_id = record.texture_ids[slot];
                    let view = if tex_id != PBR_INVALID_SLOT {
                        self.scene
                            .textures
                            .get(&tex_id)
                            .map(|t| &t.view)
                            .unwrap_or(&library.fallback_view)
                    } else {
                        &library.fallback_view
                    };
                    entries.push(wgpu::BindGroupEntry {
                        binding: (3 + slot) as u32,
                        resource: wgpu::BindingResource::TextureView(view),
                    });
                }

                record.bind_group = Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("BindGroup Material PBR"),
                    layout: &library.layout_object_pbr,
                    entries: &entries,
                }));
            }
        }
    }
}
