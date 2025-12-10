use std::collections::HashMap;
use wgpu;

use super::resources::{MaterialId, ShaderId};

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
}

impl Default for PipelineCache {
    fn default() -> Self {
        Self::new()
    }
}
