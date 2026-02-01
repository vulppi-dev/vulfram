use crate::core::resources::ModelComponent;

#[derive(Clone, Copy)]
pub struct DrawItem {
    pub model_id: u32,
    pub geometry_id: u32,
    pub material_id: u32,
    pub depth: f32,
    pub instance_idx: u32,
}

/// Collection of vectors to be reused across frames for draw call preparation
#[derive(Default)]
pub struct DrawCollector {
    pub standard_opaque: Vec<DrawItem>,
    pub standard_masked: Vec<DrawItem>,
    pub standard_transparent: Vec<DrawItem>,
    pub pbr_opaque: Vec<DrawItem>,
    pub pbr_masked: Vec<DrawItem>,
    pub pbr_transparent: Vec<DrawItem>,
    pub instance_data: Vec<ModelComponent>,
    pub shadow_instance_data: Vec<ModelComponent>,
    pub outline_items: Vec<(u32, u32)>,
    pub outline_instance_data: Vec<ModelComponent>,
}

impl DrawCollector {
    pub fn clear(&mut self) {
        self.standard_opaque.clear();
        self.standard_masked.clear();
        self.standard_transparent.clear();
        self.pbr_opaque.clear();
        self.pbr_masked.clear();
        self.pbr_transparent.clear();
        self.instance_data.clear();
        self.shadow_instance_data.clear();
        self.outline_items.clear();
        self.outline_instance_data.clear();
    }
}
