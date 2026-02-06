use crate::core::VulframResult;
use crate::core::cmd::{EngineCmd, EngineEvent};
use crate::core::render::graph::LogicalId;
use crate::core::resources::{
    CameraKind, CmdCameraCreateArgs, CmdMaterialCreateArgs, CmdModelCreateArgs,
    CmdPrimitiveGeometryCreateArgs, CmdTextureCreateSolidColorArgs, MaterialKind, MaterialOptions,
    MaterialSampler, PrimitiveShape, StandardOptions, ViewAnchor, ViewPosition, ViewSize,
    ViewValue,
};
use crate::core::ui::cmd::{
    CmdUiApplyOpsArgs, CmdUiContextCreateArgs, CmdUiPanelCreateArgs, CmdUiThemeDefineArgs,
};
use crate::core::ui::tree::{UiListeners, UiOp, UiOpAdd, UiOpAnimate, UiOpSet, UiSetMode};
use crate::core::ui::types::{UiRectPx, UiRenderTarget, UiThemeConfig, UiValue};
use glam::{Mat4, Vec2, Vec3, Vec4};

use crate::demos::common::{
    create_point_light_cmd, receive_responses, run_loop_with_events, send_commands,
};

pub fn run(window_id: u32) -> bool {
    let camera_world_id: u32 = 30;
    let camera_viewport_id: u32 = 32;
    let camera_target_id: u32 = 910;

    let cube_geometry_id: u32 = 930;
    let cube_material_id: u32 = 931;
    let cube_model_id: u32 = 932;

    let sphere_geometry_id: u32 = 940;
    let sphere_material_id: u32 = 941;
    let sphere_model_id: u32 = 942;

    let cylinder_geometry_id: u32 = 935;
    let cylinder_material_id: u32 = 936;
    let cylinder_model_id: u32 = 937;

    let torus_geometry_id: u32 = 945;
    let torus_material_id: u32 = 946;
    let torus_model_id: u32 = 947;

    let ui_plane_geometry_id: u32 = 950;
    let ui_plane_material_id: u32 = 951;
    let ui_plane_model_id: u32 = 952;
    let ui_texture_id: u32 = 960;
    let ui_debug_texture_id: u32 = 961;

    let context_screen_id = LogicalId::Str("ui_demo_screen_8".into());
    let context_panel_id = LogicalId::Str("ui_demo_panel_8".into());
    let panel_id = LogicalId::Str("ui_panel_8".into());
    let theme_id = LogicalId::Str("ui_theme_demo_8".into());

    let theme = UiThemeConfig::default();

    let _ = (
        &context_screen_id,
        &context_panel_id,
        &panel_id,
        camera_viewport_id,
        camera_target_id,
        ui_texture_id,
    );

    let setup_cmds = vec![
        EngineCmd::CmdCameraCreate(CmdCameraCreateArgs {
            camera_id: camera_world_id,
            label: Some("Demo8 World Camera".into()),
            transform: Mat4::look_at_rh(Vec3::new(0.0, 3.5, 9.0), Vec3::ZERO, Vec3::Y).inverse(),
            kind: CameraKind::Perspective,
            flags: 0,
            near_far: Vec2::new(0.1, 100.0),
            layer_mask: 0xFFFFFFFF,
            order: 0,
            layer: 0,
            target_texture_id: None,
            view_position: Some(ViewPosition {
                anchor: ViewAnchor {
                    x: ViewValue::Relative(0.5),
                    y: ViewValue::Relative(0.0),
                },
                size: ViewSize {
                    width: ViewValue::Relative(0.5),
                    height: ViewValue::Relative(1.0),
                },
            }),
            ortho_scale: 10.0,
        }),
        EngineCmd::CmdCameraCreate(CmdCameraCreateArgs {
            camera_id: camera_viewport_id,
            label: Some("Demo8 Viewport Camera".into()),
            transform: Mat4::look_at_rh(
                Vec3::new(0.0, 2.5, 8.0),
                Vec3::new(0.0, 0.5, 0.0),
                Vec3::Y,
            )
            .inverse(),
            kind: CameraKind::Perspective,
            flags: 0,
            near_far: Vec2::new(0.1, 100.0),
            layer_mask: 0xFFFFFFFF,
            order: -1,
            layer: 0,
            target_texture_id: Some(LogicalId::Int(camera_target_id as i64)),
            view_position: None,
            ortho_scale: 10.0,
        }),
        create_point_light_cmd(window_id, 40, Vec4::new(1.8, 3.0, 2.0, 1.0)),
        create_point_light_cmd(window_id, 41, Vec4::new(0.0, 3.5, 5.0, 1.0)),
        EngineCmd::CmdTextureCreateSolidColor(CmdTextureCreateSolidColorArgs {
            window_id,
            texture_id: ui_debug_texture_id,
            label: Some("Demo8 UI Debug".into()),
            color: Vec4::new(0.1, 0.5, 0.9, 1.0),
            srgb: Some(true),
            mode: Default::default(),
            atlas_options: None,
        }),
        EngineCmd::CmdMaterialCreate(CmdMaterialCreateArgs {
            window_id,
            material_id: cube_material_id,
            label: Some("Demo8 Cube".into()),
            kind: MaterialKind::Standard,
            options: Some(MaterialOptions::Standard(StandardOptions {
                base_color: Vec4::new(0.7, 0.2, 0.2, 1.0),
                base_sampler: Some(MaterialSampler::LinearClamp),
                emissive_color: Vec4::ZERO,
                ..Default::default()
            })),
        }),
        EngineCmd::CmdMaterialCreate(CmdMaterialCreateArgs {
            window_id,
            material_id: sphere_material_id,
            label: Some("Demo8 Sphere".into()),
            kind: MaterialKind::Standard,
            options: Some(MaterialOptions::Standard(StandardOptions {
                base_color: Vec4::new(0.2, 0.8, 0.4, 1.0),
                base_sampler: Some(MaterialSampler::LinearClamp),
                emissive_color: Vec4::ZERO,
                ..Default::default()
            })),
        }),
        EngineCmd::CmdMaterialCreate(CmdMaterialCreateArgs {
            window_id,
            material_id: cylinder_material_id,
            label: Some("Demo8 Cylinder".into()),
            kind: MaterialKind::Standard,
            options: Some(MaterialOptions::Standard(StandardOptions {
                base_color: Vec4::new(0.9, 0.7, 0.2, 1.0),
                base_sampler: Some(MaterialSampler::LinearClamp),
                emissive_color: Vec4::ZERO,
                ..Default::default()
            })),
        }),
        EngineCmd::CmdMaterialCreate(CmdMaterialCreateArgs {
            window_id,
            material_id: torus_material_id,
            label: Some("Demo8 Torus".into()),
            kind: MaterialKind::Standard,
            options: Some(MaterialOptions::Standard(StandardOptions {
                base_color: Vec4::new(0.3, 0.5, 0.9, 1.0),
                base_sampler: Some(MaterialSampler::LinearClamp),
                emissive_color: Vec4::ZERO,
                ..Default::default()
            })),
        }),
        EngineCmd::CmdMaterialCreate(CmdMaterialCreateArgs {
            window_id,
            material_id: ui_plane_material_id,
            label: Some("Demo8 UI Plane A".into()),
            kind: MaterialKind::Standard,
            options: Some(MaterialOptions::Standard(StandardOptions {
                base_color: Vec4::ZERO,
                base_tex_id: Some(ui_debug_texture_id),
                base_sampler: Some(MaterialSampler::LinearClamp),
                emissive_color: Vec4::ONE,
                emissive_tex_id: Some(ui_texture_id),
                emissive_sampler: Some(MaterialSampler::LinearClamp),
                ..Default::default()
            })),
        }),
        EngineCmd::CmdPrimitiveGeometryCreate(CmdPrimitiveGeometryCreateArgs {
            window_id,
            geometry_id: cube_geometry_id,
            label: Some("Demo8 Cube Geo".into()),
            shape: PrimitiveShape::Cube,
            options: None,
        }),
        EngineCmd::CmdPrimitiveGeometryCreate(CmdPrimitiveGeometryCreateArgs {
            window_id,
            geometry_id: sphere_geometry_id,
            label: Some("Demo8 Sphere Geo".into()),
            shape: PrimitiveShape::Sphere,
            options: None,
        }),
        EngineCmd::CmdPrimitiveGeometryCreate(CmdPrimitiveGeometryCreateArgs {
            window_id,
            geometry_id: cylinder_geometry_id,
            label: Some("Demo8 Cylinder Geo".into()),
            shape: PrimitiveShape::Cylinder,
            options: None,
        }),
        EngineCmd::CmdPrimitiveGeometryCreate(CmdPrimitiveGeometryCreateArgs {
            window_id,
            geometry_id: torus_geometry_id,
            label: Some("Demo8 Torus Geo".into()),
            shape: PrimitiveShape::Torus,
            options: None,
        }),
        EngineCmd::CmdPrimitiveGeometryCreate(CmdPrimitiveGeometryCreateArgs {
            window_id,
            geometry_id: ui_plane_geometry_id,
            label: Some("Demo8 UI Plane A".into()),
            shape: PrimitiveShape::Plane,
            options: None,
        }),
        EngineCmd::CmdModelCreate(CmdModelCreateArgs {
            window_id,
            model_id: cube_model_id,
            label: Some("Demo8 Cube".into()),
            geometry_id: cube_geometry_id,
            material_id: Some(cube_material_id),
            transform: Mat4::from_translation(Vec3::new(-2.0, 0.75, 0.0))
                * Mat4::from_scale(Vec3::splat(1.5)),
            layer_mask: 0xFFFFFFFF,
            cast_shadow: true,
            receive_shadow: true,
            cast_outline: false,
            outline_color: Vec4::ZERO,
        }),
        EngineCmd::CmdModelCreate(CmdModelCreateArgs {
            window_id,
            model_id: sphere_model_id,
            label: Some("Demo8 Sphere".into()),
            geometry_id: sphere_geometry_id,
            material_id: Some(sphere_material_id),
            transform: Mat4::from_translation(Vec3::new(2.0, 0.75, -1.5))
                * Mat4::from_scale(Vec3::splat(1.2)),
            layer_mask: 0xFFFFFFFF,
            cast_shadow: true,
            receive_shadow: true,
            cast_outline: false,
            outline_color: Vec4::ZERO,
        }),
        EngineCmd::CmdModelCreate(CmdModelCreateArgs {
            window_id,
            model_id: cylinder_model_id,
            label: Some("Demo8 Cylinder".into()),
            geometry_id: cylinder_geometry_id,
            material_id: Some(cylinder_material_id),
            transform: Mat4::from_translation(Vec3::new(-2.5, 0.75, -0.5))
                * Mat4::from_scale(Vec3::splat(1.0)),
            layer_mask: 0xFFFFFFFF,
            cast_shadow: true,
            receive_shadow: true,
            cast_outline: false,
            outline_color: Vec4::ZERO,
        }),
        EngineCmd::CmdModelCreate(CmdModelCreateArgs {
            window_id,
            model_id: torus_model_id,
            label: Some("Demo8 Torus".into()),
            geometry_id: torus_geometry_id,
            material_id: Some(torus_material_id),
            transform: Mat4::from_translation(Vec3::new(2.5, 1.0, -1.0))
                * Mat4::from_scale(Vec3::splat(1.0)),
            layer_mask: 0xFFFFFFFF,
            cast_shadow: true,
            receive_shadow: true,
            cast_outline: false,
            outline_color: Vec4::ZERO,
        }),
        EngineCmd::CmdModelCreate(CmdModelCreateArgs {
            window_id,
            model_id: ui_plane_model_id,
            label: Some("Demo8 UI Plane A".into()),
            geometry_id: ui_plane_geometry_id,
            material_id: Some(ui_plane_material_id),
            transform: Mat4::from_translation(Vec3::new(0.0, 1.2, 2.5))
                * Mat4::from_rotation_x(-0.5)
                * Mat4::from_scale(Vec3::splat(3.0)),
            layer_mask: 0xFFFFFFFF,
            cast_shadow: false,
            receive_shadow: false,
            cast_outline: false,
            outline_color: Vec4::ZERO,
        }),
        EngineCmd::CmdUiThemeDefine(CmdUiThemeDefineArgs {
            theme_id: theme_id.clone(),
            theme: theme.clone(),
        }),
        EngineCmd::CmdUiContextCreate(CmdUiContextCreateArgs {
            window_id,
            context_id: context_screen_id.clone(),
            theme_id: Some(theme_id.clone()),
            target: UiRenderTarget::Screen,
            screen_rect: UiRectPx {
                x: 0.0,
                y: 0.0,
                w: 1280.0,
                h: 720.0,
            },
            z_index: Some(10),
        }),
        EngineCmd::CmdUiContextCreate(CmdUiContextCreateArgs {
            window_id,
            context_id: context_panel_id.clone(),
            theme_id: Some(theme_id.clone()),
            target: UiRenderTarget::TextureId(LogicalId::Int(ui_texture_id as i64)),
            screen_rect: UiRectPx {
                x: 0.0,
                y: 0.0,
                w: 512.0,
                h: 512.0,
            },
            z_index: Some(0),
        }),
        EngineCmd::CmdUiPanelCreate(CmdUiPanelCreateArgs {
            panel_id: panel_id.clone(),
            context_id: context_panel_id.clone(),
            model_id: ui_plane_model_id,
            camera_id: camera_world_id,
        }),
        EngineCmd::CmdUiApplyOps(CmdUiApplyOpsArgs {
            context_id: context_screen_id.clone(),
            base_version: 0,
            ops: build_ui_ops_screen(camera_target_id, camera_viewport_id),
        }),
        EngineCmd::CmdUiApplyOps(CmdUiApplyOpsArgs {
            context_id: context_panel_id.clone(),
            base_version: 0,
            ops: build_ui_ops_panel(),
        }),
    ];

    assert_eq!(send_commands(setup_cmds), VulframResult::Success);
    let _ = receive_responses();

    let mut ui_screen_version: u32 = 1;
    let mut ui_panel_version: u32 = 1;
    let mut last_ui_tick: u64 = 0;
    let mut last_anim_tick: u64 = 0;

    run_loop_with_events(
        window_id,
        None,
        move |total_ms, _delta_ms| {
            let rotation_angle = total_ms as f32 * 0.0015;
            let secondary_angle = total_ms as f32 * 0.001;

            let mut cmds = vec![
                EngineCmd::CmdModelUpdate(crate::core::resources::CmdModelUpdateArgs {
                    window_id,
                    model_id: cube_model_id,
                    label: None,
                    geometry_id: None,
                    material_id: None,
                    transform: Some(
                        Mat4::from_translation(Vec3::new(-2.0, 0.75, 0.0))
                            * Mat4::from_rotation_y(rotation_angle)
                            * Mat4::from_rotation_x(rotation_angle * 0.7)
                            * Mat4::from_scale(Vec3::splat(1.5)),
                    ),
                    layer_mask: None,
                    cast_shadow: None,
                    receive_shadow: None,
                    cast_outline: None,
                    outline_color: None,
                }),
                EngineCmd::CmdModelUpdate(crate::core::resources::CmdModelUpdateArgs {
                    window_id,
                    model_id: sphere_model_id,
                    label: None,
                    geometry_id: None,
                    material_id: None,
                    transform: Some(
                        Mat4::from_translation(Vec3::new(2.0, 0.75, -1.5))
                            * Mat4::from_rotation_y(-secondary_angle)
                            * Mat4::from_rotation_x(secondary_angle * 0.5)
                            * Mat4::from_scale(Vec3::splat(1.2)),
                    ),
                    layer_mask: None,
                    cast_shadow: None,
                    receive_shadow: None,
                    cast_outline: None,
                    outline_color: None,
                }),
                EngineCmd::CmdModelUpdate(crate::core::resources::CmdModelUpdateArgs {
                    window_id,
                    model_id: cylinder_model_id,
                    label: None,
                    geometry_id: None,
                    material_id: None,
                    transform: Some(
                        Mat4::from_translation(Vec3::new(-2.5, 0.75, -0.5))
                            * Mat4::from_rotation_y(rotation_angle * 1.3)
                            * Mat4::from_rotation_z(rotation_angle * 0.4)
                            * Mat4::from_scale(Vec3::splat(1.0)),
                    ),
                    layer_mask: None,
                    cast_shadow: None,
                    receive_shadow: None,
                    cast_outline: None,
                    outline_color: None,
                }),
                EngineCmd::CmdModelUpdate(crate::core::resources::CmdModelUpdateArgs {
                    window_id,
                    model_id: torus_model_id,
                    label: None,
                    geometry_id: None,
                    material_id: None,
                    transform: Some(
                        Mat4::from_translation(Vec3::new(2.5, 1.0, -1.0))
                            * Mat4::from_rotation_x(secondary_angle * 0.8)
                            * Mat4::from_rotation_y(-secondary_angle * 1.2)
                            * Mat4::from_scale(Vec3::splat(1.0)),
                    ),
                    layer_mask: None,
                    cast_shadow: None,
                    receive_shadow: None,
                    cast_outline: None,
                    outline_color: None,
                }),
            ];

            if total_ms.saturating_sub(last_ui_tick) > 250 {
                last_ui_tick = total_ms;
                let frame_text = format!("Frame: {}", total_ms / 16);
                cmds.push(EngineCmd::CmdUiApplyOps(CmdUiApplyOpsArgs {
                    context_id: context_screen_id.clone(),
                    base_version: ui_screen_version,
                    ops: vec![UiOp::Set(UiOpSet {
                        id: LogicalId::Str("ui_frame".into()),
                        mode: UiSetMode::Merge,
                        variant: None,
                        style: None,
                        props: Some(Some(
                            [("value".to_string(), UiValue::String(frame_text.clone()))]
                                .into_iter()
                                .collect(),
                        )),
                        listeners: None,
                    })],
                }));
                ui_screen_version = ui_screen_version.saturating_add(1);

                cmds.push(EngineCmd::CmdUiApplyOps(CmdUiApplyOpsArgs {
                    context_id: context_panel_id.clone(),
                    base_version: ui_panel_version,
                    ops: vec![UiOp::Set(UiOpSet {
                        id: LogicalId::Str("panel_frame".into()),
                        mode: UiSetMode::Merge,
                        variant: None,
                        style: None,
                        props: Some(Some(
                            [("value".to_string(), UiValue::String(frame_text.clone()))]
                                .into_iter()
                                .collect(),
                        )),
                        listeners: None,
                    })],
                }));
                ui_panel_version = ui_panel_version.saturating_add(1);

                let _ = frame_text;
            }

            if total_ms.saturating_sub(last_anim_tick) > 2400 {
                last_anim_tick = total_ms;
                cmds.push(EngineCmd::CmdUiApplyOps(CmdUiApplyOpsArgs {
                    context_id: context_screen_id.clone(),
                    base_version: ui_screen_version,
                    ops: vec![UiOp::Animate(UiOpAnimate {
                        id: LogicalId::Str("ui_badge".into()),
                        property: "opacity".into(),
                        from: Some(0.0),
                        to: 1.0,
                        duration_ms: 300,
                        delay_ms: Some(0),
                        easing: Some("ease-out".into()),
                    })],
                }));
                ui_screen_version = ui_screen_version.saturating_add(1);

                cmds.push(EngineCmd::CmdUiApplyOps(CmdUiApplyOpsArgs {
                    context_id: context_panel_id.clone(),
                    base_version: ui_panel_version,
                    ops: vec![UiOp::Animate(UiOpAnimate {
                        id: LogicalId::Str("panel_title".into()),
                        property: "translateY".into(),
                        from: Some(-8.0),
                        to: 0.0,
                        duration_ms: 350,
                        delay_ms: Some(0),
                        easing: Some("ease-out".into()),
                    })],
                }));
                ui_panel_version = ui_panel_version.saturating_add(1);

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

fn build_ui_ops_screen(camera_target_id: u32, camera_id: u32) -> Vec<UiOp> {
    let mut ops = Vec::new();

    ops.push(UiOp::Add(UiOpAdd {
        parent: None,
        id: LogicalId::Str("screen_root".into()),
        node_type: crate::core::ui::tree::UiNodeType::Container,
        index: None,
        variant: None,
        style: None,
        props: None,
        listeners: None,
    }));

    let mut root_style = std::collections::HashMap::new();
    root_style.insert("layout".to_string(), UiValue::String("col".into()));
    root_style.insert("gap".to_string(), UiValue::Float(12.0));
    root_style.insert("padding".to_string(), UiValue::Float(16.0));
    root_style.insert("width".to_string(), UiValue::String("fill".into()));
    root_style.insert("height".to_string(), UiValue::String("fill".into()));
    ops.push(UiOp::Set(UiOpSet {
        id: LogicalId::Str("screen_root".into()),
        mode: UiSetMode::Merge,
        variant: None,
        style: Some(Some(root_style)),
        props: None,
        listeners: None,
    }));

    ops.push(UiOp::Add(UiOpAdd {
        parent: Some(LogicalId::Str("screen_root".into())),
        id: LogicalId::Str("screen_title".into()),
        node_type: crate::core::ui::tree::UiNodeType::Text,
        index: None,
        variant: None,
        style: None,
        props: Some(
            [(
                "value".to_string(),
                UiValue::String("Demo 008 - UI + Render".into()),
            )]
            .into_iter()
            .collect(),
        ),
        listeners: None,
    }));

    ops.push(UiOp::Add(UiOpAdd {
        parent: Some(LogicalId::Str("screen_root".into())),
        id: LogicalId::Str("screen_row".into()),
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

    ops.push(UiOp::Add(UiOpAdd {
        parent: Some(LogicalId::Str("screen_row".into())),
        id: LogicalId::Str("viewport_image".into()),
        node_type: crate::core::ui::tree::UiNodeType::Image,
        index: None,
        variant: None,
        style: Some(
            [
                ("width".to_string(), UiValue::Float(640.0)),
                ("height".to_string(), UiValue::Float(420.0)),
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
        listeners: Some(UiListeners {
            on_viewport_hover: Some("viewport_hover".into()),
            on_viewport_click: Some("viewport_click".into()),
            on_viewport_drag: Some("viewport_drag".into()),
            on_viewport_drag_end: Some("viewport_drag_end".into()),
            ..Default::default()
        }),
    }));

    ops.push(UiOp::Add(UiOpAdd {
        parent: Some(LogicalId::Str("screen_row".into())),
        id: LogicalId::Str("screen_controls".into()),
        node_type: crate::core::ui::tree::UiNodeType::Container,
        index: None,
        variant: None,
        style: Some(
            [
                ("layout".to_string(), UiValue::String("col".into())),
                ("gap".to_string(), UiValue::Float(8.0)),
                ("padding".to_string(), UiValue::Float(8.0)),
                ("width".to_string(), UiValue::Float(300.0)),
            ]
            .into_iter()
            .collect(),
        ),
        props: None,
        listeners: None,
    }));

    ops.push(UiOp::Add(UiOpAdd {
        parent: Some(LogicalId::Str("screen_controls".into())),
        id: LogicalId::Str("ui_badge".into()),
        node_type: crate::core::ui::tree::UiNodeType::Text,
        index: None,
        variant: None,
        style: Some(
            [("opacity".to_string(), UiValue::Float(0.0))]
                .into_iter()
                .collect(),
        ),
        props: Some(
            [("value".to_string(), UiValue::String("UI Animations".into()))]
                .into_iter()
                .collect(),
        ),
        listeners: None,
    }));

    ops.push(UiOp::Add(UiOpAdd {
        parent: Some(LogicalId::Str("screen_controls".into())),
        id: LogicalId::Str("ui_frame".into()),
        node_type: crate::core::ui::tree::UiNodeType::Text,
        index: None,
        variant: None,
        style: None,
        props: Some(
            [("value".to_string(), UiValue::String("Frame: 0".into()))]
                .into_iter()
                .collect(),
        ),
        listeners: None,
    }));

    ops.push(UiOp::Add(UiOpAdd {
        parent: Some(LogicalId::Str("screen_controls".into())),
        id: LogicalId::Str("ui_info".into()),
        node_type: crate::core::ui::tree::UiNodeType::Text,
        index: None,
        variant: None,
        style: None,
        props: Some(
            [(
                "value".to_string(),
                UiValue::String(format!(
                    "Viewport: Tex {} / Cam {}",
                    camera_target_id, camera_id
                )),
            )]
            .into_iter()
            .collect(),
        ),
        listeners: None,
    }));


    ops.push(UiOp::Animate(UiOpAnimate {
        id: LogicalId::Str("ui_badge".into()),
        property: "opacity".into(),
        from: Some(0.0),
        to: 1.0,
        duration_ms: 300,
        delay_ms: Some(0),
        easing: Some("ease-out".into()),
    }));

    ops
}

fn build_ui_ops_panel() -> Vec<UiOp> {
    let mut ops = Vec::new();

    ops.push(UiOp::Add(UiOpAdd {
        parent: None,
        id: LogicalId::Str("panel_root".into()),
        node_type: crate::core::ui::tree::UiNodeType::Container,
        index: None,
        variant: None,
        style: None,
        props: None,
        listeners: None,
    }));

    let mut root_style = std::collections::HashMap::new();
    root_style.insert("layout".to_string(), UiValue::String("col".into()));
    root_style.insert("gap".to_string(), UiValue::Float(8.0));
    root_style.insert("padding".to_string(), UiValue::Float(12.0));
    root_style.insert("width".to_string(), UiValue::String("fill".into()));
    root_style.insert("height".to_string(), UiValue::String("fill".into()));
    ops.push(UiOp::Set(UiOpSet {
        id: LogicalId::Str("panel_root".into()),
        mode: UiSetMode::Merge,
        variant: None,
        style: Some(Some(root_style)),
        props: None,
        listeners: None,
    }));

    ops.push(UiOp::Add(UiOpAdd {
        parent: Some(LogicalId::Str("panel_root".into())),
        id: LogicalId::Str("panel_title".into()),
        node_type: crate::core::ui::tree::UiNodeType::Text,
        index: None,
        variant: None,
        style: None,
        props: Some(
            [(
                "value".to_string(),
                UiValue::String("Panel in Render".into()),
            )]
            .into_iter()
            .collect(),
        ),
        listeners: None,
    }));

    ops.push(UiOp::Add(UiOpAdd {
        parent: Some(LogicalId::Str("panel_root".into())),
        id: LogicalId::Str("panel_frame".into()),
        node_type: crate::core::ui::tree::UiNodeType::Text,
        index: None,
        variant: None,
        style: None,
        props: Some(
            [("value".to_string(), UiValue::String("Frame: 0".into()))]
                .into_iter()
                .collect(),
        ),
        listeners: None,
    }));

    ops.push(UiOp::Add(UiOpAdd {
        parent: Some(LogicalId::Str("panel_root".into())),
        id: LogicalId::Str("panel_button".into()),
        node_type: crate::core::ui::tree::UiNodeType::Button,
        index: None,
        variant: None,
        style: None,
        props: Some(
            [("label".to_string(), UiValue::String("Click Panel A".into()))]
                .into_iter()
                .collect(),
        ),
        listeners: Some(UiListeners {
            on_click: Some("panel_a_click".into()),
            ..Default::default()
        }),
    }));

    ops.push(UiOp::Add(UiOpAdd {
        parent: Some(LogicalId::Str("panel_root".into())),
        id: LogicalId::Str("panel_input".into()),
        node_type: crate::core::ui::tree::UiNodeType::Input,
        index: None,
        variant: None,
        style: None,
        props: Some(
            [("value".to_string(), UiValue::String("Test input".into()))]
                .into_iter()
                .collect(),
        ),
        listeners: Some(UiListeners {
            on_change_commit: Some("panel_a_input_change".into()),
            ..Default::default()
        }),
    }));

    ops.push(UiOp::Animate(UiOpAnimate {
        id: LogicalId::Str("panel_title".into()),
        property: "translateY".into(),
        from: Some(-8.0),
        to: 0.0,
        duration_ms: 350,
        delay_ms: Some(0),
        easing: Some("ease-out".into()),
    }));

    ops
}
