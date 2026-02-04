use serde::{Deserialize, Serialize};

use crate::core::render::graph::LogicalId;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum UiValue {
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UiRectPx {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum UiRenderTarget {
    TextureId(LogicalId),
    Screen,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum UiThemeSource {
    InlineJson(String),
    InlineMsgPack(Vec<u8>),
    AssetRef(LogicalId),
}
