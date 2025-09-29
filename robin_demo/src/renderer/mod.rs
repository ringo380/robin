// Metal Renderer Module for macOS-native game engine

pub mod metal_renderer;
pub mod shaders;
pub mod mesh;
pub mod texture_atlas;
pub mod error_handling;

pub use metal_renderer::MetalRenderer;
pub use mesh::{Mesh, Vertex};
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
    // Enhanced movement system
    velocity: Vector3<f32>,
    angular_velocity: Vector3<f32>,
    movement_momentum: f32,
    mouse_sensitivity: f32,
    smooth_factor: f32,
    max_velocity: f32,
    acceleration: f32,
    friction: f32,
    // Smooth look parameters
    target_yaw: f32,
    target_pitch: f32,
    look_smooth_factor: f32,
}

impl Camera {
    pub fn new(width: f32, height: f32) -> Self {
        let initial_yaw = -std::f32::consts::FRAC_PI_2;
        let initial_pitch = -0.3;

        Self {
            eye: Point3::new(0.0, 15.0, 10.0),     // Elevated position behind terrain
            target: Point3::new(0.0, 8.0, 0.0),   // Looking down at terrain center
            up: Vector3::new(0.0, 1.0, 0.0),
            aspect: width / height,
            fovy: 60.0,
            znear: 0.1,
            zfar: 1000.0,
            yaw: initial_yaw,
            pitch: initial_pitch,
            // Enhanced movement parameters
            velocity: Vector3::new(0.0, 0.0, 0.0),
            angular_velocity: Vector3::new(0.0, 0.0, 0.0),
            movement_momentum: 0.85,               // How much momentum is retained (0-1)
            mouse_sensitivity: 0.002,              // Mouse look sensitivity
            smooth_factor: 0.15,                   // Movement smoothing factor
            max_velocity: 25.0,                    // Maximum movement speed
            acceleration: 45.0,                    // Movement acceleration
            friction: 12.0,                        // Movement friction/deceleration
            // Smooth look parameters
            target_yaw: initial_yaw,
            target_pitch: initial_pitch,
            look_smooth_factor: 0.2,               // Mouse look smoothing
        }
    }

    pub fn build_view_projection_matrix(&self) -> Matrix4<f32> {
        use cgmath::{perspective, Deg};

        let view = Matrix4::look_at_rh(self.eye, self.target, self.up);
        let proj = perspective(Deg(self.fovy), self.aspect, self.znear, self.zfar);
        proj * view
    }

    pub fn update_from_input(&mut self, forward: f32, right: f32, up: f32, mouse_dx: f32, mouse_dy: f32) {
        self.update_smooth_look(mouse_dx, mouse_dy);
        self.update_smooth_movement(forward, right, up, 1.0 / 60.0); // Assume 60 FPS for now
    }

    /// Enhanced update method with proper delta time
    pub fn update_from_input_with_delta(&mut self, forward: f32, right: f32, up: f32, mouse_dx: f32, mouse_dy: f32, delta_time: f32) {
        self.update_smooth_look(mouse_dx, mouse_dy);
        self.update_smooth_movement(forward, right, up, delta_time);
    }

    /// Enhanced smooth camera movement with momentum and acceleration
    pub fn update_smooth_movement(&mut self, forward: f32, right: f32, up: f32, delta_time: f32) {
        use cgmath::InnerSpace;

        // Calculate current direction vectors from camera orientation
        let forward_dir = Vector3::new(
            self.yaw.cos() * self.pitch.cos(),
            self.pitch.sin(),
            self.yaw.sin() * self.pitch.cos(),
        ).normalize();
        let right_dir = forward_dir.cross(Vector3::new(0.0, 1.0, 0.0)).normalize();
        let up_dir = Vector3::new(0.0, 1.0, 0.0);

        // Calculate desired velocity based on input
        let mut desired_velocity = Vector3::new(0.0, 0.0, 0.0);
        desired_velocity += forward_dir * forward * self.max_velocity;
        desired_velocity += right_dir * right * self.max_velocity;
        desired_velocity += up_dir * up * self.max_velocity;

        // Apply acceleration toward desired velocity
        let velocity_diff = desired_velocity - self.velocity;
        let acceleration_force = velocity_diff * self.acceleration * delta_time;
        self.velocity += acceleration_force;

        // Apply friction when no input is given
        if forward.abs() < 0.1 && right.abs() < 0.1 && up.abs() < 0.1 {
            self.velocity *= 1.0 - (self.friction * delta_time).min(1.0);
        }

        // Apply momentum damping to prevent infinite acceleration
        self.velocity *= self.movement_momentum;

        // Clamp velocity to maximum
        let speed = self.velocity.magnitude();
        if speed > self.max_velocity {
            self.velocity = self.velocity.normalize() * self.max_velocity;
        }

        // Update position with smooth interpolation
        let position_delta = self.velocity * delta_time;
        self.eye += position_delta;

        // Update target based on current orientation
        self.target = self.eye + forward_dir;
    }

    /// Smooth mouse look with momentum
    pub fn update_smooth_look(&mut self, mouse_dx: f32, mouse_dy: f32) {
        // Update target angles with mouse input
        self.target_yaw += mouse_dx * self.mouse_sensitivity;
        self.target_pitch -= mouse_dy * self.mouse_sensitivity;

        // Clamp pitch to reasonable limits
        self.target_pitch = self.target_pitch.clamp(-1.55, 1.55); // ~89 degrees

        // Smoothly interpolate to target angles
        let yaw_diff = self.target_yaw - self.yaw;
        let pitch_diff = self.target_pitch - self.pitch;

        self.yaw += yaw_diff * self.look_smooth_factor;
        self.pitch += pitch_diff * self.look_smooth_factor;
    }

    /// Get current velocity for external systems (e.g., sound effects, particle systems)
    pub fn get_velocity(&self) -> Vector3<f32> {
        self.velocity
    }

    /// Get current speed for external systems
    pub fn get_speed(&self) -> f32 {
        use cgmath::InnerSpace;
        self.velocity.magnitude()
    }

    /// Adjust camera sensitivity at runtime
    pub fn set_mouse_sensitivity(&mut self, sensitivity: f32) {
        self.mouse_sensitivity = sensitivity.max(0.0001); // Prevent zero sensitivity
    }

    /// Adjust movement parameters at runtime
    pub fn set_movement_parameters(&mut self, max_velocity: f32, acceleration: f32, friction: f32) {
        self.max_velocity = max_velocity.max(0.1);
        self.acceleration = acceleration.max(0.1);
        self.friction = friction.max(0.1);
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