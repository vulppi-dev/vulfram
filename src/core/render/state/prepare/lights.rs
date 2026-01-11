use super::super::RenderState;
use super::super::light::FrustumPlane;
use crate::core::resources::geometry::Frustum;

impl RenderState {
    pub(crate) fn prepare_lights(&mut self, _device: &wgpu::Device) {
        let (light_count, lights_vec) = {
            let mut count = 0;
            let mut lights = Vec::new();

            let mut sorted_lights: Vec<_> = self.scene.lights.iter().collect();
            sorted_lights.sort_by_key(|(id, _)| *id);

            let mut shadow_counter = 0u32;
            for (_, record) in sorted_lights {
                let mut light_data = record.data;
                if record.cast_shadow {
                    light_data.shadow_index = shadow_counter;
                    shadow_counter += 1;
                } else {
                    light_data.shadow_index = 0xFFFFFFFF;
                }
                lights.push(light_data);
                count += 1;
            }
            (count, lights)
        };

        let light_system = match self.light_system.as_mut() {
            Some(sys) => sys,
            None => return,
        };

        if !lights_vec.is_empty() {
            light_system.lights.write_slice(0, &lights_vec);
        }
        light_system.light_count = light_count;

        let mut sorted_cameras: Vec<_> = self.scene.cameras.iter().collect();
        sorted_cameras.sort_by_key(|(_, record)| record.order);

        let mut frustums = Vec::new();
        for (_, camera_record) in &sorted_cameras {
            let frustum = Frustum::from_view_projection(camera_record.data.view_projection);
            frustums.push(FrustumPlane {
                data: frustum.planes[0],
            });
            frustums.push(FrustumPlane {
                data: frustum.planes[1],
            });
            frustums.push(FrustumPlane {
                data: frustum.planes[2],
            });
            frustums.push(FrustumPlane {
                data: frustum.planes[3],
            });
            frustums.push(FrustumPlane {
                data: frustum.planes[4],
            });
            frustums.push(FrustumPlane {
                data: frustum.planes[5],
            });
        }

        if !frustums.is_empty() {
            light_system.camera_frustums.write_slice(0, &frustums);
        }
        light_system.camera_count = sorted_cameras.len() as u32;

        let max_lights = 128; // TBD: make configurable or dynamic
        light_system.max_lights_per_camera = max_lights;

        let params = [
            light_count as u32,
            light_system.camera_count,
            max_lights,
            0, // Padding
        ];

        let params_buffer = light_system.params_buffer.as_ref().unwrap();
        light_system
            .queue
            .write_buffer(params_buffer, 0, bytemuck::cast_slice(&params));
    }
}
