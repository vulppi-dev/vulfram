use std::collections::HashMap;
use wgpu;

use super::resources::{MaterialId, MaterialResource, ShaderId, ShaderResource};

// MARK: - Pipeline Cache Key

/// Key for caching render pipelines
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct PipelineCacheKey {
    pub shader_id: ShaderId,
    pub material_id: MaterialId,
}

impl PipelineCacheKey {
    pub fn new(shader_id: ShaderId, material_id: MaterialId) -> Self {
        Self {
            shader_id,
            material_id,
        }
    }
}

// MARK: - Pipeline Cache

/// Cache for render pipelines
///
/// Pipelines are created lazily on first use and cached for reuse.
/// A pipeline is uniquely identified by shader + material combination.
pub struct PipelineCache {
    pipelines: HashMap<PipelineCacheKey, wgpu::RenderPipeline>,
}

impl PipelineCache {
    pub fn new() -> Self {
        Self {
            pipelines: HashMap::new(),
        }
    }

    /// Get a cached pipeline
    pub fn get(&self, key: &PipelineCacheKey) -> Option<&wgpu::RenderPipeline> {
        self.pipelines.get(key)
    }

    /// Insert a pipeline into the cache
    pub fn insert(&mut self, key: PipelineCacheKey, pipeline: wgpu::RenderPipeline) {
        self.pipelines.insert(key, pipeline);
    }

    /// Check if a pipeline is cached
    pub fn contains_key(&self, key: &PipelineCacheKey) -> bool {
        self.pipelines.contains_key(key)
    }

    /// Remove all pipelines related to a shader
    pub fn remove_shader_pipelines(&mut self, shader_id: ShaderId) {
        self.pipelines.retain(|key, _| key.shader_id != shader_id);
    }

    /// Remove all pipelines related to a material
    pub fn remove_material_pipelines(&mut self, material_id: MaterialId) {
        self.pipelines
            .retain(|key, _| key.material_id != material_id);
    }

    /// Clear all pipelines
    pub fn clear(&mut self) {
        self.pipelines.clear();
    }

    /// Get number of cached pipelines
    pub fn len(&self) -> usize {
        self.pipelines.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.pipelines.is_empty()
    }

    /// Get or create a render pipeline (lazy creation)
    ///
    /// This method creates pipelines lazily on first use and caches them for reuse.
    /// A pipeline is uniquely identified by shader + material combination.
    pub fn get_or_create(
        &mut self,
        shader_id: ShaderId,
        material_id: MaterialId,
        device: &wgpu::Device,
        shader: &ShaderResource,
        material: &MaterialResource,
        surface_format: wgpu::TextureFormat,
    ) -> Result<&wgpu::RenderPipeline, String> {
        let key = PipelineCacheKey::new(shader_id, material_id);

        // If already exists, return from cache
        if self.pipelines.contains_key(&key) {
            return Ok(self.pipelines.get(&key).unwrap());
        }

        // Create pipeline layout from shader bind group layouts
        let bind_group_layout_refs: Vec<&wgpu::BindGroupLayout> =
            shader.bind_group_layouts.iter().collect();

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some(&format!("Pipeline Layout S{} M{}", shader_id, material_id)),
            bind_group_layouts: &bind_group_layout_refs,
            push_constant_ranges: &[],
        });

        // Create render pipeline
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(&format!("Render Pipeline S{} M{}", shader_id, material_id)),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader.module,
                entry_point: Some("vs_main"),
                buffers: &[shader.vertex_buffer_layout.clone()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader.module,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: material.pipeline_spec.blend,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: material.pipeline_spec.primitive,
            depth_stencil: material.pipeline_spec.depth_stencil.clone(),
            multisample: material.pipeline_spec.multisample,
            multiview: None,
            cache: None,
        });

        // Insert into cache
        self.pipelines.insert(key.clone(), pipeline);

        Ok(self.pipelines.get(&key).unwrap())
    }
}

impl Default for PipelineCache {
    fn default() -> Self {
        Self::new()
    }
}
