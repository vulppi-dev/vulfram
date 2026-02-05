use crate::core::VulframResult;
use crate::core::buffers::state::UploadType;
use crate::core::cmd::EngineCmd;
use crate::core::resources::{
    CmdGeometryCreateArgs, CmdModelCreateArgs, CmdPoseUpdateArgs, GeometryPrimitiveEntry,
};
use glam::{Mat4, Vec3, Vec4};

use crate::demos::common::{
    build_skinned_plane, create_ambient_light_cmd, create_camera_cmd, create_point_light_cmd,
    create_shadow_config_cmd, create_standard_material_cmd, receive_responses, run_loop,
    send_commands, upload_buffer,
};

pub fn run(window_id: u32) -> bool {
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
