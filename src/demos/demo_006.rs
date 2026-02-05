use crate::core::VulframResult;
use crate::core::cmd::{EngineCmd, EngineEvent};
use crate::core::render::graph::LogicalId;
use crate::core::resources::{CameraKind, CmdCameraCreateArgs};
use crate::core::ui::cmd::{CmdUiApplyOpsArgs, CmdUiContextCreateArgs, CmdUiThemeDefineArgs};
use crate::core::ui::tree::{UiListeners, UiOp, UiOpAdd, UiOpSet, UiSetMode};
use crate::core::ui::types::{UiRectPx, UiRenderTarget, UiThemeSource, UiValue};
use glam::{Mat4, Vec2, Vec3};

use crate::demos::common::{receive_responses, run_loop_with_events, send_commands};

pub fn run(window_id: u32) -> bool {
    let camera_id: u32 = 10;
    let context_id = LogicalId::Str("ui_demo_screen".into());
    let theme_id = LogicalId::Str("ui_theme_screen".into());

    let setup_cmds = vec![
        EngineCmd::CmdCameraCreate(CmdCameraCreateArgs {
            camera_id,
            label: Some("UI Screen Camera".into()),
            transform: Mat4::look_at_rh(Vec3::new(0.0, 2.0, 6.0), Vec3::ZERO, Vec3::Y).inverse(),
            kind: CameraKind::Perspective,
            flags: 0,
            near_far: Vec2::new(0.1, 100.0),
            layer_mask: 0xFFFFFFFF,
            order: 0,
            layer: 0,
            target_texture_id: None,
            view_position: None,
            ortho_scale: 10.0,
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
            ops: build_ui_ops(),
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

fn build_ui_ops() -> Vec<UiOp> {
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
    style.insert("gap".to_string(), UiValue::Float(10.0));
    style.insert("padding".to_string(), UiValue::Float(16.0));
    style.insert("width".to_string(), UiValue::String("fill".into()));
    style.insert("height".to_string(), UiValue::String("fill".into()));
    style.insert("align".to_string(), UiValue::String("start".into()));
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
        id: LogicalId::Str("title".into()),
        node_type: crate::core::ui::tree::UiNodeType::Text,
        index: None,
        variant: None,
        style: None,
        props: Some(
            [(
                "value".to_string(),
                UiValue::String("Demo 6: UI Screen".into()),
            )]
            .into_iter()
            .collect(),
        ),
        listeners: None,
    }));

    ops.push(UiOp::Add(UiOpAdd {
        parent: Some(LogicalId::Str("main".into())),
        id: LogicalId::Str("name".into()),
        node_type: crate::core::ui::tree::UiNodeType::Input,
        index: None,
        variant: None,
        style: None,
        props: Some(
            [("value".to_string(), UiValue::String("".into()))]
                .into_iter()
                .collect(),
        ),
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
        props: Some(
            [
                ("value".to_string(), UiValue::Float(0.5)),
                ("min".to_string(), UiValue::Float(0.0)),
                ("max".to_string(), UiValue::Float(1.0)),
            ]
            .into_iter()
            .collect(),
        ),
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
        props: Some(
            [("value".to_string(), UiValue::Bool(false))]
                .into_iter()
                .collect(),
        ),
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
        props: Some(
            [("label".to_string(), UiValue::String("Submit".into()))]
                .into_iter()
                .collect(),
        ),
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
