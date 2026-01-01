mod core;

use crate::core::VulframResult;
use crate::core::cmd::{CommandResponse, CommandResponseEnvelope, EngineCmd, EngineCmdEnvelope};
use crate::core::resources::{
    CameraKind, CmdCameraCreateArgs, CmdLightCreateArgs, CmdModelCreateArgs,
    CmdPrimitiveGeometryCreateArgs, CmdShadowConfigureArgs, LightKind, PrimitiveShape,
    ShadowConfig,
};
use crate::core::window::{CmdWindowCloseArgs, CmdWindowCreateArgs};
use glam::{Mat4, Vec2, Vec3, Vec4};
use rmp_serde::{from_slice, to_vec_named};
use std::sync::Mutex;
use std::time::{Duration, Instant};

static ENGINE_GUARD: Mutex<()> = Mutex::new(());

fn main() {
    let _lock = ENGINE_GUARD.lock().unwrap();

    println!("Initializing Vulfram engine...");
    assert_eq!(core::vulfram_init(), VulframResult::Success);

    println!("Creating window...");
    let window_id: u32 = 1;
    let create_cmd = EngineCmd::CmdWindowCreate(CmdWindowCreateArgs {
        window_id,
        title: "Vulfram Render Test".into(),
        size: glam::UVec2::new(1280, 720),
        ..Default::default()
    });
    assert_eq!(send_commands(vec![create_cmd]), VulframResult::Success);

    // Give some time for window to be created and confirm it
    pump_for(Duration::from_millis(200));
    wait_for_confirmation(window_id);
    println!("Window confirmed.");

    println!("Creating geometry, camera and models...");
    let geometry_cube: u32 = 1;
    let geometry_plane: u32 = 2;
    let camera_id: u32 = 1;
    let model_cube: u32 = 1;
    let model_plane: u32 = 2;
    let light_id: u32 = 1;

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
        // 3. Create a directional light for shadows
        EngineCmd::CmdLightCreate(CmdLightCreateArgs {
            window_id,
            light_id,
            kind: Some(LightKind::Directional),
            position: Some(Vec4::new(0.0, 15.0, 5.0, 1.0)),
            direction: Some(Vec4::new(0.0, -1.0, -0.3, 0.0)),
            color: Some(Vec4::new(1.0, 1.0, 1.0, 1.0)),
            intensity: Some(1.0),
            range: Some(100.0),
            spot_inner_outer: Some(Vec2::new(0.7, 0.9)),
            layer_mask: 0xFFFFFFFF,
            cast_shadow: true,
        }),
        // 4. Create models
        EngineCmd::CmdModelCreate(CmdModelCreateArgs {
            window_id,
            model_id: model_cube,
            geometry_id: geometry_cube,
            material_id: None,
            transform: Mat4::from_translation(Vec3::new(0.0, 1.0, 0.0)),
            layer_mask: 0xFFFFFFFF,
            cast_shadow: true,
            receive_shadow: true,
        }),
        EngineCmd::CmdModelCreate(CmdModelCreateArgs {
            window_id,
            model_id: model_plane,
            geometry_id: geometry_plane,
            material_id: None,
            transform: Mat4::from_rotation_x(-std::f32::consts::FRAC_PI_2)
                * Mat4::from_scale(Vec3::new(10.0, 10.0, 1.0)),
            layer_mask: 0xFFFFFFFF,
            cast_shadow: true,
            receive_shadow: true,
        }),
        // 5. Configure Shadows
        EngineCmd::CmdShadowConfigure(CmdShadowConfigureArgs {
            window_id,
            config: ShadowConfig {
                tile_resolution: 2048,
                atlas_tiles_w: 4,
                atlas_tiles_h: 4,
                atlas_layers: 1,
                virtual_grid_size: 1,
            },
        }),
    ];

    assert_eq!(send_commands(setup_cmds), VulframResult::Success);
    let _ = receive_responses();

    println!("Rendering for 10 seconds...");
    let start_time = Instant::now();
    let mut last_frame_time = Instant::now();
    let mut total_ms: u64 = 0;

    while start_time.elapsed() < Duration::from_secs(10) {
        let now = Instant::now();
        let delta_ms = now.duration_since(last_frame_time).as_millis() as u32;
        last_frame_time = now;
        total_ms += delta_ms as u64;

        // Update cube rotation and position to see shadow moving
        let angle = (total_ms as f32 / 1000.0) * 1.0;
        let x_pos = (total_ms as f32 / 1000.0).sin() * 2.0;
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
        let _ = send_commands(vec![update_cmd]);

        // Dynamic shadow reconfiguration test at 5 seconds
        if total_ms >= 5000 && total_ms < 5016 {
            println!("Dynamically changing shadow resolution to 128px...");
            let reconfig_cmd = EngineCmd::CmdShadowConfigure(CmdShadowConfigureArgs {
                window_id,
                config: ShadowConfig {
                    tile_resolution: 128,
                    atlas_tiles_w: 4,
                    atlas_tiles_h: 4,
                    atlas_layers: 1,
                    virtual_grid_size: 1,
                },
            });
            let _ = send_commands(vec![reconfig_cmd]);
        }

        assert_eq!(
            core::vulfram_tick(total_ms, delta_ms),
            VulframResult::Success
        );

        // Process any responses or events (optional for this test)
        let _ = receive_responses();

        std::thread::sleep(Duration::from_millis(16)); // ~60 FPS
    }

    println!("Closing window...");
    let close_cmd = EngineCmd::CmdWindowClose(CmdWindowCloseArgs { window_id });
    assert_eq!(send_commands(vec![close_cmd]), VulframResult::Success);
    pump_for(Duration::from_millis(100));

    println!("Disposing engine...");
    assert_eq!(core::vulfram_dispose(), VulframResult::Success);
    println!("Vulfram test completed successfully.");
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
    for response in &responses {
        println!("Command response: {:?}", response);
    }
    responses
}
