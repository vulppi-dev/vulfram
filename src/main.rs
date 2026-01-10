mod core;

use crate::core::VulframResult;
use crate::core::buffers::state::UploadType;
use crate::core::cmd::{CommandResponse, CommandResponseEnvelope, EngineCmd, EngineCmdEnvelope};
use crate::core::resources::shadow::{CmdShadowConfigureArgs, ShadowConfig};
use crate::core::resources::{
    CameraKind, CmdCameraCreateArgs, CmdLightCreateArgs, CmdMaterialCreateArgs, CmdModelCreateArgs,
    CmdPrimitiveGeometryCreateArgs, CmdTextureCreateFromBufferArgs, ForwardAtlasOptions, LightKind,
    MaterialKind, MaterialOptions, MaterialSampler, PbrOptions, PrimitiveShape, StandardOptions,
    SurfaceType, TextureCreateMode,
};
use crate::core::system::{CmdNotificationSendArgs, NotificationLevel};
use crate::core::window::{CmdWindowCloseArgs, CmdWindowCreateArgs};
use glam::{Mat4, Vec2, Vec3, Vec4};
use rmp_serde::{from_slice, to_vec_named};
use std::fs;
use std::sync::Mutex;
use std::time::{Duration, Instant};

static ENGINE_GUARD: Mutex<()> = Mutex::new(());

fn main() {
    let _lock = ENGINE_GUARD.lock().unwrap();

    assert_eq!(core::vulfram_init(), VulframResult::Success);

    let window_id: u32 = 1;
    let create_cmd = EngineCmd::CmdWindowCreate(CmdWindowCreateArgs {
        window_id,
        title: "Vulfram Render Test".into(),
        size: glam::UVec2::new(1280, 720),
        resizable: true,
        initial_state: crate::core::window::EngineWindowState::Maximized,
        ..Default::default()
    });
    assert_eq!(send_commands(vec![create_cmd]), VulframResult::Success);

    // Give some time for window to be created and confirm it
    pump_for(Duration::from_millis(200));
    wait_for_confirmation(window_id);

    let geometry_cube: u32 = 1;
    let geometry_plane: u32 = 2;
    let geometry_sphere: u32 = 3;
    let camera_id: u32 = 1;
    let model_cube: u32 = 1;
    let model_plane: u32 = 2;
    let model_light_marker: u32 = 3;
    let material_cube: u32 = 10;
    let material_plane: u32 = 11;
    let material_masked: u32 = 12;
    let material_transparent: u32 = 13;
    let texture_test: u32 = 20;
    let texture_atlas: u32 = 21;
    let texture_normal: u32 = 22;
    let texture_alpha: u32 = 23;
    let texture_buffer: u64 = 1;
    let texture_atlas_buffer: u64 = 2;
    let texture_normal_buffer: u64 = 3;
    let texture_alpha_buffer: u64 = 4;

    let texture_bytes = fs::read("assets/colo_test_texture.png")
        .expect("failed to read assets/colo_test_texture.png");
    let normal_bytes = fs::read("assets/normal_test_texture.png")
        .expect("failed to read assets/normal_test_texture.png");
    let alpha_bytes = fs::read("assets/alpha_test_texture.png")
        .expect("failed to read assets/alpha_test_texture.png");
    assert_eq!(
        core::vulfram_upload_buffer(
            texture_buffer,
            UploadType::ImageData as u32,
            texture_bytes.as_ptr(),
            texture_bytes.len()
        ),
        VulframResult::Success
    );
    assert_eq!(
        core::vulfram_upload_buffer(
            texture_atlas_buffer,
            UploadType::ImageData as u32,
            texture_bytes.as_ptr(),
            texture_bytes.len()
        ),
        VulframResult::Success
    );
    assert_eq!(
        core::vulfram_upload_buffer(
            texture_normal_buffer,
            UploadType::ImageData as u32,
            normal_bytes.as_ptr(),
            normal_bytes.len()
        ),
        VulframResult::Success
    );
    assert_eq!(
        core::vulfram_upload_buffer(
            texture_alpha_buffer,
            UploadType::ImageData as u32,
            alpha_bytes.as_ptr(),
            alpha_bytes.len()
        ),
        VulframResult::Success
    );

    let setup_cmds = vec![
        // 1. Create geometries
        EngineCmd::CmdPrimitiveGeometryCreate(CmdPrimitiveGeometryCreateArgs {
            window_id,
            geometry_id: geometry_cube,
            shape: PrimitiveShape::Cube,
            options: None,
        }),
        EngineCmd::CmdPrimitiveGeometryCreate(CmdPrimitiveGeometryCreateArgs {
            window_id,
            geometry_id: geometry_plane,
            shape: PrimitiveShape::Plane,
            options: None,
        }),
        EngineCmd::CmdPrimitiveGeometryCreate(CmdPrimitiveGeometryCreateArgs {
            window_id,
            geometry_id: geometry_sphere,
            shape: PrimitiveShape::Sphere,
            options: None,
        }),
        // 2. Create a camera
        EngineCmd::CmdCameraCreate(CmdCameraCreateArgs {
            camera_id,
            transform: Mat4::look_at_rh(
                Vec3::new(0.0, 14.0, 18.0),
                Vec3::new(0.0, 0.0, 0.0),
                Vec3::Y,
            )
            .inverse(),
            kind: CameraKind::Perspective,
            flags: 0,
            near_far: Vec2::new(0.1, 100.0),
            layer_mask: 0xFFFFFFFF,
            order: 0,
            view_position: None,
            ortho_scale: 10.0,
        }),
        // 3.1 Create a point light
        EngineCmd::CmdLightCreate(CmdLightCreateArgs {
            window_id,
            light_id: 2,
            kind: Some(LightKind::Point),
            position: Some(Vec4::new(5.0, 5.0, 0.0, 1.0)),
            direction: None,
            color: Some(Vec4::new(1.0, 1.0, 1.0, 1.0)),
            ground_color: None,
            intensity: Some(10.0),
            range: Some(20.0),
            spot_inner_outer: None,
            layer_mask: 0xFFFFFFFF,
            cast_shadow: true,
        }),
        // 3.6 Create a point light in center above cube
        EngineCmd::CmdLightCreate(CmdLightCreateArgs {
            window_id,
            light_id: 3,
            kind: Some(LightKind::Point),
            position: Some(Vec4::new(0.0, 5.0, -4.0, 1.0)),
            direction: None,
            color: Some(Vec4::new(0.0, 0.0, 1.0, 1.0)),
            ground_color: None,
            intensity: Some(10.0),
            range: Some(20.0),
            spot_inner_outer: None,
            layer_mask: 0xFFFFFFFF,
            cast_shadow: true,
        }),
        EngineCmd::CmdLightCreate(CmdLightCreateArgs {
            window_id,
            light_id: 4,
            kind: Some(LightKind::Point),
            position: Some(Vec4::new(0.0, 5.0, 4.0, 1.0)),
            direction: None,
            color: Some(Vec4::new(1.0, 0.0, 0.0, 1.0)),
            ground_color: None,
            intensity: Some(10.0),
            range: Some(20.0),
            spot_inner_outer: None,
            layer_mask: 0xFFFFFFFF,
            cast_shadow: true,
        }),
        EngineCmd::CmdTextureCreateFromBuffer(CmdTextureCreateFromBufferArgs {
            window_id,
            texture_id: texture_test,
            buffer_id: texture_buffer,
            srgb: Some(true),
            mode: TextureCreateMode::Standalone,
            atlas_options: None,
        }),
        EngineCmd::CmdTextureCreateFromBuffer(CmdTextureCreateFromBufferArgs {
            window_id,
            texture_id: texture_atlas,
            buffer_id: texture_atlas_buffer,
            srgb: Some(true),
            mode: TextureCreateMode::ForwardAtlas,
            atlas_options: Some(ForwardAtlasOptions {
                tile_px: 256,
                layers: 1,
            }),
        }),
        EngineCmd::CmdTextureCreateFromBuffer(CmdTextureCreateFromBufferArgs {
            window_id,
            texture_id: texture_normal,
            buffer_id: texture_normal_buffer,
            srgb: Some(false),
            mode: TextureCreateMode::Standalone,
            atlas_options: None,
        }),
        EngineCmd::CmdTextureCreateFromBuffer(CmdTextureCreateFromBufferArgs {
            window_id,
            texture_id: texture_alpha,
            buffer_id: texture_alpha_buffer,
            srgb: Some(true),
            mode: TextureCreateMode::Standalone,
            atlas_options: None,
        }),
        // 3.7 Create a soft pink standard material for the cube
        EngineCmd::CmdMaterialCreate(CmdMaterialCreateArgs {
            window_id,
            material_id: material_cube,
            kind: MaterialKind::Standard,
            options: Some(MaterialOptions::Standard(StandardOptions {
                base_color: Vec4::ONE,
                base_tex_id: Some(texture_test),
                base_sampler: Some(MaterialSampler::LinearClamp),
                normal_tex_id: Some(texture_normal),
                normal_sampler: Some(MaterialSampler::LinearClamp),
                ..Default::default()
            })),
        }),
        EngineCmd::CmdMaterialCreate(CmdMaterialCreateArgs {
            window_id,
            material_id: material_plane,
            kind: MaterialKind::Pbr,
            options: Some(MaterialOptions::Pbr(PbrOptions {
                base_color: Vec4::ONE,
                emissive_color: Vec4::ZERO,
                metallic: 0.8,
                roughness: 0.1,
                ao: 1.0,
                normal_scale: 1.0,
                surface_type: SurfaceType::Opaque,
                base_tex_id: Some(texture_atlas),
                base_sampler: Some(MaterialSampler::LinearClamp),
                normal_tex_id: Some(texture_normal),
                normal_sampler: Some(MaterialSampler::LinearClamp),
                ..Default::default()
            })),
        }),
        EngineCmd::CmdMaterialCreate(CmdMaterialCreateArgs {
            window_id,
            material_id: material_masked,
            kind: MaterialKind::Standard,
            options: Some(MaterialOptions::Standard(StandardOptions {
                base_color: Vec4::ONE,
                surface_type: SurfaceType::Masked,
                base_tex_id: Some(texture_alpha),
                base_sampler: Some(MaterialSampler::LinearClamp),
                ..Default::default()
            })),
        }),
        EngineCmd::CmdMaterialCreate(CmdMaterialCreateArgs {
            window_id,
            material_id: material_transparent,
            kind: MaterialKind::Standard,
            options: Some(MaterialOptions::Standard(StandardOptions {
                base_color: Vec4::new(1.0, 0.2, 0.6, 0.5),
                surface_type: SurfaceType::Transparent,
                ..Default::default()
            })),
        }),
        // 4. Create models
        EngineCmd::CmdModelCreate(CmdModelCreateArgs {
            window_id,
            model_id: model_cube,
            geometry_id: geometry_cube,
            material_id: Some(material_cube),
            transform: Mat4::from_translation(Vec3::new(0.0, 1.0, 0.0)),
            layer_mask: 0xFFFFFFFF,
            cast_shadow: true,
            receive_shadow: true,
        }),
        EngineCmd::CmdModelCreate(CmdModelCreateArgs {
            window_id,
            model_id: model_plane,
            geometry_id: geometry_plane,
            material_id: Some(material_plane),
            transform: Mat4::from_translation(Vec3::new(0.0, 0.0, 0.0))
                * Mat4::from_rotation_x(-std::f32::consts::FRAC_PI_2)
                * Mat4::from_scale(Vec3::new(12.0, 12.0, 1.0)),
            layer_mask: 0xFFFFFFFF,
            cast_shadow: true,
            receive_shadow: true,
        }),
        EngineCmd::CmdModelCreate(CmdModelCreateArgs {
            window_id,
            model_id: 4,
            geometry_id: geometry_plane,
            material_id: Some(material_masked),
            transform: Mat4::from_translation(Vec3::new(-6.0, 1.0, 0.0))
                * Mat4::from_rotation_x(-std::f32::consts::FRAC_PI_2)
                * Mat4::from_scale(Vec3::new(4.0, 4.0, 1.0)),
            layer_mask: 0xFFFFFFFF,
            cast_shadow: true,
            receive_shadow: true,
        }),
        EngineCmd::CmdModelCreate(CmdModelCreateArgs {
            window_id,
            model_id: 5,
            geometry_id: geometry_plane,
            material_id: Some(material_transparent),
            transform: Mat4::from_translation(Vec3::new(6.0, 1.0, 0.0))
                * Mat4::from_rotation_x(-std::f32::consts::FRAC_PI_2)
                * Mat4::from_scale(Vec3::new(4.0, 4.0, 1.0)),
            layer_mask: 0xFFFFFFFF,
            cast_shadow: false,
            receive_shadow: true,
        }),
        EngineCmd::CmdModelCreate(CmdModelCreateArgs {
            window_id,
            model_id: model_light_marker,
            geometry_id: geometry_sphere,
            material_id: None,
            transform: Mat4::from_translation(Vec3::new(5.0, 5.0, 0.0))
                * Mat4::from_scale(Vec3::splat(0.2)),
            layer_mask: 0xFFFFFFFF,
            cast_shadow: false,
            receive_shadow: false,
        }),
        // 5. Configure Shadows: High Res, No Smoothing
        EngineCmd::CmdShadowConfigure(CmdShadowConfigureArgs {
            window_id,
            config: ShadowConfig {
                tile_resolution: 1024,
                atlas_tiles_w: 16, // 16x16 = 256 tiles total
                atlas_tiles_h: 16, // 16x16 = 256 tiles total
                atlas_layers: 4,
                virtual_grid_size: 1,
                smoothing: 2,
            },
        }),
    ];

    assert_eq!(send_commands(setup_cmds), VulframResult::Success);
    let _ = receive_responses();

    // Test notification
    let notification_cmd = EngineCmd::CmdNotificationSend(CmdNotificationSendArgs {
        id: Some("test-notif".into()),
        title: "🦊 Vulfram Core".into(),
        body: "Notificações implementadas com sucesso! Clique para testar o evento.".into(),
        level: NotificationLevel::Success,
        timeout: Some(5000),
    });
    assert_eq!(
        send_commands(vec![notification_cmd]),
        VulframResult::Success
    );
    let _ = receive_responses();

    let start_time = Instant::now();
    let mut last_frame_time = Instant::now();
    let mut total_ms: u64 = 0;

    while start_time.elapsed() < Duration::from_secs(20) {
        let now = Instant::now();
        let delta_ms = now.duration_since(last_frame_time).as_millis() as u32;
        last_frame_time = now;
        total_ms += delta_ms as u64;

        // Update cube rotation and position to see shadow moving
        let angle = (total_ms as f32 / 1000.0) * 1.0;
        let x_pos = (total_ms as f32 / 1000.0).sin() * 4.0;
        let rotation = Mat4::from_translation(Vec3::new(x_pos, 1.5, 0.0))
            * Mat4::from_euler(glam::EulerRot::XYZ, angle, angle * 0.5, 0.0);
        let update_cmd = EngineCmd::CmdModelUpdate(crate::core::resources::CmdModelUpdateArgs {
            window_id,
            model_id: model_cube,
            geometry_id: None,
            material_id: None,
            transform: Some(rotation),
            layer_mask: None,
            cast_shadow: None,
            receive_shadow: None,
        });

        // Update plane rotation
        let plane_angle = (total_ms as f32 / 4000.0) * 0.5;
        let plane_rotation = Mat4::from_rotation_y(plane_angle)
            * Mat4::from_rotation_x(-std::f32::consts::FRAC_PI_2)
            * Mat4::from_scale(Vec3::new(20.0, 20.0, 1.0));
        let plane_update = EngineCmd::CmdModelUpdate(crate::core::resources::CmdModelUpdateArgs {
            window_id,
            model_id: model_plane,
            geometry_id: None,
            material_id: None,
            transform: Some(plane_rotation),
            layer_mask: None,
            cast_shadow: None,
            receive_shadow: None,
        });

        // Update point light position
        let light_x = (total_ms as f32 / 1000.0).cos() * 5.0;
        let light_z = (total_ms as f32 / 1000.0).sin() * 5.0;
        let light_update = EngineCmd::CmdLightUpdate(crate::core::resources::CmdLightUpdateArgs {
            window_id,
            light_id: 2,
            kind: None,
            position: Some(Vec4::new(light_x, 5.0, light_z, 1.0)),
            direction: None,
            color: None,
            ground_color: None,
            intensity: None,
            range: None,
            spot_inner_outer: None,
            layer_mask: None,
            cast_shadow: None,
        });

        let marker_update = EngineCmd::CmdModelUpdate(crate::core::resources::CmdModelUpdateArgs {
            window_id,
            model_id: model_light_marker,
            geometry_id: None,
            material_id: None,
            transform: Some(
                Mat4::from_translation(Vec3::new(light_x, 5.0, light_z))
                    * Mat4::from_scale(Vec3::splat(0.2)),
            ),
            layer_mask: None,
            cast_shadow: None,
            receive_shadow: None,
        });

        // Demo Gizmos
        let mut gizmo_cmds = vec![
            // Axis at origin
            EngineCmd::CmdGizmoDrawLine(crate::core::cmd::gizmo::CmdGizmoDrawLineArgs {
                start: Vec3::ZERO,
                end: Vec3::X * 3.0,
                color: Vec4::new(1.0, 0.2, 0.2, 1.0), // Soft Red
            }),
            EngineCmd::CmdGizmoDrawLine(crate::core::cmd::gizmo::CmdGizmoDrawLineArgs {
                start: Vec3::ZERO,
                end: Vec3::Y * 3.0,
                color: Vec4::new(0.2, 1.0, 0.2, 1.0), // Soft Green
            }),
            EngineCmd::CmdGizmoDrawLine(crate::core::cmd::gizmo::CmdGizmoDrawLineArgs {
                start: Vec3::ZERO,
                end: Vec3::Z * 3.0,
                color: Vec4::new(0.2, 0.2, 1.0, 1.0), // Soft Blue
            }),
            // AABB around the moving cube
            EngineCmd::CmdGizmoDrawAabb(crate::core::cmd::gizmo::CmdGizmoDrawAabbArgs {
                min: Vec3::new(x_pos - 0.6, 1.0 - 0.6, -0.6),
                max: Vec3::new(x_pos + 0.6, 2.0 + 0.6, 0.6),
                color: Vec4::new(1.0, 1.0, 0.0, 1.0), // Yellow
            }),
        ];

        let mut all_cmds = vec![update_cmd, plane_update, light_update, marker_update];
        all_cmds.append(&mut gizmo_cmds);

        let _ = send_commands(all_cmds);

        let start_time = Instant::now();
        assert_eq!(
            core::vulfram_tick(total_ms, delta_ms),
            VulframResult::Success
        );

        // Process any responses or events (optional for this test)
        let _ = receive_responses();
        let events = receive_events();
        for event in events {
            if let crate::core::cmd::EngineEvent::System(sys_event) = event {
                println!("Received system event: {:?}", sys_event);
            }
        }

        let elapsed = start_time.elapsed();
        let target_frame_time = Duration::from_millis(16);
        if elapsed < target_frame_time {
            std::thread::sleep(target_frame_time - elapsed);
        }
    }

    let close_cmd = EngineCmd::CmdWindowClose(CmdWindowCloseArgs { window_id });
    assert_eq!(send_commands(vec![close_cmd]), VulframResult::Success);
    pump_for(Duration::from_millis(100));

    assert_eq!(core::vulfram_dispose(), VulframResult::Success);
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

    // Reclaim the buffer allocated by the core
    let bytes = unsafe { Box::from_raw(std::slice::from_raw_parts_mut(ptr as *mut u8, len)) };
    let responses = from_slice(&bytes).expect("failed to deserialize responses");
    responses
}

fn receive_events() -> Vec<crate::core::cmd::EngineEvent> {
    let mut ptr = std::ptr::null();
    let mut len: usize = 0;
    let result = core::vulfram_receive_events(&mut ptr, &mut len);

    if result != VulframResult::Success || len == 0 {
        return Vec::new();
    }

    // Reclaim the buffer allocated by the core
    let bytes = unsafe { Box::from_raw(std::slice::from_raw_parts_mut(ptr as *mut u8, len)) };
    let events = from_slice(&bytes).expect("failed to deserialize events");
    events
}
