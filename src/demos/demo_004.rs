use std::cell::RefCell;
use std::rc::Rc;

use crate::core::VulframResult;
use crate::core::audio::{
    AudioPlayModeDto, AudioSpatialParamsDto, CmdAudioListenerCreateArgs,
    CmdAudioResourceCreateArgs, CmdAudioSourceCreateArgs, CmdAudioSourcePlayArgs,
    CmdAudioSourceStopArgs,
};
use crate::core::cmd::{EngineCmd, EngineEvent};
use crate::core::input::events::{ElementState, KeyboardEvent};
use crate::core::render::cmd::CmdRenderGraphSetArgs;
use crate::core::resources::{
    CmdEnvironmentUpdateArgs, CmdMaterialCreateArgs, CmdModelCreateArgs, CmdModelUpdateArgs,
    CmdPrimitiveGeometryCreateArgs, CmdTextureCreateFromBufferArgs, EnvironmentConfig,
    MaterialKind, MaterialOptions, MaterialSampler, MsaaConfig, PrimitiveShape, SkyboxConfig,
    SkyboxMode, StandardOptions, TextureCreateMode,
};
use crate::core::system::events::SystemEvent;
use glam::{Mat4, Quat, Vec3, Vec4};

use crate::demos::common::{
    create_ambient_light_cmd, create_camera_cmd, create_floor_cmd, create_point_light_cmd,
    create_shadow_config_cmd, create_standard_material_cmd, receive_responses,
    run_loop_with_events, send_commands, upload_binary_bytes, upload_texture_bytes,
};
use crate::demos::demo_004_graph::{build_demo_graph, build_post_config};

pub fn run(window_id: u32) -> bool {
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

    let graph = build_demo_graph();

    let mut setup_cmds = vec![
        EngineCmd::CmdRenderGraphSet(CmdRenderGraphSetArgs { window_id, graph }),
        create_camera_cmd(
            camera_id,
            "Demo 004 Camera",
            Mat4::look_at_rh(Vec3::new(0.0, 4.5, 14.0), Vec3::ZERO, Vec3::Y).inverse(),
        ),
        create_point_light_cmd(window_id, 2, Vec4::new(0.0, 5.0, 2.0, 1.0)),
        create_ambient_light_cmd(window_id, 3, Vec4::new(0.3, 0.3, 0.3, 1.0), 0.6),
        create_standard_material_cmd(
            window_id,
            floor_material_id,
            "Floor",
            Vec4::new(0.08, 0.08, 0.08, 1.0),
            None,
            None,
        ),
        create_standard_material_cmd(
            window_id,
            material_id,
            "Material",
            Vec4::new(0.9, 0.25, 0.2, 1.0),
            None,
            None,
        ),
        create_standard_material_cmd(
            window_id,
            emissive_material_id,
            "Emissive",
            Vec4::new(0.2, 0.8, 1.0, 1.0),
            None,
            Some(Vec4::new(2.0, 2.0, 2.0, 1.0)),
        ),
        EngineCmd::CmdPrimitiveGeometryCreate(CmdPrimitiveGeometryCreateArgs {
            window_id,
            geometry_id,
            label: Some("Cube".into()),
            shape: PrimitiveShape::Cube,
            options: None,
        }),
        EngineCmd::CmdPrimitiveGeometryCreate(CmdPrimitiveGeometryCreateArgs {
            window_id,
            geometry_id: emitter_geometry_id,
            label: Some("Emitter".into()),
            shape: PrimitiveShape::Sphere,
            options: None,
        }),
        EngineCmd::CmdModelCreate(CmdModelCreateArgs {
            window_id,
            model_id: emitter_model_id,
            label: Some("Emitter".into()),
            geometry_id: emitter_geometry_id,
            material_id: Some(emitter_material_id),
            transform: Mat4::from_translation(emitter_pos) * Mat4::from_scale(Vec3::splat(0.4)),
            layer_mask: 0xFFFFFFFF,
            cast_shadow: false,
            receive_shadow: false,
            cast_outline: false,
            outline_color: Vec4::ZERO,
        }),
        create_floor_cmd(window_id, geometry_id, floor_material_id),
        create_shadow_config_cmd(window_id),
        EngineCmd::CmdMaterialCreate(CmdMaterialCreateArgs {
            window_id,
            material_id: emitter_material_id,
            label: Some("Emitter Material".into()),
            kind: MaterialKind::Standard,
            options: Some(MaterialOptions::Standard(StandardOptions {
                base_color: Vec4::new(1.0, 0.6, 0.1, 1.0),
                base_sampler: Some(MaterialSampler::LinearClamp),
                emissive_color: Vec4::new(2.0, 1.0, 0.2, 1.0),
                ..Default::default()
            })),
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
            spatial: AudioSpatialParamsDto {
                min_distance: 1.0,
                max_distance: 30.0,
                rolloff: 1.0,
                ..AudioSpatialParamsDto::default()
            },
        }),
        EngineCmd::CmdAudioListenerCreate(CmdAudioListenerCreateArgs {
            window_id,
            model_id: listener_model_id,
        }),
        EngineCmd::CmdModelCreate(CmdModelCreateArgs {
            window_id,
            model_id: listener_model_id,
            label: Some("Listener".into()),
            geometry_id,
            material_id: Some(material_id),
            transform: Mat4::from_translation(Vec3::new(0.0, -1.6, 6.0))
                * Mat4::from_scale(Vec3::splat(0.01)),
            layer_mask: 0xFFFFFFFF,
            cast_shadow: false,
            receive_shadow: false,
            cast_outline: false,
            outline_color: Vec4::ZERO,
        }),
    ];

    let mut post_config = build_post_config();

    let audio_state_events = Rc::new(RefCell::new((false, false, false)));
    let audio_state_frame = audio_state_events.clone();

    let skybox_bytes = include_bytes!("../../assets/skybox.exr");
    upload_texture_bytes(skybox_bytes, skybox_buffer_id);
    setup_cmds.push(EngineCmd::CmdTextureCreateFromBuffer(
        CmdTextureCreateFromBufferArgs {
            window_id,
            texture_id: skybox_texture_id,
            label: Some("Skybox".into()),
            buffer_id: skybox_buffer_id,
            srgb: Some(true),
            mode: TextureCreateMode::Standalone,
            atlas_options: None,
        },
    ));

    let audio_bytes = include_bytes!("../../assets/audio.wav");
    upload_binary_bytes(audio_bytes, audio_buffer_id);

    for (model_id, pos, color) in cube_models.iter() {
        setup_cmds.push(EngineCmd::CmdMaterialCreate(CmdMaterialCreateArgs {
            window_id,
            material_id: model_id + 1000,
            label: Some(format!("Cube Material {}", model_id)),
            kind: MaterialKind::Standard,
            options: Some(MaterialOptions::Standard(StandardOptions {
                base_color: *color,
                base_sampler: Some(MaterialSampler::LinearClamp),
                emissive_color: Vec4::ZERO,
                ..Default::default()
            })),
        }));

        setup_cmds.push(EngineCmd::CmdModelCreate(CmdModelCreateArgs {
            window_id,
            model_id: *model_id,
            label: Some(format!("Cube {}", model_id)),
            geometry_id,
            material_id: Some(model_id + 1000),
            transform: Mat4::from_translation(*pos) * Mat4::from_scale(Vec3::splat(1.1)),
            layer_mask: 0xFFFFFFFF,
            cast_shadow: true,
            receive_shadow: true,
            cast_outline: true,
            outline_color: Vec4::new(0.0, 0.0, 0.0, 1.0),
        }));
    }

    setup_cmds.push(EngineCmd::CmdEnvironmentUpdate(CmdEnvironmentUpdateArgs {
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

    assert_eq!(send_commands(setup_cmds), VulframResult::Success);
    let _ = receive_responses();

    run_loop_with_events(
        window_id,
        None,
        move |total_ms, _delta_ms| {
            let time_f = total_ms as f32 / 1000.0;
            let mut cmds = Vec::new();

            if total_ms % 2000 < 16 {
                let intensity = 0.2 + (time_f * 0.6).sin().abs() * 1.2;
                post_config.bloom_intensity = intensity;
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
