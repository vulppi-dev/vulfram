use serde::{Deserialize, Serialize};

use crate::core::render::graph::LogicalId;

use super::tree::UiEventKind;
use super::types::UiValue;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UiEvent {
    pub window_id: LogicalId,
    pub context_id: LogicalId,
    pub label: String,
    pub kind: UiEventKind,
    pub node_id: Option<LogicalId>,
    pub value: Option<UiValue>,
}
