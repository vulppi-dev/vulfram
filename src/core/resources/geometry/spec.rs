use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Debug, Clone, Deserialize_repr, Serialize_repr)]
#[repr(u32)]
pub enum GeometryPrimitiveType {
    Index = 0,
    Position,
    Normal,
    Tangent,
    Color,
    UV,
    SkinJoints,
    SkinWeights,
}
