use super::branches;
use crate::core::resources::SurfaceType;

pub(crate) fn draw_batches(
    render_pass: &mut wgpu::RenderPass,
    scene: &crate::core::render::state::RenderScene,
    library: &crate::core::render::state::ResourceLibrary,
    collector: &crate::core::render::state::DrawCollector,
    bindings: &crate::core::render::state::BindingSystem,
    vertex_sys: &mut crate::core::resources::VertexAllocatorSystem,
    frame_index: u64,
    device: &wgpu::Device,
    cache: &mut crate::core::render::cache::RenderCache,
) {
    // 1. PBR Opaque
    draw_group(
        render_pass,
        &collector.pbr_opaque,
        SurfaceType::Opaque,
        true, // is_pbr
        scene,
        bindings,
        vertex_sys,
        frame_index,
        device,
        cache,
        library,
    );

    // 2. PBR Masked
    draw_group(
        render_pass,
        &collector.pbr_masked,
        SurfaceType::Masked,
        true,
        scene,
        bindings,
        vertex_sys,
        frame_index,
        device,
        cache,
        library,
    );

    // 3. Standard Opaque
    draw_group(
        render_pass,
        &collector.standard_opaque,
        SurfaceType::Opaque,
        false, // is_pbr
        scene,
        bindings,
        vertex_sys,
        frame_index,
        device,
        cache,
        library,
    );

    // 4. Standard Masked
    draw_group(
        render_pass,
        &collector.standard_masked,
        SurfaceType::Masked,
        false,
        scene,
        bindings,
        vertex_sys,
        frame_index,
        device,
        cache,
        library,
    );

    // 5. PBR Transparent
    draw_group(
        render_pass,
        &collector.pbr_transparent,
        SurfaceType::Transparent,
        true,
        scene,
        bindings,
        vertex_sys,
        frame_index,
        device,
        cache,
        library,
    );

    // 6. Standard Transparent
    draw_group(
        render_pass,
        &collector.standard_transparent,
        SurfaceType::Transparent,
        false,
        scene,
        bindings,
        vertex_sys,
        frame_index,
        device,
        cache,
        library,
    );
}

fn draw_group(
    render_pass: &mut wgpu::RenderPass,
    items: &[crate::core::render::state::DrawItem],
    surface_type: SurfaceType,
    is_pbr: bool,
    scene: &crate::core::render::state::RenderScene,
    bindings: &crate::core::render::state::BindingSystem,
    vertex_sys: &mut crate::core::resources::VertexAllocatorSystem,
    frame_index: u64,
    device: &wgpu::Device,
    cache: &mut crate::core::render::cache::RenderCache,
    library: &crate::core::render::state::ResourceLibrary,
) {
    if items.is_empty() {
        return;
    }

    let pipeline = if is_pbr {
        branches::pbr::get_pipeline(cache, frame_index, device, library, surface_type)
    } else {
        branches::standard::get_pipeline(cache, frame_index, device, library, surface_type)
    };
    render_pass.set_pipeline(pipeline);

    let mut i = 0;
    while i < items.len() {
        let batch_start = i;
        let item = &items[i];
        let mat_id = item.material_id;
        let geom_id = item.geometry_id;

        while i < items.len() && items[i].material_id == mat_id && items[i].geometry_id == geom_id {
            i += 1;
        }
        let batch_count = (i - batch_start) as u32;

        if is_pbr {
            if let Some(material) = scene.materials_pbr.get(&mat_id) {
                if let Some(group) = material.bind_group.as_ref() {
                    let material_offset = bindings.material_pbr_pool.get_offset(mat_id) as u32;
                    render_pass.set_bind_group(1, group, &[material_offset]);
                }
            }
        } else {
            if let Some(material) = scene.materials_standard.get(&mat_id) {
                if let Some(group) = material.bind_group.as_ref() {
                    let material_offset = bindings.material_standard_pool.get_offset(mat_id) as u32;
                    render_pass.set_bind_group(1, group, &[material_offset]);
                }
            }
        }

        if let Ok(Some(index_info)) = vertex_sys.index_info(geom_id) {
            if vertex_sys.bind(render_pass, geom_id).is_ok() {
                let first_instance = items[batch_start].instance_idx;
                render_pass.draw_indexed(
                    0..index_info.count,
                    0,
                    first_instance..(first_instance + batch_count),
                );
            }
        }
    }
}
