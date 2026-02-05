use crate::core::VulframResult;
use crate::core::cmd::{EngineCmd, EngineEvent};
use crate::core::render::graph::LogicalId;
use crate::core::resources::{
    CameraKind, CmdCameraCreateArgs, CmdMaterialCreateArgs, CmdModelCreateArgs,
    CmdPrimitiveGeometryCreateArgs, MaterialKind, MaterialOptions, MaterialSampler, PrimitiveShape,
    StandardOptions,
};
use crate::core::ui::cmd::{CmdUiApplyOpsArgs, CmdUiContextCreateArgs, CmdUiThemeDefineArgs};
use crate::core::ui::tree::{UiOp, UiOpAdd, UiOpSet, UiSetMode};
use crate::core::ui::types::{UiRectPx, UiRenderTarget, UiThemeSource, UiValue};
use glam::{Mat4, Vec2, Vec3, Vec4};

use crate::demos::common::{
    create_point_light_cmd, receive_responses, run_loop_with_events, send_commands,
};

pub fn run(window_id: u32) -> bool {
    let camera_id: u32 = 20;
    let cube_geometry_id: u32 = 820;
    let cube_material_id: u32 = 821;
    let cube_model_id: u32 = 822;
    let camera_target_id: u32 = 900;

    let context_id = LogicalId::Str("viewport_demo".into());
    let theme_id = LogicalId::Str("ui_theme_viewport".into());

    let setup_cmds = vec![
        EngineCmd::CmdCameraCreate(CmdCameraCreateArgs {
            camera_id,
            label: Some("Viewport Camera".into()),
            transform: Mat4::look_at_rh(Vec3::new(0.0, 2.0, 6.0), Vec3::ZERO, Vec3::Y).inverse(),
            kind: CameraKind::Perspective,
            flags: 0,
            near_far: Vec2::new(0.1, 100.0),
            layer_mask: 0xFFFFFFFF,
            order: 0,
            layer: 0,
            target_texture_id: Some(LogicalId::Int(camera_target_id as i64)),
            view_position: None,
            ortho_scale: 10.0,
        }),
        create_point_light_cmd(window_id, 22, Vec4::new(2.0, 3.0, 4.0, 1.0)),
        EngineCmd::CmdMaterialCreate(CmdMaterialCreateArgs {
            window_id,
            material_id: cube_material_id,
            label: Some("Viewport Cube".into()),
            kind: MaterialKind::Standard,
            options: Some(MaterialOptions::Standard(StandardOptions {
                base_color: Vec4::new(0.2, 0.6, 0.9, 1.0),
                base_sampler: Some(MaterialSampler::LinearClamp),
                emissive_color: Vec4::ZERO,
                ..Default::default()
            })),
        }),
        EngineCmd::CmdPrimitiveGeometryCreate(CmdPrimitiveGeometryCreateArgs {
            window_id,
            geometry_id: cube_geometry_id,
            label: Some("Viewport Cube".into()),
            shape: PrimitiveShape::Cube,
            options: None,
        }),
        EngineCmd::CmdModelCreate(CmdModelCreateArgs {
            window_id,
            model_id: cube_model_id,
            label: Some("Viewport Cube".into()),
            geometry_id: cube_geometry_id,
            material_id: Some(cube_material_id),
            transform: Mat4::from_translation(Vec3::new(0.0, 0.5, 0.0))
                * Mat4::from_scale(Vec3::splat(1.5)),
            layer_mask: 0xFFFFFFFF,
            cast_shadow: true,
            receive_shadow: true,
            cast_outline: false,
            outline_color: Vec4::ZERO,
        }),
        EngineCmd::CmdUiThemeDefine(CmdUiThemeDefineArgs {
            theme_id: theme_id.clone(),
            source: UiThemeSource::InlineJson("{}".into()),
        }),
        EngineCmd::CmdUiContextCreate(CmdUiContextCreateArgs {
            window_id,
            context_id: context_id.clone(),
            theme_id: theme_id.clone(),
            target: UiRenderTarget::Screen,
            screen_rect: UiRectPx {
                x: 0.0,
                y: 0.0,
                w: 1280.0,
                h: 720.0,
            },
            z_index: Some(10),
        }),
        EngineCmd::CmdUiApplyOps(CmdUiApplyOpsArgs {
            context_id: context_id.clone(),
            base_version: 0,
            ops: build_ui_ops(camera_target_id, camera_id),
        }),
    ];

    assert_eq!(send_commands(setup_cmds), VulframResult::Success);
    let _ = receive_responses();

    run_loop_with_events(
        window_id,
        None,
        |total_ms, _delta_ms| {
            let rotation_angle = total_ms as f32 * 0.001;
            let cmds = vec![EngineCmd::CmdModelUpdate(
                crate::core::resources::CmdModelUpdateArgs {
                    window_id,
                    model_id: cube_model_id,
                    label: None,
                    geometry_id: None,
                    material_id: None,
                    transform: Some(
                        Mat4::from_translation(Vec3::new(0.0, 0.5, 0.0))
                            * Mat4::from_rotation_y(rotation_angle)
                            * Mat4::from_rotation_x(rotation_angle * 0.7)
                            * Mat4::from_scale(Vec3::splat(1.5)),
                    ),
                    layer_mask: None,
                    cast_shadow: None,
                    receive_shadow: None,
                    cast_outline: None,
                    outline_color: None,
                },
            )];
            if total_ms < 100 {
                println!("Sending update command with angle: {}", rotation_angle);
            }
            cmds
        },
        |event| match event {
            EngineEvent::Ui(ui) => {
                println!("UiEvent: {:?}", ui);
                false
            }
            _ => false,
        },
    )
}

fn build_ui_ops(camera_target_id: u32, camera_id: u32) -> Vec<UiOp> {
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

    // Container horizontal para viewports
    ops.push(UiOp::Add(UiOpAdd {
        parent: Some(LogicalId::Str("main".into())),
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

    // Painel lateral com viewport pequeno e controles
    ops.push(UiOp::Add(UiOpAdd {
        parent: Some(LogicalId::Str("viewports_row".into())),
        id: LogicalId::Str("sidebar".into()),
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
        props: None,
        listeners: None,
    }));

    ops.push(UiOp::Add(UiOpAdd {
        parent: Some(LogicalId::Str("sidebar".into())),
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
        parent: Some(LogicalId::Str("sidebar".into())),
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

    // Separador
    ops.push(UiOp::Add(UiOpAdd {
        parent: Some(LogicalId::Str("sidebar".into())),
        id: LogicalId::Str("sep1".into()),
        node_type: crate::core::ui::tree::UiNodeType::Separator,
        index: None,
        variant: None,
        style: None,
        props: None,
        listeners: None,
    }));

    // Info sobre a câmera
    ops.push(UiOp::Add(UiOpAdd {
        parent: Some(LogicalId::Str("sidebar".into())),
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
        parent: Some(LogicalId::Str("sidebar".into())),
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

    // Separador
    ops.push(UiOp::Add(UiOpAdd {
        parent: Some(LogicalId::Str("sidebar".into())),
        id: LogicalId::Str("sep2".into()),
        node_type: crate::core::ui::tree::UiNodeType::Separator,
        index: None,
        variant: None,
        style: None,
        props: None,
        listeners: None,
    }));

    // Botões de exemplo
    ops.push(UiOp::Add(UiOpAdd {
        parent: Some(LogicalId::Str("sidebar".into())),
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
        parent: Some(LogicalId::Str("sidebar".into())),
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
        parent: Some(LogicalId::Str("sidebar".into())),
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
