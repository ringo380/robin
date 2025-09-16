use crate::engine::math::{Vec3, Mat4};

pub struct Camera {
    pub position: Vec3,
    pub target: Vec3,
    pub up: Vec3,
    pub fov: f32,
    pub aspect: f32,
    pub near: f32,
    pub far: f32,
}

impl Camera {
    pub fn new(aspect: f32) -> Self {
        Self {
            position: Vec3::new(0.0, 0.0, 3.0),
            target: Vec3::new(0.0, 0.0, 0.0),
            up: Vec3::new(0.0, 1.0, 0.0),
            fov: 45.0,
            aspect,
            near: 0.1,
            far: 100.0,
        }
    }

    pub fn view_matrix(&self) -> Mat4 {
        cgmath::Matrix4::look_at_rh(
            cgmath::Point3::new(self.position.x, self.position.y, self.position.z),
            cgmath::Point3::new(self.target.x, self.target.y, self.target.z),
            self.up,
        )
    }

    pub fn projection_matrix(&self) -> Mat4 {
        cgmath::perspective(
            cgmath::Deg(self.fov),
            self.aspect,
            self.near,
            self.far,
        )
    }
}