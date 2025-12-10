use glam::Mat4;
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

impl BindingKey {
    pub fn new(component_id: ComponentId, shader_id: ShaderId, resource_ids: Vec<u32>) -> Self {
        Self {
            component_id,
            shader_id,
            resource_ids,
        }
    }
}

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

    /// Get mutable binding info for a key
    pub fn get_mut(&mut self, key: &BindingKey) -> Option<&mut BindingInfo> {
        self.bindings.get_mut(key)
    }

    /// Insert binding info
    pub fn insert(&mut self, key: BindingKey, info: BindingInfo) {
        self.bindings.insert(key, info);
    }

    /// Check if a binding exists
    pub fn contains_key(&self, key: &BindingKey) -> bool {
        self.bindings.contains_key(key)
    }

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

    /// Get number of cached bindings
    pub fn len(&self) -> usize {
        self.bindings.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.bindings.is_empty()
    }

    /// Get or update binding info (lazy creation/update)
    ///
    /// This is the core lazy update function that:
    /// 1. Checks if binding exists in cache
    /// 2. Checks if component is dirty
    /// 3. Allocates buffer space if needed
    /// 4. Creates bind groups if needed
    /// 5. Writes data to GPU buffers
    /// 6. Returns binding info
    pub fn get_or_update(
        &mut self,
        key: &BindingKey,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        shader: &mut ShaderResource,
        camera_opt: Option<&CameraInstance>,
        model_opt: Option<&MeshInstance>,
    ) -> Result<&BindingInfo, String> {
        use bytemuck;

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

        // Group 0: Camera/Global uniforms
        if let Some(camera) = camera_opt {
            let needs_allocation = binding_info.group_0_offset.is_none();

            if needs_allocation {
                // Allocate
                let size = std::mem::size_of::<Mat4>() * 2; // proj + view
                let offset = shader.uniform_buffers.allocate(
                    device,
                    0,
                    key.component_id,
                    size as u64,
                    shader.shader_id,
                )?;
                binding_info.group_0_offset = Some(offset);
            }

            // Write data (always write if dirty)
            if let Some(buffer) = shader.uniform_buffers.get_buffer(0) {
                let offset = binding_info.group_0_offset.unwrap();

                // Pack camera matrices
                let data = [camera.proj_mat, camera.view_mat];
                let bytes = bytemuck::cast_slice(&data);

                queue.write_buffer(buffer, offset, bytes);
            }

            // Create bind group only if needed (first time or buffer recreated)
            if binding_info.bind_group_0.is_none() && !shader.bind_group_layouts.is_empty() {
                let offset = binding_info.group_0_offset.unwrap();
                let size = (std::mem::size_of::<Mat4>() * 2) as u64;

                if let Some(buffer) = shader.uniform_buffers.get_buffer(0) {
                    binding_info.bind_group_0 =
                        Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
                            label: Some(&format!("Bind Group 0 - Camera {}", key.component_id)),
                            layout: &shader.bind_group_layouts[0],
                            entries: &[wgpu::BindGroupEntry {
                                binding: 0,
                                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                                    buffer,
                                    offset,
                                    size: Some(std::num::NonZeroU64::new(size).unwrap()),
                                }),
                            }],
                        }));
                }
            }
        }

        // Group 2: Model/Instance uniforms
        if let Some(model) = model_opt {
            let needs_allocation = binding_info.group_2_offset.is_none();

            if needs_allocation {
                // Allocate
                let size = std::mem::size_of::<Mat4>(); // model matrix
                let offset = shader.uniform_buffers.allocate(
                    device,
                    2,
                    key.component_id,
                    size as u64,
                    shader.shader_id,
                )?;
                binding_info.group_2_offset = Some(offset);
            }

            // Write data (always write if dirty)
            if let Some(buffer) = shader.uniform_buffers.get_buffer(2) {
                let offset = binding_info.group_2_offset.unwrap();

                // Pack model matrix
                let data = [model.model_mat];
                let bytes = bytemuck::cast_slice(&data);

                queue.write_buffer(buffer, offset, bytes);
            }

            // Create bind group only if needed (first time or buffer recreated)
            if binding_info.bind_group_2.is_none() && shader.bind_group_layouts.len() > 2 {
                let offset = binding_info.group_2_offset.unwrap();
                let size = std::mem::size_of::<Mat4>() as u64;

                if let Some(buffer) = shader.uniform_buffers.get_buffer(2) {
                    binding_info.bind_group_2 =
                        Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
                            label: Some(&format!("Bind Group 2 - Model {}", key.component_id)),
                            layout: &shader.bind_group_layouts[2],
                            entries: &[wgpu::BindGroupEntry {
                                binding: 0,
                                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                                    buffer,
                                    offset,
                                    size: Some(std::num::NonZeroU64::new(size).unwrap()),
                                }),
                            }],
                        }));
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
    pub fn allocate(
        &mut self,
        device: &wgpu::Device,
        group: u32,
        id: u32,
        size: u64,
        shader_id: ShaderId,
    ) -> Result<u64, String> {
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

            // TODO: Copy old data to new buffer if needed
            // For now, data will be rewritten on next update
        }

        Ok(offset)
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

    /// Get allocator for a group
    pub fn get_allocator(&self, group: u32) -> Option<&BufferAllocator> {
        match group {
            0 => Some(&self.group_0_allocator),
            1 => Some(&self.group_1_allocator),
            2 => Some(&self.group_2_allocator),
            _ => None,
        }
    }
}

impl Default for ShaderUniformBuffers {
    fn default() -> Self {
        Self::new()
    }
}
