use crate::core::VulframResult;
use crate::core::cmd::EngineCmd;
use crate::core::render::gizmos::CmdGizmoDrawLineArgs;
use crate::core::resources::shadow::{CmdShadowConfigureArgs, ShadowConfig};
use crate::core::resources::{
    CameraKind, CmdCameraCreateArgs, CmdLightCreateArgs, CmdMaterialCreateArgs,
    CmdModelCreateArgs, CmdTextureCreateFromBufferArgs, LightKind, MaterialKind, MaterialOptions,
    MaterialSampler, StandardOptions, TextureCreateMode,
};
use glam::{Mat4, Vec2, Vec3, Vec4};
use rand::Rng;

use super::loop_utils::send_commands;

pub fn create_window(window_id: u32, title: &str) {
    let create_cmd = EngineCmd::CmdWindowCreate(crate::core::window::CmdWindowCreateArgs {
        window_id,
        title: title.into(),
        size: glam::UVec2::new(1280, 720),
        resizable: true,
        initial_state: crate::core::window::EngineWindowState::Maximized,
        ..Default::default()
    });
    assert_eq!(send_commands(vec![create_cmd]), VulframResult::Success);
}

pub fn create_camera_cmd(camera_id: u32, label: &str, transform: Mat4) -> EngineCmd {
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

pub fn create_point_light_cmd(window_id: u32, light_id: u32, position: Vec4) -> EngineCmd {
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

pub fn create_ambient_light_cmd(
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

pub fn create_standard_material_cmd(
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

pub fn create_texture_cmd(window_id: u32, texture_id: u32, label: &str, buffer_id: u64) -> EngineCmd {
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

pub fn create_floor_cmd(window_id: u32, geometry_id: u32, material_id: u32) -> EngineCmd {
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

pub fn create_instanced_cubes(
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

pub fn create_shadow_config_cmd(window_id: u32) -> EngineCmd {
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

pub fn draw_axes_gizmos() -> Vec<EngineCmd> {
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

pub fn default_camera_transform() -> Mat4 {
    Mat4::look_at_rh(
        Vec3::new(0.0, 10.0, 15.0),
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::Y,
    )
    .inverse()
}

pub struct CubeData {
    pub id: u32,
    pub initial_pos: Vec3,
    pub phase: f32,
}
