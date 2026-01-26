use std::sync::mpsc;

use wgpu::BufferUsages;

use crate::core::profiling::state::TickProfiling;

pub struct GpuProfiler {
    query_set: wgpu::QuerySet,
    resolve: wgpu::Buffer,
    readback: wgpu::Buffer,
    query_count: u32,
    window_count: usize,
    timestamp_period: f32,
}

impl GpuProfiler {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, window_count: usize) -> Self {
        let query_count = Self::required_query_count(window_count);
        let query_set = device.create_query_set(&wgpu::QuerySetDescriptor {
            label: Some("GpuProfiler QuerySet"),
            ty: wgpu::QueryType::Timestamp,
            count: query_count,
        });
        let buffer_size =
            (query_count as wgpu::BufferAddress) * wgpu::QUERY_SIZE as wgpu::BufferAddress;
        let resolve = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("GpuProfiler Resolve"),
            size: buffer_size,
            usage: BufferUsages::QUERY_RESOLVE | BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let readback = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("GpuProfiler Readback"),
            size: buffer_size,
            usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        Self {
            query_set,
            resolve,
            readback,
            query_count,
            window_count,
            timestamp_period: queue.get_timestamp_period(),
        }
    }

    pub fn required_query_count(window_count: usize) -> u32 {
        if window_count == 0 {
            0
        } else {
            2 + (window_count as u32) * 6
        }
    }

    pub fn ensure_capacity(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        window_count: usize,
    ) {
        if window_count == 0 {
            return;
        }
        let needed = Self::required_query_count(window_count);
        if needed == self.query_count && window_count == self.window_count {
            return;
        }
        *self = Self::new(device, queue, window_count);
    }

    pub fn query_set(&self) -> &wgpu::QuerySet {
        &self.query_set
    }

    pub fn query_count(&self) -> u32 {
        self.query_count
    }

    pub fn readback_buffer(&self) -> &wgpu::Buffer {
        &self.readback
    }

    pub fn resolve_buffer(&self) -> &wgpu::Buffer {
        &self.resolve
    }

    pub fn buffer_size(&self) -> wgpu::BufferAddress {
        (self.query_count as wgpu::BufferAddress) * wgpu::QUERY_SIZE as wgpu::BufferAddress
    }

    pub fn readback_and_update(&self, device: &wgpu::Device, profiling: &mut TickProfiling) {
        if self.query_count == 0 {
            return;
        }
        let slice = self.readback.slice(..);
        let (sender, receiver) = mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = sender.send(result);
        });
        let _ = device.poll(wgpu::PollType::wait_indefinitely());
        if receiver.recv().ok() != Some(Ok(())) {
            self.readback.unmap();
            return;
        }

        let data = slice.get_mapped_range();
        let timestamps: &[u64] = bytemuck::cast_slice(&data);
        let period = self.timestamp_period as f64;

        let mut gpu_shadow_ns = 0u64;
        let mut gpu_light_cull_ns = 0u64;
        let mut gpu_forward_ns = 0u64;
        let mut gpu_compose_ns = 0u64;

        if timestamps.len() >= 2 {
            let start = timestamps[0];
            let end = timestamps[1];
            if end >= start {
                gpu_shadow_ns = ((end - start) as f64 * period) as u64;
            }
        }

        for window_index in 0..self.window_count {
            let base = 2 + window_index * 6;
            if timestamps.len() <= base + 5 {
                break;
            }
            let light_start = timestamps[base];
            let light_end = timestamps[base + 1];
            if light_end >= light_start {
                gpu_light_cull_ns = gpu_light_cull_ns
                    .saturating_add(((light_end - light_start) as f64 * period) as u64);
            }

            let forward_start = timestamps[base + 2];
            let forward_end = timestamps[base + 3];
            if forward_end >= forward_start {
                gpu_forward_ns = gpu_forward_ns
                    .saturating_add(((forward_end - forward_start) as f64 * period) as u64);
            }

            let compose_start = timestamps[base + 4];
            let compose_end = timestamps[base + 5];
            if compose_end >= compose_start {
                gpu_compose_ns = gpu_compose_ns
                    .saturating_add(((compose_end - compose_start) as f64 * period) as u64);
            }
        }

        profiling.gpu_shadow_ns = gpu_shadow_ns;
        profiling.gpu_light_cull_ns = gpu_light_cull_ns;
        profiling.gpu_forward_ns = gpu_forward_ns;
        profiling.gpu_compose_ns = gpu_compose_ns;
        profiling.gpu_total_ns = gpu_shadow_ns
            .saturating_add(gpu_light_cull_ns)
            .saturating_add(gpu_forward_ns)
            .saturating_add(gpu_compose_ns);

        drop(data);
        self.readback.unmap();
    }
}
