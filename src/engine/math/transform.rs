use crate::engine::math::{Vec3, Vec2, Mat4};

pub struct Transform3D {
    pub position: Vec3,
    pub rotation: Vec3,
    pub scale: Vec3,
}

impl Transform3D {
    pub fn new() -> Self {
        Self {
            position: Vec3::new(0.0, 0.0, 0.0),
            rotation: Vec3::new(0.0, 0.0, 0.0),
            scale: Vec3::new(1.0, 1.0, 1.0),
        }
    }

    pub fn to_matrix(&self) -> Mat4 {
        let translation = Mat4::from_translation(self.position);
        let rotation_x = Mat4::from_angle_x(cgmath::Rad(self.rotation.x));
        let rotation_y = Mat4::from_angle_y(cgmath::Rad(self.rotation.y));
        let rotation_z = Mat4::from_angle_z(cgmath::Rad(self.rotation.z));
        let rotation = rotation_z * rotation_y * rotation_x;
        let scale = Mat4::from_nonuniform_scale(self.scale.x, self.scale.y, self.scale.z);
        
        translation * rotation * scale
    }
}