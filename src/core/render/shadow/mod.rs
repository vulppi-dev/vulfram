use crate::core::resources::{AtlasDesc, AtlasSystem, StorageBufferPool};
use wgpu::{Device, Queue, TextureFormat, TextureUsages};

/// Entry in the GPU page table for Virtual Shadow Maps
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ShadowPageEntry {
    /// [scale.x, scale.y, offset.x, offset.y]
    pub scale_offset: [f32; 4],
    /// Index into the texture array
    pub layer_index: u32,
    /// Padding for 16-byte alignment (20 + 12 = 32 bytes total)
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
    /// Capacity in number of pages the table can hold
    pub table_capacity: u32,
}

impl ShadowManager {
    pub fn new(device: &Device, queue: &Queue, table_capacity: u32) -> Self {
        // Initialize Atlas for Shadow Maps (using Depth32Float)
        let atlas_desc = AtlasDesc {
            label: Some("Shadow Atlas"),
            format: TextureFormat::Depth32Float,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            tile_px: 512,
            tiles_w: 8, // 4096px width (approx with pitch)
            tiles_h: 8, // 4096px height
            layers: 1,
        };

        let atlas = AtlasSystem::new(device, atlas_desc);

        // Initialize Page Table (Storage Buffer)
        let alignment = device.limits().min_storage_buffer_offset_alignment as u64;
        let page_table = StorageBufferPool::new(
            device,
            queue,
            Some(table_capacity),
            alignment,
        );

        Self {
            atlas,
            page_table,
            table_capacity,
        }
    }

    /// Uploads the entire page table or partial updates to the GPU
    pub fn update_page_table(&mut self, entries: &[ShadowPageEntry]) {
        self.page_table.write_slice(0, entries);
    }

    pub fn begin_frame(&mut self, frame_index: u64) {
        self.page_table.begin_frame(frame_index);
    }
}
