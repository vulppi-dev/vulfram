use crate::core::render::graph::LogicalId;
use crate::core::render::state::RenderScene;
use crate::core::resources::TextureRecord;

pub fn map_camera_targets(render_scene: &mut RenderScene) {
    let mut mappings = Vec::new();
    for (camera_id, record) in render_scene.cameras.iter() {
        let Some(target_id) = record.target_texture_id.as_ref() else {
            continue;
        };
        let texture_id = match target_id {
            LogicalId::Int(value) => {
                if *value < 0 || *value > u32::MAX as i64 {
                    log::warn!("Camera target {:?} out of u32 range", target_id);
                    continue;
                }
                *value as u32
            }
            LogicalId::Str(_) => {
                log::warn!("Camera target {:?} must be an int to map to texture id", target_id);
                continue;
            }
        };

        let target = match record.post_target.as_ref().or(record.render_target.as_ref()) {
            Some(target) => target,
            None => {
                log::warn!("Camera {} has no render target yet", camera_id);
                continue;
            }
        };

        if let Some(existing) = render_scene.textures.get(&texture_id) {
            if existing.label.as_deref() != Some("camera_target") {
                log::warn!(
                    "Texture id {} already in use; skipping camera target mapping",
                    texture_id
                );
                continue;
            }
        }

        let size = target._texture.size();
        mappings.push((
            texture_id,
            TextureRecord {
                label: Some("camera_target".into()),
                _size: size,
                _format: target.format,
                _texture: target._texture.clone(),
                view: target.view.clone(),
            },
        ));
    }

    for (texture_id, record) in mappings {
        render_scene.textures.insert(texture_id, record);
        invalidate_material_bind_groups(render_scene, texture_id);
    }
}

fn invalidate_material_bind_groups(render_scene: &mut RenderScene, texture_id: u32) {
    for record in render_scene.materials_standard.values_mut() {
        if record.texture_ids.iter().any(|id| *id == texture_id) {
            record.bind_group = None;
        }
    }
    for record in render_scene.materials_pbr.values_mut() {
        if record.texture_ids.iter().any(|id| *id == texture_id) {
            record.bind_group = None;
        }
    }
}
