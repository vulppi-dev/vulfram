use crate::core::render::graph::LogicalId;
use crate::core::ui::tree::{UiOp, UiOpAdd, UiOpSet, UiSetMode};
use crate::core::ui::types::UiValue;

pub fn build_ui_ops(camera_target_id: u32, camera_id: u32) -> Vec<UiOp> {
    let mut ops = Vec::new();

    ops.push(UiOp::Add(UiOpAdd {
        parent: None,
        id: LogicalId::Str("main".into()),
        node_type: crate::core::ui::tree::UiNodeType::Container,
        index: None,
        variant: None,
        style: None,
        props: None,
        listeners: None,
    }));

    let mut style = std::collections::HashMap::new();
    style.insert("layout".to_string(), UiValue::String("col".into()));
    style.insert("gap".to_string(), UiValue::Float(20.0));
    style.insert("padding".to_string(), UiValue::Float(24.0));
    style.insert("width".to_string(), UiValue::String("fill".into()));
    style.insert("height".to_string(), UiValue::String("fill".into()));
    ops.push(UiOp::Set(UiOpSet {
        id: LogicalId::Str("main".into()),
        mode: UiSetMode::Merge,
        variant: None,
        style: Some(Some(style)),
        props: None,
        listeners: None,
    }));

    // Título principal
    ops.push(UiOp::Add(UiOpAdd {
        parent: Some(LogicalId::Str("main".into())),
        id: LogicalId::Str("title".into()),
        node_type: crate::core::ui::tree::UiNodeType::Text,
        index: None,
        variant: None,
        style: None,
        props: Some(
            [(
                "value".to_string(),
                UiValue::String("🎥 Virtual Viewports Demo".into()),
            )]
            .into_iter()
            .collect(),
        ),
        listeners: None,
    }));

    // Descrição
    ops.push(UiOp::Add(UiOpAdd {
        parent: Some(LogicalId::Str("main".into())),
        id: LogicalId::Str("desc".into()),
        node_type: crate::core::ui::tree::UiNodeType::Text,
        index: None,
        variant: None,
        style: None,
        props: Some(
            [(
                "value".to_string(),
                UiValue::String(
                    "Câmeras renderizando para texturas e exibidas em widgets Image".into(),
                ),
            )]
            .into_iter()
            .collect(),
        ),
        listeners: None,
    }));

    // Dock root
    ops.push(UiOp::Add(UiOpAdd {
        parent: Some(LogicalId::Str("main".into())),
        id: LogicalId::Str("dock_root".into()),
        node_type: crate::core::ui::tree::UiNodeType::Dock,
        index: None,
        variant: None,
        style: None,
        props: Some(
            [("activeIndex".to_string(), UiValue::Int(0))]
                .into_iter()
                .collect(),
        ),
        listeners: Some(crate::core::ui::tree::UiListeners {
            on_change: Some("dock_change".into()),
            ..Default::default()
        }),
    }));

    // Tab: Viewports
    ops.push(UiOp::Add(UiOpAdd {
        parent: Some(LogicalId::Str("dock_root".into())),
        id: LogicalId::Str("tab_viewports".into()),
        node_type: crate::core::ui::tree::UiNodeType::Container,
        index: None,
        variant: None,
        style: Some(
            [
                ("layout".to_string(), UiValue::String("col".into())),
                ("gap".to_string(), UiValue::Float(12.0)),
            ]
            .into_iter()
            .collect(),
        ),
        props: Some(
            [("title".to_string(), UiValue::String("Viewports".into()))]
                .into_iter()
                .collect(),
        ),
        listeners: None,
    }));

    // Tab: Controles
    ops.push(UiOp::Add(UiOpAdd {
        parent: Some(LogicalId::Str("dock_root".into())),
        id: LogicalId::Str("tab_controls".into()),
        node_type: crate::core::ui::tree::UiNodeType::Container,
        index: None,
        variant: None,
        style: Some(
            [
                ("layout".to_string(), UiValue::String("col".into())),
                ("gap".to_string(), UiValue::Float(12.0)),
            ]
            .into_iter()
            .collect(),
        ),
        props: Some(
            [("title".to_string(), UiValue::String("Controles".into()))]
                .into_iter()
                .collect(),
        ),
        listeners: None,
    }));

    // Container horizontal para viewports
    ops.push(UiOp::Add(UiOpAdd {
        parent: Some(LogicalId::Str("tab_viewports".into())),
        id: LogicalId::Str("viewports_row".into()),
        node_type: crate::core::ui::tree::UiNodeType::Container,
        index: None,
        variant: None,
        style: Some(
            [
                ("layout".to_string(), UiValue::String("row".into())),
                ("gap".to_string(), UiValue::Float(16.0)),
            ]
            .into_iter()
            .collect(),
        ),
        props: None,
        listeners: None,
    }));

    // Container do viewport grande
    ops.push(UiOp::Add(UiOpAdd {
        parent: Some(LogicalId::Str("viewports_row".into())),
        id: LogicalId::Str("large_viewport_container".into()),
        node_type: crate::core::ui::tree::UiNodeType::Container,
        index: None,
        variant: None,
        style: Some(
            [
                ("layout".to_string(), UiValue::String("col".into())),
                ("gap".to_string(), UiValue::Float(8.0)),
            ]
            .into_iter()
            .collect(),
        ),
        props: None,
        listeners: None,
    }));

    ops.push(UiOp::Add(UiOpAdd {
        parent: Some(LogicalId::Str("large_viewport_container".into())),
        id: LogicalId::Str("large_label".into()),
        node_type: crate::core::ui::tree::UiNodeType::Text,
        index: None,
        variant: None,
        style: None,
        props: Some(
            [(
                "value".to_string(),
                UiValue::String("📺 Viewport Principal (512x512)".into()),
            )]
            .into_iter()
            .collect(),
        ),
        listeners: None,
    }));

    ops.push(UiOp::Add(UiOpAdd {
        parent: Some(LogicalId::Str("large_viewport_container".into())),
        id: LogicalId::Str("viewport".into()),
        node_type: crate::core::ui::tree::UiNodeType::Image,
        index: None,
        variant: None,
        style: Some(
            [
                ("width".to_string(), UiValue::Float(512.0)),
                ("height".to_string(), UiValue::Float(512.0)),
            ]
            .into_iter()
            .collect(),
        ),
        props: Some(
            [
                (
                    "textureId".to_string(),
                    UiValue::Int(camera_target_id as i64),
                ),
                ("cameraId".to_string(), UiValue::Int(camera_id as i64)),
            ]
            .into_iter()
            .collect(),
        ),
        listeners: Some(crate::core::ui::tree::UiListeners {
            on_viewport_hover: Some("viewport_hover".into()),
            on_viewport_click: Some("viewport_click".into()),
            on_viewport_drag: Some("viewport_drag".into()),
            on_viewport_drag_end: Some("viewport_drag_end".into()),
            ..Default::default()
        }),
    }));

    // Mini viewport
    ops.push(UiOp::Add(UiOpAdd {
        parent: Some(LogicalId::Str("viewports_row".into())),
        id: LogicalId::Str("mini_viewport_container".into()),
        node_type: crate::core::ui::tree::UiNodeType::Container,
        index: None,
        variant: None,
        style: Some(
            [
                ("layout".to_string(), UiValue::String("col".into())),
                ("gap".to_string(), UiValue::Float(8.0)),
            ]
            .into_iter()
            .collect(),
        ),
        props: None,
        listeners: None,
    }));

    ops.push(UiOp::Add(UiOpAdd {
        parent: Some(LogicalId::Str("mini_viewport_container".into())),
        id: LogicalId::Str("small_label".into()),
        node_type: crate::core::ui::tree::UiNodeType::Text,
        index: None,
        variant: None,
        style: None,
        props: Some(
            [(
                "value".to_string(),
                UiValue::String("🔍 Mini Viewport (256x256)".into()),
            )]
            .into_iter()
            .collect(),
        ),
        listeners: None,
    }));

    ops.push(UiOp::Add(UiOpAdd {
        parent: Some(LogicalId::Str("mini_viewport_container".into())),
        id: LogicalId::Str("viewport2".into()),
        node_type: crate::core::ui::tree::UiNodeType::Image,
        index: None,
        variant: None,
        style: Some(
            [
                ("width".to_string(), UiValue::Float(256.0)),
                ("height".to_string(), UiValue::Float(256.0)),
            ]
            .into_iter()
            .collect(),
        ),
        props: Some(
            [
                (
                    "textureId".to_string(),
                    UiValue::Int(camera_target_id as i64),
                ),
                ("cameraId".to_string(), UiValue::Int(camera_id as i64)),
            ]
            .into_iter()
            .collect(),
        ),
        listeners: None,
    }));

    // Info e controles no tab separado
    ops.push(UiOp::Add(UiOpAdd {
        parent: Some(LogicalId::Str("tab_controls".into())),
        id: LogicalId::Str("info_title".into()),
        node_type: crate::core::ui::tree::UiNodeType::Text,
        index: None,
        variant: None,
        style: None,
        props: Some(
            [(
                "value".to_string(),
                UiValue::String("ℹ️ Info da Câmera".into()),
            )]
            .into_iter()
            .collect(),
        ),
        listeners: None,
    }));

    ops.push(UiOp::Add(UiOpAdd {
        parent: Some(LogicalId::Str("tab_controls".into())),
        id: LogicalId::Str("camera_info".into()),
        node_type: crate::core::ui::tree::UiNodeType::Text,
        index: None,
        variant: None,
        style: None,
        props: Some(
            [(
                "value".to_string(),
                UiValue::String(format!(
                    "ID: {}\nTarget: {}\nTipo: Perspective",
                    camera_id, camera_target_id
                )),
            )]
            .into_iter()
            .collect(),
        ),
        listeners: None,
    }));

    ops.push(UiOp::Add(UiOpAdd {
        parent: Some(LogicalId::Str("tab_controls".into())),
        id: LogicalId::Str("sep2".into()),
        node_type: crate::core::ui::tree::UiNodeType::Separator,
        index: None,
        variant: None,
        style: None,
        props: None,
        listeners: None,
    }));

    ops.push(UiOp::Add(UiOpAdd {
        parent: Some(LogicalId::Str("tab_controls".into())),
        id: LogicalId::Str("btn_label".into()),
        node_type: crate::core::ui::tree::UiNodeType::Text,
        index: None,
        variant: None,
        style: None,
        props: Some(
            [("value".to_string(), UiValue::String("🎮 Controles".into()))]
                .into_iter()
                .collect(),
        ),
        listeners: None,
    }));

    ops.push(UiOp::Add(UiOpAdd {
        parent: Some(LogicalId::Str("tab_controls".into())),
        id: LogicalId::Str("btn1".into()),
        node_type: crate::core::ui::tree::UiNodeType::Button,
        index: None,
        variant: None,
        style: None,
        props: Some(
            [(
                "label".to_string(),
                UiValue::String("Botão Exemplo 1".into()),
            )]
            .into_iter()
            .collect(),
        ),
        listeners: None,
    }));

    ops.push(UiOp::Add(UiOpAdd {
        parent: Some(LogicalId::Str("tab_controls".into())),
        id: LogicalId::Str("btn2".into()),
        node_type: crate::core::ui::tree::UiNodeType::Button,
        index: None,
        variant: None,
        style: None,
        props: Some(
            [(
                "label".to_string(),
                UiValue::String("Botão Exemplo 2".into()),
            )]
            .into_iter()
            .collect(),
        ),
        listeners: None,
    }));

    ops
}
