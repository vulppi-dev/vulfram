use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};

#[derive(Debug, Clone, Hash, PartialEq, Eq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum LogicalId {
    Str(String),
    Int(i64),
}

impl std::fmt::Display for LogicalId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogicalId::Str(value) => write!(f, "{}", value),
            LogicalId::Int(value) => write!(f, "{}", value),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum RenderGraphResourceKind {
    Texture,
    Buffer,
    Attachment,
}

impl Default for RenderGraphResourceKind {
    fn default() -> Self {
        RenderGraphResourceKind::Texture
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum RenderGraphLifetime {
    Frame,
    Persistent,
}

impl Default for RenderGraphLifetime {
    fn default() -> Self {
        RenderGraphLifetime::Frame
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum RenderGraphEdgeReason {
    ReadAfterWrite,
    WriteAfterRead,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum RenderGraphValue {
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
}

impl From<&str> for RenderGraphValue {
    fn from(value: &str) -> Self {
        RenderGraphValue::String(value.to_string())
    }
}

impl From<String> for RenderGraphValue {
    fn from(value: String) -> Self {
        RenderGraphValue::String(value)
    }
}

impl From<bool> for RenderGraphValue {
    fn from(value: bool) -> Self {
        RenderGraphValue::Bool(value)
    }
}

impl From<i64> for RenderGraphValue {
    fn from(value: i64) -> Self {
        RenderGraphValue::Int(value)
    }
}

impl From<f64> for RenderGraphValue {
    fn from(value: f64) -> Self {
        RenderGraphValue::Float(value)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderGraphResource {
    pub res_id: LogicalId,
    #[serde(default)]
    pub kind: RenderGraphResourceKind,
    #[serde(default)]
    pub lifetime: RenderGraphLifetime,
    #[serde(default)]
    pub alias_group: Option<LogicalId>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderGraphNode {
    pub node_id: LogicalId,
    pub pass_id: String,
    #[serde(default)]
    pub inputs: Vec<LogicalId>,
    #[serde(default)]
    pub outputs: Vec<LogicalId>,
    #[serde(default)]
    pub params: HashMap<String, RenderGraphValue>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderGraphEdge {
    pub from_node_id: LogicalId,
    pub to_node_id: LogicalId,
    #[serde(default)]
    pub reason: Option<RenderGraphEdgeReason>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderGraphDesc {
    pub graph_id: LogicalId,
    pub nodes: Vec<RenderGraphNode>,
    pub edges: Vec<RenderGraphEdge>,
    #[serde(default)]
    pub resources: Vec<RenderGraphResource>,
    #[serde(default)]
    pub fallback: bool,
}

#[derive(Debug, Clone)]
pub struct RenderGraphPlan {
    pub nodes: Vec<RenderGraphNode>,
    pub order: Vec<usize>,
}

impl RenderGraphPlan {
    pub fn has_pass(&self, pass_id: &str) -> bool {
        self.nodes.iter().any(|node| node.pass_id == pass_id)
    }
}

#[derive(Debug, Clone)]
pub enum RenderGraphApplyResult {
    Applied,
    FallbackUsed(String),
}

#[derive(Debug, Clone)]
pub struct RenderGraphState {
    fallback: RenderGraphPlan,
    active: RenderGraphPlan,
    uses_fallback: bool,
}

impl RenderGraphState {
    pub fn new() -> Self {
        let fallback_desc = fallback_graph();
        let fallback = validate_graph(&fallback_desc).expect("Fallback graph must be valid");
        Self {
            active: fallback.clone(),
            fallback,
            uses_fallback: true,
        }
    }

    pub fn apply_graph(&mut self, desc: RenderGraphDesc) -> Result<RenderGraphApplyResult, String> {
        match validate_graph(&desc) {
            Ok(plan) => {
                self.active = plan;
                self.uses_fallback = false;
                Ok(RenderGraphApplyResult::Applied)
            }
            Err(err) => {
                if desc.fallback {
                    self.active = self.fallback.clone();
                    self.uses_fallback = true;
                    Ok(RenderGraphApplyResult::FallbackUsed(err))
                } else {
                    Err(err)
                }
            }
        }
    }

    pub fn reset_to_fallback(&mut self) {
        self.active = self.fallback.clone();
        self.uses_fallback = true;
    }

    pub fn plan(&self) -> &RenderGraphPlan {
        &self.active
    }
}

pub fn validate_graph(desc: &RenderGraphDesc) -> Result<RenderGraphPlan, String> {
    let mut node_ids: HashSet<LogicalId> = HashSet::new();
    for node in &desc.nodes {
        if !node_ids.insert(node.node_id.clone()) {
            return Err(format!("Duplicate node_id: {}", node.node_id));
        }
    }

    let mut res_ids: HashSet<LogicalId> = HashSet::new();
    for res in &desc.resources {
        if !res_ids.insert(res.res_id.clone()) {
            return Err(format!("Duplicate res_id: {}", res.res_id));
        }
    }

    let mut node_index: HashMap<LogicalId, usize> = HashMap::new();
    for (idx, node) in desc.nodes.iter().enumerate() {
        node_index.insert(node.node_id.clone(), idx);
        if !is_known_pass(&node.pass_id) {
            return Err(format!("Unknown pass_id: {}", node.pass_id));
        }
    }

    for edge in &desc.edges {
        if !node_index.contains_key(&edge.from_node_id) {
            return Err(format!("Edge from unknown node: {}", edge.from_node_id));
        }
        if !node_index.contains_key(&edge.to_node_id) {
            return Err(format!("Edge to unknown node: {}", edge.to_node_id));
        }
    }

    for node in &desc.nodes {
        for input in &node.inputs {
            res_ids.insert(input.clone());
        }
        for output in &node.outputs {
            res_ids.insert(output.clone());
        }
    }

    let order = topo_sort(&desc.nodes, &desc.edges)?;

    Ok(RenderGraphPlan {
        nodes: desc.nodes.clone(),
        order,
    })
}

fn topo_sort(nodes: &[RenderGraphNode], edges: &[RenderGraphEdge]) -> Result<Vec<usize>, String> {
    let mut indegree = vec![0usize; nodes.len()];
    let mut adjacency: Vec<Vec<usize>> = vec![Vec::new(); nodes.len()];
    let mut index_map: HashMap<LogicalId, usize> = HashMap::new();

    for (idx, node) in nodes.iter().enumerate() {
        index_map.insert(node.node_id.clone(), idx);
    }

    for edge in edges {
        let from = *index_map
            .get(&edge.from_node_id)
            .ok_or_else(|| format!("Edge from unknown node: {}", edge.from_node_id))?;
        let to = *index_map
            .get(&edge.to_node_id)
            .ok_or_else(|| format!("Edge to unknown node: {}", edge.to_node_id))?;
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
        return Err("Render graph contains a cycle".into());
    }

    Ok(order)
}

fn is_known_pass(pass_id: &str) -> bool {
    matches!(
        pass_id,
        "shadow"
            | "light-cull"
            | "skybox"
            | "forward"
            | "outline"
            | "ssao"
            | "ssao-blur"
            | "post"
            | "compose"
    )
}

pub fn fallback_graph() -> RenderGraphDesc {
    RenderGraphDesc {
        graph_id: LogicalId::Str("fallback".into()),
        nodes: vec![
            RenderGraphNode {
                node_id: LogicalId::Str("shadow_pass".into()),
                pass_id: "shadow".into(),
                inputs: Vec::new(),
                outputs: vec![LogicalId::Str("shadow_atlas".into())],
                params: HashMap::new(),
            },
            RenderGraphNode {
                node_id: LogicalId::Str("light_cull_pass".into()),
                pass_id: "light-cull".into(),
                inputs: Vec::new(),
                outputs: Vec::new(),
                params: HashMap::new(),
            },
            RenderGraphNode {
                node_id: LogicalId::Str("forward_pass".into()),
                pass_id: "forward".into(),
                inputs: vec![LogicalId::Str("shadow_atlas".into())],
                outputs: vec![
                    LogicalId::Str("hdr_color".into()),
                    LogicalId::Str("depth".into()),
                ],
                params: HashMap::new(),
            },
            RenderGraphNode {
                node_id: LogicalId::Str("outline_pass".into()),
                pass_id: "outline".into(),
                inputs: vec![LogicalId::Str("depth".into())],
                outputs: vec![LogicalId::Str("outline_color".into())],
                params: HashMap::new(),
            },
            RenderGraphNode {
                node_id: LogicalId::Str("ssao_pass".into()),
                pass_id: "ssao".into(),
                inputs: vec![LogicalId::Str("depth".into())],
                outputs: vec![LogicalId::Str("ssao_raw".into())],
                params: HashMap::new(),
            },
            RenderGraphNode {
                node_id: LogicalId::Str("ssao_blur_pass".into()),
                pass_id: "ssao-blur".into(),
                inputs: vec![
                    LogicalId::Str("ssao_raw".into()),
                    LogicalId::Str("depth".into()),
                ],
                outputs: vec![LogicalId::Str("ssao_blur".into())],
                params: HashMap::new(),
            },
            RenderGraphNode {
                node_id: LogicalId::Str("post_pass".into()),
                pass_id: "post".into(),
                inputs: vec![
                    LogicalId::Str("hdr_color".into()),
                    LogicalId::Str("outline_color".into()),
                    LogicalId::Str("ssao_blur".into()),
                ],
                outputs: vec![LogicalId::Str("post_color".into())],
                params: HashMap::new(),
            },
            RenderGraphNode {
                node_id: LogicalId::Str("compose_pass".into()),
                pass_id: "compose".into(),
                inputs: vec![LogicalId::Str("post_color".into())],
                outputs: vec![LogicalId::Str("swapchain".into())],
                params: HashMap::new(),
            },
        ],
        edges: vec![
            RenderGraphEdge {
                from_node_id: LogicalId::Str("shadow_pass".into()),
                to_node_id: LogicalId::Str("forward_pass".into()),
                reason: Some(RenderGraphEdgeReason::ReadAfterWrite),
            },
            RenderGraphEdge {
                from_node_id: LogicalId::Str("forward_pass".into()),
                to_node_id: LogicalId::Str("outline_pass".into()),
                reason: Some(RenderGraphEdgeReason::ReadAfterWrite),
            },
            RenderGraphEdge {
                from_node_id: LogicalId::Str("forward_pass".into()),
                to_node_id: LogicalId::Str("ssao_pass".into()),
                reason: Some(RenderGraphEdgeReason::ReadAfterWrite),
            },
            RenderGraphEdge {
                from_node_id: LogicalId::Str("ssao_pass".into()),
                to_node_id: LogicalId::Str("ssao_blur_pass".into()),
                reason: Some(RenderGraphEdgeReason::ReadAfterWrite),
            },
            RenderGraphEdge {
                from_node_id: LogicalId::Str("ssao_blur_pass".into()),
                to_node_id: LogicalId::Str("post_pass".into()),
                reason: Some(RenderGraphEdgeReason::ReadAfterWrite),
            },
            RenderGraphEdge {
                from_node_id: LogicalId::Str("outline_pass".into()),
                to_node_id: LogicalId::Str("post_pass".into()),
                reason: Some(RenderGraphEdgeReason::ReadAfterWrite),
            },
            RenderGraphEdge {
                from_node_id: LogicalId::Str("post_pass".into()),
                to_node_id: LogicalId::Str("compose_pass".into()),
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
