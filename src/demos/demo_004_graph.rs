use std::collections::HashMap;

use crate::core::render::graph::{
    LogicalId, RenderGraphDesc, RenderGraphEdge, RenderGraphEdgeReason, RenderGraphLifetime,
    RenderGraphNode, RenderGraphResource, RenderGraphResourceKind,
};
use crate::core::resources::PostProcessConfig;

pub fn build_demo_graph() -> RenderGraphDesc {
    RenderGraphDesc {
        graph_id: LogicalId::Str("demo_graph".into()),
        nodes: vec![
            RenderGraphNode {
                node_id: LogicalId::Str("shadow".into()),
                pass_id: "shadow".to_string(),
                inputs: vec![],
                outputs: vec![LogicalId::Str("shadow_atlas".into())],
                params: HashMap::new(),
            },
            RenderGraphNode {
                node_id: LogicalId::Str("forward".into()),
                pass_id: "forward".to_string(),
                inputs: vec![LogicalId::Str("shadow_atlas".into())],
                outputs: vec![LogicalId::Str("hdr_color".into()), LogicalId::Str("depth".into())],
                params: HashMap::new(),
            },
            RenderGraphNode {
                node_id: LogicalId::Str("outline".into()),
                pass_id: "outline".to_string(),
                inputs: vec![LogicalId::Str("depth".into())],
                outputs: vec![LogicalId::Str("outline_color".into())],
                params: HashMap::new(),
            },
            RenderGraphNode {
                node_id: LogicalId::Str("ssao".into()),
                pass_id: "ssao".to_string(),
                inputs: vec![LogicalId::Str("depth".into())],
                outputs: vec![LogicalId::Str("ssao_raw".into())],
                params: HashMap::new(),
            },
            RenderGraphNode {
                node_id: LogicalId::Str("ssao_blur".into()),
                pass_id: "ssao-blur".to_string(),
                inputs: vec![LogicalId::Str("ssao_raw".into()), LogicalId::Str("depth".into())],
                outputs: vec![LogicalId::Str("ssao_blur".into())],
                params: HashMap::new(),
            },
            RenderGraphNode {
                node_id: LogicalId::Str("bloom".into()),
                pass_id: "bloom".to_string(),
                inputs: vec![LogicalId::Str("hdr_color".into())],
                outputs: vec![LogicalId::Str("bloom_color".into())],
                params: HashMap::new(),
            },
            RenderGraphNode {
                node_id: LogicalId::Str("post".into()),
                pass_id: "post".to_string(),
                inputs: vec![
                    LogicalId::Str("hdr_color".into()),
                    LogicalId::Str("outline_color".into()),
                    LogicalId::Str("ssao_blur".into()),
                    LogicalId::Str("bloom_color".into()),
                ],
                outputs: vec![LogicalId::Str("post_color".into())],
                params: HashMap::new(),
            },
            RenderGraphNode {
                node_id: LogicalId::Str("compose".into()),
                pass_id: "compose".to_string(),
                inputs: vec![LogicalId::Str("post_color".into())],
                outputs: vec![LogicalId::Str("swapchain".into())],
                params: HashMap::new(),
            },
        ],
        edges: vec![
            RenderGraphEdge {
                from_node_id: LogicalId::Str("shadow".into()),
                to_node_id: LogicalId::Str("forward".into()),
                reason: Some(RenderGraphEdgeReason::ReadAfterWrite),
            },
            RenderGraphEdge {
                from_node_id: LogicalId::Str("forward".into()),
                to_node_id: LogicalId::Str("outline".into()),
                reason: Some(RenderGraphEdgeReason::ReadAfterWrite),
            },
            RenderGraphEdge {
                from_node_id: LogicalId::Str("forward".into()),
                to_node_id: LogicalId::Str("ssao".into()),
                reason: Some(RenderGraphEdgeReason::ReadAfterWrite),
            },
            RenderGraphEdge {
                from_node_id: LogicalId::Str("ssao".into()),
                to_node_id: LogicalId::Str("ssao_blur".into()),
                reason: Some(RenderGraphEdgeReason::ReadAfterWrite),
            },
            RenderGraphEdge {
                from_node_id: LogicalId::Str("ssao_blur".into()),
                to_node_id: LogicalId::Str("post".into()),
                reason: Some(RenderGraphEdgeReason::ReadAfterWrite),
            },
            RenderGraphEdge {
                from_node_id: LogicalId::Str("forward".into()),
                to_node_id: LogicalId::Str("bloom".into()),
                reason: Some(RenderGraphEdgeReason::ReadAfterWrite),
            },
            RenderGraphEdge {
                from_node_id: LogicalId::Str("bloom".into()),
                to_node_id: LogicalId::Str("post".into()),
                reason: Some(RenderGraphEdgeReason::ReadAfterWrite),
            },
            RenderGraphEdge {
                from_node_id: LogicalId::Str("outline".into()),
                to_node_id: LogicalId::Str("post".into()),
                reason: Some(RenderGraphEdgeReason::ReadAfterWrite),
            },
            RenderGraphEdge {
                from_node_id: LogicalId::Str("post".into()),
                to_node_id: LogicalId::Str("compose".into()),
                reason: Some(RenderGraphEdgeReason::ReadAfterWrite),
            },
        ],
        resources: vec![
            RenderGraphResource {
                res_id: LogicalId::Str("shadow_atlas".into()),
                kind: RenderGraphResourceKind::Texture,
                lifetime: RenderGraphLifetime::Frame,
                alias_group: None,
            },
            RenderGraphResource {
                res_id: LogicalId::Str("hdr_color".into()),
                kind: RenderGraphResourceKind::Texture,
                lifetime: RenderGraphLifetime::Frame,
                alias_group: None,
            },
            RenderGraphResource {
                res_id: LogicalId::Str("depth".into()),
                kind: RenderGraphResourceKind::Texture,
                lifetime: RenderGraphLifetime::Frame,
                alias_group: None,
            },
            RenderGraphResource {
                res_id: LogicalId::Str("outline_color".into()),
                kind: RenderGraphResourceKind::Texture,
                lifetime: RenderGraphLifetime::Frame,
                alias_group: None,
            },
            RenderGraphResource {
                res_id: LogicalId::Str("ssao_raw".into()),
                kind: RenderGraphResourceKind::Texture,
                lifetime: RenderGraphLifetime::Frame,
                alias_group: None,
            },
            RenderGraphResource {
                res_id: LogicalId::Str("ssao_blur".into()),
                kind: RenderGraphResourceKind::Texture,
                lifetime: RenderGraphLifetime::Frame,
                alias_group: None,
            },
            RenderGraphResource {
                res_id: LogicalId::Str("bloom_color".into()),
                kind: RenderGraphResourceKind::Texture,
                lifetime: RenderGraphLifetime::Frame,
                alias_group: None,
            },
            RenderGraphResource {
                res_id: LogicalId::Str("post_color".into()),
                kind: RenderGraphResourceKind::Texture,
                lifetime: RenderGraphLifetime::Frame,
                alias_group: None,
            },
            RenderGraphResource {
                res_id: LogicalId::Str("swapchain".into()),
                kind: RenderGraphResourceKind::Attachment,
                lifetime: RenderGraphLifetime::Frame,
                alias_group: None,
            },
        ],
        fallback: true,
    }
}

pub fn build_post_config() -> PostProcessConfig {
    let mut post_config = PostProcessConfig::default();
    post_config.filter_enabled = true;
    post_config.filter_exposure = 1.0;
    post_config.filter_gamma = 2.2;
    post_config.filter_saturation = 1.1;
    post_config.filter_contrast = 1.1;
    post_config.filter_vignette = 0.12;
    post_config.filter_grain = 0.02;
    post_config.filter_chromatic_aberration = 0.3;
    post_config.filter_blur = 0.0;
    post_config.filter_sharpen = 0.15;
    post_config.filter_tonemap_mode = 1;
    post_config.outline_enabled = true;
    post_config.outline_strength = 1.0;
    post_config.outline_threshold = 0.3;
    post_config.outline_width = 1.0;
    post_config.outline_quality = 0.0;
    post_config.filter_posterize_steps = 0.0;
    post_config.cell_shading = false;
    post_config.ssao_enabled = true;
    post_config.ssao_strength = 1.0;
    post_config.ssao_radius = 0.75;
    post_config.ssao_bias = 0.02;
    post_config.ssao_power = 1.3;
    post_config.ssao_blur_radius = 2.0;
    post_config.ssao_blur_depth_threshold = 0.02;
    post_config.bloom_enabled = true;
    post_config.bloom_threshold = 1.0;
    post_config.bloom_knee = 0.5;
    post_config.bloom_intensity = 0.9;
    post_config.bloom_scatter = 0.7;
    post_config
}
