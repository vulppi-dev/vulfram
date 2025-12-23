// MARK: - Uniform Buffer Pool

use bytemuck::{Pod, bytes_of};
use std::marker::PhantomData;
use std::ops::Range;
use wgpu::{Buffer, BufferDescriptor, BufferUsages, Device, Queue};

#[derive(Debug)]
pub struct UniformBufferPool<T: Pod> {
    buffer: Buffer,
    capacity: u32,
    item_size: u64,
    device: wgpu::Device,
    queue: wgpu::Queue,
    _phantom: PhantomData<T>,
}

impl<T: Pod> UniformBufferPool<T> {
    pub fn new(device: &Device, queue: &Queue, initial_capacity: Option<u32>) -> Self {
        let capacity = initial_capacity.unwrap_or(4);
        let item_size = std::mem::size_of::<T>() as u64;

        assert!(item_size > 0, "item_size must be greater than 0");

        let buffer_size = capacity as u64 * item_size;
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("UniformBufferPool"),
            size: buffer_size,
            usage: wgpu::BufferUsages::COPY_SRC
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::UNIFORM,
            mapped_at_creation: false,
        });

        Self {
            buffer,
            capacity,
            item_size,
            device: device.clone(),
            queue: queue.clone(),
            _phantom: PhantomData,
        }
    }

    pub fn write_bytes(&mut self, index: u32, data: &[u8]) {
        assert!(
            data.len() as u64 <= self.item_size,
            "data size exceeds item_size"
        );

        if index + 1 > self.capacity {
            self.scale_to_capacity(index + 1);
        }

        let offset = index as u64 * self.item_size;
        self.queue.write_buffer(&self.buffer, offset, data);
    }

    pub fn write(&mut self, index: u32, value: &T) {
        let data = bytes_of(value);
        self.write_bytes(index, data);
    }

    pub fn write_slice(&mut self, start_index: u32, values: &[T]) {
        if values.is_empty() {
            return;
        }

        let end_index = start_index + values.len() as u32;
        if end_index > self.capacity {
            self.scale_to_capacity(end_index);
        }

        for (i, value) in values.iter().enumerate() {
            let index = start_index + i as u32;
            let offset = index as u64 * self.item_size;
            self.queue
                .write_buffer(&self.buffer, offset, bytes_of(value));
        }
    }

    pub fn capacity(&self) -> u32 {
        self.capacity
    }

    pub fn item_size(&self) -> u64 {
        self.item_size
    }

    pub fn buffer(&self) -> &Buffer {
        &self.buffer
    }

    pub fn get_offset(&self, index: u32) -> u64 {
        index as u64 * self.item_size
    }

    pub fn buffer_size(&self) -> u64 {
        self.capacity as u64 * self.item_size
    }

    fn scale_to_capacity(&mut self, required_capacity: u32) {
        if required_capacity <= self.capacity {
            return;
        }

        let new_capacity = self.calculate_next_capacity(required_capacity);
        let new_size = new_capacity as u64 * self.item_size;

        let new_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("UniformBufferPool (resized)"),
            size: new_size,
            usage: wgpu::BufferUsages::COPY_SRC
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::UNIFORM,
            mapped_at_creation: false,
        });

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("UniformBufferPool resize encoder"),
            });

        encoder.copy_buffer_to_buffer(
            &self.buffer,
            0,
            &new_buffer,
            0,
            self.capacity as u64 * self.item_size,
        );

        self.queue.submit(Some(encoder.finish()));

        self.buffer = new_buffer;
        self.capacity = new_capacity;
    }

    fn calculate_next_capacity(&self, required_capacity: u32) -> u32 {
        let mut new_capacity = self.capacity.max(1) * 2;

        while new_capacity < required_capacity {
            new_capacity = new_capacity.checked_mul(2).expect("capacity overflow");
        }

        new_capacity
    }
}
