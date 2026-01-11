mod bindings;
mod lights;
mod materials;

use crate::core::resources::FrameComponent;
use super::RenderState;

impl RenderState {
    pub fn prepare_render(
        &mut self,
        device: &wgpu::Device,
        frame_spec: FrameComponent,
        with_shadows: bool,
    ) {
        // 0. Pre-prepare lights
        self.prepare_lights(device);

        let bindings = match self.bindings.as_mut() {
            Some(b) => b,
            None => return,
        };

        if self.library.is_none() {
            return;
        }

        // 1. Upload global data
        bindings.frame_pool.write(0, &frame_spec);

        let mut any_pool_resized = false;
        let mut check_pool = |name: &'static str, current_version: u64| {
            let entry = bindings
                .pool_versions
                .entry(name)
                .or_insert(current_version);
            if *entry != current_version {
                *entry = current_version;
                any_pool_resized = true;
            }
        };

        check_pool("frame", bindings.frame_pool.version());
        check_pool("camera", bindings.camera_pool.version());
        check_pool("model", bindings.model_pool.version());
        check_pool("instance", bindings.instance_pool.version());
        check_pool("shadow_instance", bindings.shadow_instance_pool.version());
        check_pool("mat_std", bindings.material_standard_pool.version());
        check_pool("mat_std_in", bindings.material_standard_inputs.version());
        check_pool("mat_pbr", bindings.material_pbr_pool.version());
        check_pool("mat_pbr_in", bindings.material_pbr_inputs.version());

        if let Some(light_system) = self.light_system.as_ref() {
            check_pool("light_cull", light_system.lights.version());
            check_pool("light_indices", light_system.visible_indices.version());
            check_pool("light_counts", light_system.visible_counts.version());
            check_pool("light_params", light_system.light_params.version());
        }

        if let Some(shadow_manager) = self.shadow.as_ref() {
            check_pool("shadow_params", shadow_manager.params_pool.version());
            check_pool("shadow_page_table", shadow_manager.page_table.version());
            check_pool("shadow_point_vp", shadow_manager.point_light_vp.version());
        }

        if with_shadows != bindings.last_with_shadows {
            bindings.last_with_shadows = with_shadows;
            bindings.shared_group = None;
        }

        if any_pool_resized {
            bindings.shared_group = None;
            bindings.model_bind_group = None;
            bindings.shadow_model_bind_group = None;
            // Clear material bind groups
            for mat in self.scene.materials_standard.values_mut() {
                mat.bind_group = None;
            }
            for mat in self.scene.materials_pbr.values_mut() {
                mat.bind_group = None;
            }
        }

        // 2. Upload camera data
        for (id, record) in &mut self.scene.cameras {
            if record.is_dirty {
                bindings.camera_pool.write(*id, &record.data);
                record.clear_dirty();
            }
        }

        // 3. Upload model data
        for (id, record) in &mut self.scene.models {
            if record.is_dirty {
                bindings.model_pool.write(*id, &record.data);
                record.clear_dirty();
                if let Some(shadow) = self.shadow.as_mut() {
                    shadow.mark_dirty();
                }
            }
        }

        // 4. Update and upload materials
        self.prepare_materials(device);

        // 5. Build/Update bind groups
        self.update_bind_groups(device, with_shadows);
    }
}
