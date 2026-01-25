mod core;

use crate::core::buffers::state::UploadType;
use crate::core::cmd::{
    CommandResponse, CommandResponseEnvelope, EngineCmd, EngineCmdEnvelope, EngineEvent,
};
use crate::core::render::gizmos::{CmdGizmoDrawAabbArgs, CmdGizmoDrawLineArgs};
use crate::core::resources::shadow::{CmdShadowConfigureArgs, ShadowConfig};
use crate::core::resources::{
    CameraKind, CmdCameraCreateArgs, CmdLightCreateArgs, CmdMaterialCreateArgs, CmdModelCreateArgs,
    CmdModelUpdateArgs, CmdPrimitiveGeometryCreateArgs, CmdTextureCreateFromBufferArgs, LightKind,
    MaterialKind, MaterialOptions, MaterialSampler, PrimitiveShape, StandardOptions,
    TextureCreateMode,
};
use crate::core::window::{CmdWindowCloseArgs, CmdWindowCreateArgs, WindowEvent};
use crate::core::VulframResult;
use glam::{Mat4, UVec2, Vec2, Vec3, Vec4};
use rand::Rng;
use rmp_serde::{from_slice, to_vec_named};
use std::fs;
use std::sync::Mutex;
use std::time::{Duration, Instant};

static ENGINE_GUARD: Mutex<()> = Mutex::new(());

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DemoKind {
    Demo001,
    Demo002,
}

impl DemoKind {
    fn from_str(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "demo_001" | "demo001" | "1" => Some(Self::Demo001),
            "demo_002" | "demo002" | "2" => Some(Self::Demo002),
            _ => None,
        }
    }

    fn title(self) -> &'static str {
        match self {
            Self::Demo001 => "Vulfram Demo 001",
            Self::Demo002 => "Vulfram Demo 002",
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
        ),
    ];

    setup_cmds.push(create_floor_cmd(window_id, geometry_cube, material_instance));
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

        setup_cmds.push(EngineCmd::CmdPrimitiveGeometryCreate(CmdPrimitiveGeometryCreateArgs {
            window_id,
            geometry_id,
            label: Some(label.clone()),
            shape: *shape,
            options: None,
        }));

        setup_cmds.push(create_standard_material_cmd(
            window_id,
            material_id,
            &format!("{} Material", label),
            color,
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
        }));

        primitive_models.push((model_id, position));
    }

    assert_eq!(send_commands(setup_cmds), VulframResult::Success);
    let _ = receive_responses();

    run_loop(window_id, None, |total_ms, _delta_ms| {
        let mut frame_cmds = vec![];
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
            }));
        }

        frame_cmds
    })
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
            ..Default::default()
        })),
    })
}

fn create_texture_cmd(
    window_id: u32,
    texture_id: u32,
    label: &str,
    buffer_id: u64,
) -> EngineCmd {
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
        }));
    }

    (cubes, cmds)
}

fn create_shadow_config_cmd(window_id: u32) -> EngineCmd {
    EngineCmd::CmdShadowConfigure(CmdShadowConfigureArgs {
        window_id,
        config: ShadowConfig {
            tile_resolution: 512,
            atlas_tiles_w: 16,
            atlas_tiles_h: 16,
            atlas_layers: 2,
            virtual_grid_size: 1,
            smoothing: 2,
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

fn upload_texture(path: &str, buffer_id: u64) {
    let texture_bytes = fs::read(path).expect("failed to read texture");
    assert_eq!(
        core::vulfram_upload_buffer(
            buffer_id,
            UploadType::ImageData as u32,
            texture_bytes.as_ptr(),
            texture_bytes.len()
        ),
        VulframResult::Success
    );
}

fn run_loop<F>(window_id: u32, max_duration: Option<Duration>, mut on_frame: F) -> bool
where
    F: FnMut(u64, u32) -> Vec<EngineCmd>,
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

        if handle_close_events(window_id) {
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

fn handle_close_events(window_id: u32) -> bool {
    let events = receive_events();
    for event in events {
        match event {
            EngineEvent::Window(WindowEvent::OnCloseRequest { window_id: id }) if id == window_id => {
                return true
            }
            EngineEvent::System(sys_event) => {
                println!("Received system event: {:?}", sys_event);
            }
            _ => {}
        }
    }

    false
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
