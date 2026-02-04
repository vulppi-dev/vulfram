use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct EguiVertex {
    pub position: [f32; 2],
    pub uv: [f32; 2],
    pub color: u32,
}

impl From<egui::epaint::Vertex> for EguiVertex {
    fn from(value: egui::epaint::Vertex) -> Self {
        let color = u32::from_le_bytes(value.color.to_array());
        Self {
            position: [value.pos.x, value.pos.y],
            uv: [value.uv.x, value.uv.y],
            color,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, PartialEq)]
pub struct UniformBuffer {
    pub screen_size_in_points: [f32; 2],
    pub dithering: u32,
    pub predictable_texture_filtering: u32,
}

impl UniformBuffer {
    pub fn zeroed() -> Self {
        Self {
            screen_size_in_points: [0.0, 0.0],
            dithering: 0,
            predictable_texture_filtering: 0,
        }
    }
}

#[derive(Debug)]
pub struct SlicedBuffer {
    pub buffer: wgpu::Buffer,
    pub capacity: u64,
}
