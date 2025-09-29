pub mod transform;
pub mod bounds;
// Removed non-existent modules: vector, matrix

// Import specific items from cgmath to avoid conflicts
pub use cgmath::{
    Vector2, Vector3, Vector4, Matrix4, Quaternion,
    Rad, Deg, Zero, One, InnerSpace, EuclideanSpace, MetricSpace,
    SquareMatrix, Basis2, Basis3, Rotation, Rotation2, Rotation3,
    perspective, ortho, frustum,
};
pub use transform::*;
pub use bounds::*;
// Removed unused imports: vector::*, matrix::*

pub type Vec2 = cgmath::Vector2<f32>;
pub type Vec3 = cgmath::Vector3<f32>;
pub type Vec4 = cgmath::Vector4<f32>;
pub type Point3 = cgmath::Point3<f32>;
pub type Mat4 = cgmath::Matrix4<f32>;
pub type Quat = cgmath::Quaternion<f32>;