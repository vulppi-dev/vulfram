/// Handle to an allocated region in the forward material atlas
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ForwardAtlasHandle {
    pub(crate) index: u32,
    pub(crate) generation: u32,
}

/// Description for creating a ForwardAtlasSystem
#[derive(Debug, Clone)]
pub struct ForwardAtlasDesc {
    pub label: Option<&'static str>,
    pub format: wgpu::TextureFormat,
    pub usage: wgpu::TextureUsages,
    pub tile_px: u32,
    pub layers: u32,
}

/// Information about a relocation after a repack
#[derive(Debug, Clone)]
pub struct ForwardAtlasRelocation {
    pub handle: ForwardAtlasHandle,
    pub _old_layer: u32,
    pub _old_rect_tiles: (u32, u32, u32, u32),
    pub _new_layer: u32,
    pub _new_rect_tiles: (u32, u32, u32, u32),
}

#[derive(Debug, Clone)]
struct ForwardAtlasSlot {
    generation: u32,
    alive: bool,
    layer: u32,
    rect_tiles: (u32, u32, u32, u32), // (x, y, w, h) in tiles
}

/// A sub-allocator for a 2D Texture Array divided into tiles with internal guards
#[allow(dead_code)]
pub struct ForwardAtlasSystem {
    _texture: wgpu::Texture,
    view: wgpu::TextureView,
    layer_views: Vec<wgpu::TextureView>, // Cached views for each layer

    // Config (clamped by hardware limits)
    tile_px: u32,
    pitch_px: u32,
    guard_px: u32,
    tiles_w: u32,
    tiles_h: u32,
    layers: u32,
    format: wgpu::TextureFormat,

    // State
    slots: Vec<ForwardAtlasSlot>,
    free_slots: Vec<u32>,
    layers_occupied: Vec<Vec<bool>>, // [layer][y * tiles_w + x]
    free_tiles_total: u32,

    // Diagnostics
    repack_count: u64,
}

impl ForwardAtlasSystem {
    pub const GUARD_PX: u32 = 8;

    pub fn new(device: &wgpu::Device, desc: ForwardAtlasDesc) -> Self {
        let guard_px = Self::GUARD_PX;
        let pitch_px = desc.tile_px + (guard_px * 2);

        // Clamp by hardware limits
        let max_dim = device.limits().max_texture_dimension_2d;
        let max_layers = device.limits().max_texture_array_layers;

        let actual_tiles_w = max_dim / pitch_px;
        let actual_tiles_h = max_dim / pitch_px;
        let actual_layers = desc.layers.min(max_layers);

        let atlas_w_px = actual_tiles_w * pitch_px;
        let atlas_h_px = actual_tiles_h * pitch_px;

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: desc.label,
            size: wgpu::Extent3d {
                width: atlas_w_px,
                height: atlas_h_px,
                depth_or_array_layers: actual_layers,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: desc.format,
            usage: desc.usage,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: desc.label.map(|l| format!("{l} View")).as_deref(),
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            ..Default::default()
        });

        let layer_size = (actual_tiles_w * actual_tiles_h) as usize;
        let layers_occupied = vec![vec![false; layer_size]; actual_layers as usize];

        let mut layer_views = Vec::with_capacity(actual_layers as usize);
        for i in 0..actual_layers {
            layer_views.push(texture.create_view(&wgpu::TextureViewDescriptor {
                label: desc.label.map(|l| format!("{l} Layer View {i}")).as_deref(),
                dimension: Some(wgpu::TextureViewDimension::D2),
                base_array_layer: i,
                array_layer_count: Some(1),
                ..Default::default()
            }));
        }

        Self {
            _texture: texture,
            view,
            layer_views,
            tile_px: desc.tile_px,
            pitch_px,
            guard_px,
            tiles_w: actual_tiles_w,
            tiles_h: actual_tiles_h,
            layers: actual_layers,
            format: desc.format,
            slots: Vec::new(),
            free_slots: Vec::new(),
            layers_occupied,
            free_tiles_total: actual_tiles_w * actual_tiles_h * actual_layers,
            repack_count: 0,
        }
    }

    #[allow(dead_code)]
    pub fn view(&self) -> &wgpu::TextureView {
        &self.view
    }

    pub fn texture(&self) -> &wgpu::Texture {
        &self._texture
    }

    #[allow(dead_code)]
    pub fn layer_view(&self, layer: u32) -> Option<&wgpu::TextureView> {
        self.layer_views.get(layer as usize)
    }

    /// Allocate a region of tiles. Returns None if capacity is insufficient or if fragmentation
    /// occurs and repack cannot solve it.
    pub fn alloc(
        &mut self,
        w: u32,
        h: u32,
    ) -> Option<(ForwardAtlasHandle, Vec<ForwardAtlasRelocation>)> {
        if w == 0 || h == 0 || w > self.tiles_w || h > self.tiles_h {
            return None;
        }

        let required = w * h;
        if self.free_tiles_total < required {
            return None;
        }

        // Try normal path (first-fit in any layer)
        if let Some((layer, x, y)) = self.find_free_region(w, h) {
            return Some((self.create_slot(layer, x, y, w, h), Vec::new()));
        }

        // If failed but we have total space, it's fragmentation. Trigger repack.
        let relocations = self.repack();
        self.repack_count += 1;

        // Try again after repack
        if let Some((layer, x, y)) = self.find_free_region(w, h) {
            Some((self.create_slot(layer, x, y, w, h), relocations))
        } else {
            // Still no space even after repack (should be rare if logic is correct)
            None
        }
    }

    /// Free an allocated region
    pub fn free(&mut self, handle: ForwardAtlasHandle) -> bool {
        let (alive, layer, rect_tiles) = if let Some(slot) = self.slots.get(handle.index as usize) {
            if slot.generation == handle.generation {
                (slot.alive, slot.layer, slot.rect_tiles)
            } else {
                return false;
            }
        } else {
            return false;
        };

        if alive {
            self.mark_region(
                layer,
                rect_tiles.0,
                rect_tiles.1,
                rect_tiles.2,
                rect_tiles.3,
                false,
            );
            self.free_tiles_total += rect_tiles.2 * rect_tiles.3;
            self.free_slots.push(handle.index);

            let slot = &mut self.slots[handle.index as usize];
            slot.alive = false;
            return true;
        }

        false
    }

    /// Global repack of all alive slots. Returns a list of relocations.
    pub fn repack(&mut self) -> Vec<ForwardAtlasRelocation> {
        // 1. Collect all alive slots and their data
        let mut items: Vec<(u32, u32, u32, u32, (u32, u32, u32, u32))> = self
            .slots
            .iter()
            .enumerate()
            .filter(|(_, s)| s.alive)
            .map(|(i, s)| {
                (
                    i as u32,
                    s.layer,
                    s.rect_tiles.2,
                    s.rect_tiles.3,
                    s.rect_tiles,
                )
            })
            .collect();

        // 2. Reset occupancy
        for layer in &mut self.layers_occupied {
            for cell in layer.iter_mut() {
                *cell = false;
            }
        }

        // 3. Sort bigger first to reduce fragmentation
        items.sort_by_key(|(_, _, w, h, _)| std::cmp::Reverse(w * h));

        let mut relocations = Vec::new();

        for (index, old_layer, w, h, old_rect) in items {
            if let Some((new_layer, x, y)) = self.find_free_region(w, h) {
                self.mark_region(new_layer, x, y, w, h, true);
                let slot = &mut self.slots[index as usize];
                let old_rect_tiles = slot.rect_tiles;
                slot.layer = new_layer;
                slot.rect_tiles = (x, y, w, h);
                if old_layer != new_layer || old_rect_tiles != slot.rect_tiles {
                    relocations.push(ForwardAtlasRelocation {
                        handle: ForwardAtlasHandle {
                            index,
                            generation: slot.generation,
                        },
                        _old_layer: old_layer,
                        _old_rect_tiles: old_rect,
                        _new_layer: new_layer,
                        _new_rect_tiles: slot.rect_tiles,
                    });
                }
            }
        }

        // Recompute free tiles
        let mut used = 0;
        for layer in &self.layers_occupied {
            used += layer.iter().filter(|v| **v).count() as u32;
        }
        self.free_tiles_total = self.tiles_w * self.tiles_h * self.layers - used;

        relocations
    }

    /// Returns (scale_x, scale_y, bias_x, bias_y, layer)
    pub fn get_uv_transform(
        &self,
        handle: ForwardAtlasHandle,
    ) -> Option<(f32, f32, f32, f32, u32)> {
        let slot = self.slots.get(handle.index as usize)?;
        if !slot.alive || slot.generation != handle.generation {
            return None;
        }

        let rect = slot.rect_tiles;
        let inner_x = (rect.0 * self.pitch_px + self.guard_px) as f32;
        let inner_y = (rect.1 * self.pitch_px + self.guard_px) as f32;
        let inner_w = (rect.2 * self.tile_px) as f32;
        let inner_h = (rect.3 * self.tile_px) as f32;

        let atlas_w = (self.tiles_w * self.pitch_px) as f32;
        let atlas_h = (self.tiles_h * self.pitch_px) as f32;

        let scale_x = inner_w / atlas_w;
        let scale_y = inner_h / atlas_h;
        let bias_x = inner_x / atlas_w;
        let bias_y = inner_y / atlas_h;

        Some((scale_x, scale_y, bias_x, bias_y, slot.layer))
    }

    /// Returns (x, y, w, h, layer) in pixels for writing into the atlas.
    pub fn get_copy_rect(&self, handle: ForwardAtlasHandle) -> Option<(u32, u32, u32, u32, u32)> {
        let slot = self.slots.get(handle.index as usize)?;
        if !slot.alive || slot.generation != handle.generation {
            return None;
        }

        let rect = slot.rect_tiles;
        let x = rect.0 * self.pitch_px + self.guard_px;
        let y = rect.1 * self.pitch_px + self.guard_px;
        let w = rect.2 * self.tile_px;
        let h = rect.3 * self.tile_px;

        Some((x, y, w, h, slot.layer))
    }

    pub fn info(&self) -> (u32, u32, u32, u32, u32, wgpu::TextureFormat) {
        (
            self.tile_px,
            self.pitch_px,
            self.tiles_w,
            self.tiles_h,
            self.layers,
            self.format,
        )
    }

    pub fn grow_layers(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        new_layers: u32,
    ) -> bool {
        if new_layers <= self.layers {
            return false;
        }

        let atlas_w_px = self.tiles_w * self.pitch_px;
        let atlas_h_px = self.tiles_h * self.pitch_px;

        let new_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Forward Atlas (Grow)"),
            size: wgpu::Extent3d {
                width: atlas_w_px,
                height: atlas_h_px,
                depth_or_array_layers: new_layers,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: self.format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Forward Atlas Grow Copy"),
        });
        encoder.copy_texture_to_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &self._texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyTextureInfo {
                texture: &new_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::Extent3d {
                width: atlas_w_px,
                height: atlas_h_px,
                depth_or_array_layers: self.layers,
            },
        );
        queue.submit(Some(encoder.finish()));

        self._texture = new_texture;
        self.view = self._texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("Forward Atlas View"),
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            ..Default::default()
        });

        self.layer_views.clear();
        for i in 0..new_layers {
            self.layer_views.push(
                self._texture.create_view(&wgpu::TextureViewDescriptor {
                    label: Some(&format!("Forward Atlas Layer View {i}")),
                    dimension: Some(wgpu::TextureViewDimension::D2),
                    base_array_layer: i,
                    array_layer_count: Some(1),
                    ..Default::default()
                }),
            );
        }

        let layer_size = (self.tiles_w * self.tiles_h) as usize;
        let current_layers = self.layers;
        for _ in current_layers..new_layers {
            self.layers_occupied.push(vec![false; layer_size]);
        }
        self.free_tiles_total += (new_layers - current_layers) * self.tiles_w * self.tiles_h;
        self.layers = new_layers;
        true
    }

    fn create_slot(&mut self, layer: u32, x: u32, y: u32, w: u32, h: u32) -> ForwardAtlasHandle {
        let index = if let Some(i) = self.free_slots.pop() {
            i
        } else {
            self.slots.push(ForwardAtlasSlot {
                generation: 0,
                alive: false,
                layer,
                rect_tiles: (0, 0, 0, 0),
            });
            (self.slots.len() - 1) as u32
        };
        let generation;
        {
            let slot = &mut self.slots[index as usize];
            slot.generation = slot.generation.wrapping_add(1);
            slot.alive = true;
            slot.layer = layer;
            slot.rect_tiles = (x, y, w, h);
            generation = slot.generation;
        }

        self.mark_region(layer, x, y, w, h, true);
        self.free_tiles_total -= w * h;

        ForwardAtlasHandle { index, generation }
    }

    fn find_free_region(&self, w: u32, h: u32) -> Option<(u32, u32, u32)> {
        for layer in 0..self.layers {
            for y in 0..=self.tiles_h - h {
                for x in 0..=self.tiles_w - w {
                    if self.is_region_free(layer, x, y, w, h) {
                        return Some((layer, x, y));
                    }
                }
            }
        }
        None
    }

    fn is_region_free(&self, layer: u32, x: u32, y: u32, w: u32, h: u32) -> bool {
        let layer_index = layer as usize;
        let occ = &self.layers_occupied[layer_index];
        for yy in y..(y + h) {
            for xx in x..(x + w) {
                let idx = (yy * self.tiles_w + xx) as usize;
                if occ[idx] {
                    return false;
                }
            }
        }
        true
    }

    fn mark_region(&mut self, layer: u32, x: u32, y: u32, w: u32, h: u32, value: bool) {
        let layer_index = layer as usize;
        let occ = &mut self.layers_occupied[layer_index];
        for yy in y..(y + h) {
            for xx in x..(x + w) {
                let idx = (yy * self.tiles_w + xx) as usize;
                occ[idx] = value;
            }
        }
    }
}
