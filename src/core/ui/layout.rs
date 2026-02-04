use super::tree::UiStyle;
use super::types::UiValue;

#[derive(Clone, Copy)]
pub enum SizeSpec {
    Auto,
    Fill,
    Px(f32),
}

pub fn resolve_layout(
    layout_value: &str,
    style: Option<&UiStyle>,
    wrap: bool,
) -> (egui::Layout, bool) {
    if layout_value == "grid" {
        return (egui::Layout::top_down(egui::Align::Min), true);
    }

    let align_value = style
        .and_then(|style| style.get("align"))
        .and_then(ui_value_string)
        .unwrap_or_else(|| "start".into());
    let justify_value = style
        .and_then(|style| style.get("justify"))
        .and_then(ui_value_string)
        .unwrap_or_else(|| "start".into());

    let cross_align = align_from_string(&align_value);
    let (main_align, main_justify) = justify_from_string(&justify_value);
    let mut layout = match layout_value {
        "row" => egui::Layout::left_to_right(cross_align),
        "reverse-row" => egui::Layout::right_to_left(cross_align),
        "reverse-col" => egui::Layout::bottom_up(cross_align),
        _ => egui::Layout::top_down(cross_align),
    };
    layout = layout
        .with_main_wrap(wrap)
        .with_main_align(main_align)
        .with_main_justify(main_justify);

    if align_value == "stretch" {
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

pub fn resolve_size(ui: &egui::Ui, style: Option<&UiStyle>) -> (f32, f32, bool) {
    let width_spec = style
        .and_then(|style| style.get("width"))
        .map(parse_size)
        .unwrap_or(SizeSpec::Auto);
    let height_spec = style
        .and_then(|style| style.get("height"))
        .map(parse_size)
        .unwrap_or(SizeSpec::Auto);

    let width = match width_spec {
        SizeSpec::Auto => ui.available_width(),
        SizeSpec::Fill => ui.available_width(),
        SizeSpec::Px(value) => value,
    };
    let height = match height_spec {
        SizeSpec::Auto => ui.available_height(),
        SizeSpec::Fill => ui.available_height(),
        SizeSpec::Px(value) => value,
    };
    let has_size = !matches!(width_spec, SizeSpec::Auto) || !matches!(height_spec, SizeSpec::Auto);
    (width.max(0.0), height.max(0.0), has_size)
}

fn align_from_string(value: &str) -> egui::Align {
    match value {
        "center" => egui::Align::Center,
        "end" => egui::Align::Max,
        _ => egui::Align::Min,
    }
}

fn justify_from_string(value: &str) -> (egui::Align, bool) {
    match value {
        "center" => (egui::Align::Center, false),
        "end" => (egui::Align::Max, false),
        "stretch" | "fill" => (egui::Align::Min, true),
        _ => (egui::Align::Min, false),
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
