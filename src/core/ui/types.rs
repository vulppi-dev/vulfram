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
#[serde(rename_all = "camelCase")]
pub struct UiThemeConfig {
    #[serde(default)]
    pub fonts: Vec<UiThemeFont>,
    #[serde(default)]
    pub font_families: std::collections::HashMap<String, Vec<String>>,
    #[serde(default)]
    pub text_styles: std::collections::HashMap<String, UiTextStyle>,
    pub debug: Option<bool>,
}

impl Default for UiThemeConfig {
    fn default() -> Self {
        Self {
            fonts: Vec::new(),
            font_families: std::collections::HashMap::new(),
            text_styles: std::collections::HashMap::new(),
            debug: None,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UiThemeFont {
    pub name: String,
    pub data: Vec<u8>,
    pub family: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UiTextStyle {
    pub size: f32,
    pub family: Option<String>,
}
