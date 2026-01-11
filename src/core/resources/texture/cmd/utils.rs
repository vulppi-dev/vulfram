use crate::core::resources::texture::{ForwardAtlasDesc, ForwardAtlasSystem};

pub(crate) fn ensure_forward_atlas<'a>(
    render_state: &'a mut crate::core::render::state::RenderState,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    desc: &ForwardAtlasDesc,
) -> Result<&'a mut ForwardAtlasSystem, String> {
    if render_state.forward_atlas.is_none() {
        render_state.forward_atlas = Some(ForwardAtlasSystem::new(device, desc.clone()));
    }

    let atlas = render_state
        .forward_atlas
        .as_mut()
        .expect("atlas just created");
    let info = atlas.info();
    let config_matches = info.0 == desc.tile_px && info.5 == desc.format;

    if config_matches {
        let desired_layers = desc.layers.min(device.limits().max_texture_array_layers);
        if desired_layers > info.4 {
            let _ = atlas.grow_layers(device, queue, desired_layers);
        }
        Ok(atlas)
    } else {
        Err("Forward atlas already initialized with different config".into())
    }
}

pub(crate) fn mark_materials_dirty(scene: &mut crate::core::render::state::RenderScene, texture_id: u32) {
    for record in scene.materials_standard.values_mut() {
        if record.texture_ids.iter().any(|id| *id == texture_id) {
            record.bind_group = None;
            record.mark_dirty();
        }
    }
    for record in scene.materials_pbr.values_mut() {
        if record.texture_ids.iter().any(|id| *id == texture_id) {
            record.bind_group = None;
            record.mark_dirty();
        }
    }
}
