/// Handle to an allocated region in the atlas
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AtlasHandle {
    pub(crate) index: u32,
    pub(crate) generation: u32,
}

/// Description for creating an AtlasSystem
#[derive(Debug, Clone)]
pub struct AtlasDesc {
    pub label: Option<&'static str>,
    pub format: wgpu::TextureFormat,
    pub usage: wgpu::TextureUsages,
    pub tile_px: u32,
    pub tiles_w: u32,
    pub tiles_h: u32,
    pub layers: u32,
}

/// Information about a relocation after a repack
#[derive(Debug, Clone)]
pub struct AtlasRelocation {
    pub handle: AtlasHandle,
    pub old_layer: u32,
    pub old_rect_tiles: (u32, u32, u32, u32),
    pub new_layer: u32,
    pub new_rect_tiles: (u32, u32, u32, u32),
}

#[derive(Debug, Clone)]
struct AtlasSlot {
    generation: u32,
    alive: bool,
    layer: u32,
    rect_tiles: (u32, u32, u32, u32), // (x, y, w, h) in tiles
}

/// A sub-allocator for a 2D Texture Array divided into tiles with internal guards
pub struct AtlasSystem {
    texture: wgpu::Texture,
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
    slots: Vec<AtlasSlot>,
    free_slots: Vec<u32>,
    layers_occupied: Vec<Vec<bool>>, // [layer][y * tiles_w + x]
    free_tiles_total: u32,

    // Diagnostics
    repack_count: u64,
}

impl AtlasSystem {
    pub const GUARD_PX: u32 = 8;

    pub fn new(device: &wgpu::Device, desc: AtlasDesc) -> Self {
        let guard_px = Self::GUARD_PX;
        let pitch_px = desc.tile_px + (guard_px * 2);

        // Clamp by hardware limits
        let max_dim = device.limits().max_texture_dimension_2d;
        let max_layers = device.limits().max_texture_array_layers;

        let actual_tiles_w = desc.tiles_w.min(max_dim / pitch_px);
        let actual_tiles_h = desc.tiles_h.min(max_dim / pitch_px);
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
            label: desc.label.map(|l| format!("{} View", l)).as_deref(),
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            ..Default::default()
        });

        let layer_size = (actual_tiles_w * actual_tiles_h) as usize;
        let layers_occupied = vec![vec![false; layer_size]; actual_layers as usize];

        let mut layer_views = Vec::with_capacity(actual_layers as usize);
        for i in 0..actual_layers {
            layer_views.push(
                texture.create_view(&wgpu::TextureViewDescriptor {
                    label: desc
                        .label
                        .map(|l| format!("{} Layer View {}", l, i))
                        .as_deref(),
                    dimension: Some(wgpu::TextureViewDimension::D2),
                    base_array_layer: i,
                    array_layer_count: Some(1),
                    ..Default::default()
                }),
            );
        }

        Self {
            texture,
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

    pub fn layer_view(&self, layer: u32) -> Option<&wgpu::TextureView> {
        self.layer_views.get(layer as usize)
    }

    /// Allocate a region of tiles. Returns None if capacity is insufficient or if fragmentation
    /// occurs and repack cannot solve it.
    pub fn alloc(&mut self, w: u32, h: u32) -> Option<(AtlasHandle, Vec<AtlasRelocation>)> {
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
    pub fn free(&mut self, handle: AtlasHandle) -> bool {
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
    pub fn repack(&mut self) -> Vec<AtlasRelocation> {
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

        // 2. Sort by area descending (largest first)
        items.sort_by(|a, b| (b.2 * b.3).cmp(&(a.2 * a.3)));

        // 3. Reset occupation
        for grid in &mut self.layers_occupied {
            grid.fill(false);
        }

        let mut relocations = Vec::new();

        // 4. Realoc everything
        for (index, old_layer, w, h, old_rect) in items {
            if let Some((new_layer, new_x, new_y)) = self.find_free_region(w, h) {
                self.mark_region(new_layer, new_x, new_y, w, h, true);

                let slot = &mut self.slots[index as usize];
                let moved = new_layer != old_layer || (new_x, new_y, w, h) != old_rect;

                if moved {
                    relocations.push(AtlasRelocation {
                        handle: AtlasHandle {
                            index,
                            generation: slot.generation,
                        },
                        old_layer,
                        old_rect_tiles: old_rect,
                        new_layer,
                        new_rect_tiles: (new_x, new_y, w, h),
                    });

                    slot.layer = new_layer;
                    slot.rect_tiles = (new_x, new_y, w, h);
                }
            } else {
                // This shouldn't happen if free_tiles_total was checked before calling repack,
                // unless we have very complex shapes that don't pack.
                // For safety, mark as dead if it doesn't fit anymore.
                self.slots[index as usize].alive = false;
            }
        }

        relocations
    }

    // -------------------------------------------------------------------------
    // Queries
    // -------------------------------------------------------------------------

    pub fn get_uv_transform(&self, handle: AtlasHandle) -> Option<(f32, f32, f32, f32, u32)> {
        let slot = self.slots.get(handle.index as usize)?;
        if !slot.alive || slot.generation != handle.generation {
            return None;
        }

        let (tx, ty, tw, th) = slot.rect_tiles;
        let atlas_w = (self.tiles_w * self.pitch_px) as f32;
        let atlas_h = (self.tiles_h * self.pitch_px) as f32;

        // Inner rect in pixels
        let inner_x = (tx * self.pitch_px + self.guard_px) as f32;
        let inner_y = (ty * self.pitch_px + self.guard_px) as f32;
        let inner_w = (tw * self.pitch_px - 2 * self.guard_px) as f32;
        let inner_h = (th * self.pitch_px - 2 * self.guard_px) as f32;

        let scale_x = inner_w / atlas_w;
        let scale_y = inner_h / atlas_h;
        let bias_x = inner_x / atlas_w;
        let bias_y = inner_y / atlas_h;

        Some((scale_x, scale_y, bias_x, bias_y, slot.layer))
    }

    pub fn texture(&self) -> &wgpu::Texture {
        &self.texture
    }

    pub fn view(&self) -> &wgpu::TextureView {
        &self.view
    }

    pub fn info(&self) -> AtlasInfo {
        AtlasInfo {
            tiles_w: self.tiles_w,
            tiles_h: self.tiles_h,
            layers: self.layers,
            tile_px: self.tile_px,
            pitch_px: self.pitch_px,
            free_tiles: self.free_tiles_total,
            repack_count: self.repack_count,
            format: self.format,
        }
    }

    // -------------------------------------------------------------------------
    // Internal Helpers
    // -------------------------------------------------------------------------

    fn find_free_region(&self, w: u32, h: u32) -> Option<(u32, u32, u32)> {
        for layer in 0..self.layers {
            let grid = &self.layers_occupied[layer as usize];

            for y in 0..=(self.tiles_h - h) {
                for x in 0..=(self.tiles_w - w) {
                    if self.is_region_free(grid, x, y, w, h) {
                        return Some((layer, x, y));
                    }
                }
            }
        }
        None
    }

    fn is_region_free(&self, grid: &[bool], x: u32, y: u32, w: u32, h: u32) -> bool {
        for dy in 0..h {
            let offset = ((y + dy) * self.tiles_w + x) as usize;
            for dx in 0..w {
                if grid[offset + dx as usize] {
                    return false;
                }
            }
        }
        true
    }

    fn mark_region(&mut self, layer: u32, x: u32, y: u32, w: u32, h: u32, occupied: bool) {
        let grid = &mut self.layers_occupied[layer as usize];
        for dy in 0..h {
            let offset = ((y + dy) * self.tiles_w + x) as usize;
            for dx in 0..w {
                grid[offset + dx as usize] = occupied;
            }
        }
    }

    fn create_slot(&mut self, layer: u32, x: u32, y: u32, w: u32, h: u32) -> AtlasHandle {
        self.mark_region(layer, x, y, w, h, true);
        self.free_tiles_total -= w * h;

        if let Some(index) = self.free_slots.pop() {
            let slot = &mut self.slots[index as usize];
            slot.generation += 1;
            slot.alive = true;
            slot.layer = layer;
            slot.rect_tiles = (x, y, w, h);
            AtlasHandle {
                index,
                generation: slot.generation,
            }
        } else {
            let index = self.slots.len() as u32;
            self.slots.push(AtlasSlot {
                generation: 0,
                alive: true,
                layer,
                rect_tiles: (x, y, w, h),
            });
            AtlasHandle {
                index,
                generation: 0,
            }
        }
    }
}

pub struct AtlasInfo {
    pub tiles_w: u32,
    pub tiles_h: u32,
    pub layers: u32,
    pub tile_px: u32,
    pub pitch_px: u32,
    pub free_tiles: u32,
    pub repack_count: u64,
    pub format: wgpu::TextureFormat,
}
