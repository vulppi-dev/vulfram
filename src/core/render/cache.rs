use std::collections::HashMap;

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
    max_unused_frames: u64,
}

impl RenderCache {
    pub fn new() -> Self {
        Self {
            pipelines: HashMap::new(),
            max_unused_frames: 3,
        }
    }

    pub fn set_max_unused_frames(&mut self, frames: u64) {
        self.max_unused_frames = frames.max(1);
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

    pub fn mark_used(&mut self, key: PipelineKey, frame_index: u64) -> bool {
        if let Some(entry) = self.pipelines.get_mut(&key) {
            entry.last_used_frame = frame_index;
            true
        } else {
            false
        }
    }

    pub fn gc(&mut self, frame_index: u64) {
        let max_unused = self.max_unused_frames;
        self.pipelines.retain(|_, entry| {
            frame_index.saturating_sub(entry.last_used_frame) <= max_unused
        });
    }

    pub fn clear(&mut self) {
        self.pipelines.clear();
    }
}
