use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::core::render::graph::LogicalId;

use super::types::UiValue;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum UiNodeType {
    Container,
    Text,
    Button,
    Input,
    Slider,
    Checkbox,
    Select,
    Image,
    Scroll,
    Separator,
    Spacer,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UiNode {
    pub id: LogicalId,
    pub parent: Option<LogicalId>,
    pub children: Vec<LogicalId>,
    pub node_type: UiNodeType,
    pub variant: Option<String>,
    pub style: Option<UiStyle>,
    pub props: Option<UiProps>,
    pub listeners: Option<UiListeners>,
}

impl UiNode {
    pub fn new_root() -> Self {
        UiNode {
            id: LogicalId::Str("root".into()),
            parent: None,
            children: Vec::new(),
            node_type: UiNodeType::Container,
            variant: None,
            style: None,
            props: None,
            listeners: None,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UiListeners {
    pub on_click: Option<String>,
    pub on_change: Option<String>,
    pub on_change_commit: Option<String>,
    pub on_submit: Option<String>,
    pub on_focus: Option<String>,
    pub on_blur: Option<String>,
}

pub type UiStyle = HashMap<String, UiValue>;
pub type UiProps = HashMap<String, UiValue>;

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum UiSetMode {
    Merge,
    Replace,
}

impl Default for UiSetMode {
    fn default() -> Self {
        UiSetMode::Merge
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum UiEventKind {
    Click,
    Change,
    Submit,
    Focus,
    Blur,
    AnimComplete,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "op", rename_all = "kebab-case")]
pub enum UiOp {
    Add(UiOpAdd),
    Remove(UiOpRemove),
    Clear(UiOpClear),
    Move(UiOpMove),
    Set(UiOpSet),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UiOpAdd {
    pub parent: Option<LogicalId>,
    pub id: LogicalId,
    pub node_type: UiNodeType,
    pub index: Option<u32>,
    pub variant: Option<String>,
    pub style: Option<UiStyle>,
    pub props: Option<UiProps>,
    pub listeners: Option<UiListeners>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UiOpRemove {
    pub id: LogicalId,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UiOpClear {
    pub id: LogicalId,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UiOpMove {
    pub id: LogicalId,
    pub parent: Option<LogicalId>,
    pub index: Option<u32>,
    pub step: Option<i32>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UiOpSet {
    pub id: LogicalId,
    #[serde(default)]
    pub mode: UiSetMode,
    pub variant: Option<Option<String>>,
    pub style: Option<Option<UiStyle>>,
    pub props: Option<Option<UiProps>>,
    pub listeners: Option<Option<UiListeners>>,
}

#[derive(Debug, Clone, Default)]
pub struct UiTreeState {
    pub nodes: HashMap<LogicalId, UiNode>,
    pub version: u32,
}

impl UiTreeState {
    pub fn with_root() -> Self {
        let mut nodes = HashMap::new();
        nodes.insert(LogicalId::Str("root".into()), UiNode::new_root());
        UiTreeState { nodes, version: 0 }
    }
}

pub fn apply_ops(tree: &mut UiTreeState, ops: &[UiOp]) -> Result<(), String> {
    for op in ops {
        match op {
            UiOp::Add(payload) => apply_add(tree, payload)?,
            UiOp::Remove(payload) => apply_remove(tree, payload)?,
            UiOp::Clear(payload) => apply_clear(tree, payload)?,
            UiOp::Move(payload) => apply_move(tree, payload)?,
            UiOp::Set(payload) => apply_set(tree, payload)?,
        }
    }
    Ok(())
}

fn resolve_parent_id(parent: &Option<LogicalId>) -> LogicalId {
    parent
        .clone()
        .unwrap_or_else(|| LogicalId::Str("root".into()))
}

fn apply_add(tree: &mut UiTreeState, op: &UiOpAdd) -> Result<(), String> {
    if tree.nodes.contains_key(&op.id) {
        return Err(format!("Node {} already exists", op.id));
    }
    let parent_id = resolve_parent_id(&op.parent);
    let parent = match tree.nodes.get_mut(&parent_id) {
        Some(node) => node,
        None => return Err(format!("Parent {} not found", parent_id)),
    };

    let node = UiNode {
        id: op.id.clone(),
        parent: Some(parent_id.clone()),
        children: Vec::new(),
        node_type: op.node_type.clone(),
        variant: op.variant.clone(),
        style: op.style.clone(),
        props: op.props.clone(),
        listeners: op.listeners.clone(),
    };

    insert_child(parent, op.id.clone(), op.index);
    tree.nodes.insert(op.id.clone(), node);
    Ok(())
}

fn apply_remove(tree: &mut UiTreeState, op: &UiOpRemove) -> Result<(), String> {
    if op.id == LogicalId::Str("root".into()) {
        return Err("Cannot remove root node".into());
    }
    remove_subtree(tree, &op.id)
}

fn apply_clear(tree: &mut UiTreeState, op: &UiOpClear) -> Result<(), String> {
    let children = {
        let node = match tree.nodes.get(&op.id) {
            Some(n) => n,
            None => return Err(format!("Node {} not found", op.id)),
        };
        node.children.clone()
    };

    for child in children {
        remove_subtree(tree, &child)?;
    }
    Ok(())
}

fn apply_move(tree: &mut UiTreeState, op: &UiOpMove) -> Result<(), String> {
    let node_parent_id = {
        let node = match tree.nodes.get(&op.id) {
            Some(n) => n,
            None => return Err(format!("Node {} not found", op.id)),
        };
        node.parent
            .clone()
            .ok_or_else(|| "Root cannot be moved".to_string())?
    };

    let target_parent_id = op.parent.clone().unwrap_or(node_parent_id.clone());

    if !tree.nodes.contains_key(&target_parent_id) {
        return Err(format!("Parent {} not found", target_parent_id));
    }

    if node_parent_id != target_parent_id {
        if is_descendant(tree, &op.id, &target_parent_id) {
            return Err("Cannot move node into its own subtree".into());
        }
        detach_child(tree, &node_parent_id, &op.id)?;
        attach_child(tree, &target_parent_id, &op.id, op.index)?;
        if let Some(node) = tree.nodes.get_mut(&op.id) {
            node.parent = Some(target_parent_id);
        }
        return Ok(());
    }

    reorder_child(tree, &node_parent_id, &op.id, op.index, op.step)?;
    Ok(())
}

fn apply_set(tree: &mut UiTreeState, op: &UiOpSet) -> Result<(), String> {
    let node = match tree.nodes.get_mut(&op.id) {
        Some(n) => n,
        None => return Err(format!("Node {} not found", op.id)),
    };

    if let Some(variant) = &op.variant {
        node.variant = variant.clone();
    }

    if let Some(style) = &op.style {
        match style {
            None => node.style = None,
            Some(map) => match op.mode {
                UiSetMode::Merge => merge_map(&mut node.style, Some(map.clone())),
                UiSetMode::Replace => node.style = Some(map.clone()),
            },
        }
    }

    if let Some(props) = &op.props {
        match props {
            None => node.props = None,
            Some(map) => match op.mode {
                UiSetMode::Merge => merge_map(&mut node.props, Some(map.clone())),
                UiSetMode::Replace => node.props = Some(map.clone()),
            },
        }
    }

    if let Some(listeners) = &op.listeners {
        match listeners {
            None => node.listeners = None,
            Some(values) => match op.mode {
                UiSetMode::Merge => merge_listeners(&mut node.listeners, Some(values.clone())),
                UiSetMode::Replace => node.listeners = Some(values.clone()),
            },
        }
    }

    Ok(())
}

fn merge_map(
    current: &mut Option<HashMap<String, UiValue>>,
    update: Option<HashMap<String, UiValue>>,
) {
    match (current.as_mut(), update) {
        (None, Some(map)) => {
            *current = Some(map);
        }
        (Some(existing), Some(map)) => {
            for (key, value) in map {
                existing.insert(key, value);
            }
        }
        (_, None) => {}
    }
}

fn merge_listeners(current: &mut Option<UiListeners>, update: Option<UiListeners>) {
    match (current.as_mut(), update) {
        (None, Some(listeners)) => {
            *current = Some(listeners);
        }
        (Some(existing), Some(listeners)) => {
            if listeners.on_click.is_some() {
                existing.on_click = listeners.on_click;
            }
            if listeners.on_change.is_some() {
                existing.on_change = listeners.on_change;
            }
            if listeners.on_change_commit.is_some() {
                existing.on_change_commit = listeners.on_change_commit;
            }
            if listeners.on_submit.is_some() {
                existing.on_submit = listeners.on_submit;
            }
            if listeners.on_focus.is_some() {
                existing.on_focus = listeners.on_focus;
            }
            if listeners.on_blur.is_some() {
                existing.on_blur = listeners.on_blur;
            }
        }
        (_, None) => {}
    }
}

fn insert_child(parent: &mut UiNode, child_id: LogicalId, index: Option<u32>) {
    let insert_index = index
        .map(|value| value as usize)
        .unwrap_or_else(|| parent.children.len());
    let insert_index = insert_index.min(parent.children.len());
    parent.children.insert(insert_index, child_id);
}

fn detach_child(
    tree: &mut UiTreeState,
    parent_id: &LogicalId,
    child_id: &LogicalId,
) -> Result<(), String> {
    let parent = match tree.nodes.get_mut(parent_id) {
        Some(node) => node,
        None => return Err(format!("Parent {} not found", parent_id)),
    };
    if let Some(index) = parent.children.iter().position(|id| id == child_id) {
        parent.children.remove(index);
        return Ok(());
    }
    Err(format!(
        "Node {} not found in parent {}",
        child_id, parent_id
    ))
}

fn attach_child(
    tree: &mut UiTreeState,
    parent_id: &LogicalId,
    child_id: &LogicalId,
    index: Option<u32>,
) -> Result<(), String> {
    let parent = match tree.nodes.get_mut(parent_id) {
        Some(node) => node,
        None => return Err(format!("Parent {} not found", parent_id)),
    };
    insert_child(parent, child_id.clone(), index);
    Ok(())
}

fn reorder_child(
    tree: &mut UiTreeState,
    parent_id: &LogicalId,
    child_id: &LogicalId,
    index: Option<u32>,
    step: Option<i32>,
) -> Result<(), String> {
    let parent = match tree.nodes.get_mut(parent_id) {
        Some(node) => node,
        None => return Err(format!("Parent {} not found", parent_id)),
    };

    let old_index = match parent.children.iter().position(|id| id == child_id) {
        Some(idx) => idx,
        None => {
            return Err(format!(
                "Node {} not found in parent {}",
                child_id, parent_id
            ));
        }
    };

    let mut new_index = old_index;
    if let Some(target) = index {
        new_index = target as usize;
    } else if let Some(delta) = step {
        let max_index = parent.children.len().saturating_sub(1) as i32;
        let desired = old_index as i32 + delta;
        let clamped = desired.clamp(0, max_index);
        new_index = clamped as usize;
    }

    new_index = new_index.min(parent.children.len().saturating_sub(1));
    if new_index == old_index {
        return Ok(());
    }

    parent.children.remove(old_index);
    parent.children.insert(new_index, child_id.clone());
    Ok(())
}

fn remove_subtree(tree: &mut UiTreeState, node_id: &LogicalId) -> Result<(), String> {
    let children = {
        let node = match tree.nodes.get(node_id) {
            Some(n) => n,
            None => return Err(format!("Node {} not found", node_id)),
        };
        node.children.clone()
    };

    for child in children {
        remove_subtree(tree, &child)?;
    }

    let parent_id = match tree.nodes.get(node_id).and_then(|n| n.parent.clone()) {
        Some(pid) => pid,
        None => return Err("Cannot remove root node".into()),
    };
    detach_child(tree, &parent_id, node_id)?;
    tree.nodes.remove(node_id);
    Ok(())
}

fn is_descendant(tree: &UiTreeState, node_id: &LogicalId, candidate_parent: &LogicalId) -> bool {
    let mut stack = vec![node_id.clone()];
    while let Some(current) = stack.pop() {
        if &current == candidate_parent {
            return true;
        }
        if let Some(node) = tree.nodes.get(&current) {
            for child in &node.children {
                stack.push(child.clone());
            }
        }
    }
    false
}
