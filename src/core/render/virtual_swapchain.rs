#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};

use crate::core::render::graph::LogicalId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RenderLevelId(pub u32);

impl RenderLevelId {
    pub const ROOT: RenderLevelId = RenderLevelId(0);
}

#[derive(Debug, Clone)]
pub struct RenderLevelAllocator {
    next: u32,
}

impl Default for RenderLevelAllocator {
    fn default() -> Self {
        Self { next: RenderLevelId::ROOT.0 + 1 }
    }
}

impl RenderLevelAllocator {
    pub fn allocate(&mut self) -> RenderLevelId {
        let id = RenderLevelId(self.next);
        self.next = self.next.saturating_add(1);
        id
    }

    pub fn reset(&mut self) {
        self.next = RenderLevelId::ROOT.0 + 1;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SwapchainVirtualRoot {
    pub root_id: LogicalId,
    pub window_id: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SwapchainVirtualGraph {
    pub root: SwapchainVirtualRoot,
    pub levels: Vec<SwapchainVirtualLevel>,
    #[serde(default)]
    pub texture_registry: SwapchainVirtualTextureRegistry,
    #[serde(default)]
    pub cycle_policy: SwapchainCyclePolicy,
    #[serde(default)]
    pub max_depth: Option<u32>,
    #[serde(default)]
    pub auto_generated: bool,
}

impl SwapchainVirtualGraph {
    pub fn build_level_plans(&self) -> Result<Vec<SwapchainVirtualLevelPlan>, String> {
        let mut plans = Vec::with_capacity(self.levels.len());
        for level in &self.levels {
            plans.push(level.build_plan()?);
        }
        Ok(plans)
    }

    pub fn build_execution_plan(&self) -> Result<SwapchainVirtualExecutionPlan, String> {
        let mut level_plans = self.build_level_plans()?;
        level_plans.sort_by_key(|plan| plan.level_id.0);
        Ok(SwapchainVirtualExecutionPlan { level_plans })
    }

    pub fn validate_consistency(&self) -> Result<(), String> {
        for level in &self.levels {
            level.build_plan()?;
        }
        Ok(())
    }

    pub fn register_ui_target(
        &mut self,
        level_id: RenderLevelId,
        logical_id: LogicalId,
    ) {
        self.texture_registry
            .upsert(level_id, logical_id, SwapchainVirtualTextureUsage::UiTarget);
    }

    pub fn register_camera_target(
        &mut self,
        level_id: RenderLevelId,
        logical_id: LogicalId,
    ) {
        self.texture_registry
            .upsert(level_id, logical_id, SwapchainVirtualTextureUsage::CameraTarget);
    }

    pub fn register_compose_target(
        &mut self,
        level_id: RenderLevelId,
        logical_id: LogicalId,
    ) {
        self.texture_registry
            .upsert(level_id, logical_id, SwapchainVirtualTextureUsage::ComposeTarget);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SwapchainVirtualLevel {
    pub level_id: RenderLevelId,
    pub nodes: Vec<SwapchainVirtualNode>,
    pub edges: Vec<SwapchainVirtualEdge>,
    #[serde(default)]
    pub dirty: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SwapchainCyclePolicy {
    FrameLag,
}

impl Default for SwapchainCyclePolicy {
    fn default() -> Self {
        SwapchainCyclePolicy::FrameLag
    }
}

impl SwapchainVirtualLevel {
    pub fn build_plan(&self) -> Result<SwapchainVirtualLevelPlan, String> {
        let mut node_ids = HashSet::new();
        for node in &self.nodes {
            if !node_ids.insert(node.node_id.clone()) {
                return Err(format!("Duplicate node_id in level {:?}", self.level_id));
            }
        }

        let mut index_map = HashMap::new();
        for (idx, node) in self.nodes.iter().enumerate() {
            index_map.insert(node.node_id.clone(), idx);
        }

        for edge in &self.edges {
            if !index_map.contains_key(&edge.from_node_id) {
                return Err(format!(
                    "Edge from unknown node {} in level {:?}",
                    edge.from_node_id, self.level_id
                ));
            }
            if !index_map.contains_key(&edge.to_node_id) {
                return Err(format!(
                    "Edge to unknown node {} in level {:?}",
                    edge.to_node_id, self.level_id
                ));
            }
        }

        let order = topo_sort(&self.nodes, &self.edges, &index_map)?;

        Ok(SwapchainVirtualLevelPlan {
            level_id: self.level_id,
            nodes: self.nodes.clone(),
            order,
        })
    }
}

#[derive(Debug, Clone)]
pub struct SwapchainVirtualLevelPlan {
    pub level_id: RenderLevelId,
    pub nodes: Vec<SwapchainVirtualNode>,
    pub order: Vec<usize>,
}

#[derive(Debug, Clone)]
pub struct SwapchainVirtualExecutionPlan {
    pub level_plans: Vec<SwapchainVirtualLevelPlan>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SwapchainVirtualTextureRegistry {
    pub entries: Vec<SwapchainVirtualTextureEntry>,
}

impl SwapchainVirtualTextureRegistry {
    pub fn upsert(
        &mut self,
        level_id: RenderLevelId,
        logical_id: LogicalId,
        usage: SwapchainVirtualTextureUsage,
    ) {
        if let Some(entry) = self
            .entries
            .iter_mut()
            .find(|entry| entry.level_id == level_id && entry.logical_id == logical_id)
        {
            entry.usage = usage;
            return;
        }

        self.entries.push(SwapchainVirtualTextureEntry {
            level_id,
            logical_id,
            usage,
        });
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SwapchainVirtualTextureEntry {
    pub level_id: RenderLevelId,
    pub logical_id: LogicalId,
    pub usage: SwapchainVirtualTextureUsage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SwapchainVirtualTextureUsage {
    UiTarget,
    CameraTarget,
    ComposeTarget,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SwapchainOrderKey {
    pub layer: i32,
    pub z_index: i32,
    pub depth_level: u32,
    pub order: i32,
}

impl Default for SwapchainOrderKey {
    fn default() -> Self {
        Self {
            layer: 0,
            z_index: 0,
            depth_level: 0,
            order: 0,
        }
    }
}

impl SwapchainOrderKey {
    pub fn with_depth(mut self, depth_level: u32) -> Self {
        self.depth_level = depth_level;
        self
    }

    pub fn with_layer(mut self, layer: i32) -> Self {
        self.layer = layer;
        self
    }

    pub fn with_z_index(mut self, z_index: i32) -> Self {
        self.z_index = z_index;
        self
    }

    pub fn with_order(mut self, order: i32) -> Self {
        self.order = order;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SwapchainVirtualNode {
    pub node_id: LogicalId,
    pub kind: SwapchainVirtualNodeKind,
    #[serde(default)]
    pub order: SwapchainOrderKey,
}

impl SwapchainVirtualNode {
    pub fn ui_context(node_id: LogicalId, context_id: LogicalId) -> Self {
        Self {
            node_id,
            kind: SwapchainVirtualNodeKind::UiContext { context_id },
            order: SwapchainOrderKey::default(),
        }
    }

    pub fn camera_viewport(node_id: LogicalId, camera_id: u32) -> Self {
        Self {
            node_id,
            kind: SwapchainVirtualNodeKind::CameraViewport { camera_id },
            order: SwapchainOrderKey::default(),
        }
    }

    pub fn panel_plane(node_id: LogicalId, panel_id: LogicalId) -> Self {
        Self {
            node_id,
            kind: SwapchainVirtualNodeKind::PanelPlane { panel_id },
            order: SwapchainOrderKey::default(),
        }
    }

    pub fn compose_target(node_id: LogicalId, target_id: LogicalId) -> Self {
        Self {
            node_id,
            kind: SwapchainVirtualNodeKind::ComposeTarget { target_id },
            order: SwapchainOrderKey::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SwapchainVirtualNodeKind {
    UiContext { context_id: LogicalId },
    CameraViewport { camera_id: u32 },
    PanelPlane { panel_id: LogicalId },
    ComposeTarget { target_id: LogicalId },
}

impl SwapchainVirtualNodeKind {
    pub fn is_panel(&self) -> bool {
        matches!(self, SwapchainVirtualNodeKind::PanelPlane { .. })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SwapchainVirtualEdge {
    pub from_node_id: LogicalId,
    pub to_node_id: LogicalId,
}

fn topo_sort(
    nodes: &[SwapchainVirtualNode],
    edges: &[SwapchainVirtualEdge],
    index_map: &HashMap<LogicalId, usize>,
) -> Result<Vec<usize>, String> {
    let mut indegree = vec![0usize; nodes.len()];
    let mut adjacency: Vec<Vec<usize>> = vec![Vec::new(); nodes.len()];

    for edge in edges {
        let from = *index_map
            .get(&edge.from_node_id)
            .ok_or_else(|| format!("Edge from unknown node {}", edge.from_node_id))?;
        let to = *index_map
            .get(&edge.to_node_id)
            .ok_or_else(|| format!("Edge to unknown node {}", edge.to_node_id))?;
        adjacency[from].push(to);
        indegree[to] += 1;
    }

    let mut queue = VecDeque::new();
    for (idx, &deg) in indegree.iter().enumerate() {
        if deg == 0 {
            queue.push_back(idx);
        }
    }

    let mut order = Vec::with_capacity(nodes.len());
    while let Some(node) = queue.pop_front() {
        order.push(node);
        for &next in &adjacency[node] {
            indegree[next] -= 1;
            if indegree[next] == 0 {
                queue.push_back(next);
            }
        }
    }

    if order.len() != nodes.len() {
        return Err("Swapchain virtual graph contains a cycle".into());
    }

    Ok(order)
}

pub fn detect_cycles(level: &SwapchainVirtualLevel) -> Result<(), String> {
    let mut index_map = HashMap::new();
    for (idx, node) in level.nodes.iter().enumerate() {
        index_map.insert(node.node_id.clone(), idx);
    }

    match topo_sort(&level.nodes, &level.edges, &index_map) {
        Ok(_) => Ok(()),
        Err(message) => Err(message),
    }
}
