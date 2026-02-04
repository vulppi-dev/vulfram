use crate::core::VulframResult;
use crate::core::cmd::{EngineCmd, EngineEvent};
use crate::core::render::graph::LogicalId;
use crate::core::resources::{
    CmdMaterialCreateArgs, CmdModelCreateArgs, CmdPrimitiveGeometryCreateArgs,
    CmdTextureCreateSolidColorArgs, MaterialKind, MaterialOptions, MaterialSampler, PrimitiveShape,
    StandardOptions,
};
use crate::core::ui::cmd::{CmdUiApplyOpsArgs, CmdUiContextCreateArgs, CmdUiThemeDefineArgs};
use crate::core::ui::tree::{
    UiListeners, UiOp, UiOpAdd, UiOpSet, UiSetMode,
};
use crate::core::ui::types::{UiRectPx, UiRenderTarget, UiThemeSource, UiValue};
use glam::{Mat4, Vec3, Vec4};

use crate::demos::common::{
    create_camera_cmd, create_point_light_cmd, receive_responses, run_loop_with_events,
    send_commands,
};

pub fn run(window_id: u32) -> bool {
    let camera_id: u32 = 1;
    let plane_geometry_id: u32 = 700;
    let plane_model_id: u32 = 701;
    let plane_material_id: u32 = 702;
    let ui_texture_id: u32 = 750;
    let debug_texture_id: u32 = 751;

    let context_id = LogicalId::Str("ui_demo".into());
    let theme_id = LogicalId::Str("ui_theme".into());

    let setup_cmds = vec![
        create_camera_cmd(
            camera_id,
            "UI Camera",
            Mat4::look_at_rh(Vec3::new(0.0, 4.5, 10.0), Vec3::ZERO, Vec3::Y).inverse(),
        ),
        create_point_light_cmd(window_id, 2, Vec4::new(0.0, 3.0, 4.0, 1.0)),
        EngineCmd::CmdTextureCreateSolidColor(CmdTextureCreateSolidColorArgs {
            window_id,
            texture_id: debug_texture_id,
            label: Some("UI Debug Solid".into()),
            color: Vec4::new(0.1, 0.8, 0.2, 1.0),
            srgb: Some(true),
            mode: Default::default(),
            atlas_options: None,
        }),
        EngineCmd::CmdMaterialCreate(CmdMaterialCreateArgs {
            window_id,
            material_id: plane_material_id,
            label: Some("UI Material".into()),
            kind: MaterialKind::Standard,
            options: Some(MaterialOptions::Standard(StandardOptions {
                base_color: Vec4::ZERO,
                base_tex_id: Some(debug_texture_id),
                base_sampler: Some(MaterialSampler::LinearClamp),
                emissive_color: Vec4::ONE,
                emissive_tex_id: Some(ui_texture_id),
                emissive_sampler: Some(MaterialSampler::LinearClamp),
                ..Default::default()
            })),
        }),
        EngineCmd::CmdPrimitiveGeometryCreate(CmdPrimitiveGeometryCreateArgs {
            window_id,
            geometry_id: plane_geometry_id,
            label: Some("UI Plane".into()),
            shape: PrimitiveShape::Plane,
            options: None,
        }),
        EngineCmd::CmdModelCreate(CmdModelCreateArgs {
            window_id,
            model_id: plane_model_id,
            label: Some("UI Plane".into()),
            geometry_id: plane_geometry_id,
            material_id: Some(plane_material_id),
            transform: Mat4::from_translation(Vec3::new(0.0, 0.5, 0.0))
                * Mat4::from_euler(glam::EulerRot::XYZ, -0.6, 0.0, 0.0)
                * Mat4::from_scale(Vec3::splat(4.0)),
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
            target: UiRenderTarget::TextureId(LogicalId::Int(ui_texture_id as i64)),
            screen_rect: UiRectPx {
                x: 32.0,
                y: 32.0,
                w: 512.0,
                h: 512.0,
            },
            z_index: Some(10),
        }),
        EngineCmd::CmdUiApplyOps(CmdUiApplyOpsArgs {
            context_id: context_id.clone(),
            base_version: 0,
            ops: build_ui_ops(debug_texture_id),
        }),
    ];

    assert_eq!(send_commands(setup_cmds), VulframResult::Success);
    let _ = receive_responses();

    run_loop_with_events(
        window_id,
        None,
        |_total_ms, _delta_ms| vec![],
        |event| match event {
            EngineEvent::Ui(ui) => {
                println!("UiEvent: {:?}", ui);
                false
            }
            _ => false,
        },
    )
}

fn build_ui_ops(debug_texture_id: u32) -> Vec<UiOp> {
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
    style.insert("gap".to_string(), UiValue::Float(8.0));
    style.insert("padding".to_string(), UiValue::Float(12.0));
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

    ops.push(UiOp::Add(UiOpAdd {
        parent: Some(LogicalId::Str("main".into())),
        id: LogicalId::Str("preview".into()),
        node_type: crate::core::ui::tree::UiNodeType::Image,
        index: None,
        variant: None,
        style: Some(
            [
                ("width".to_string(), UiValue::Float(96.0)),
                ("height".to_string(), UiValue::Float(96.0)),
            ]
            .into_iter()
            .collect(),
        ),
        props: Some(
            [("textureId".to_string(), UiValue::Int(debug_texture_id as i64))]
                .into_iter()
                .collect(),
        ),
        listeners: None,
    }));

    ops.push(UiOp::Add(UiOpAdd {
        parent: Some(LogicalId::Str("main".into())),
        id: LogicalId::Str("title".into()),
        node_type: crate::core::ui::tree::UiNodeType::Text,
        index: None,
        variant: None,
        style: None,
        props: Some([
            ("value".to_string(), UiValue::String("Demo UI".into())),
        ]
        .into_iter()
        .collect()),
        listeners: None,
    }));

    ops.push(UiOp::Add(UiOpAdd {
        parent: Some(LogicalId::Str("main".into())),
        id: LogicalId::Str("name".into()),
        node_type: crate::core::ui::tree::UiNodeType::Input,
        index: None,
        variant: None,
        style: None,
        props: Some([
            ("value".to_string(), UiValue::String("".into())),
        ]
        .into_iter()
        .collect()),
        listeners: Some(UiListeners {
            on_click: None,
            on_change: None,
            on_change_commit: Some("NameChanged".into()),
            on_submit: Some("NameSubmit".into()),
            on_focus: None,
            on_blur: None,
        }),
    }));

    ops.push(UiOp::Add(UiOpAdd {
        parent: Some(LogicalId::Str("main".into())),
        id: LogicalId::Str("slider".into()),
        node_type: crate::core::ui::tree::UiNodeType::Slider,
        index: None,
        variant: None,
        style: None,
        props: Some([
            ("value".to_string(), UiValue::Float(0.5)),
            ("min".to_string(), UiValue::Float(0.0)),
            ("max".to_string(), UiValue::Float(1.0)),
        ]
        .into_iter()
        .collect()),
        listeners: Some(UiListeners {
            on_click: None,
            on_change: Some("SliderChanged".into()),
            on_change_commit: None,
            on_submit: None,
            on_focus: None,
            on_blur: None,
        }),
    }));

    ops.push(UiOp::Add(UiOpAdd {
        parent: Some(LogicalId::Str("main".into())),
        id: LogicalId::Str("flag".into()),
        node_type: crate::core::ui::tree::UiNodeType::Checkbox,
        index: None,
        variant: None,
        style: None,
        props: Some([
            ("value".to_string(), UiValue::Bool(false)),
        ]
        .into_iter()
        .collect()),
        listeners: Some(UiListeners {
            on_click: None,
            on_change: Some("FlagChanged".into()),
            on_change_commit: None,
            on_submit: None,
            on_focus: None,
            on_blur: None,
        }),
    }));

    ops.push(UiOp::Add(UiOpAdd {
        parent: Some(LogicalId::Str("main".into())),
        id: LogicalId::Str("submit".into()),
        node_type: crate::core::ui::tree::UiNodeType::Button,
        index: None,
        variant: None,
        style: None,
        props: Some([
            ("label".to_string(), UiValue::String("Submit".into())),
        ]
        .into_iter()
        .collect()),
        listeners: Some(UiListeners {
            on_click: Some("Submit".into()),
            on_change: None,
            on_change_commit: None,
            on_submit: None,
            on_focus: None,
            on_blur: None,
        }),
    }));

    ops
}
