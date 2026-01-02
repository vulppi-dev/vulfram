pub fn default_layer_mask() -> u32 {
    0xFFFFFFFF
}

pub fn default_true() -> bool {
    true
}

pub fn wgpu_projection_correction() -> glam::Mat4 {
    glam::Mat4::from_cols(
        glam::vec4(1.0, 0.0, 0.0, 0.0),
        glam::vec4(0.0, 1.0, 0.0, 0.0),
        glam::vec4(0.0, 0.0, 0.5, 0.0),
        glam::vec4(0.0, 0.0, 0.5, 1.0),
    )
}
