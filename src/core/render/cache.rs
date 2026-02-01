use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u64)]
pub enum ShaderId {
    Compose = 0,
    Post,
    Outline,
    Shadow,
    LightCull,
    ForwardStandard,
    ForwardPbr,
    Gizmo,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PipelineKey {
    pub shader_id: u64,
    pub color_format: wgpu::TextureFormat,
    pub depth_format: Option<wgpu::TextureFormat>,
    pub sample_count: u32,
    pub topology: wgpu::PrimitiveTopology,
    pub cull_mode: Option<wgpu::Face>,
    pub front_face: wgpu::FrontFace,
    pub depth_write_enabled: bool,
    pub depth_compare: wgpu::CompareFunction,
    pub blend: Option<wgpu::BlendState>,
}

#[derive(Debug)]
struct PipelineEntry {
    pipeline: wgpu::RenderPipeline,
    last_used_frame: u64,
}

#[derive(Debug)]
pub struct RenderCache {
    pipelines: HashMap<PipelineKey, PipelineEntry>,
    compute_pipelines: HashMap<ComputePipelineKey, ComputePipelineEntry>,
    max_unused_frames: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ComputePipelineKey {
    pub shader_id: u64,
}

#[derive(Debug)]
struct ComputePipelineEntry {
    pipeline: wgpu::ComputePipeline,
    last_used_frame: u64,
}

impl RenderCache {
    #[cfg(any(not(feature = "wasm"), target_arch = "wasm32"))]
    pub fn new() -> Self {
        Self {
            pipelines: HashMap::new(),
            compute_pipelines: HashMap::new(),
            max_unused_frames: 3,
        }
    }

    pub fn get_or_create<F>(
        &mut self,
        key: PipelineKey,
        frame_index: u64,
        create: F,
    ) -> &wgpu::RenderPipeline
    where
        F: FnOnce() -> wgpu::RenderPipeline,
    {
        let entry = self.pipelines.entry(key).or_insert_with(|| PipelineEntry {
            pipeline: create(),
            last_used_frame: frame_index,
        });

        entry.last_used_frame = frame_index;
        &entry.pipeline
    }

    pub fn gc(&mut self, frame_index: u64) {
        let max_unused = self.max_unused_frames;
        self.pipelines
            .retain(|_, entry| frame_index.saturating_sub(entry.last_used_frame) <= max_unused);
        self.compute_pipelines
            .retain(|_, entry| frame_index.saturating_sub(entry.last_used_frame) <= max_unused);
    }

    pub fn clear(&mut self) {
        self.pipelines.clear();
        self.compute_pipelines.clear();
    }

    pub fn get_or_create_compute<F>(
        &mut self,
        key: ComputePipelineKey,
        frame_index: u64,
        create: F,
    ) -> &wgpu::ComputePipeline
    where
        F: FnOnce() -> wgpu::ComputePipeline,
    {
        let entry = self
            .compute_pipelines
            .entry(key)
            .or_insert_with(|| ComputePipelineEntry {
                pipeline: create(),
                last_used_frame: frame_index,
            });

        entry.last_used_frame = frame_index;
        &entry.pipeline
    }
}
