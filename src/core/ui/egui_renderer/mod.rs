mod buffers;
mod textures;

use std::ops::Range;

use bytemuck::cast_slice;
use egui::epaint::{ClippedPrimitive, Primitive, TextureId};

use buffers::{EguiVertex, SlicedBuffer, UniformBuffer};
use textures::TextureManager;

pub struct ScreenDescriptor {
    pub size_in_pixels: [u32; 2],
    pub pixels_per_point: f32,
}

impl ScreenDescriptor {
    pub fn screen_size_in_points(&self) -> [f32; 2] {
        [
            self.size_in_pixels[0] as f32 / self.pixels_per_point,
            self.size_in_pixels[1] as f32 / self.pixels_per_point,
        ]
    }
}

pub(crate) struct DrawCall {
    clip_rect: egui::Rect,
    texture_id: TextureId,
    index_range: Range<u32>,
}

pub struct UiEguiRenderer {
    pipeline: wgpu::RenderPipeline,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    texture_bind_group_layout: wgpu::BindGroupLayout,
    vertex_buffer: SlicedBuffer,
    index_buffer: SlicedBuffer,
    previous_uniforms: UniformBuffer,
    textures: TextureManager,
    output_format: wgpu::TextureFormat,
}

impl UiEguiRenderer {
    pub fn new(device: &wgpu::Device, output_format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("egui_shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("egui.wgsl").into()),
        });

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("egui_uniform_buffer"),
            size: std::mem::size_of::<UniformBuffer>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("egui_uniform_bind_group_layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: std::num::NonZeroU64::new(
                            std::mem::size_of::<UniformBuffer>() as u64,
                        ),
                    },
                    count: None,
                }],
            });

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("egui_uniform_bind_group"),
            layout: &uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("egui_texture_bind_group_layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("egui_pipeline_layout"),
            bind_group_layouts: &[&uniform_bind_group_layout, &texture_bind_group_layout],
            immediate_size: 0,
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("egui_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<EguiVertex>() as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x2,
                            offset: 0,
                            shader_location: 0,
                        },
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x2,
                            offset: 8,
                            shader_location: 1,
                        },
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Uint32,
                            offset: 16,
                            shader_location: 2,
                        },
                    ],
                }],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main_linear_framebuffer"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: output_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        let vertex_buffer = SlicedBuffer {
            buffer: device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("egui_vertex_buffer"),
                size: 1,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
            capacity: 0,
        };

        let index_buffer = SlicedBuffer {
            buffer: device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("egui_index_buffer"),
                size: 1,
                usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
            capacity: 0,
        };

        Self {
            pipeline,
            uniform_buffer,
            uniform_bind_group,
            texture_bind_group_layout,
            vertex_buffer,
            index_buffer,
            previous_uniforms: UniformBuffer::zeroed(),
            textures: TextureManager::new(),
            output_format,
        }
    }

    pub fn output_format(&self) -> wgpu::TextureFormat {
        self.output_format
    }

    pub fn update_textures(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        textures_delta: &egui::TexturesDelta,
    ) {
        for (id, image_delta) in &textures_delta.set {
            self.textures
                .update_texture(device, queue, &self.texture_bind_group_layout, *id, image_delta);
        }
        for id in &textures_delta.free {
            self.textures.free_texture(id);
        }
    }

    pub fn update_buffers(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        paint_jobs: &[ClippedPrimitive],
        screen_descriptor: &ScreenDescriptor,
    ) -> Vec<DrawCall> {
        let uniforms = UniformBuffer {
            screen_size_in_points: screen_descriptor.screen_size_in_points(),
            dithering: 1,
            predictable_texture_filtering: 0,
        };

        if uniforms != self.previous_uniforms {
            queue.write_buffer(&self.uniform_buffer, 0, cast_slice(&[uniforms]));
            self.previous_uniforms = uniforms;
        }

        let mut vertices: Vec<EguiVertex> = Vec::new();
        let mut indices: Vec<u32> = Vec::new();
        let mut draw_calls: Vec<DrawCall> = Vec::new();

        for ClippedPrimitive { clip_rect, primitive } in paint_jobs {
            let Primitive::Mesh(mesh) = primitive else {
                continue;
            };

            let index_start = indices.len() as u32;
            let base = vertices.len() as u32;
            indices.extend(mesh.indices.iter().map(|i| *i + base));
            vertices.extend(mesh.vertices.iter().cloned().map(EguiVertex::from));
            let index_end = indices.len() as u32;

            draw_calls.push(DrawCall {
                clip_rect: *clip_rect,
                texture_id: mesh.texture_id,
                index_range: index_start..index_end,
            });
        }

        self.write_vertex_buffer(device, queue, &vertices);
        self.write_index_buffer(device, queue, &indices);

        draw_calls
    }

    pub fn render(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        target_view: &wgpu::TextureView,
        draw_calls: &[DrawCall],
        screen_descriptor: &ScreenDescriptor,
        clear_color: wgpu::Color,
    ) {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("egui_render_pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target_view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(clear_color),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });

        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.uniform_bind_group, &[]);
        pass.set_vertex_buffer(0, self.vertex_buffer.buffer.slice(..));
        pass.set_index_buffer(self.index_buffer.buffer.slice(..), wgpu::IndexFormat::Uint32);

        let target_size = screen_descriptor.size_in_pixels;
        for call in draw_calls {
            let Some(bind_group) = self.textures.texture_bind_group(&call.texture_id) else {
                continue;
            };
            if let Some(scissor) = ScissorRect::new(&call.clip_rect, screen_descriptor, target_size)
            {
                pass.set_scissor_rect(scissor.x, scissor.y, scissor.w, scissor.h);
            } else {
                continue;
            }
            pass.set_bind_group(1, bind_group, &[]);
            pass.draw_indexed(call.index_range.clone(), 0, 0..1);
        }
    }

    fn write_vertex_buffer(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        vertices: &[EguiVertex],
    ) {
        let size = (std::mem::size_of::<EguiVertex>() * vertices.len()) as u64;
        if size == 0 {
            return;
        }
        if self.vertex_buffer.capacity < size {
            self.vertex_buffer.capacity = size.next_power_of_two();
            self.vertex_buffer.buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("egui_vertex_buffer"),
                size: self.vertex_buffer.capacity,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }
        queue.write_buffer(&self.vertex_buffer.buffer, 0, cast_slice(vertices));
    }

    fn write_index_buffer(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        indices: &[u32],
    ) {
        let size = (std::mem::size_of::<u32>() * indices.len()) as u64;
        if size == 0 {
            return;
        }
        if self.index_buffer.capacity < size {
            self.index_buffer.capacity = size.next_power_of_two();
            self.index_buffer.buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("egui_index_buffer"),
                size: self.index_buffer.capacity,
                usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }
        queue.write_buffer(&self.index_buffer.buffer, 0, cast_slice(indices));
    }
}

struct ScissorRect {
    x: u32,
    y: u32,
    w: u32,
    h: u32,
}

impl ScissorRect {
    fn new(
        clip_rect: &egui::Rect,
        screen: &ScreenDescriptor,
        target_size: [u32; 2],
    ) -> Option<Self> {
        let clip_min_x = (clip_rect.min.x * screen.pixels_per_point).round() as i32;
        let clip_min_y = (clip_rect.min.y * screen.pixels_per_point).round() as i32;
        let clip_max_x = (clip_rect.max.x * screen.pixels_per_point).round() as i32;
        let clip_max_y = (clip_rect.max.y * screen.pixels_per_point).round() as i32;

        let clip_min_x = clip_min_x.clamp(0, target_size[0] as i32);
        let clip_min_y = clip_min_y.clamp(0, target_size[1] as i32);
        let clip_max_x = clip_max_x.clamp(clip_min_x, target_size[0] as i32);
        let clip_max_y = clip_max_y.clamp(clip_min_y, target_size[1] as i32);

        let w = (clip_max_x - clip_min_x) as u32;
        let h = (clip_max_y - clip_min_y) as u32;
        if w == 0 || h == 0 {
            return None;
        }

        Some(ScissorRect {
            x: clip_min_x as u32,
            y: clip_min_y as u32,
            w,
            h,
        })
    }
}
