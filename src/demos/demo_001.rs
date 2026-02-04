use crate::core::VulframResult;
use crate::core::cmd::EngineCmd;
use crate::core::render::gizmos::CmdGizmoDrawAabbArgs;
use crate::core::resources::{CmdModelUpdateArgs, CmdPrimitiveGeometryCreateArgs, PrimitiveShape};
use glam::{Mat4, Vec3, Vec4};

use crate::demos::common::{
    create_camera_cmd, create_floor_cmd, create_instanced_cubes, create_point_light_cmd,
    create_shadow_config_cmd, create_standard_material_cmd, create_texture_cmd,
    default_camera_transform, draw_axes_gizmos, receive_responses, run_loop, send_commands,
    upload_texture,
};

pub fn run(window_id: u32) -> bool {
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
