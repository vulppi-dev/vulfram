use crate::core::VulframResult;
use crate::core::cmd::EngineCmd;
use crate::core::resources::{
    CmdEnvironmentUpdateArgs, CmdModelCreateArgs, CmdPrimitiveGeometryCreateArgs,
    EnvironmentConfig, MsaaConfig, PostProcessConfig, PrimitiveShape, SkyboxConfig, SkyboxMode,
};
use glam::{Mat4, Vec3, Vec4};

use crate::demos::common::{
    create_ambient_light_cmd, create_camera_cmd, create_point_light_cmd, create_shadow_config_cmd,
    create_standard_material_cmd, draw_axes_gizmos, receive_responses, run_loop, send_commands,
};

pub fn run(window_id: u32) -> bool {
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
            frame_cmds.push(EngineCmd::CmdModelUpdate(
                crate::core::resources::CmdModelUpdateArgs {
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
                },
            ));
        }

        frame_cmds
    })
}
