use std::collections::HashMap;

#[derive(Debug, Clone, Copy)]
pub struct BoneAllocation {
    pub offset: u32,
    pub count: u32,
}

#[derive(Debug, Default)]
pub struct SkinningSystem {
    allocations: HashMap<u32, BoneAllocation>,
    free_list: Vec<BoneAllocation>,
    next_offset: u32,
}

impl SkinningSystem {
    pub const MAX_BONES_PER_MODEL: u32 = 256;

    pub fn ensure_allocation(&mut self, model_id: u32, bone_count: u32) -> BoneAllocation {
        if let Some(existing) = self.allocations.get(&model_id).copied() {
            if existing.count >= bone_count {
                return existing;
            }
            self.free_list.push(existing);
            self.allocations.remove(&model_id);
        }

        self.merge_free_list();

        if let Some(idx) = self
            .free_list
            .iter()
            .position(|alloc| alloc.count >= bone_count)
        {
            let alloc = self.free_list.swap_remove(idx);
            let chosen = BoneAllocation {
                offset: alloc.offset,
                count: alloc.count,
            };
            self.allocations.insert(model_id, chosen);
            return chosen;
        }

        let alloc = BoneAllocation {
            offset: self.next_offset,
            count: bone_count,
        };
        self.next_offset = self.next_offset.saturating_add(bone_count);
        self.allocations.insert(model_id, alloc);
        alloc
    }

    pub fn release(&mut self, model_id: u32) {
        if let Some(alloc) = self.allocations.remove(&model_id) {
            self.free_list.push(alloc);
        }
    }

    pub fn clear(&mut self) {
        self.allocations.clear();
        self.free_list.clear();
        self.next_offset = 0;
    }

    fn merge_free_list(&mut self) {
        if self.free_list.len() < 2 {
            return;
        }
        self.free_list.sort_by_key(|alloc| alloc.offset);
        let mut merged = Vec::with_capacity(self.free_list.len());
        let mut current = self.free_list[0];
        for alloc in self.free_list.iter().skip(1) {
            if current.offset + current.count == alloc.offset {
                current.count = current.count.saturating_add(alloc.count);
            } else {
                merged.push(current);
                current = *alloc;
            }
        }
        merged.push(current);
        self.free_list = merged;
    }
}
