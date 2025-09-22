// Metal Renderer Module for macOS-native game engine

pub mod metal_renderer;
pub mod shaders;
pub mod mesh;
pub mod texture_atlas;

pub use metal_renderer::MetalRenderer;
pub use mesh::Mesh;
pub use texture_atlas::TextureAtlas;

use cgmath::{Matrix4, Vector3, Point3};

// Core rendering types optimized for Apple Silicon
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Uniforms {
    pub view_proj: [[f32; 4]; 4],
    pub view_pos: [f32; 4],
    pub light_pos: [f32; 4],
    pub time: f32,
    pub ambient_factor: f32,  // Ambient lighting intensity (0.0 - 1.0)
    pub light_intensity: f32, // Light source intensity
    pub _padding0: f32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceData {
    pub model_matrix: [[f32; 4]; 4],
    pub material_id: u32,
    pub _padding: [u32; 3],
}

// Frustum culling structures
#[derive(Debug, Clone, Copy)]
pub struct Plane {
    pub normal: Vector3<f32>,
    pub distance: f32,
}

impl Plane {
    pub fn new(normal: Vector3<f32>, distance: f32) -> Self {
        Self { normal, distance }
    }

    pub fn from_point_normal(point: Point3<f32>, normal: Vector3<f32>) -> Self {
        use cgmath::InnerSpace;
        let normalized = normal.normalize();
        let point_vec = Vector3::new(point.x, point.y, point.z);
        let distance = normalized.dot(point_vec);
        Self { normal: normalized, distance }
    }

    pub fn distance_to_point(&self, point: Point3<f32>) -> f32 {
        use cgmath::InnerSpace;
        let point_vec = Vector3::new(point.x, point.y, point.z);
        self.normal.dot(point_vec) - self.distance
    }
}

#[derive(Debug, Clone)]
pub struct Frustum {
    pub planes: [Plane; 6], // left, right, bottom, top, near, far
}

impl Frustum {
    pub fn from_view_projection_matrix(view_proj: &Matrix4<f32>) -> Self {
        // Extract frustum planes from view-projection matrix
        // Using the standard method: plane equations from matrix rows
        let m = view_proj;

        let planes = [
            // Left plane: m3 + m0
            Plane::new(
                Vector3::new(m[3][0] + m[0][0], m[3][1] + m[0][1], m[3][2] + m[0][2]),
                m[3][3] + m[0][3]
            ),
            // Right plane: m3 - m0
            Plane::new(
                Vector3::new(m[3][0] - m[0][0], m[3][1] - m[0][1], m[3][2] - m[0][2]),
                m[3][3] - m[0][3]
            ),
            // Bottom plane: m3 + m1
            Plane::new(
                Vector3::new(m[3][0] + m[1][0], m[3][1] + m[1][1], m[3][2] + m[1][2]),
                m[3][3] + m[1][3]
            ),
            // Top plane: m3 - m1
            Plane::new(
                Vector3::new(m[3][0] - m[1][0], m[3][1] - m[1][1], m[3][2] - m[1][2]),
                m[3][3] - m[1][3]
            ),
            // Near plane: m3 + m2
            Plane::new(
                Vector3::new(m[3][0] + m[2][0], m[3][1] + m[2][1], m[3][2] + m[2][2]),
                m[3][3] + m[2][3]
            ),
            // Far plane: m3 - m2
            Plane::new(
                Vector3::new(m[3][0] - m[2][0], m[3][1] - m[2][1], m[3][2] - m[2][2]),
                m[3][3] - m[2][3]
            ),
        ];

        // Normalize planes
        let mut normalized_planes = [Plane::new(Vector3::new(0.0, 0.0, 0.0), 0.0); 6];
        for (i, plane) in planes.iter().enumerate() {
            use cgmath::InnerSpace;
            let length = plane.normal.magnitude();
            if length > 0.0 {
                normalized_planes[i] = Plane::new(
                    plane.normal / length,
                    plane.distance / length
                );
            } else {
                normalized_planes[i] = *plane;
            }
        }

        Self { planes: normalized_planes }
    }

    pub fn intersects_aabb(&self, aabb: &AABB) -> bool {
        // Test AABB against all 6 frustum planes
        for plane in &self.planes {
            // Get the positive vertex (farthest point in direction of plane normal)
            let positive_vertex = Point3::new(
                if plane.normal.x >= 0.0 { aabb.max.x } else { aabb.min.x },
                if plane.normal.y >= 0.0 { aabb.max.y } else { aabb.min.y },
                if plane.normal.z >= 0.0 { aabb.max.z } else { aabb.min.z },
            );

            // If positive vertex is behind plane, AABB is completely outside frustum
            if plane.distance_to_point(positive_vertex) < 0.0 {
                return false;
            }
        }
        true // AABB intersects or is inside frustum
    }
}

#[derive(Debug, Clone, Copy)]
pub struct AABB {
    pub min: Point3<f32>,
    pub max: Point3<f32>,
}

impl AABB {
    pub fn new(min: Point3<f32>, max: Point3<f32>) -> Self {
        Self { min, max }
    }

    pub fn from_chunk_coords(chunk_x: i32, chunk_y: i32, chunk_z: i32, chunk_size: usize) -> Self {
        let size = chunk_size as f32;
        let min = Point3::new(
            chunk_x as f32 * size,
            chunk_y as f32 * size,
            chunk_z as f32 * size,
        );
        let max = Point3::new(
            min.x + size,
            min.y + size,
            min.z + size,
        );
        Self { min, max }
    }

    pub fn center(&self) -> Point3<f32> {
        Point3::new(
            (self.min.x + self.max.x) * 0.5,
            (self.min.y + self.max.y) * 0.5,
            (self.min.z + self.max.z) * 0.5,
        )
    }

    pub fn size(&self) -> Vector3<f32> {
        Vector3::new(
            self.max.x - self.min.x,
            self.max.y - self.min.y,
            self.max.z - self.min.z,
        )
    }
}

// Camera optimized for first-person games
pub struct Camera {
    pub eye: Point3<f32>,
    pub target: Point3<f32>,
    pub up: Vector3<f32>,
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
    pub yaw: f32,
    pub pitch: f32,
}

impl Camera {
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            eye: Point3::new(0.0, 10.0, 0.0),
            target: Point3::new(0.0, 10.0, -1.0),
            up: Vector3::new(0.0, 1.0, 0.0),
            aspect: width / height,
            fovy: 60.0,
            znear: 0.1,
            zfar: 1000.0,
            yaw: -std::f32::consts::FRAC_PI_2,
            pitch: 0.0,
        }
    }

    pub fn build_view_projection_matrix(&self) -> Matrix4<f32> {
        use cgmath::{perspective, Deg};

        let view = Matrix4::look_at_rh(self.eye, self.target, self.up);
        let proj = perspective(Deg(self.fovy), self.aspect, self.znear, self.zfar);
        proj * view
    }

    pub fn update_from_input(&mut self, forward: f32, right: f32, up: f32, mouse_dx: f32, mouse_dy: f32) {
        use cgmath::InnerSpace;

        // Mouse look with smooth movement
        self.yaw += mouse_dx * 0.002;
        self.pitch -= mouse_dy * 0.002;
        self.pitch = self.pitch.clamp(-1.5, 1.5);

        // Calculate movement vectors
        let forward_dir = Vector3::new(
            self.yaw.cos() * self.pitch.cos(),
            self.pitch.sin(),
            self.yaw.sin() * self.pitch.cos(),
        ).normalize();

        let right_dir = forward_dir.cross(Vector3::new(0.0, 1.0, 0.0)).normalize();
        let up_dir = Vector3::new(0.0, 1.0, 0.0);

        // Apply movement
        let movement_speed = 0.2;
        self.eye += forward_dir * forward * movement_speed;
        self.eye += right_dir * right * movement_speed;
        self.eye += up_dir * up * movement_speed;

        // Update target
        self.target = self.eye + forward_dir;
    }

    pub fn get_forward_vector(&self) -> Vector3<f32> {
        use cgmath::InnerSpace;

        Vector3::new(
            self.yaw.cos() * self.pitch.cos(),
            self.pitch.sin(),
            self.yaw.sin() * self.pitch.cos(),
        ).normalize()
    }

    pub fn get_frustum(&self) -> Frustum {
        let view_proj = self.build_view_projection_matrix();
        Frustum::from_view_projection_matrix(&view_proj)
    }
}