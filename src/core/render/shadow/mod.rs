use crate::core::resources::{
    AtlasDesc, AtlasHandle, AtlasSystem, StorageBufferPool, UniformBufferPool,
};
use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec4Swizzles};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use wgpu::{Device, Queue, TextureFormat, TextureUsages};

pub mod cmd;
pub use cmd::*;

/// Configuration for the Shadow Manager
#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub struct ShadowConfig {
    pub tile_resolution: u32,
    pub atlas_tiles_w: u32,
    pub atlas_tiles_h: u32,
    pub atlas_layers: u32,
    pub virtual_grid_size: u32,
    pub smoothing: u32,
}

impl Default for ShadowConfig {
    fn default() -> Self {
        Self {
            tile_resolution: 1024,
            atlas_tiles_w: 8,
            atlas_tiles_h: 8,
            atlas_layers: 1,
            virtual_grid_size: 1,
            smoothing: 1,
        }
    }
}

/// Uniform data for shadow parameters in the shader
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct ShadowParams {
    pub virtual_grid_size: f32,
    pub pcf_range: i32,
    pub _padding: [f32; 2],
}

/// Unique identifier for a virtual shadow page
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ShadowPageKey {
    pub light_id: u32,
    pub x: u32,
    pub y: u32,
}

/// State of an allocated shadow page
#[derive(Debug, Clone)]
pub struct ShadowPageRecord {
    pub atlas_handle: AtlasHandle,
    pub last_frame_used: u64,
    pub is_dirty: bool,
}

/// Entry in the GPU page table for Virtual Shadow Maps
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ShadowPageEntry {
    /// [scale.x, scale.y, offset.x, offset.y]
    pub scale_offset: [f32; 4],
    /// Index into the texture array
    pub layer_index: u32,
    /// Padding for 16-byte alignment
    pub _padding: [u32; 3],
}

impl Default for ShadowPageEntry {
    fn default() -> Self {
        Self {
            scale_offset: [0.0; 4],
            layer_index: 0,
            _padding: [0; 3],
        }
    }
}

/// Manages Virtual Shadow Maps paging and atlas allocation
pub struct ShadowManager {
    pub atlas: AtlasSystem,
    pub page_table: StorageBufferPool<ShadowPageEntry>,
    pub params_pool: UniformBufferPool<ShadowParams>,
    pub table_capacity: u32,
    pub is_dirty: bool,

    // Virtual to Physical mapping
    pub cache: HashMap<ShadowPageKey, ShadowPageRecord>,

    pub config: ShadowConfig,
}

impl ShadowManager {
    pub fn new(device: &Device, queue: &Queue, table_capacity: u32) -> Self {
        let config = ShadowConfig::default();
        let atlas_desc = AtlasDesc {
            label: Some("Shadow Atlas"),
            format: TextureFormat::Depth32Float,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            tile_px: config.tile_resolution,
            tiles_w: config.atlas_tiles_w,
            tiles_h: config.atlas_tiles_h,
            layers: config.atlas_layers,
        };

        let atlas = AtlasSystem::new(device, atlas_desc);
        let alignment = device.limits().min_uniform_buffer_offset_alignment as u64;
        let storage_alignment = device.limits().min_storage_buffer_offset_alignment as u64;

        let page_table =
            StorageBufferPool::new(device, queue, Some(table_capacity), storage_alignment);
        let mut params_pool = UniformBufferPool::new(device, queue, Some(1), alignment);

        params_pool.write(
            0,
            &ShadowParams {
                virtual_grid_size: config.virtual_grid_size as f32,
                pcf_range: config.smoothing as i32,
                _padding: [0.0; 2],
            },
        );

        Self {
            atlas,
            page_table,
            params_pool,
            table_capacity,
            cache: HashMap::new(),
            config,
            is_dirty: true,
        }
    }

    pub fn configure(&mut self, device: &Device, config: ShadowConfig) {
        let needs_atlas_rebuild = config.tile_resolution != self.config.tile_resolution
            || config.atlas_tiles_w != self.config.atlas_tiles_w
            || config.atlas_tiles_h != self.config.atlas_tiles_h
            || config.atlas_layers != self.config.atlas_layers;

        self.config = config;

        self.params_pool.write(
            0,
            &ShadowParams {
                virtual_grid_size: config.virtual_grid_size as f32,
                pcf_range: config.smoothing as i32,
                _padding: [0.0; 2],
            },
        );

        if needs_atlas_rebuild {
            let atlas_desc = AtlasDesc {
                label: Some("Shadow Atlas"),
                format: TextureFormat::Depth32Float,
                usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
                tile_px: config.tile_resolution,
                tiles_w: config.atlas_tiles_w,
                tiles_h: config.atlas_tiles_h,
                layers: config.atlas_layers,
            };
            self.atlas = AtlasSystem::new(device, atlas_desc);
            self.cache.clear();
            self.is_dirty = true;
        }
    }

    /// Identifies which virtual pages are required for a given light and camera.
    /// Returns a list of (x, y) coordinates in the virtual grid.
    pub fn identify_required_pages(
        &self,
        light_view_proj: Mat4,
        camera_inv_view_proj: Mat4,
    ) -> Vec<(u32, u32)> {
        // ... (rest of the method using self.config.virtual_grid_size)
        // Note: I will need to replace self.virtual_grid_size with self.config.virtual_grid_size in the implementation
        // 1. Get camera frustum corners in world space (NDC cube -> World)
        let ndc_corners = [
            glam::vec3(-1.0, -1.0, 0.0), // Near bottom left
            glam::vec3(1.0, -1.0, 0.0),  // Near bottom right
            glam::vec3(-1.0, 1.0, 0.0),  // Near top left
            glam::vec3(1.0, 1.0, 0.0),   // Near top right
            glam::vec3(-1.0, -1.0, 1.0), // Far bottom left
            glam::vec3(1.0, -1.0, 1.0),  // Far bottom right
            glam::vec3(-1.0, 1.0, 1.0),  // Far top left
            glam::vec3(1.0, 1.0, 1.0),   // Far top right
        ];

        let mut world_corners = [glam::Vec3::ZERO; 8];
        for i in 0..8 {
            let world_pos = camera_inv_view_proj * ndc_corners[i].extend(1.0);
            world_corners[i] = world_pos.xyz() / world_pos.w;
        }

        // 2. Transform world corners to light NDC space
        let mut min_ndc = glam::vec2(1.0, 1.0);
        let mut max_ndc = glam::vec2(-1.0, -1.0);

        for corner in world_corners {
            let light_ndc = light_view_proj * corner.extend(1.0);
            let ndc = light_ndc.xy() / light_ndc.w;

            min_ndc = min_ndc.min(ndc);
            max_ndc = max_ndc.max(ndc);
        }

        // 3. Clamp to light viewport [-1, 1]
        let min_x = min_ndc.x.max(-1.0);
        let max_x = max_ndc.x.min(1.0);
        let min_y = min_ndc.y.max(-1.0);
        let max_y = max_ndc.y.min(1.0);

        if min_x > max_x || min_y > max_y {
            return Vec::new();
        }

        // 4. Convert NDC to virtual grid coordinates
        // NDC [-1, 1] -> Grid [0, virtual_grid_size]
        let grid_min_x =
            (((min_x + 1.0) * 0.5) * self.config.virtual_grid_size as f32).floor() as u32;
        let grid_max_x =
            (((max_x + 1.0) * 0.5) * self.config.virtual_grid_size as f32).ceil() as u32;
        let grid_min_y =
            (((min_y + 1.0) * 0.5) * self.config.virtual_grid_size as f32).floor() as u32;
        let grid_max_y =
            (((max_y + 1.0) * 0.5) * self.config.virtual_grid_size as f32).ceil() as u32;

        let mut required = Vec::new();
        for y in grid_min_y..grid_max_y.min(self.config.virtual_grid_size) {
            for x in grid_min_x..grid_max_x.min(self.config.virtual_grid_size) {
                required.push((x, y));
            }
        }

        required
    }

    /// Calculates the View-Projection matrix for a specific virtual page.
    /// This "zooms in" the light's base projection to the specific page area.
    pub fn get_page_view_projection(
        &self,
        light_view: Mat4,
        light_proj: Mat4,
        x: u32,
        y: u32,
    ) -> Mat4 {
        // Calculate the range in NDC space [-1, 1] for this page
        let s = self.config.virtual_grid_size as f32;
        let x_min = -1.0 + (x as f32 * 2.0 / s);
        let x_max = -1.0 + ((x + 1) as f32 * 2.0 / s);
        let y_min = -1.0 + (y as f32 * 2.0 / s);
        let y_max = -1.0 + ((y + 1) as f32 * 2.0 / s);

        // Create a scale and bias matrix to transform the base projection
        // We want to map the [x_min, x_max] range to [-1, 1]
        let scale_x = 2.0 / (x_max - x_min);
        let scale_y = 2.0 / (y_max - y_min);
        let offset_x = -(x_max + x_min) / (x_max - x_min);
        let offset_y = -(y_max + y_min) / (y_max - y_min);

        let custom_proj = Mat4::from_cols(
            glam::vec4(scale_x, 0.0, 0.0, 0.0),
            glam::vec4(0.0, scale_y, 0.0, 0.0),
            glam::vec4(0.0, 0.0, 1.0, 0.0),
            glam::vec4(offset_x, offset_y, 0.0, 1.0),
        );

        custom_proj * light_proj * light_view
    }

    /// Requests a tile for a specific virtual page.

    /// If the page is already cached, returns its handle.
    /// If not, tries to allocate a new one.
    pub fn request_page(
        &mut self,
        light_id: u32,
        x: u32,
        y: u32,
        frame_index: u64,
    ) -> Option<AtlasHandle> {
        let key = ShadowPageKey { light_id, x, y };

        if let Some(record) = self.cache.get_mut(&key) {
            record.last_frame_used = frame_index;
            return Some(record.atlas_handle);
        }

        // New page needed
        if let Some((handle, relocations)) = self.atlas.alloc(1, 1) {
            // Handle relocations if repack happened
            for relocation in relocations {
                self.update_cache_after_relocation(relocation);
            }

            self.cache.insert(
                key,
                ShadowPageRecord {
                    atlas_handle: handle,
                    last_frame_used: frame_index,
                    is_dirty: true,
                },
            );

            return Some(handle);
        }

        None
    }

    fn update_cache_after_relocation(
        &mut self,
        relocation: crate::core::resources::AtlasRelocation,
    ) {
        for record in self.cache.values_mut() {
            if record.atlas_handle == relocation.handle {
                record.is_dirty = true; // Must re-render since it moved
            }
        }
    }

    /// Synchronizes the GPU page table with the current cache state
    pub fn sync_table(&mut self) {
        let mut entries = vec![ShadowPageEntry::default(); self.table_capacity as usize];

        for (key, record) in &self.cache {
            // Linear mapping of light+page to table index
            let id = (key.light_id * self.config.virtual_grid_size * self.config.virtual_grid_size
                + key.y * self.config.virtual_grid_size
                + key.x)
                % self.table_capacity;

            if let Some(transform) = self.atlas.get_uv_transform(record.atlas_handle) {
                entries[id as usize] = ShadowPageEntry {
                    scale_offset: [transform.0, transform.1, transform.2, transform.3],
                    layer_index: transform.4,
                    _padding: [0; 3],
                };
            }
        }

        self.page_table.write_slice(0, &entries);
    }

    pub fn begin_frame(&mut self, frame_index: u64) {
        self.page_table.begin_frame(frame_index);
        self.params_pool.begin_frame(frame_index);
    }

    pub fn mark_dirty(&mut self) {
        self.is_dirty = true;
    }

    pub fn clear_dirty(&mut self) {
        self.is_dirty = false;
    }
}
