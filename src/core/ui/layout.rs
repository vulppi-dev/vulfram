use super::tree::UiStyle;
use super::types::UiValue;
use egui::scroll_area::ScrollBarVisibility;

#[derive(Clone, Copy, Debug)]
pub enum SizeSpec {
    Auto,
    Fill,
    Px(f32),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum UiLayoutKind {
    Row,
    ReverseRow,
    Col,
    ReverseCol,
    Grid,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum UiAlignKind {
    Start,
    Center,
    End,
    Stretch,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum UiJustifyKind {
    Start,
    Center,
    End,
    Stretch,
}

#[derive(Clone, Debug)]
pub struct UiStyleCacheEntry {
    pub layout_kind: UiLayoutKind,
    pub align: UiAlignKind,
    pub justify: UiJustifyKind,
    pub wrap: bool,
    pub gap: egui::Vec2,
    pub padding: egui::Margin,
    pub width: SizeSpec,
    pub height: SizeSpec,
    pub display: bool,
    pub visible: bool,
    pub opacity: f32,
    pub translate_y: f32,
    pub z_index: i32,
    pub scroll_x: bool,
    pub scroll_y: bool,
    pub scroll_bar_visibility: ScrollBarVisibility,
    pub font_size: Option<f32>,
    pub text_style: Option<String>,
}

pub fn build_style_cache(style: Option<&UiStyle>) -> UiStyleCacheEntry {
    let layout_value = style
        .and_then(|style| style.get("layout"))
        .and_then(ui_value_string)
        .unwrap_or_else(|| "col".into());
    let layout_kind = match layout_value.as_str() {
        "row" => UiLayoutKind::Row,
        "reverse-row" => UiLayoutKind::ReverseRow,
        "reverse-col" => UiLayoutKind::ReverseCol,
        "grid" => UiLayoutKind::Grid,
        _ => UiLayoutKind::Col,
    };

    let align = style
        .and_then(|style| style.get("align"))
        .and_then(ui_value_string)
        .map(parse_align)
        .unwrap_or(UiAlignKind::Start);
    let justify = style
        .and_then(|style| style.get("justify"))
        .and_then(ui_value_string)
        .map(parse_justify)
        .unwrap_or(UiJustifyKind::Start);
    let wrap = style
        .and_then(|style| style.get("wrap"))
        .and_then(ui_value_bool)
        .unwrap_or(false);

    let gap = resolve_gap(style);
    let padding = resolve_padding(style);
    let width = style
        .and_then(|style| style.get("width"))
        .map(parse_size)
        .unwrap_or(SizeSpec::Auto);
    let height = style
        .and_then(|style| style.get("height"))
        .map(parse_size)
        .unwrap_or(SizeSpec::Auto);

    let display = style
        .and_then(|style| style.get("display"))
        .and_then(ui_value_string)
        .map(|value| value != "none")
        .unwrap_or(true);
    let visible = style
        .and_then(|style| style.get("visible"))
        .and_then(ui_value_bool)
        .unwrap_or(true);
    let opacity = style
        .and_then(|style| style.get("opacity"))
        .and_then(ui_value_float)
        .unwrap_or(1.0)
        .clamp(0.0, 1.0);
    let translate_y = style
        .and_then(|style| style.get("translateY"))
        .and_then(ui_value_float)
        .unwrap_or(0.0);
    let z_index = style
        .and_then(|style| style.get("zIndex"))
        .and_then(ui_value_i32)
        .unwrap_or(0);
    let scroll_x = style
        .and_then(|style| style.get("scrollX"))
        .and_then(ui_value_bool)
        .unwrap_or(false);
    let scroll_y = style
        .and_then(|style| style.get("scrollY"))
        .and_then(ui_value_bool)
        .unwrap_or(true);
    let scroll_bar_visibility = style
        .and_then(|style| style.get("scrollbar"))
        .and_then(ui_value_string)
        .map(parse_scroll_visibility)
        .unwrap_or_default();
    let font_size = style
        .and_then(|style| style.get("fontSize"))
        .and_then(ui_value_float);
    let text_style = style
        .and_then(|style| style.get("textStyle"))
        .and_then(ui_value_string);

    UiStyleCacheEntry {
        layout_kind,
        align,
        justify,
        wrap,
        gap,
        padding,
        width,
        height,
        display,
        visible,
        opacity,
        translate_y,
        z_index,
        scroll_x,
        scroll_y,
        scroll_bar_visibility,
        font_size,
        text_style,
    }
}

pub fn apply_style_overrides(entry: &mut UiStyleCacheEntry, overrides: &UiStyle) {
    if let Some(value) = overrides.get("opacity").and_then(ui_value_float) {
        entry.opacity = value.clamp(0.0, 1.0);
    }
    if let Some(value) = overrides.get("translateY").and_then(ui_value_float) {
        entry.translate_y = value;
    }
}

pub fn resolve_layout_from_cache(entry: &UiStyleCacheEntry) -> (egui::Layout, bool) {
    if entry.layout_kind == UiLayoutKind::Grid {
        return (egui::Layout::top_down(egui::Align::Min), true);
    }

    let cross_align = align_from_kind(entry.align);
    let (main_align, main_justify) = justify_from_kind(entry.justify);
    let mut layout = match entry.layout_kind {
        UiLayoutKind::Row => egui::Layout::left_to_right(cross_align),
        UiLayoutKind::ReverseRow => egui::Layout::right_to_left(cross_align),
        UiLayoutKind::ReverseCol => egui::Layout::bottom_up(cross_align),
        UiLayoutKind::Col | UiLayoutKind::Grid => egui::Layout::top_down(cross_align),
    };
    layout = layout
        .with_main_wrap(entry.wrap)
        .with_main_align(main_align)
        .with_main_justify(main_justify);

    if entry.align == UiAlignKind::Stretch {
        layout = layout.with_cross_justify(true);
    }

    (layout, false)
}

pub fn resolve_gap(style: Option<&UiStyle>) -> egui::Vec2 {
    let gap = style
        .and_then(|style| style.get("gap"))
        .and_then(ui_value_float)
        .unwrap_or(4.0);
    let gap_x = style
        .and_then(|style| style.get("gapX"))
        .and_then(ui_value_float)
        .unwrap_or(gap);
    let gap_y = style
        .and_then(|style| style.get("gapY"))
        .and_then(ui_value_float)
        .unwrap_or(gap);
    egui::vec2(gap_x, gap_y)
}

pub fn resolve_padding(style: Option<&UiStyle>) -> egui::Margin {
    let padding = style
        .and_then(|style| style.get("padding"))
        .and_then(ui_value_float)
        .unwrap_or(0.0);
    let padding_x = style
        .and_then(|style| style.get("paddingX"))
        .and_then(ui_value_float)
        .unwrap_or(padding);
    let padding_y = style
        .and_then(|style| style.get("paddingY"))
        .and_then(ui_value_float)
        .unwrap_or(padding);
    egui::Margin {
        left: padding_x,
        right: padding_x,
        top: padding_y,
        bottom: padding_y,
    }
}

pub fn resolve_size_from_cache(
    ui: &egui::Ui,
    entry: &UiStyleCacheEntry,
) -> (f32, f32, bool) {
    let width = match entry.width {
        SizeSpec::Auto => ui.available_width(),
        SizeSpec::Fill => ui.available_width(),
        SizeSpec::Px(value) => value,
    };
    let height = match entry.height {
        SizeSpec::Auto => ui.available_height(),
        SizeSpec::Fill => ui.available_height(),
        SizeSpec::Px(value) => value,
    };
    let has_size = !matches!(entry.width, SizeSpec::Auto) || !matches!(entry.height, SizeSpec::Auto);
    (width.max(0.0), height.max(0.0), has_size)
}

fn align_from_kind(value: UiAlignKind) -> egui::Align {
    match value {
        UiAlignKind::Center => egui::Align::Center,
        UiAlignKind::End => egui::Align::Max,
        _ => egui::Align::Min,
    }
}

fn justify_from_kind(value: UiJustifyKind) -> (egui::Align, bool) {
    match value {
        UiJustifyKind::Center => (egui::Align::Center, false),
        UiJustifyKind::End => (egui::Align::Max, false),
        UiJustifyKind::Stretch => (egui::Align::Min, true),
        _ => (egui::Align::Min, false),
    }
}

fn parse_align(value: String) -> UiAlignKind {
    match value.as_str() {
        "center" => UiAlignKind::Center,
        "end" => UiAlignKind::End,
        "stretch" => UiAlignKind::Stretch,
        _ => UiAlignKind::Start,
    }
}

fn parse_justify(value: String) -> UiJustifyKind {
    match value.as_str() {
        "center" => UiJustifyKind::Center,
        "end" => UiJustifyKind::End,
        "stretch" | "fill" => UiJustifyKind::Stretch,
        _ => UiJustifyKind::Start,
    }
}

fn parse_scroll_visibility(value: String) -> ScrollBarVisibility {
    match value.as_str() {
        "hidden" => ScrollBarVisibility::AlwaysHidden,
        "visible" => ScrollBarVisibility::AlwaysVisible,
        _ => ScrollBarVisibility::VisibleWhenNeeded,
    }
}

fn parse_size(value: &UiValue) -> SizeSpec {
    match value {
        UiValue::String(value) => {
            let value = value.trim();
            if value.eq_ignore_ascii_case("auto") {
                SizeSpec::Auto
            } else if value.eq_ignore_ascii_case("fill") {
                SizeSpec::Fill
            } else if let Some(px) = value.strip_suffix("px") {
                px.trim()
                    .parse::<f32>()
                    .map(SizeSpec::Px)
                    .unwrap_or(SizeSpec::Auto)
            } else {
                value
                    .parse::<f32>()
                    .map(SizeSpec::Px)
                    .unwrap_or(SizeSpec::Auto)
            }
        }
        UiValue::Float(value) => SizeSpec::Px(*value as f32),
        UiValue::Int(value) => SizeSpec::Px(*value as f32),
        _ => SizeSpec::Auto,
    }
}

fn ui_value_string(value: &UiValue) -> Option<String> {
    match value {
        UiValue::String(value) => Some(value.clone()),
        _ => None,
    }
}

fn ui_value_float(value: &UiValue) -> Option<f32> {
    match value {
        UiValue::Float(value) => Some(*value as f32),
        UiValue::Int(value) => Some(*value as f32),
        _ => None,
    }
}

fn ui_value_bool(value: &UiValue) -> Option<bool> {
    match value {
        UiValue::Bool(value) => Some(*value),
        _ => None,
    }
}

fn ui_value_i32(value: &UiValue) -> Option<i32> {
    match value {
        UiValue::Int(value) => i32::try_from(*value).ok(),
        UiValue::Float(value) => {
            if *value >= i32::MIN as f64 && *value <= i32::MAX as f64 {
                Some(*value as i32)
            } else {
                None
            }
        }
        _ => None,
    }
}
