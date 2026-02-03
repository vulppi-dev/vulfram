mod core;

use crate::core::VulframResult;
use crate::core::audio::{
    AudioPlayModeDto, AudioSpatialParamsDto, CmdAudioListenerCreateArgs,
    CmdAudioResourceCreateArgs, CmdAudioSourceCreateArgs, CmdAudioSourcePlayArgs,
    CmdAudioSourceStopArgs,
};
use crate::core::buffers::state::UploadType;
use crate::core::cmd::{
    CommandResponse, CommandResponseEnvelope, EngineCmd, EngineCmdEnvelope, EngineEvent,
};
use crate::core::input::events::{ElementState, KeyboardEvent};
use crate::core::render::cmd::CmdRenderGraphSetArgs;
use crate::core::render::gizmos::{CmdGizmoDrawAabbArgs, CmdGizmoDrawLineArgs};
use crate::core::render::graph::{
    LogicalId, RenderGraphDesc, RenderGraphEdge, RenderGraphEdgeReason, RenderGraphLifetime,
    RenderGraphNode, RenderGraphResource, RenderGraphResourceKind,
};
use crate::core::resources::shadow::{CmdShadowConfigureArgs, ShadowConfig};
use crate::core::resources::{
    CameraKind, CmdCameraCreateArgs, CmdCameraUpdateArgs, CmdEnvironmentUpdateArgs,
    CmdGeometryCreateArgs, CmdLightCreateArgs, CmdMaterialCreateArgs, CmdModelCreateArgs,
    CmdModelUpdateArgs, CmdPoseUpdateArgs, CmdPrimitiveGeometryCreateArgs,
    CmdTextureCreateFromBufferArgs, EnvironmentConfig, GeometryPrimitiveEntry, LightKind,
    MaterialKind, MaterialOptions, MaterialSampler, MsaaConfig, PostProcessConfig, PrimitiveShape,
    SkyboxConfig, SkyboxMode, StandardOptions, TextureCreateMode,
};
use crate::core::system::events::SystemEvent;
use crate::core::window::{CmdWindowCloseArgs, CmdWindowCreateArgs, WindowEvent};
use bytemuck::cast_slice;
use glam::{Mat4, Quat, UVec2, Vec2, Vec3, Vec4};
use rand::Rng;
use rmp_serde::{from_slice, to_vec_named};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use std::rc::Rc;
use std::sync::Mutex;
use std::time::{Duration, Instant};

static ENGINE_GUARD: Mutex<()> = Mutex::new(());

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DemoKind {
    Demo001,
    Demo002,
    Demo003,
    Demo004,
}

impl DemoKind {
    fn from_str(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "demo_001" | "demo001" | "1" => Some(Self::Demo001),
            "demo_002" | "demo002" | "2" => Some(Self::Demo002),
            "demo_003" | "demo003" | "3" => Some(Self::Demo003),
            "demo_004" | "demo004" | "4" => Some(Self::Demo004),
            _ => None,
        }
    }

    fn title(self) -> &'static str {
        match self {
            Self::Demo001 => "Vulfram Demo 001",
            Self::Demo002 => "Vulfram Demo 002",
            Self::Demo003 => "Vulfram Demo 003",
            Self::Demo004 => "Vulfram Demo 004",
        }
    }
}

fn main() {
    let _lock = ENGINE_GUARD.lock().unwrap();

    assert_eq!(core::vulfram_init(), VulframResult::Success);

    let demo = select_demo();
    let window_id: u32 = 1;

    create_window(window_id, demo.title());
    pump_for(Duration::from_millis(200));
    wait_for_confirmation(window_id);

    let close_sent = match demo {
        DemoKind::Demo001 => demo_001(window_id),
        DemoKind::Demo002 => demo_002(window_id),
        DemoKind::Demo003 => demo_003(window_id),
        DemoKind::Demo004 => demo_004(window_id),
    };

    if !close_sent {
        let close_cmd = EngineCmd::CmdWindowClose(CmdWindowCloseArgs { window_id });
        let _ = send_commands(vec![close_cmd]);
    }
    pump_for(Duration::from_millis(100));

    assert_eq!(core::vulfram_dispose(), VulframResult::Success);
}

fn select_demo() -> DemoKind {
    if let Some(arg) = std::env::args().nth(1) {
        if let Some(demo) = DemoKind::from_str(&arg) {
            println!("Selected demo from args: {:?}", demo);
            return demo;
        }
    }

    if let Ok(value) = std::env::var("VULFRAM_DEMO") {
        if let Some(demo) = DemoKind::from_str(&value) {
            println!("Selected demo from env: {:?}", demo);
            return demo;
        }
    }

    DemoKind::Demo001
}

fn create_window(window_id: u32, title: &str) {
    let create_cmd = EngineCmd::CmdWindowCreate(CmdWindowCreateArgs {
        window_id,
        title: title.into(),
        size: UVec2::new(1280, 720),
        resizable: true,
        initial_state: crate::core::window::EngineWindowState::Maximized,
        ..Default::default()
    });
    assert_eq!(send_commands(vec![create_cmd]), VulframResult::Success);
}

fn demo_001(window_id: u32) -> bool {
    let geometry_cube: u32 = 1;
    let camera_id: u32 = 1;
    let material_instance: u32 = 10;
    let texture_test: u32 = 20;
    let texture_buffer: u64 = 1;

    upload_texture("assets/colo_test_texture.png", texture_buffer);

    let mut setup_cmds = vec![
        EngineCmd::CmdPrimitiveGeometryCreate(CmdPrimitiveGeometryCreateArgs {
            window_id,
            geometry_id: geometry_cube,
            label: Some("Default Cube".to_string()),
            shape: PrimitiveShape::Cube,
            options: None,
        }),
        create_camera_cmd(camera_id, "Main Camera", default_camera_transform()),
        create_point_light_cmd(window_id, 2, Vec4::new(0.0, 8.0, 0.0, 1.0)),
        create_texture_cmd(window_id, texture_test, "Test Texture", texture_buffer),
        create_standard_material_cmd(
            window_id,
            material_instance,
            "Test Material",
            Vec4::ONE,
            Some(texture_test),
            None,
        ),
    ];

    setup_cmds.push(create_floor_cmd(
        window_id,
        geometry_cube,
        material_instance,
    ));
    let (mut cubes, cube_cmds) =
        create_instanced_cubes(window_id, geometry_cube, material_instance);
    setup_cmds.extend(cube_cmds);
    setup_cmds.push(create_shadow_config_cmd(window_id));

    assert_eq!(send_commands(setup_cmds), VulframResult::Success);
    let _ = receive_responses();

    run_loop(window_id, None, |total_ms, _delta_ms| {
        let mut frame_cmds = vec![];
        frame_cmds.extend(draw_axes_gizmos());
        frame_cmds.push(EngineCmd::CmdGizmoDrawAabb(CmdGizmoDrawAabbArgs {
            min: Vec3::splat(-5.0),
            max: Vec3::splat(5.0),
            color: Vec4::new(1.0, 1.0, 1.0, 0.2),
        }));

        let time_f = total_ms as f32 / 1000.0;
        for cube in &mut cubes {
            let offset_y = (time_f + cube.phase).sin() * 0.5;
            let rotation = time_f * 2.0 + cube.phase;

            frame_cmds.push(EngineCmd::CmdModelUpdate(CmdModelUpdateArgs {
                window_id,
                model_id: cube.id,
                label: None,
                geometry_id: None,
                material_id: None,
                transform: Some(
                    Mat4::from_translation(cube.initial_pos + Vec3::new(0.0, offset_y, 0.0))
                        * Mat4::from_euler(glam::EulerRot::XYZ, rotation, rotation * 0.5, 0.0)
                        * Mat4::from_scale(Vec3::splat(0.4)),
                ),
                layer_mask: None,
                cast_shadow: None,
                receive_shadow: None,
                cast_outline: None,
                outline_color: None,
            }));
        }

        frame_cmds
    })
}

fn demo_002(window_id: u32) -> bool {
    let camera_id: u32 = 1;

    let shapes = [
        PrimitiveShape::Cube,
        PrimitiveShape::Plane,
        PrimitiveShape::Sphere,
        PrimitiveShape::Cylinder,
        PrimitiveShape::Torus,
        PrimitiveShape::Pyramid,
    ];

    let base_geometry_id: u32 = 100;
    let base_model_id: u32 = 200;
    let base_material_id: u32 = 300;

    let mut setup_cmds = vec![
        EngineCmd::CmdEnvironmentUpdate(CmdEnvironmentUpdateArgs {
            window_id,
            config: EnvironmentConfig {
                msaa: MsaaConfig {
                    enabled: true,
                    sample_count: 4,
                },
                skybox: SkyboxConfig {
                    mode: SkyboxMode::None,
                    intensity: 1.0,
                    rotation: 0.0,
                    ground_color: Vec3::new(0.02, 0.03, 0.04),
                    horizon_color: Vec3::new(0.12, 0.16, 0.22),
                    sky_color: Vec3::new(0.2, 0.35, 0.6),
                    cubemap_texture_id: None,
                },
                post: PostProcessConfig {
                    filter_enabled: true,
                    filter_exposure: 1.0,
                    filter_gamma: 2.2,
                    filter_saturation: 1.05,
                    filter_contrast: 1.1,
                    filter_vignette: 0.12,
                    filter_grain: 0.02,
                    filter_chromatic_aberration: 0.2,
                    filter_blur: 0.0,
                    filter_sharpen: 0.1,
                    filter_tonemap_mode: 1,
                    outline_enabled: false,
                    outline_strength: 0.0,
                    outline_threshold: 0.2,
                    outline_width: 1.0,
                    outline_quality: 0.0,
                    filter_posterize_steps: 0.0,
                    cell_shading: false,
                    ssao_enabled: false,
                    ssao_strength: 1.0,
                    ssao_radius: 0.75,
                    ssao_bias: 0.025,
                    ssao_power: 1.5,
                    ssao_blur_radius: 2.0,
                    ssao_blur_depth_threshold: 0.02,
                    bloom_enabled: true,
                    bloom_threshold: 1.0,
                    bloom_knee: 0.5,
                    bloom_intensity: 0.8,
                    bloom_scatter: 0.7,
                },
            },
        }),
        create_camera_cmd(
            camera_id,
            "Primitives Camera",
            Mat4::look_at_rh(Vec3::new(0.0, 4.0, 12.0), Vec3::ZERO, Vec3::Y).inverse(),
        ),
        create_point_light_cmd(window_id, 2, Vec4::new(5.0, 8.0, 6.0, 1.0)),
        create_ambient_light_cmd(window_id, 3, Vec4::new(0.25, 0.25, 0.25, 1.0), 0.6),
        create_shadow_config_cmd(window_id),
    ];

    let palette = [
        Vec4::new(0.85, 0.25, 0.2, 1.0),
        Vec4::new(0.2, 0.6, 0.9, 1.0),
        Vec4::new(0.95, 0.75, 0.25, 1.0),
        Vec4::new(0.3, 0.85, 0.45, 1.0),
        Vec4::new(0.75, 0.35, 0.9, 1.0),
        Vec4::new(0.9, 0.55, 0.2, 1.0),
    ];

    let spacing = 2.8_f32;
    let start_x = -((shapes.len() - 1) as f32) * spacing * 0.5;
    let mut primitive_models = Vec::new();

    for (index, shape) in shapes.iter().enumerate() {
        let geometry_id = base_geometry_id + index as u32;
        let model_id = base_model_id + index as u32;
        let material_id = base_material_id + index as u32;
        let label = format!("{:?}", shape);
        let color = palette[index % palette.len()];

        setup_cmds.push(EngineCmd::CmdPrimitiveGeometryCreate(
            CmdPrimitiveGeometryCreateArgs {
                window_id,
                geometry_id,
                label: Some(label.clone()),
                shape: *shape,
                options: None,
            },
        ));

        setup_cmds.push(create_standard_material_cmd(
            window_id,
            material_id,
            &format!("{} Material", label),
            color,
            None,
            None,
        ));

        let position = Vec3::new(start_x + spacing * index as f32, 0.0, 0.0);
        setup_cmds.push(EngineCmd::CmdModelCreate(CmdModelCreateArgs {
            window_id,
            model_id,
            label: Some(label.clone()),
            geometry_id,
            material_id: Some(material_id),
            transform: Mat4::from_translation(position),
            layer_mask: 0xFFFFFFFF,
            cast_shadow: true,
            receive_shadow: true,
            cast_outline: false,
            outline_color: Vec4::ZERO,
        }));

        primitive_models.push((model_id, position));
    }

    assert_eq!(send_commands(setup_cmds), VulframResult::Success);
    let _ = receive_responses();

    run_loop(window_id, None, |total_ms, _delta_ms| {
        let mut frame_cmds = vec![];
        frame_cmds.extend(draw_axes_gizmos());
        let time_f = total_ms as f32 / 1000.0;

        for (index, (model_id, position)) in primitive_models.iter().enumerate() {
            let rotation = time_f * 0.6 + index as f32 * 0.3;
            frame_cmds.push(EngineCmd::CmdModelUpdate(CmdModelUpdateArgs {
                window_id,
                model_id: *model_id,
                label: None,
                geometry_id: None,
                material_id: None,
                transform: Some(
                    Mat4::from_translation(*position)
                        * Mat4::from_euler(glam::EulerRot::XYZ, rotation * 0.4, rotation, 0.0)
                        * Mat4::from_scale(Vec3::splat(1.2)),
                ),
                layer_mask: None,
                cast_shadow: None,
                receive_shadow: None,
                cast_outline: None,
                outline_color: None,
            }));
        }

        frame_cmds
    })
}

fn demo_003(window_id: u32) -> bool {
    let geometry_id: u32 = 400;
    let model_id: u32 = 401;
    let material_id: u32 = 402;
    let camera_id: u32 = 1;
    let bone_count: u32 = 16;

    let (positions, normals, uvs, joints, weights, indices) =
        build_skinned_plane(64, 64, 10.0, bone_count);

    upload_buffer(2000, UploadType::VertexData, &positions);
    upload_buffer(2001, UploadType::VertexData, &normals);
    upload_buffer(2002, UploadType::VertexData, &uvs);
    upload_buffer(2003, UploadType::VertexData, &joints);
    upload_buffer(2004, UploadType::VertexData, &weights);
    upload_buffer(2005, UploadType::IndexData, &indices);

    let setup_cmds = vec![
        EngineCmd::CmdGeometryCreate(CmdGeometryCreateArgs {
            window_id,
            geometry_id,
            label: Some("Skinned Plane".into()),
            entries: vec![
                GeometryPrimitiveEntry {
                    primitive_type: crate::core::resources::GeometryPrimitiveType::Index,
                    buffer_id: 2005,
                },
                GeometryPrimitiveEntry {
                    primitive_type: crate::core::resources::GeometryPrimitiveType::Position,
                    buffer_id: 2000,
                },
                GeometryPrimitiveEntry {
                    primitive_type: crate::core::resources::GeometryPrimitiveType::Normal,
                    buffer_id: 2001,
                },
                GeometryPrimitiveEntry {
                    primitive_type: crate::core::resources::GeometryPrimitiveType::UV,
                    buffer_id: 2002,
                },
                GeometryPrimitiveEntry {
                    primitive_type: crate::core::resources::GeometryPrimitiveType::SkinJoints,
                    buffer_id: 2003,
                },
                GeometryPrimitiveEntry {
                    primitive_type: crate::core::resources::GeometryPrimitiveType::SkinWeights,
                    buffer_id: 2004,
                },
            ],
        }),
        create_camera_cmd(
            camera_id,
            "Skinned Camera",
            Mat4::look_at_rh(Vec3::new(0.0, 6.0, 12.0), Vec3::ZERO, Vec3::Y).inverse(),
        ),
        create_point_light_cmd(window_id, 2, Vec4::new(0.0, 6.0, 0.0, 1.0)),
        create_ambient_light_cmd(window_id, 3, Vec4::new(0.3, 0.3, 0.3, 1.0), 0.4),
        create_standard_material_cmd(
            window_id,
            material_id,
            "Skinned Material",
            Vec4::ONE,
            None,
            None,
        ),
        EngineCmd::CmdModelCreate(CmdModelCreateArgs {
            window_id,
            model_id,
            label: Some("Skinned Plane".into()),
            geometry_id,
            material_id: Some(material_id),
            transform: Mat4::IDENTITY,
            layer_mask: crate::core::resources::common::default_layer_mask(),
            cast_shadow: true,
            receive_shadow: true,
            cast_outline: false,
            outline_color: Vec4::ZERO,
        }),
        create_shadow_config_cmd(window_id),
    ];

    assert_eq!(send_commands(setup_cmds), VulframResult::Success);
    let _ = receive_responses();

    let pose_buffer_id: u64 = 9000;

    run_loop(window_id, None, |total_ms, _delta_ms| {
        let time_f = total_ms as f32 / 1000.0;
        let mut bones: Vec<Mat4> = Vec::with_capacity(bone_count as usize);
        for i in 0..bone_count {
            let phase = time_f * 4.5 + i as f32 * 1.4;
            let offset_y = phase.sin() * 1.2;
            bones.push(Mat4::from_translation(Vec3::new(0.0, offset_y, 0.0)));
        }

        upload_buffer(pose_buffer_id, UploadType::Raw, &bones);

        vec![EngineCmd::CmdPoseUpdate(CmdPoseUpdateArgs {
            window_id,
            model_id,
            bone_count,
            matrices_buffer_id: pose_buffer_id,
        })]
    })
}

fn demo_004(window_id: u32) -> bool {
    let geometry_id: u32 = 500;
    let material_id: u32 = 502;
    let floor_material_id: u32 = 503;
    let emissive_material_id: u32 = 504;
    let skybox_texture_id: u32 = 900;
    let skybox_buffer_id: u64 = 9000;
    let audio_id: u32 = 910;
    let audio_source_id: u32 = 911;
    let audio_buffer_id: u64 = 9100;
    let listener_model_id: u32 = 920;
    let emitter_geometry_id: u32 = 930;
    let emitter_material_id: u32 = 931;
    let emitter_model_id: u32 = 932;
    let emitter_pos = Vec3::new(8.0, -5.2, 8.0);
    let cube_models = [
        (
            501,
            Vec3::new(-2.5, 0.0, 0.0),
            Vec4::new(1.0, 0.1, 0.1, 1.0),
        ),
        (502, Vec3::new(0.0, 0.0, 0.0), Vec4::new(0.1, 1.0, 0.1, 1.0)),
        (503, Vec3::new(2.5, 0.0, 0.0), Vec4::new(0.1, 0.1, 1.0, 1.0)),
        (
            504,
            Vec3::new(-0.6, 0.2, -0.2),
            Vec4::new(1.0, 0.6, 0.1, 1.0),
        ),
        (505, Vec3::new(0.4, 0.1, 0.3), Vec4::new(0.6, 1.0, 0.9, 1.0)),
        (
            506,
            Vec3::new(0.0, -0.1, 0.8),
            Vec4::new(0.9, 0.4, 1.0, 1.0),
        ),
    ];
    let camera_id: u32 = 1;

    let graph = RenderGraphDesc {
        graph_id: LogicalId::Str("demo_004_graph".into()),
        nodes: vec![
            RenderGraphNode {
                node_id: LogicalId::Str("shadow_pass".into()),
                pass_id: "shadow".into(),
                inputs: Vec::new(),
                outputs: vec![LogicalId::Str("shadow_atlas".into())],
                params: HashMap::new(),
            },
            RenderGraphNode {
                node_id: LogicalId::Str("light_cull_pass".into()),
                pass_id: "light-cull".into(),
                inputs: Vec::new(),
                outputs: Vec::new(),
                params: HashMap::new(),
            },
            RenderGraphNode {
                node_id: LogicalId::Str("skybox_pass".into()),
                pass_id: "skybox".into(),
                inputs: Vec::new(),
                outputs: Vec::new(),
                params: HashMap::new(),
            },
            RenderGraphNode {
                node_id: LogicalId::Str("forward_pass".into()),
                pass_id: "forward".into(),
                inputs: vec![LogicalId::Str("shadow_atlas".into())],
                outputs: vec![
                    LogicalId::Str("hdr_color".into()),
                    LogicalId::Str("depth".into()),
                ],
                params: HashMap::new(),
            },
            RenderGraphNode {
                node_id: LogicalId::Str("outline_pass".into()),
                pass_id: "outline".into(),
                inputs: vec![LogicalId::Str("depth".into())],
                outputs: vec![LogicalId::Str("outline_color".into())],
                params: HashMap::new(),
            },
            RenderGraphNode {
                node_id: LogicalId::Str("ssao_pass".into()),
                pass_id: "ssao".into(),
                inputs: vec![LogicalId::Str("depth".into())],
                outputs: vec![LogicalId::Str("ssao_raw".into())],
                params: HashMap::new(),
            },
            RenderGraphNode {
                node_id: LogicalId::Str("ssao_blur_pass".into()),
                pass_id: "ssao-blur".into(),
                inputs: vec![
                    LogicalId::Str("ssao_raw".into()),
                    LogicalId::Str("depth".into()),
                ],
                outputs: vec![LogicalId::Str("ssao_blur".into())],
                params: HashMap::new(),
            },
            RenderGraphNode {
                node_id: LogicalId::Str("bloom_pass".into()),
                pass_id: "bloom".into(),
                inputs: vec![LogicalId::Str("hdr_color".into())],
                outputs: vec![LogicalId::Str("bloom_color".into())],
                params: HashMap::new(),
            },
            RenderGraphNode {
                node_id: LogicalId::Str("post_pass".into()),
                pass_id: "post".into(),
                inputs: vec![
                    LogicalId::Str("hdr_color".into()),
                    LogicalId::Str("outline_color".into()),
                    LogicalId::Str("ssao_blur".into()),
                    LogicalId::Str("bloom_color".into()),
                ],
                outputs: vec![LogicalId::Str("post_color".into())],
                params: HashMap::new(),
            },
            RenderGraphNode {
                node_id: LogicalId::Str("compose_pass".into()),
                pass_id: "compose".into(),
                inputs: vec![LogicalId::Str("post_color".into())],
                outputs: vec![LogicalId::Str("swapchain".into())],
                params: HashMap::new(),
            },
        ],
        edges: vec![
            RenderGraphEdge {
                from_node_id: LogicalId::Str("shadow_pass".into()),
                to_node_id: LogicalId::Str("forward_pass".into()),
                reason: Some(RenderGraphEdgeReason::ReadAfterWrite),
            },
            RenderGraphEdge {
                from_node_id: LogicalId::Str("forward_pass".into()),
                to_node_id: LogicalId::Str("outline_pass".into()),
                reason: Some(RenderGraphEdgeReason::ReadAfterWrite),
            },
            RenderGraphEdge {
                from_node_id: LogicalId::Str("forward_pass".into()),
                to_node_id: LogicalId::Str("ssao_pass".into()),
                reason: Some(RenderGraphEdgeReason::ReadAfterWrite),
            },
            RenderGraphEdge {
                from_node_id: LogicalId::Str("ssao_pass".into()),
                to_node_id: LogicalId::Str("ssao_blur_pass".into()),
                reason: Some(RenderGraphEdgeReason::ReadAfterWrite),
            },
            RenderGraphEdge {
                from_node_id: LogicalId::Str("ssao_blur_pass".into()),
                to_node_id: LogicalId::Str("post_pass".into()),
                reason: Some(RenderGraphEdgeReason::ReadAfterWrite),
            },
            RenderGraphEdge {
                from_node_id: LogicalId::Str("forward_pass".into()),
                to_node_id: LogicalId::Str("bloom_pass".into()),
                reason: Some(RenderGraphEdgeReason::ReadAfterWrite),
            },
            RenderGraphEdge {
                from_node_id: LogicalId::Str("bloom_pass".into()),
                to_node_id: LogicalId::Str("post_pass".into()),
                reason: Some(RenderGraphEdgeReason::ReadAfterWrite),
            },
            RenderGraphEdge {
                from_node_id: LogicalId::Str("outline_pass".into()),
                to_node_id: LogicalId::Str("post_pass".into()),
                reason: Some(RenderGraphEdgeReason::ReadAfterWrite),
            },
            RenderGraphEdge {
                from_node_id: LogicalId::Str("post_pass".into()),
                to_node_id: LogicalId::Str("compose_pass".into()),
                reason: Some(RenderGraphEdgeReason::ReadAfterWrite),
            },
        ],
        resources: vec![
            RenderGraphResource {
                res_id: LogicalId::Str("shadow_atlas".into()),
                kind: RenderGraphResourceKind::Texture,
                lifetime: RenderGraphLifetime::Frame,
                alias_group: None,
            },
            RenderGraphResource {
                res_id: LogicalId::Str("hdr_color".into()),
                kind: RenderGraphResourceKind::Texture,
                lifetime: RenderGraphLifetime::Frame,
                alias_group: None,
            },
            RenderGraphResource {
                res_id: LogicalId::Str("post_color".into()),
                kind: RenderGraphResourceKind::Texture,
                lifetime: RenderGraphLifetime::Frame,
                alias_group: None,
            },
            RenderGraphResource {
                res_id: LogicalId::Str("outline_color".into()),
                kind: RenderGraphResourceKind::Texture,
                lifetime: RenderGraphLifetime::Frame,
                alias_group: None,
            },
            RenderGraphResource {
                res_id: LogicalId::Str("ssao_raw".into()),
                kind: RenderGraphResourceKind::Texture,
                lifetime: RenderGraphLifetime::Frame,
                alias_group: None,
            },
            RenderGraphResource {
                res_id: LogicalId::Str("ssao_blur".into()),
                kind: RenderGraphResourceKind::Texture,
                lifetime: RenderGraphLifetime::Frame,
                alias_group: None,
            },
            RenderGraphResource {
                res_id: LogicalId::Str("bloom_color".into()),
                kind: RenderGraphResourceKind::Texture,
                lifetime: RenderGraphLifetime::Frame,
                alias_group: None,
            },
            RenderGraphResource {
                res_id: LogicalId::Str("depth".into()),
                kind: RenderGraphResourceKind::Texture,
                lifetime: RenderGraphLifetime::Frame,
                alias_group: None,
            },
            RenderGraphResource {
                res_id: LogicalId::Str("swapchain".into()),
                kind: RenderGraphResourceKind::Attachment,
                lifetime: RenderGraphLifetime::Frame,
                alias_group: None,
            },
        ],
        fallback: true,
    };

    let post_config = PostProcessConfig {
        filter_enabled: false,
        filter_exposure: 1.0,
        filter_gamma: 1.0,
        filter_saturation: 1.0,
        filter_contrast: 1.0,
        filter_vignette: 0.0,
        filter_grain: 0.0,
        filter_chromatic_aberration: 0.0,
        filter_blur: 0.0,
        filter_sharpen: 0.0,
        filter_tonemap_mode: 1,
        outline_enabled: true,
        outline_strength: 0.6,
        outline_threshold: 0.0,
        outline_width: 2.0,
        outline_quality: 1.0,
        filter_posterize_steps: 1.0,
        cell_shading: false,
        ssao_enabled: true,
        ssao_strength: 0.75,
        ssao_radius: 0.9,
        ssao_bias: 0.02,
        ssao_power: 1.3,
        ssao_blur_radius: 2.0,
        ssao_blur_depth_threshold: 0.02,
        bloom_enabled: true,
        bloom_threshold: 1.0,
        bloom_knee: 0.8,
        bloom_intensity: 1.0,
        bloom_scatter: 1.0,
    };

    let audio_bytes = load_texture_bytes("assets/audio.wav");
    upload_binary_bytes(&audio_bytes, audio_buffer_id);

    let setup_cmds = vec![
        EngineCmd::CmdEnvironmentUpdate(CmdEnvironmentUpdateArgs {
            window_id,
            config: EnvironmentConfig {
                msaa: MsaaConfig {
                    enabled: true,
                    sample_count: 4,
                },
                skybox: SkyboxConfig {
                    mode: SkyboxMode::Procedural,
                    intensity: 1.0,
                    rotation: 0.0,
                    ground_color: Vec3::new(1.0, 0.0, 0.0),
                    horizon_color: Vec3::new(1.00, 1.0, 1.0),
                    sky_color: Vec3::new(0.18, 0.32, 0.55),
                    cubemap_texture_id: None,
                },
                post: post_config.clone(),
            },
        }),
        EngineCmd::CmdRenderGraphSet(CmdRenderGraphSetArgs { window_id, graph }),
        EngineCmd::CmdPrimitiveGeometryCreate(CmdPrimitiveGeometryCreateArgs {
            window_id,
            geometry_id,
            label: Some("Graph Cube".into()),
            shape: PrimitiveShape::Cube,
            options: None,
        }),
        EngineCmd::CmdPrimitiveGeometryCreate(CmdPrimitiveGeometryCreateArgs {
            window_id,
            geometry_id: emitter_geometry_id,
            label: Some("Audio Emitter".into()),
            shape: PrimitiveShape::Sphere,
            options: None,
        }),
        create_camera_cmd(
            camera_id,
            "Graph Camera",
            Mat4::look_at_rh(Vec3::new(0.0, 3.5, 8.0), Vec3::ZERO, Vec3::Y).inverse(),
        ),
        create_point_light_cmd(window_id, 2, Vec4::new(0.0, 5.0, 2.0, 1.0)),
        create_ambient_light_cmd(window_id, 3, Vec4::new(0.3, 0.3, 0.3, 1.0), 0.6),
        create_standard_material_cmd(
            window_id,
            material_id,
            "Graph Material",
            Vec4::ONE,
            None,
            None,
        ),
        create_standard_material_cmd(
            window_id,
            floor_material_id,
            "Graph Floor Material",
            Vec4::ONE,
            None,
            None,
        ),
        create_standard_material_cmd(
            window_id,
            emissive_material_id,
            "Graph Emissive Material",
            Vec4::ONE,
            None,
            Some(Vec4::new(5.0, 5.0, 5.0, 1.0)),
        ),
        create_standard_material_cmd(
            window_id,
            emitter_material_id,
            "Audio Emitter Material",
            Vec4::new(1.0, 0.8, 0.2, 1.0),
            None,
            Some(Vec4::new(2.5, 1.6, 0.4, 1.0)),
        ),
        EngineCmd::CmdModelCreate(CmdModelCreateArgs {
            window_id,
            model_id: cube_models[0].0,
            label: Some("Graph Cube R".into()),
            geometry_id,
            material_id: Some(material_id),
            transform: Mat4::from_translation(cube_models[0].1),
            layer_mask: 0xFFFFFFFF,
            cast_shadow: true,
            receive_shadow: true,
            cast_outline: true,
            outline_color: cube_models[0].2,
        }),
        EngineCmd::CmdModelCreate(CmdModelCreateArgs {
            window_id,
            model_id: cube_models[1].0,
            label: Some("Graph Cube G".into()),
            geometry_id,
            material_id: Some(material_id),
            transform: Mat4::from_translation(cube_models[1].1),
            layer_mask: 0xFFFFFFFF,
            cast_shadow: true,
            receive_shadow: true,
            cast_outline: true,
            outline_color: cube_models[1].2,
        }),
        EngineCmd::CmdModelCreate(CmdModelCreateArgs {
            window_id,
            model_id: cube_models[2].0,
            label: Some("Graph Cube B".into()),
            geometry_id,
            material_id: Some(emissive_material_id),
            transform: Mat4::from_translation(cube_models[2].1),
            layer_mask: 0xFFFFFFFF,
            cast_shadow: true,
            receive_shadow: true,
            cast_outline: true,
            outline_color: cube_models[2].2,
        }),
        EngineCmd::CmdModelCreate(CmdModelCreateArgs {
            window_id,
            model_id: cube_models[3].0,
            label: Some("Graph Cube D".into()),
            geometry_id,
            material_id: Some(material_id),
            transform: Mat4::from_translation(cube_models[3].1),
            layer_mask: 0xFFFFFFFF,
            cast_shadow: true,
            receive_shadow: true,
            cast_outline: true,
            outline_color: cube_models[3].2,
        }),
        EngineCmd::CmdModelCreate(CmdModelCreateArgs {
            window_id,
            model_id: cube_models[4].0,
            label: Some("Graph Cube E".into()),
            geometry_id,
            material_id: Some(material_id),
            transform: Mat4::from_translation(cube_models[4].1),
            layer_mask: 0xFFFFFFFF,
            cast_shadow: true,
            receive_shadow: true,
            cast_outline: true,
            outline_color: cube_models[4].2,
        }),
        EngineCmd::CmdModelCreate(CmdModelCreateArgs {
            window_id,
            model_id: cube_models[5].0,
            label: Some("Graph Cube F".into()),
            geometry_id,
            material_id: Some(material_id),
            transform: Mat4::from_translation(cube_models[5].1),
            layer_mask: 0xFFFFFFFF,
            cast_shadow: true,
            receive_shadow: true,
            cast_outline: true,
            outline_color: cube_models[5].2,
        }),
        EngineCmd::CmdModelCreate(CmdModelCreateArgs {
            window_id,
            model_id: listener_model_id,
            label: Some("Audio Listener".into()),
            geometry_id,
            material_id: Some(material_id),
            transform: Mat4::from_translation(Vec3::new(0.0, 3.5, 8.0)),
            layer_mask: 0,
            cast_shadow: false,
            receive_shadow: false,
            cast_outline: false,
            outline_color: Vec4::ZERO,
        }),
        EngineCmd::CmdModelCreate(CmdModelCreateArgs {
            window_id,
            model_id: emitter_model_id,
            label: Some("Audio Emitter Sphere".into()),
            geometry_id: emitter_geometry_id,
            material_id: Some(emitter_material_id),
            transform: Mat4::from_translation(emitter_pos) * Mat4::from_scale(Vec3::splat(0.6)),
            layer_mask: 0xFFFFFFFF,
            cast_shadow: false,
            receive_shadow: true,
            cast_outline: false,
            outline_color: Vec4::ZERO,
        }),
        create_floor_cmd(window_id, geometry_id, floor_material_id),
        create_shadow_config_cmd(window_id),
        EngineCmd::CmdAudioListenerCreate(CmdAudioListenerCreateArgs {
            window_id,
            model_id: listener_model_id,
        }),
        EngineCmd::CmdAudioResourceCreate(CmdAudioResourceCreateArgs {
            resource_id: audio_id,
            buffer_id: audio_buffer_id,
            total_bytes: None,
            offset_bytes: None,
        }),
        EngineCmd::CmdAudioSourceCreate(CmdAudioSourceCreateArgs {
            window_id,
            source_id: audio_source_id,
            model_id: emitter_model_id,
            position: emitter_pos,
            velocity: Vec3::ZERO,
            orientation: Quat::IDENTITY,
            gain: 1.0,
            pitch: 1.0,
            spatial: AudioSpatialParamsDto::default(),
        }),
    ];

    assert_eq!(send_commands(setup_cmds), VulframResult::Success);
    let responses = receive_responses();
    for response in responses {
        if let CommandResponse::RenderGraphSet(result) = response.response {
            println!(
                "RenderGraphSet: success={} fallback={} message={}",
                result.success, result.fallback_used, result.message
            );
        }
    }

    let skybox_bytes: std::sync::Arc<Mutex<Option<Vec<u8>>>> =
        std::sync::Arc::new(Mutex::new(None));
    {
        let skybox_bytes = std::sync::Arc::clone(&skybox_bytes);
        std::thread::spawn(move || {
            let bytes = load_texture_bytes("assets/skybox.exr");
            if let Ok(mut slot) = skybox_bytes.lock() {
                *slot = Some(bytes);
            }
        });
    }

    let audio_state = Rc::new(RefCell::new((false, false, false)));

    let audio_state_frame = Rc::clone(&audio_state);
    let audio_state_events = Rc::clone(&audio_state);
    run_loop_with_events(
        window_id,
        None,
        move |total_ms, _delta_ms| {
            let time_f = total_ms as f32 / 1000.0;
            let mut cmds = Vec::new();
            let camera_radius = 8.0;
            let camera_base_height = 3.0;
            let camera_angle = time_f * 0.35;
            let camera_height = camera_base_height + (time_f * 0.7).sin() * 1.25;
            let camera_pos = Vec3::new(
                camera_radius * camera_angle.cos(),
                camera_height,
                camera_radius * camera_angle.sin(),
            );
            let camera_transform = Mat4::look_at_rh(camera_pos, Vec3::ZERO, Vec3::Y).inverse();
            cmds.push(EngineCmd::CmdCameraUpdate(CmdCameraUpdateArgs {
                camera_id,
                label: None,
                transform: Some(camera_transform),
                kind: None,
                flags: None,
                near_far: None,
                layer_mask: None,
                order: None,
                view_position: None,
                ortho_scale: None,
            }));
            cmds.push(EngineCmd::CmdModelUpdate(CmdModelUpdateArgs {
                window_id,
                model_id: listener_model_id,
                label: None,
                geometry_id: None,
                material_id: None,
                transform: Some(camera_transform),
                layer_mask: None,
                cast_shadow: None,
                receive_shadow: None,
                cast_outline: None,
                outline_color: None,
            }));
            if let Ok(mut slot) = skybox_bytes.lock() {
                if let Some(bytes) = slot.take() {
                    upload_texture_bytes(&bytes, skybox_buffer_id);
                    cmds.push(EngineCmd::CmdTextureCreateFromBuffer(
                        CmdTextureCreateFromBufferArgs {
                            window_id,
                            texture_id: skybox_texture_id,
                            label: Some("Skybox Texture".into()),
                            buffer_id: skybox_buffer_id,
                            srgb: Some(false),
                            mode: TextureCreateMode::Standalone,
                            atlas_options: None,
                        },
                    ));
                    cmds.push(EngineCmd::CmdEnvironmentUpdate(CmdEnvironmentUpdateArgs {
                        window_id,
                        config: EnvironmentConfig {
                            msaa: MsaaConfig {
                                enabled: true,
                                sample_count: 4,
                            },
                            skybox: SkyboxConfig {
                                mode: SkyboxMode::Cubemap,
                                intensity: 1.0,
                                rotation: 0.0,
                                ground_color: Vec3::new(0.01, 0.02, 0.03),
                                horizon_color: Vec3::new(0.08, 0.12, 0.18),
                                sky_color: Vec3::new(0.18, 0.32, 0.55),
                                cubemap_texture_id: Some(skybox_texture_id),
                            },
                            post: post_config.clone(),
                        },
                    }));
                }
            }
            for (index, (model_id, base_pos, _outline)) in cube_models.iter().enumerate() {
                let wobble = time_f + index as f32 * 0.6;
                let position = *base_pos + Vec3::new(0.0, wobble.sin() * 0.25, 0.0);
                let transform = Mat4::from_translation(position)
                    * Mat4::from_euler(
                        glam::EulerRot::XYZ,
                        wobble * 0.9,
                        wobble * 0.6,
                        wobble * 0.3,
                    )
                    * Mat4::from_scale(Vec3::splat(1.15));
                cmds.push(EngineCmd::CmdModelUpdate(CmdModelUpdateArgs {
                    window_id,
                    model_id: *model_id,
                    label: None,
                    geometry_id: None,
                    material_id: None,
                    transform: Some(transform),
                    layer_mask: None,
                    cast_shadow: None,
                    receive_shadow: None,
                    cast_outline: None,
                    outline_color: None,
                }));
            }
            {
                let mut state = audio_state_frame.borrow_mut();
                if state.0 && state.1 != state.2 {
                    state.2 = state.1;
                    if state.1 {
                        cmds.push(EngineCmd::CmdAudioSourcePlay(CmdAudioSourcePlayArgs {
                            source_id: audio_source_id,
                            resource_id: audio_id,
                            timeline_id: None,
                            intensity: 1.0,
                            delay_ms: None,
                            mode: AudioPlayModeDto::Loop,
                        }));
                    } else {
                        cmds.push(EngineCmd::CmdAudioSourceStop(CmdAudioSourceStopArgs {
                            source_id: audio_source_id,
                            timeline_id: None,
                        }));
                    }
                }
            }

            cmds
        },
        move |event| {
            match &event {
                EngineEvent::System(SystemEvent::AudioReady {
                    resource_id: ready_id,
                    success,
                    message,
                }) if *ready_id == audio_id => {
                    let mut state = audio_state_events.borrow_mut();
                    state.0 = *success;
                    println!("AudioReady: success={} message={}", success, message);
                }
                EngineEvent::Keyboard(KeyboardEvent::OnInput {
                    window_id: id,
                    key_code,
                    state: ElementState::Pressed,
                    ..
                }) if *id == window_id && *key_code == 62 => {
                    let mut state = audio_state_events.borrow_mut();
                    state.1 = !state.1;
                }
                _ => {}
            }
            false
        },
    )
}

fn create_camera_cmd(camera_id: u32, label: &str, transform: Mat4) -> EngineCmd {
    EngineCmd::CmdCameraCreate(CmdCameraCreateArgs {
        camera_id,
        label: Some(label.to_string()),
        transform,
        kind: CameraKind::Perspective,
        flags: 0,
        near_far: Vec2::new(0.1, 100.0),
        layer_mask: 0xFFFFFFFF,
        order: 0,
        view_position: None,
        ortho_scale: 10.0,
    })
}

fn create_point_light_cmd(window_id: u32, light_id: u32, position: Vec4) -> EngineCmd {
    EngineCmd::CmdLightCreate(CmdLightCreateArgs {
        window_id,
        light_id,
        label: Some("Point Light".to_string()),
        kind: Some(LightKind::Point),
        position: Some(position),
        direction: None,
        color: Some(Vec4::new(1.0, 1.0, 1.0, 1.0)),
        ground_color: None,
        intensity: Some(20.0),
        range: Some(30.0),
        spot_inner_outer: None,
        layer_mask: 0xFFFFFFFF,
        cast_shadow: true,
    })
}

fn create_ambient_light_cmd(
    window_id: u32,
    light_id: u32,
    color: Vec4,
    intensity: f32,
) -> EngineCmd {
    EngineCmd::CmdLightCreate(CmdLightCreateArgs {
        window_id,
        light_id,
        label: Some("Ambient Light".to_string()),
        kind: Some(LightKind::Ambient),
        position: None,
        direction: None,
        color: Some(color),
        ground_color: None,
        intensity: Some(intensity),
        range: Some(1.0),
        spot_inner_outer: None,
        layer_mask: 0xFFFFFFFF,
        cast_shadow: false,
    })
}

fn create_standard_material_cmd(
    window_id: u32,
    material_id: u32,
    label: &str,
    base_color: Vec4,
    base_tex_id: Option<u32>,
    emissive_color: Option<Vec4>,
) -> EngineCmd {
    EngineCmd::CmdMaterialCreate(CmdMaterialCreateArgs {
        window_id,
        material_id,
        label: Some(label.to_string()),
        kind: MaterialKind::Standard,
        options: Some(MaterialOptions::Standard(StandardOptions {
            base_color,
            base_tex_id,
            base_sampler: Some(MaterialSampler::LinearClamp),
            emissive_color: emissive_color.unwrap_or(Vec4::ZERO),
            ..Default::default()
        })),
    })
}

fn create_texture_cmd(window_id: u32, texture_id: u32, label: &str, buffer_id: u64) -> EngineCmd {
    EngineCmd::CmdTextureCreateFromBuffer(CmdTextureCreateFromBufferArgs {
        window_id,
        texture_id,
        label: Some(label.to_string()),
        buffer_id,
        srgb: Some(true),
        mode: TextureCreateMode::Standalone,
        atlas_options: None,
    })
}

fn create_floor_cmd(window_id: u32, geometry_id: u32, material_id: u32) -> EngineCmd {
    EngineCmd::CmdModelCreate(CmdModelCreateArgs {
        window_id,
        model_id: 2000,
        label: Some("Floor".to_string()),
        geometry_id,
        material_id: Some(material_id),
        transform: Mat4::from_translation(Vec3::new(0.0, -6.0, 0.0))
            * Mat4::from_scale(Vec3::new(20.0, 0.1, 20.0)),
        layer_mask: 0xFFFFFFFF,
        cast_shadow: false,
        receive_shadow: true,
        cast_outline: false,
        outline_color: Vec4::ZERO,
    })
}

fn create_instanced_cubes(
    window_id: u32,
    geometry_id: u32,
    material_id: u32,
) -> (Vec<CubeData>, Vec<EngineCmd>) {
    let mut rng = rand::rng();
    let mut cmds = Vec::new();
    let mut cubes = Vec::new();

    for i in 0..100 {
        let x: f32 = rng.random_range(-5.0..5.0);
        let y: f32 = rng.random_range(-5.0..5.0);
        let z: f32 = rng.random_range(-5.0..5.0);

        let rot_x: f32 = rng.random_range(0.0..std::f32::consts::TAU);
        let rot_y: f32 = rng.random_range(0.0..std::f32::consts::TAU);

        let model_id = 100 + i;

        cubes.push(CubeData {
            id: model_id,
            initial_pos: Vec3::new(x, y, z),
            phase: rng.random_range(0.0..std::f32::consts::TAU),
        });

        cmds.push(EngineCmd::CmdModelCreate(CmdModelCreateArgs {
            window_id,
            model_id,
            label: Some(format!("Cube {}", i)),
            geometry_id,
            material_id: Some(material_id),
            transform: Mat4::from_translation(Vec3::new(x, y, z))
                * Mat4::from_euler(glam::EulerRot::XYZ, rot_x, rot_y, 0.0)
                * Mat4::from_scale(Vec3::splat(0.4)),
            layer_mask: 0xFFFFFFFF,
            cast_shadow: true,
            receive_shadow: true,
            cast_outline: false,
            outline_color: Vec4::ZERO,
        }));
    }

    (cubes, cmds)
}

fn create_shadow_config_cmd(window_id: u32) -> EngineCmd {
    EngineCmd::CmdShadowConfigure(CmdShadowConfigureArgs {
        window_id,
        config: ShadowConfig {
            tile_resolution: 2048,
            atlas_tiles_w: 16,
            atlas_tiles_h: 16,
            atlas_layers: 2,
            virtual_grid_size: 1,
            smoothing: 1,
            normal_bias: 0.01,
        },
    })
}

fn draw_axes_gizmos() -> Vec<EngineCmd> {
    vec![
        EngineCmd::CmdGizmoDrawLine(CmdGizmoDrawLineArgs {
            start: Vec3::ZERO,
            end: Vec3::X * 5.0,
            color: Vec4::new(1.0, 0.0, 0.0, 1.0),
        }),
        EngineCmd::CmdGizmoDrawLine(CmdGizmoDrawLineArgs {
            start: Vec3::ZERO,
            end: Vec3::Y * 5.0,
            color: Vec4::new(0.0, 1.0, 0.0, 1.0),
        }),
        EngineCmd::CmdGizmoDrawLine(CmdGizmoDrawLineArgs {
            start: Vec3::ZERO,
            end: Vec3::Z * 5.0,
            color: Vec4::new(0.0, 0.0, 1.0, 1.0),
        }),
    ]
}

fn default_camera_transform() -> Mat4 {
    Mat4::look_at_rh(
        Vec3::new(0.0, 10.0, 15.0),
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::Y,
    )
    .inverse()
}

fn load_texture_bytes(path: &str) -> Vec<u8> {
    fs::read(path).expect("failed to read texture")
}

fn upload_texture_bytes(bytes: &[u8], buffer_id: u64) {
    assert_eq!(
        core::vulfram_upload_buffer(
            buffer_id,
            upload_type_to_u32(UploadType::ImageData),
            bytes.as_ptr(),
            bytes.len()
        ),
        VulframResult::Success
    );
}

fn upload_binary_bytes(bytes: &[u8], buffer_id: u64) {
    assert_eq!(
        core::vulfram_upload_buffer(
            buffer_id,
            upload_type_to_u32(UploadType::BinaryAsset),
            bytes.as_ptr(),
            bytes.len()
        ),
        VulframResult::Success
    );
}

fn upload_texture(path: &str, buffer_id: u64) {
    let texture_bytes = load_texture_bytes(path);
    upload_texture_bytes(&texture_bytes, buffer_id);
}

fn upload_buffer<T: bytemuck::Pod>(buffer_id: u64, upload_type: UploadType, data: &[T]) {
    let bytes = cast_slice(data);
    assert_eq!(
        core::vulfram_upload_buffer(
            buffer_id,
            upload_type_to_u32(upload_type),
            bytes.as_ptr() as *const u8,
            bytes.len()
        ),
        VulframResult::Success
    );
}

fn upload_type_to_u32(upload_type: UploadType) -> u32 {
    match upload_type {
        UploadType::Raw => 0,
        UploadType::ShaderSource => 1,
        UploadType::GeometryData => 2,
        UploadType::VertexData => 3,
        UploadType::IndexData => 4,
        UploadType::ImageData => 5,
        UploadType::BinaryAsset => 6,
    }
}

fn build_skinned_plane(
    grid_x: u32,
    grid_z: u32,
    size: f32,
    bone_count: u32,
) -> (
    Vec<[f32; 3]>,
    Vec<[f32; 3]>,
    Vec<[f32; 2]>,
    Vec<[u16; 4]>,
    Vec<[f32; 4]>,
    Vec<u32>,
) {
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut joints = Vec::new();
    let mut weights = Vec::new();
    let mut indices = Vec::new();

    let step_x = 1.0 / grid_x as f32;
    let step_z = 1.0 / grid_z as f32;
    let half = size * 0.5;
    let segments = (bone_count - 1).max(1) as f32;
    let smoothstep = |t: f32| t * t * (3.0 - 2.0 * t);

    for z in 0..=grid_z {
        for x in 0..=grid_x {
            let fx = x as f32 * step_x;
            let fz = z as f32 * step_z;
            let pos_x = fx * size - half;
            let pos_z = fz * size - half;

            positions.push([pos_x, 0.0, pos_z]);
            normals.push([0.0, 1.0, 0.0]);
            uvs.push([fx, fz]);

            let bone_f = fx * segments;
            let bone_idx = bone_f.floor() as u32;
            let next_idx = (bone_idx + 1).min(bone_count - 1);
            let t = smoothstep(bone_f - bone_idx as f32);
            joints.push([bone_idx as u16, next_idx as u16, 0, 0]);
            weights.push([1.0 - t, t, 0.0, 0.0]);
        }
    }

    let verts_x = grid_x + 1;
    for z in 0..grid_z {
        for x in 0..grid_x {
            let i0 = z * verts_x + x;
            let i1 = i0 + 1;
            let i2 = i0 + verts_x;
            let i3 = i2 + 1;
            indices.extend_from_slice(&[i0, i2, i1, i1, i2, i3]);
        }
    }

    (positions, normals, uvs, joints, weights, indices)
}

fn run_loop<F>(window_id: u32, max_duration: Option<Duration>, on_frame: F) -> bool
where
    F: FnMut(u64, u32) -> Vec<EngineCmd>,
{
    run_loop_with_events(window_id, max_duration, on_frame, |_| false)
}

fn run_loop_with_events<F, G>(
    window_id: u32,
    max_duration: Option<Duration>,
    mut on_frame: F,
    mut on_event: G,
) -> bool
where
    F: FnMut(u64, u32) -> Vec<EngineCmd>,
    G: FnMut(EngineEvent) -> bool,
{
    let start_time = Instant::now();
    let mut last_frame_time = Instant::now();
    let mut total_ms: u64 = 0;
    let target_frame_time = Duration::from_millis(16);

    loop {
        if let Some(max_duration) = max_duration {
            if start_time.elapsed() >= max_duration {
                return false;
            }
        }

        let now = Instant::now();
        let delta_ms = now.duration_since(last_frame_time).as_millis() as u32;
        last_frame_time = now;
        total_ms += delta_ms as u64;

        let frame_cmds = on_frame(total_ms, delta_ms);
        if !frame_cmds.is_empty() {
            let _ = send_commands(frame_cmds);
        }

        let tick_start = Instant::now();
        assert_eq!(
            core::vulfram_tick(total_ms, delta_ms),
            VulframResult::Success
        );

        let _ = receive_responses();
        if total_ms % 1000 == 0 {
            if let Some(profiling) = get_profiling() {
                println!("Profiling: {:?}", profiling);
            }
        }

        if handle_events(window_id, &mut on_event) {
            let _ = send_commands(vec![EngineCmd::CmdWindowClose(CmdWindowCloseArgs {
                window_id,
            })]);
            return true;
        }

        let elapsed = tick_start.elapsed();
        if elapsed < target_frame_time {
            std::thread::sleep(target_frame_time - elapsed);
        }
    }
}

fn is_close_event(window_id: u32, event: &EngineEvent) -> bool {
    match event {
        EngineEvent::Window(WindowEvent::OnCloseRequest { window_id: id }) if *id == window_id => {
            true
        }
        EngineEvent::Keyboard(KeyboardEvent::OnInput {
            window_id: id,
            key_code,
            state: ElementState::Pressed,
            ..
        }) if *id == window_id && *key_code == 106 => true,
        _ => false,
    }
}

fn handle_events<F>(window_id: u32, on_event: &mut F) -> bool
where
    F: FnMut(EngineEvent) -> bool,
{
    let events = receive_events();
    for event in events {
        if is_close_event(window_id, &event) {
            return true;
        }
        if on_event(event) {
            return true;
        }
    }
    false
}

fn get_profiling() -> Option<core::profiling::cmd::ProfilingData> {
    let mut ptr = std::ptr::null();
    let mut len: usize = 0;
    let result = core::vulfram_get_profiling(&mut ptr, &mut len);
    if result != VulframResult::Success || len == 0 {
        return None;
    }
    let bytes = unsafe { Box::from_raw(std::slice::from_raw_parts_mut(ptr as *mut u8, len)) };
    let profiling = from_slice(&bytes).ok()?;
    Some(profiling)
}

fn pump_for(duration: Duration) {
    let start = Instant::now();
    let mut total_ms: u64 = 0;
    while start.elapsed() < duration {
        assert_eq!(core::vulfram_tick(total_ms, 16), VulframResult::Success);
        total_ms += 16;
        std::thread::sleep(Duration::from_millis(16));
    }
}

fn wait_for_confirmation(_window_id: u32) {
    for _ in 0..100 {
        let responses = receive_responses();
        for response in responses {
            match response.response {
                CommandResponse::WindowCreate(res) => {
                    if res.success {
                        return;
                    } else {
                        panic!("Window creation failed: {}", res.message);
                    }
                }
                _ => {}
            }
        }
        std::thread::sleep(Duration::from_millis(10));
        assert_eq!(core::vulfram_tick(0, 0), VulframResult::Success);
    }
}

fn send_commands(cmds: Vec<EngineCmd>) -> VulframResult {
    let envelopes: Vec<EngineCmdEnvelope> = cmds
        .into_iter()
        .enumerate()
        .map(|(idx, cmd)| EngineCmdEnvelope {
            id: idx as u64,
            cmd,
        })
        .collect();
    let data = to_vec_named(&envelopes).expect("failed to serialize commands");
    core::vulfram_send_queue(data.as_ptr(), data.len())
}

fn receive_responses() -> Vec<CommandResponseEnvelope> {
    let mut ptr = std::ptr::null();
    let mut len: usize = 0;
    let result = core::vulfram_receive_queue(&mut ptr, &mut len);

    if result != VulframResult::Success || len == 0 {
        return Vec::new();
    }

    let bytes = unsafe { Box::from_raw(std::slice::from_raw_parts_mut(ptr as *mut u8, len)) };
    let responses = from_slice(&bytes).expect("failed to deserialize responses");
    responses
}

fn receive_events() -> Vec<EngineEvent> {
    let mut ptr = std::ptr::null();
    let mut len: usize = 0;
    let result = core::vulfram_receive_events(&mut ptr, &mut len);

    if result != VulframResult::Success || len == 0 {
        return Vec::new();
    }

    let bytes = unsafe { Box::from_raw(std::slice::from_raw_parts_mut(ptr as *mut u8, len)) };
    let events = from_slice(&bytes).expect("failed to deserialize events");
    events
}

struct CubeData {
    id: u32,
    initial_pos: Vec3,
    phase: f32,
}
