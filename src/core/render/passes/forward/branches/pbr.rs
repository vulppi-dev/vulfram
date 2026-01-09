use crate::core::render::cache::{PipelineKey, RenderCache, ShaderId};
use crate::core::render::state::ResourceLibrary;
use crate::core::resources::{SurfaceType, VertexStream};

pub fn get_pipeline<'a>(
    cache: &'a mut RenderCache,
    frame_index: u64,
    device: &wgpu::Device,
    library: &ResourceLibrary,
    surface: SurfaceType,
) -> &'a wgpu::RenderPipeline {
    let (blend, depth_write, depth_compare, cull_mode) = match surface {
        SurfaceType::Transparent => (
            Some(wgpu::BlendState::ALPHA_BLENDING),
            false,
            wgpu::CompareFunction::Greater,
            None,
        ),
        _ => (
            None,
            true,
            wgpu::CompareFunction::Greater, // Reverse Z
            Some(wgpu::Face::Back),
        ),
    };
    let key = PipelineKey {
        shader_id: ShaderId::ForwardPbr as u64,
        color_format: wgpu::TextureFormat::Rgba16Float,
        depth_format: Some(wgpu::TextureFormat::Depth32Float), // Reverse Z
        sample_count: 1,
        topology: wgpu::PrimitiveTopology::TriangleList,
        cull_mode,
        front_face: wgpu::FrontFace::Ccw,
        depth_write_enabled: depth_write,
        depth_compare,
        blend,
    };

    cache.get_or_create(key, frame_index, || {
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Forward PBR Pipeline"),
            layout: Some(&library.forward_pbr_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &library.forward_pbr_shader,
                entry_point: Some("vs_main"),
                buffers: &[
                    // 0: Position
                    wgpu::VertexBufferLayout {
                        array_stride: VertexStream::Position.stride_bytes(),
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &[wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x3,
                            offset: 0,
                            shader_location: 0,
                        }],
                    },
                    // 1: Normal
                    wgpu::VertexBufferLayout {
                        array_stride: VertexStream::Normal.stride_bytes(),
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &[wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x3,
                            offset: 0,
                            shader_location: 1,
                        }],
                    },
                    // 2: Tangent
                    wgpu::VertexBufferLayout {
                        array_stride: VertexStream::Tangent.stride_bytes(),
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &[wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x4,
                            offset: 0,
                            shader_location: 2,
                        }],
                    },
                    // 3: Color0
                    wgpu::VertexBufferLayout {
                        array_stride: VertexStream::Color0.stride_bytes(),
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &[wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x4,
                            offset: 0,
                            shader_location: 3,
                        }],
                    },
                    // 4: UV0
                    wgpu::VertexBufferLayout {
                        array_stride: VertexStream::UV0.stride_bytes(),
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &[wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x2,
                            offset: 0,
                            shader_location: 4,
                        }],
                    },
                    // 5: UV1
                    wgpu::VertexBufferLayout {
                        array_stride: VertexStream::UV1.stride_bytes(),
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &[wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x2,
                            offset: 0,
                            shader_location: 5,
                        }],
                    },
                    // 6: Joints
                    wgpu::VertexBufferLayout {
                        array_stride: VertexStream::Joints.stride_bytes(),
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &[wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Uint16x4,
                            offset: 0,
                            shader_location: 6,
                        }],
                    },
                    // 7: Weights
                    wgpu::VertexBufferLayout {
                        array_stride: VertexStream::Weights.stride_bytes(),
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &[wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x4,
                            offset: 0,
                            shader_location: 7,
                        }],
                    },
                ],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &library.forward_pbr_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: key.color_format,
                    blend: key.blend,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: key.topology,
                strip_index_format: None,
                front_face: key.front_face,
                cull_mode: key.cull_mode,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: key.depth_format.map(|format| wgpu::DepthStencilState {
                format,
                depth_write_enabled: key.depth_write_enabled,
                depth_compare: key.depth_compare,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: key.sample_count,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview_mask: None,
            cache: None,
        })
    })
}
