mod core;

use crate::core::VulframResult;
use crate::core::buffers::state::UploadType;
use crate::core::cmd::{CommandResponse, CommandResponseEnvelope, EngineCmd, EngineCmdEnvelope};
use crate::core::render::gizmos::{CmdGizmoDrawAabbArgs, CmdGizmoDrawLineArgs};
use crate::core::resources::shadow::{CmdShadowConfigureArgs, ShadowConfig};
use crate::core::resources::{
    CameraKind, CmdCameraCreateArgs, CmdLightCreateArgs, CmdMaterialCreateArgs, CmdModelCreateArgs,
    CmdModelUpdateArgs, CmdPrimitiveGeometryCreateArgs, CmdTextureCreateFromBufferArgs, LightKind,
    MaterialKind, MaterialOptions, MaterialSampler, PrimitiveShape, StandardOptions,
    TextureCreateMode,
};
use crate::core::window::{CmdWindowCloseArgs, CmdWindowCreateArgs};
use glam::{Mat4, Vec2, Vec3, Vec4};
use rand::Rng;
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
    let camera_id: u32 = 1;
    let material_instance: u32 = 10;
    let texture_test: u32 = 20;
    let texture_buffer: u64 = 1;

    let texture_bytes = fs::read("assets/colo_test_texture.png")
        .expect("failed to read assets/colo_test_texture.png");
    assert_eq!(
        core::vulfram_upload_buffer(
            texture_buffer,
            UploadType::ImageData as u32,
            texture_bytes.as_ptr(),
            texture_bytes.len()
        ),
        VulframResult::Success
    );

    let mut setup_cmds = vec![
        // 1. Create cube geometry
        EngineCmd::CmdPrimitiveGeometryCreate(CmdPrimitiveGeometryCreateArgs {
            window_id,
            geometry_id: geometry_cube,
            label: Some("Default Cube".to_string()),
            shape: PrimitiveShape::Cube,
            options: None,
        }),
        // 2. Create a camera
        EngineCmd::CmdCameraCreate(CmdCameraCreateArgs {
            camera_id,
            label: Some("Main Camera".to_string()),
            transform: Mat4::look_at_rh(
                Vec3::new(0.0, 10.0, 15.0),
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
        // 3. Create a point light
        EngineCmd::CmdLightCreate(CmdLightCreateArgs {
            window_id,
            light_id: 2,
            label: Some("Point Light".to_string()),
            kind: Some(LightKind::Point),
            position: Some(Vec4::new(0.0, 8.0, 0.0, 1.0)),
            direction: None,
            color: Some(Vec4::new(1.0, 1.0, 1.0, 1.0)),
            ground_color: None,
            intensity: Some(20.0),
            range: Some(30.0),
            spot_inner_outer: None,
            layer_mask: 0xFFFFFFFF,
            cast_shadow: true,
        }),
        // 4. Create texture and material
        EngineCmd::CmdTextureCreateFromBuffer(CmdTextureCreateFromBufferArgs {
            window_id,
            texture_id: texture_test,
            label: Some("Test Texture".to_string()),
            buffer_id: texture_buffer,
            srgb: Some(true),
            mode: TextureCreateMode::Standalone,
            atlas_options: None,
        }),
        EngineCmd::CmdMaterialCreate(CmdMaterialCreateArgs {
            window_id,
            material_id: material_instance,
            label: Some("Test Material".to_string()),
            kind: MaterialKind::Standard,
            options: Some(MaterialOptions::Standard(StandardOptions {
                base_color: Vec4::ONE,
                base_tex_id: Some(texture_test),
                base_sampler: Some(MaterialSampler::LinearClamp),
                ..Default::default()
            })),
        }),
    ];

    // Floor for shadows
    setup_cmds.push(EngineCmd::CmdModelCreate(CmdModelCreateArgs {
        window_id,
        model_id: 2000,
        label: Some("Floor".to_string()),
        geometry_id: geometry_cube,
        material_id: Some(material_instance),
        transform: Mat4::from_translation(Vec3::new(0.0, -6.0, 0.0))
            * Mat4::from_scale(Vec3::new(20.0, 0.1, 20.0)),
        layer_mask: 0xFFFFFFFF,
        cast_shadow: false,
        receive_shadow: true,
    }));

    // 5. Create random cubes (Instancing Test)
    let mut rng = rand::rng();
    struct CubeData {
        id: u32,
        initial_pos: Vec3,
        phase: f32,
    }
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

        setup_cmds.push(EngineCmd::CmdModelCreate(CmdModelCreateArgs {
            window_id,
            model_id,
            label: Some(format!("Cube {}", i)),
            geometry_id: geometry_cube,
            material_id: Some(material_instance),
            transform: Mat4::from_translation(Vec3::new(x, y, z))
                * Mat4::from_euler(glam::EulerRot::XYZ, rot_x, rot_y, 0.0)
                * Mat4::from_scale(Vec3::splat(0.4)),
            layer_mask: 0xFFFFFFFF,
            cast_shadow: true,
            receive_shadow: true,
        }));
    }

    // 6. Configure Shadows
    setup_cmds.push(EngineCmd::CmdShadowConfigure(CmdShadowConfigureArgs {
        window_id,
        config: ShadowConfig {
            tile_resolution: 512,
            atlas_tiles_w: 16,
            atlas_tiles_h: 16,
            atlas_layers: 2,
            virtual_grid_size: 1,
            smoothing: 2,
        },
    }));

    assert_eq!(send_commands(setup_cmds), VulframResult::Success);
    let _ = receive_responses();

    let start_time = Instant::now();
    let mut last_frame_time = Instant::now();
    let mut total_ms: u64 = 0;

    while start_time.elapsed() < Duration::from_secs(30) {
        let now = Instant::now();
        let delta_ms = now.duration_since(last_frame_time).as_millis() as u32;
        last_frame_time = now;
        total_ms += delta_ms as u64;

        // Simple scene update
        let mut frame_cmds = vec![];

        // Axes demo (Gizmo) - sent every frame because gizmos are immediate
        frame_cmds.push(EngineCmd::CmdGizmoDrawLine(CmdGizmoDrawLineArgs {
            start: Vec3::ZERO,
            end: Vec3::X * 5.0,
            color: Vec4::new(1.0, 0.0, 0.0, 1.0),
        }));
        frame_cmds.push(EngineCmd::CmdGizmoDrawLine(CmdGizmoDrawLineArgs {
            start: Vec3::ZERO,
            end: Vec3::Y * 5.0,
            color: Vec4::new(0.0, 1.0, 0.0, 1.0),
        }));
        frame_cmds.push(EngineCmd::CmdGizmoDrawLine(CmdGizmoDrawLineArgs {
            start: Vec3::ZERO,
            end: Vec3::Z * 5.0,
            color: Vec4::new(0.0, 0.0, 1.0, 1.0),
        }));

        // Spawn area AABB (Gizmo)
        frame_cmds.push(EngineCmd::CmdGizmoDrawAabb(CmdGizmoDrawAabbArgs {
            min: Vec3::splat(-5.0),
            max: Vec3::splat(5.0),
            color: Vec4::new(1.0, 1.0, 1.0, 0.2),
        }));

        // Move the cubes
        let time_f = total_ms as f32 / 1000.0;
        for cube in &cubes {
            let offset_y = (time_f + cube.phase).sin() * 0.5;
            let rotation = time_f * 2.0 + cube.phase;

            frame_cmds.push(EngineCmd::CmdModelUpdate(CmdModelUpdateArgs {
                window_id,
                model_id: cube.id,
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
            }));
        }

        let _ = send_commands(frame_cmds);

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
