use crate::core::render::graph::LogicalId;

use crate::core::ui::tree::UiTreeState;
use crate::core::ui::types::UiValue;

pub(super) fn ui_value_string(value: &UiValue) -> Option<String> {
    match value {
        UiValue::String(value) => Some(value.clone()),
        _ => None,
    }
}

pub(super) fn ui_value_bool(value: &UiValue) -> Option<bool> {
    match value {
        UiValue::Bool(value) => Some(*value),
        _ => None,
    }
}

pub(super) fn ui_value_float(value: &UiValue) -> Option<f32> {
    match value {
        UiValue::Float(value) => Some(*value as f32),
        UiValue::Int(value) => Some(*value as f32),
        _ => None,
    }
}

pub(super) fn ui_value_u32(value: &UiValue) -> Option<u32> {
    match value {
        UiValue::Int(value) => u32::try_from(*value).ok(),
        UiValue::Float(value) => {
            if *value >= 0.0 && *value <= u32::MAX as f64 {
                Some(*value as u32)
            } else {
                None
            }
        }
        _ => None,
    }
}

pub(super) fn update_node_prop(
    tree: &mut UiTreeState,
    node_id: &LogicalId,
    key: &str,
    value: UiValue,
) {
    if let Some(node) = tree.nodes.get_mut(node_id) {
        match node.props.as_mut() {
            Some(props) => {
                props.insert(key.to_string(), value);
            }
            None => {
                let mut props = std::collections::HashMap::new();
                props.insert(key.to_string(), value);
                node.props = Some(props);
            }
        }
    }
}
