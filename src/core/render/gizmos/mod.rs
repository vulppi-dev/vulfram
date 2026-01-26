use bytemuck::{Pod, Zeroable};
use glam::{Vec3, Vec4};
use serde::{Deserialize, Serialize};

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct GizmoVertex {
    pub position: Vec3,
    pub _pad: f32, // Explicit padding to make it 16 bytes aligned for color (Vec4)
    pub color: Vec4,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdGizmoDrawLineArgs {
    pub start: Vec3,
    pub end: Vec3,
    pub color: Vec4,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdGizmoDrawAabbArgs {
    pub min: Vec3,
    pub max: Vec3,
    pub color: Vec4,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdResultGizmoDraw {
    pub status: u32,
}

pub struct GizmoSystem {
    vertices: Vec<GizmoVertex>,
    buffer: Option<wgpu::Buffer>,
    capacity: usize,
}

impl GizmoSystem {
    #[cfg(any(not(feature = "wasm"), target_arch = "wasm32"))]
    pub fn new() -> Self {
        Self {
            vertices: Vec::with_capacity(1024),
            buffer: None,
            capacity: 0,
        }
    }

    pub fn add_line(&mut self, start: Vec3, end: Vec3, color: Vec4) {
        self.vertices.push(GizmoVertex {
            position: start,
            _pad: 0.0,
            color,
        });
        self.vertices.push(GizmoVertex {
            position: end,
            _pad: 0.0,
            color,
        });
    }

    pub fn add_aabb(&mut self, min: Vec3, max: Vec3, color: Vec4) {
        let corners = [
            Vec3::new(min.x, min.y, min.z),
            Vec3::new(max.x, min.y, min.z),
            Vec3::new(max.x, max.y, min.z),
            Vec3::new(min.x, max.y, min.z),
            Vec3::new(min.x, min.y, max.z),
            Vec3::new(max.x, min.y, max.z),
            Vec3::new(max.x, max.y, max.z),
            Vec3::new(min.x, max.y, max.z),
        ];

        // Bottom square
        self.add_line(corners[0], corners[1], color);
        self.add_line(corners[1], corners[2], color);
        self.add_line(corners[2], corners[3], color);
        self.add_line(corners[3], corners[0], color);

        // Top square
        self.add_line(corners[4], corners[5], color);
        self.add_line(corners[5], corners[6], color);
        self.add_line(corners[6], corners[7], color);
        self.add_line(corners[7], corners[4], color);

        // Connection lines
        self.add_line(corners[0], corners[4], color);
        self.add_line(corners[1], corners[5], color);
        self.add_line(corners[2], corners[6], color);
        self.add_line(corners[3], corners[7], color);
    }

    pub fn clear(&mut self) {
        self.vertices.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.vertices.is_empty()
    }

    pub fn prepare(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        if self.vertices.is_empty() {
            return;
        }

        if self.buffer.is_none() || self.capacity < self.vertices.len() {
            self.capacity = self.vertices.len().next_power_of_two();
            let size = self.capacity * std::mem::size_of::<GizmoVertex>();
            self.buffer = Some(device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Gizmo Vertex Buffer"),
                size: size as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }));
        }

        if let Some(buffer) = &self.buffer {
            queue.write_buffer(buffer, 0, bytemuck::cast_slice(&self.vertices));
        }
    }

    pub fn draw<'a>(&'a self, rpass: &mut wgpu::RenderPass<'a>) {
        if let Some(buffer) = &self.buffer {
            if !self.vertices.is_empty() {
                rpass.set_vertex_buffer(0, buffer.slice(..));
                rpass.draw(0..self.vertices.len() as u32, 0..1);
            }
        }
    }
}
