use std::collections::{HashMap, HashSet};
use wgpu;

use super::allocator::BufferAllocator;
use super::components::{CameraInstance, ComponentId, MeshInstance};
use super::resources::{MaterialId, ShaderId, ShaderResource};

// MARK: - Binding Key

/// Unique key identifying a binding combination
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct BindingKey {
    pub component_id: ComponentId,
    pub shader_id: ShaderId,
    pub resource_ids: Vec<u32>,
}

// BindingKey is constructed inline where needed

// MARK: - Binding Info

/// Information about buffer bindings and bind groups for a specific combination
#[derive(Debug)]
pub struct BindingInfo {
    /// Offset in the shader's group 0 buffer (global/camera)
    pub group_0_offset: Option<u64>,
    /// Offset in the shader's group 1 buffer (mesh)
    pub group_1_offset: Option<u64>,
    /// Offset in the shader's group 3 buffer (instance) - Reserved for future
    pub group_3_offset: Option<u64>,
    /// Offset in the shader's group 4 buffer (material custom)
    pub group_4_offset: Option<u64>,
    /// Cached bind group for group 0
    pub bind_group_0: Option<wgpu::BindGroup>,
    /// Cached bind group for group 1
    pub bind_group_1: Option<wgpu::BindGroup>,
    /// Cached bind group for group 2 (textures/samplers)
    pub bind_group_2: Option<wgpu::BindGroup>,
    /// Cached bind group for group 3 (instancing)
    pub bind_group_3: Option<wgpu::BindGroup>,
    /// Cached bind group for group 4 (material custom)
    pub bind_group_4: Option<wgpu::BindGroup>,
}

impl BindingInfo {
    pub fn new() -> Self {
        Self {
            group_0_offset: None,
            group_1_offset: None,
            group_3_offset: None,
            group_4_offset: None,
            bind_group_0: None,
            bind_group_1: None,
            bind_group_2: None,
            bind_group_3: None,
            bind_group_4: None,
        }
    }
}

// MARK: - Binding Manager

/// Manages all bindings between components, shaders, and resources
pub struct BindingManager {
    /// Map of binding key to binding info
    bindings: HashMap<BindingKey, BindingInfo>,
}

impl BindingManager {
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
        }
    }

    /// Get binding info for a key
    pub fn get(&self, key: &BindingKey) -> Option<&BindingInfo> {
        self.bindings.get(key)
    }

    // Removed unused methods: get_mut, insert, contains_key

    /// Remove all bindings related to a shader
    pub fn remove_shader_bindings(&mut self, shader_id: ShaderId) {
        self.bindings.retain(|key, _| key.shader_id != shader_id);
    }

    /// Remove all bindings related to a component
    pub fn remove_component_bindings(&mut self, component_id: ComponentId) {
        self.bindings
            .retain(|key, _| key.component_id != component_id);
    }

    /// Remove all bindings related to a material (via resource_ids)
    pub fn remove_material_bindings(&mut self, material_id: MaterialId) {
        self.bindings
            .retain(|key, _| !key.resource_ids.contains(&material_id));
    }

    /// Remove all bindings related to a geometry (via resource_ids)
    pub fn remove_geometry_bindings(&mut self, geometry_id: u32) {
        self.bindings
            .retain(|key, _| !key.resource_ids.contains(&geometry_id));
    }

    /// Get all shader IDs that have bindings with a component
    pub fn get_shaders_for_component(&self, component_id: ComponentId) -> Vec<ShaderId> {
        self.bindings
            .keys()
            .filter(|key| key.component_id == component_id)
            .map(|key| key.shader_id)
            .collect::<HashSet<_>>()
            .into_iter()
            .collect()
    }

    /// Clear all bindings
    pub fn clear(&mut self) {
        self.bindings.clear();
    }

    /// Invalidate bind group 2 for specific materials
    /// This forces recreation of texture/sampler bind groups on next render
    pub fn invalidate_bind_group_2_for_materials(
        &mut self,
        material_ids: &[super::resources::MaterialId],
    ) {
        for (key, binding_info) in self.bindings.iter_mut() {
            // Check if any of the material_ids is in the resource_ids
            let has_material = material_ids
                .iter()
                .any(|mid| key.resource_ids.contains(mid));
            if has_material {
                binding_info.bind_group_2 = None;
            }
        }
    }

    /// Get or update binding info (lazy creation/update) - FLEXIBLE VERSION
    ///
    /// This is the core lazy update function that:
    /// 1. Checks if binding exists in cache
    /// 2. Checks if component is dirty
    /// 3. Allocates buffer space based on shader's uniform_layouts
    /// 4. Writes data using automatic uniform injection (camera_view, model_transform, etc.)
    /// 5. Writes material custom uniforms
    /// 6. Creates bind groups if needed
    /// 7. Returns binding info
    pub fn get_or_update(
        &mut self,
        key: &BindingKey,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        shader: &mut ShaderResource,
        material_opt: Option<&super::resources::MaterialResource>,
        textures: &std::collections::HashMap<
            super::resources::TextureId,
            super::resources::TextureResource,
        >,
        samplers: &std::collections::HashMap<
            super::resources::SamplerId,
            super::resources::SamplerResource,
        >,
        time: f32,
        delta_time: f32,
        camera_opt: Option<&CameraInstance>,
        model_opt: Option<&MeshInstance>,
        material_uniforms: Option<&std::collections::HashMap<String, super::buffers::UniformValue>>,
    ) -> Result<&BindingInfo, String> {
        // Check if binding exists
        let exists = self.bindings.contains_key(key);

        // Check if dirty
        let is_dirty = camera_opt.map(|c| c.is_dirty).unwrap_or(false)
            || model_opt.map(|m| m.is_dirty).unwrap_or(false);

        // If exists and not dirty, return cached
        if exists && !is_dirty {
            return Ok(self.bindings.get(key).unwrap());
        }

        // Need to allocate/update
        let mut binding_info = if exists {
            self.bindings.remove(key).unwrap()
        } else {
            BindingInfo::new()
        };

        // Process each uniform buffer layout defined in the shader
        for layout in &shader.uniform_layouts {
            let group = layout.group;

            // Determine which offset/bind_group fields to use
            let (offset_field, bind_group_field) = match group {
                super::buffers::GROUP_GLOBAL => (
                    &mut binding_info.group_0_offset,
                    &mut binding_info.bind_group_0,
                ),
                super::buffers::GROUP_MESH => (
                    &mut binding_info.group_1_offset,
                    &mut binding_info.bind_group_1,
                ),
                super::buffers::GROUP_INSTANCE => (
                    &mut binding_info.group_3_offset,
                    &mut binding_info.bind_group_3,
                ),
                super::buffers::GROUP_MATERIAL => (
                    &mut binding_info.group_4_offset,
                    &mut binding_info.bind_group_4,
                ),
                _ => continue, // Skip unsupported groups
            };

            let needs_allocation = offset_field.is_none();

            // Allocate buffer space if needed
            if needs_allocation {
                let (offset, needs_recreation) = shader.uniform_buffers.allocate(
                    device,
                    group,
                    key.component_id,
                    layout.total_size as u64,
                    shader.shader_id,
                )?;
                *offset_field = Some(offset);

                // If buffer was recreated, invalidate bind group to force recreation
                if needs_recreation {
                    *bind_group_field = None;
                }
            }

            // Prepare buffer data with automatic uniform injection
            let mut buffer_data = vec![0u8; layout.total_size as usize];

            // Inject automatic uniforms based on group and reserved names
            match group {
                super::buffers::GROUP_GLOBAL => {
                    // Group 0: Global/Camera data
                    if let Some(camera) = camera_opt {
                        let view_proj = camera.proj_mat * camera.view_mat;
                        let camera_pos = camera.view_mat.inverse().w_axis.truncate();

                        layout.inject_automatic_uniforms(
                            &mut buffer_data,
                            Some(time),
                            Some(delta_time),
                            Some(&camera.view_mat),
                            Some(&camera.proj_mat),
                            Some(&view_proj),
                            Some(&camera_pos),
                            None,
                            None,
                        );
                    }
                }
                super::buffers::GROUP_MESH => {
                    // Group 1: Mesh data
                    if let Some(model) = model_opt {
                        let normal_mat =
                            glam::Mat3::from_mat4(model.model_mat).inverse().transpose();

                        layout.inject_automatic_uniforms(
                            &mut buffer_data,
                            None,
                            None,
                            None,
                            None,
                            None,
                            None,
                            Some(&model.model_mat),
                            Some(&normal_mat),
                        );
                    }
                }
                super::buffers::GROUP_MATERIAL => {
                    // Group 3: Material custom uniforms
                    if let Some(uniforms) = material_uniforms {
                        if let Err(e) = pack_custom_uniforms(layout, &mut buffer_data, uniforms) {
                            log::warn!(
                                "Failed to pack material uniforms for group {}: {}",
                                group,
                                e
                            );
                        }
                    }
                }
                _ => {}
            }

            // Write buffer data to GPU
            if let Some(buffer) = shader.uniform_buffers.get_buffer(group) {
                let offset = offset_field.unwrap();
                queue.write_buffer(buffer, offset, &buffer_data);
            }

            // Create bind group if needed (first time or buffer recreated)
            // With dynamic offsets, we create ONE bind group per shader group (not per component)
            if bind_group_field.is_none() {
                // With the new architecture, bind_group_layouts is dense and group index = layout index
                // If we have groups 0 and 2, bind_group_layouts has 3 elements (0, 1 empty, 2)
                let layout_idx = group as usize;

                if let Some(buffer) = shader.uniform_buffers.get_buffer(group) {
                    if layout_idx < shader.bind_group_layouts.len() {
                        *bind_group_field =
                            Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
                                label: Some(&format!("Bind Group {}", group)),
                                layout: &shader.bind_group_layouts[layout_idx],
                                entries: &[wgpu::BindGroupEntry {
                                    binding: layout.binding,
                                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                                        buffer,
                                        offset: 0, // Dynamic offset - will be provided at draw time
                                        size: None, // Whole buffer
                                    }),
                                }],
                            }));
                    }
                }
            }
        }

        // PHASE 8B-8E: Create bind group 2 for textures/samplers (if shader has them)
        // Check if shader has group 2 bindings (textures or samplers)
        let has_group_2 =
            !shader.texture_bindings.is_empty() || !shader.sampler_bindings.is_empty();

        if has_group_2 && binding_info.bind_group_2.is_none() {
            if let Some(material) = material_opt {
                let mut entries = Vec::new();

                // PHASE 8C: Create entries for textures
                for texture_binding in shader
                    .texture_bindings
                    .iter()
                    .filter(|t| t.group == super::buffers::GROUP_TEXTURES)
                {
                    // Find corresponding texture in material by binding index
                    // We assume material.textures order matches shader texture_bindings order
                    let texture_idx = shader
                        .texture_bindings
                        .iter()
                        .filter(|t| t.group == super::buffers::GROUP_TEXTURES)
                        .position(|t| t.binding == texture_binding.binding);

                    if let Some(idx) = texture_idx {
                        if idx < material.textures.len() {
                            let texture_id = material.textures[idx];
                            if let Some(texture_resource) = textures.get(&texture_id) {
                                entries.push(wgpu::BindGroupEntry {
                                    binding: texture_binding.binding,
                                    resource: wgpu::BindingResource::TextureView(
                                        &texture_resource.view,
                                    ),
                                });
                            } else {
                                log::warn!("Texture {} not found in resources", texture_id);
                            }
                        } else {
                            log::warn!(
                                "Material texture index {} out of bounds (has {} textures)",
                                idx,
                                material.textures.len()
                            );
                        }
                    }
                }

                // PHASE 8D: Create entries for samplers
                for sampler_binding in shader
                    .sampler_bindings
                    .iter()
                    .filter(|s| s.group == super::buffers::GROUP_TEXTURES)
                {
                    // Find corresponding sampler in material by binding index
                    // We assume material.samplers order matches shader sampler_bindings order
                    let sampler_idx = shader
                        .sampler_bindings
                        .iter()
                        .filter(|s| s.group == super::buffers::GROUP_TEXTURES)
                        .position(|s| s.binding == sampler_binding.binding);

                    if let Some(idx) = sampler_idx {
                        if idx < material.samplers.len() {
                            let sampler_id = material.samplers[idx];
                            if let Some(sampler_resource) = samplers.get(&sampler_id) {
                                entries.push(wgpu::BindGroupEntry {
                                    binding: sampler_binding.binding,
                                    resource: wgpu::BindingResource::Sampler(
                                        &sampler_resource.sampler,
                                    ),
                                });
                            } else {
                                log::warn!("Sampler {} not found in resources", sampler_id);
                            }
                        } else {
                            log::warn!(
                                "Material sampler index {} out of bounds (has {} samplers)",
                                idx,
                                material.samplers.len()
                            );
                        }
                    }
                }

                // PHASE 8E: Create bind group 2 if we have entries
                if !entries.is_empty() {
                    let layout_idx = super::buffers::GROUP_TEXTURES as usize;
                    if layout_idx < shader.bind_group_layouts.len() {
                        binding_info.bind_group_2 =
                            Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
                                label: Some("Bind Group 2 (Textures/Samplers)"),
                                layout: &shader.bind_group_layouts[layout_idx],
                                entries: &entries,
                            }));
                    }
                }
            }
        }

        // Insert back into cache
        self.bindings.insert(key.clone(), binding_info);

        Ok(self.bindings.get(key).unwrap())
    }
}

impl Default for BindingManager {
    fn default() -> Self {
        Self::new()
    }
}

// MARK: - Helper Functions

/// Pack custom uniform values into buffer following layout
fn pack_custom_uniforms(
    layout: &super::buffers::UniformBufferLayout,
    buffer_data: &mut [u8],
    values: &std::collections::HashMap<String, super::buffers::UniformValue>,
) -> Result<(), String> {
    for field in &layout.fields {
        if let Some(value) = values.get(&field.name) {
            // Check type match
            if !value.matches_type(field.field_type) {
                return Err(format!(
                    "Type mismatch for '{}': expected {:?}",
                    field.name, field.field_type
                ));
            }

            // Get bytes and copy to buffer
            let bytes = value.as_bytes();
            let offset = field.offset as usize;
            let end = offset + bytes.len();

            if end > buffer_data.len() {
                return Err(format!(
                    "Buffer overflow for field '{}': offset {} + size {} > buffer size {}",
                    field.name,
                    offset,
                    bytes.len(),
                    buffer_data.len()
                ));
            }

            buffer_data[offset..end].copy_from_slice(&bytes);
        }
    }
    Ok(())
}

// MARK: - Shader Uniform Buffers

/// Shader-owned uniform buffers with allocators per group
pub struct ShaderUniformBuffers {
    /// Buffer compartilhado para uniforms do grupo 0 (camera/global)
    pub group_0_buffer: Option<wgpu::Buffer>,
    pub group_0_allocator: BufferAllocator,

    /// Buffer compartilhado para uniforms do grupo 1 (mesh)
    pub group_1_buffer: Option<wgpu::Buffer>,
    pub group_1_allocator: BufferAllocator,

    /// Buffer compartilhado para uniforms do grupo 4 (material custom)
    pub group_4_buffer: Option<wgpu::Buffer>,
    pub group_4_allocator: BufferAllocator,
}

impl ShaderUniformBuffers {
    /// Create new uniform buffers (buffers are created on-demand)
    pub fn new() -> Self {
        Self {
            group_0_buffer: None,
            group_0_allocator: BufferAllocator::new(4096), // 4KB inicial
            group_1_buffer: None,
            group_1_allocator: BufferAllocator::new(16384), // 16KB inicial
            group_4_buffer: None,
            group_4_allocator: BufferAllocator::new(8192), // 8KB inicial
        }
    }

    /// Ensure buffer exists for a group
    fn ensure_buffer(&mut self, device: &wgpu::Device, group: u32, label: &str) {
        let (buffer_opt, allocator) = match group {
            0 => (&mut self.group_0_buffer, &self.group_0_allocator),
            1 => (&mut self.group_1_buffer, &self.group_1_allocator),
            4 => (&mut self.group_4_buffer, &self.group_4_allocator),
            _ => return,
        };

        if buffer_opt.is_none() {
            *buffer_opt = Some(device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(label),
                size: allocator.capacity(),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }));
        }
    }

    /// Allocate space in the appropriate group buffer
    /// Returns (offset, needs_recreation)
    pub fn allocate(
        &mut self,
        device: &wgpu::Device,
        group: u32,
        id: u32,
        size: u64,
        shader_id: ShaderId,
    ) -> Result<(u64, bool), String> {
        // Ensure buffer exists
        self.ensure_buffer(
            device,
            group,
            &format!("Shader {} Group {} Uniform Buffer", shader_id, group),
        );

        let allocator = match group {
            0 => &mut self.group_0_allocator,
            1 => &mut self.group_1_allocator,
            4 => &mut self.group_4_allocator,
            _ => return Err(format!("Invalid group {}", group)),
        };

        let (offset, needs_recreation) = allocator.allocate(id, size)?;

        // Recreate buffer if needed (grew beyond capacity)
        if needs_recreation {
            let buffer_opt = match group {
                0 => &mut self.group_0_buffer,
                1 => &mut self.group_1_buffer,
                4 => &mut self.group_4_buffer,
                _ => return Err(format!("Invalid group {}", group)),
            };

            // Create new larger buffer
            *buffer_opt = Some(device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(&format!(
                    "Shader {} Group {} Uniform Buffer",
                    shader_id, group
                )),
                size: allocator.capacity(),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }));

            // Signal that all bind groups for this shader must be recreated
            // This is handled by returning needs_recreation flag to caller
        }

        Ok((offset, needs_recreation))
    }

    /// Deallocate space in the appropriate group buffer
    pub fn deallocate(&mut self, group: u32, id: u32) {
        match group {
            0 => self.group_0_allocator.deallocate(id),
            1 => self.group_1_allocator.deallocate(id),
            4 => self.group_4_allocator.deallocate(id),
            _ => {}
        }
    }

    /// Get buffer for a group
    pub fn get_buffer(&self, group: u32) -> Option<&wgpu::Buffer> {
        match group {
            0 => self.group_0_buffer.as_ref(),
            1 => self.group_1_buffer.as_ref(),
            4 => self.group_4_buffer.as_ref(),
            _ => None,
        }
    }
}

impl Default for ShaderUniformBuffers {
    fn default() -> Self {
        Self::new()
    }
}
