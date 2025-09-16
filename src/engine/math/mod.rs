pub mod vector;
pub mod matrix;
pub mod transform;
pub mod bounds;

pub use cgmath::*;
pub use vector::*;
pub use matrix::*;
pub use transform::*;
pub use bounds::*;

pub type Vec2 = cgmath::Vector2<f32>;
pub type Vec3 = cgmath::Vector3<f32>;
pub type Vec4 = cgmath::Vector4<f32>;
pub type Point3 = cgmath::Point3<f32>;
pub type Mat4 = cgmath::Matrix4<f32>;
pub type Quat = cgmath::Quaternion<f32>;