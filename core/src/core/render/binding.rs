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
    /// Offset in the shader's group 0 buffer (camera/global)
    pub group_0_offset: Option<u64>,
    /// Offset in the shader's group 1 buffer (material)
    pub group_1_offset: Option<u64>,
    /// Offset in the shader's group 2 buffer (instance/model)
    pub group_2_offset: Option<u64>,
    /// Cached bind group for group 0
    pub bind_group_0: Option<wgpu::BindGroup>,
    /// Cached bind group for group 1
    pub bind_group_1: Option<wgpu::BindGroup>,
    /// Cached bind group for group 2
    pub bind_group_2: Option<wgpu::BindGroup>,
}

impl BindingInfo {
    pub fn new() -> Self {
        Self {
            group_0_offset: None,
            group_1_offset: None,
            group_2_offset: None,
            bind_group_0: None,
            bind_group_1: None,
            bind_group_2: None,
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
                0 => (
                    &mut binding_info.group_0_offset,
                    &mut binding_info.bind_group_0,
                ),
                1 => (
                    &mut binding_info.group_1_offset,
                    &mut binding_info.bind_group_1,
                ),
                2 => (
                    &mut binding_info.group_2_offset,
                    &mut binding_info.bind_group_2,
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

            // Inject automatic uniforms based on reserved names
            if let Some(camera) = camera_opt {
                // Calculate view_projection if needed
                let view_proj = camera.proj_mat * camera.view_mat;

                // Extract camera position from inverse of view matrix
                let camera_pos = camera.view_mat.inverse().w_axis.truncate();

                layout.inject_automatic_uniforms(
                    &mut buffer_data,
                    Some(time),
                    Some(delta_time),
                    Some(&camera.view_mat), // camera_view
                    Some(&camera.proj_mat), // camera_projection
                    Some(&view_proj),       // camera_view_projection
                    Some(&camera_pos),      // camera_position
                    None,
                    None,
                );
            }

            if let Some(model) = model_opt {
                // Calculate normal matrix if needed (inverse transpose of model matrix)
                let normal_mat = glam::Mat3::from_mat4(model.model_mat).inverse().transpose();

                layout.inject_automatic_uniforms(
                    &mut buffer_data,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    Some(&model.model_mat), // model_transform
                    Some(&normal_mat),      // model_normal
                );
            }

            // Pack material custom uniforms if provided (typically for group 1)
            if let Some(uniforms) = material_uniforms {
                if let Err(e) = pack_custom_uniforms(layout, &mut buffer_data, uniforms) {
                    log::warn!(
                        "Failed to pack material uniforms for group {}: {}",
                        group,
                        e
                    );
                }
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

    /// Buffer compartilhado para uniforms do grupo 1 (material)
    pub group_1_buffer: Option<wgpu::Buffer>,
    pub group_1_allocator: BufferAllocator,

    /// Buffer compartilhado para uniforms do grupo 2 (instance/model)
    pub group_2_buffer: Option<wgpu::Buffer>,
    pub group_2_allocator: BufferAllocator,
}

impl ShaderUniformBuffers {
    /// Create new uniform buffers (buffers are created on-demand)
    pub fn new() -> Self {
        Self {
            group_0_buffer: None,
            group_0_allocator: BufferAllocator::new(4096), // 4KB inicial
            group_1_buffer: None,
            group_1_allocator: BufferAllocator::new(16384), // 16KB inicial
            group_2_buffer: None,
            group_2_allocator: BufferAllocator::new(8192), // 8KB inicial
        }
    }

    /// Ensure buffer exists for a group
    fn ensure_buffer(&mut self, device: &wgpu::Device, group: u32, label: &str) {
        let (buffer_opt, allocator) = match group {
            0 => (&mut self.group_0_buffer, &self.group_0_allocator),
            1 => (&mut self.group_1_buffer, &self.group_1_allocator),
            2 => (&mut self.group_2_buffer, &self.group_2_allocator),
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
            2 => &mut self.group_2_allocator,
            _ => return Err(format!("Invalid group {}", group)),
        };

        let (offset, needs_recreation) = allocator.allocate(id, size)?;

        // Recreate buffer if needed (grew beyond capacity)
        if needs_recreation {
            let buffer_opt = match group {
                0 => &mut self.group_0_buffer,
                1 => &mut self.group_1_buffer,
                2 => &mut self.group_2_buffer,
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
            2 => self.group_2_allocator.deallocate(id),
            _ => {}
        }
    }

    /// Get buffer for a group
    pub fn get_buffer(&self, group: u32) -> Option<&wgpu::Buffer> {
        match group {
            0 => self.group_0_buffer.as_ref(),
            1 => self.group_1_buffer.as_ref(),
            2 => self.group_2_buffer.as_ref(),
            _ => None,
        }
    }
}

impl Default for ShaderUniformBuffers {
    fn default() -> Self {
        Self::new()
    }
}
