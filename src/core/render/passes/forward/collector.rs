use crate::core::render::state::DrawItem;
use crate::core::resources::geometry::Frustum;
use crate::core::resources::{CameraRecord, MATERIAL_FALLBACK_ID, SurfaceType};

pub(crate) fn collect_objects(
    scene: &crate::core::render::state::RenderScene,
    collector: &mut crate::core::render::state::DrawCollector,
    camera_record: &CameraRecord,
    vertex_sys: &crate::core::resources::VertexAllocatorSystem,
) -> u32 {
    let materials_standard = &scene.materials_standard;
    let materials_pbr = &scene.materials_pbr;
    let frustum = Frustum::from_view_projection(camera_record.data.view_projection);

    let mut instance_cursor = 0;

    for (model_id, model_record) in &scene.models {
        if (model_record.layer_mask & camera_record.layer_mask) == 0 {
            continue;
        }

        if let Some(aabb) = vertex_sys.aabb(model_record.geometry_id) {
            let world_aabb = aabb.transform(&model_record.data.transform);
            if !frustum.intersects_aabb(world_aabb.min, world_aabb.max) {
                continue;
            }
        }

        let material_id = model_record.material_id.unwrap_or(MATERIAL_FALLBACK_ID);

        let model_depth = {
            let clip = camera_record.data.view_projection * model_record.data.translation;
            if clip.w.abs() > 1e-5 {
                clip.z / clip.w
            } else {
                0.0
            }
        };

        if let Some(record) = materials_pbr.get(&material_id) {
            let item = DrawItem {
                model_id: *model_id,
                geometry_id: model_record.geometry_id,
                material_id,
                depth: model_depth,
                instance_idx: 0,
            };
            match record.surface_type {
                SurfaceType::Opaque => collector.pbr_opaque.push(item),
                SurfaceType::Masked => collector.pbr_masked.push(item),
                SurfaceType::Transparent => collector.pbr_transparent.push(item),
            }
            continue;
        }

        let material_id = model_record
            .material_id
            .filter(|id| materials_standard.contains_key(id))
            .unwrap_or(MATERIAL_FALLBACK_ID);

        let surface_type = materials_standard
            .get(&material_id)
            .map(|record| record.surface_type)
            .unwrap_or(SurfaceType::Opaque);

        let item = DrawItem {
            model_id: *model_id,
            geometry_id: model_record.geometry_id,
            material_id,
            depth: model_depth,
            instance_idx: 0,
        };

        match surface_type {
            SurfaceType::Opaque => collector.standard_opaque.push(item),
            SurfaceType::Masked => collector.standard_masked.push(item),
            SurfaceType::Transparent => collector.standard_transparent.push(item),
        }
    }

    // Sort and prepare instance data
    sort_collector(collector);

    let groups = [
        &mut collector.pbr_opaque,
        &mut collector.standard_opaque,
        &mut collector.pbr_masked,
        &mut collector.standard_masked,
        &mut collector.pbr_transparent,
        &mut collector.standard_transparent,
    ];

    for group in groups {
        for item in group.iter_mut() {
            item.instance_idx = instance_cursor;
            if let Some(record) = scene.models.get(&item.model_id) {
                collector.instance_data.push(record.data);
                instance_cursor += 1;
            }
        }
    }

    instance_cursor
}

fn sort_collector(collector: &mut crate::core::render::state::DrawCollector) {
    collector
        .pbr_opaque
        .sort_by_key(|a| (a.material_id, a.geometry_id));
    collector
        .standard_opaque
        .sort_by_key(|a| (a.material_id, a.geometry_id));
    collector
        .pbr_masked
        .sort_by_key(|a| (a.material_id, a.geometry_id));
    collector
        .standard_masked
        .sort_by_key(|a| (a.material_id, a.geometry_id));

    // Sort Far-to-Near (Painter's Algorithm)
    // With Reverse Z: Far is 0.0, Near is 1.0. So we sort Ascending.
    collector.standard_transparent.sort_by(|a, b| {
        a.depth
            .partial_cmp(&b.depth)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    collector.pbr_transparent.sort_by(|a, b| {
        a.depth
            .partial_cmp(&b.depth)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
}
