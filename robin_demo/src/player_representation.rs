// Player body representation for first-person games
// Creates a visible player body that matches the physics collider

use crate::renderer::mesh::{Mesh, Vertex};
use cgmath::{Matrix4, Vector3, Point3, Deg, perspective, Angle};
use nalgebra::Point3 as NaPoint3;

pub struct PlayerBodyRenderer {
    body_mesh: Mesh,
    arm_mesh: Mesh,
    is_initialized: bool,
}

impl PlayerBodyRenderer {
    pub fn new() -> Self {
        Self {
            body_mesh: Mesh::new(),
            arm_mesh: Mesh::new(),
            is_initialized: false,
        }
    }

    pub fn initialize(&mut self, device: &metal::DeviceRef) {
        if self.is_initialized {
            return;
        }

        // Create a simple capsule body that matches the physics collider
        self.create_capsule_body();
        self.create_player_arms();

        // Create GPU buffers for meshes
        self.body_mesh.create_buffers(device);
        self.arm_mesh.create_buffers(device);

        self.is_initialized = true;
    }

    fn create_capsule_body(&mut self) {
        // Create a simple capsule shape (cylinder with hemispheres on top/bottom)
        let radius = 0.3;
        let height = 1.8;
        let cylinder_height = height - 2.0 * radius;

        let segments = 16;
        let body_color = [0.2, 0.4, 0.8]; // Blue body color

        // Clear existing vertices
        self.body_mesh.vertices.clear();
        self.body_mesh.indices.clear();

        // Create cylinder body
        for i in 0..=segments {
            let angle = 2.0 * std::f32::consts::PI * i as f32 / segments as f32;
            let x = radius * angle.cos();
            let z = radius * angle.sin();

            // Bottom ring
            self.body_mesh.vertices.push(Vertex::new(
                [x, -cylinder_height / 2.0, z],
                body_color,
                [x / radius, 0.0, z / radius], // Normal pointing outward
            ));

            // Top ring
            self.body_mesh.vertices.push(Vertex::new(
                [x, cylinder_height / 2.0, z],
                body_color,
                [x / radius, 0.0, z / radius], // Normal pointing outward
            ));
        }

        // Create cylinder indices
        for i in 0..segments {
            let bottom_current = i * 2;
            let bottom_next = ((i + 1) % (segments + 1)) * 2;
            let top_current = i * 2 + 1;
            let top_next = ((i + 1) % (segments + 1)) * 2 + 1;

            // Two triangles per quad
            self.body_mesh.indices.extend(&[
                bottom_current, bottom_next, top_current,
                top_current, bottom_next, top_next,
            ]);
        }

        // Add hemisphere caps (simplified)
        let base_vertex_count = self.body_mesh.vertices.len() as u32;

        // Top hemisphere center
        self.body_mesh.vertices.push(Vertex::new(
            [0.0, cylinder_height / 2.0 + radius, 0.0],
            body_color,
            [0.0, 1.0, 0.0],
        ));

        // Bottom hemisphere center
        self.body_mesh.vertices.push(Vertex::new(
            [0.0, -cylinder_height / 2.0 - radius, 0.0],
            body_color,
            [0.0, -1.0, 0.0],
        ));

        // Connect hemispheres to cylinder edges
        for i in 0..segments {
            let top_edge = (i * 2 + 1) as u32;
            let top_edge_next = (((i + 1) % (segments + 1)) * 2 + 1) as u32;
            let bottom_edge = (i * 2) as u32;
            let bottom_edge_next = (((i + 1) % (segments + 1)) * 2) as u32;

            // Top hemisphere triangles
            self.body_mesh.indices.extend(&[
                top_edge, base_vertex_count, top_edge_next,
            ]);

            // Bottom hemisphere triangles
            self.body_mesh.indices.extend(&[
                bottom_edge_next, base_vertex_count + 1, bottom_edge,
            ]);
        }

        self.body_mesh.vertex_count = self.body_mesh.vertices.len();
        self.body_mesh.index_count = self.body_mesh.indices.len();
    }

    fn create_player_arms(&mut self) {
        // Create simple arm representations visible in first-person view
        let arm_color = [0.8, 0.6, 0.4]; // Skin tone

        self.arm_mesh.vertices.clear();
        self.arm_mesh.indices.clear();

        // Right arm (visible when holding tools)
        self.create_simple_arm(&mut self.arm_mesh, [0.5, -0.3, -0.8], arm_color, true);

        // Left arm (partially visible)
        self.create_simple_arm(&mut self.arm_mesh, [-0.3, -0.3, -0.8], arm_color, false);

        self.arm_mesh.vertex_count = self.arm_mesh.vertices.len();
        self.arm_mesh.index_count = self.arm_mesh.indices.len();
    }

    fn create_simple_arm(&self, mesh: &mut Mesh, base_position: [f32; 3], color: [f32; 3], is_primary: bool) {
        let width = if is_primary { 0.08 } else { 0.06 };
        let length = if is_primary { 0.4 } else { 0.3 };

        let base_idx = mesh.vertices.len() as u32;

        // Create a simple rectangular arm
        let positions = [
            // Front face
            [base_position[0] - width, base_position[1], base_position[2]],
            [base_position[0] + width, base_position[1], base_position[2]],
            [base_position[0] + width, base_position[1] - length, base_position[2]],
            [base_position[0] - width, base_position[1] - length, base_position[2]],
            // Back face
            [base_position[0] - width, base_position[1], base_position[2] - width],
            [base_position[0] + width, base_position[1], base_position[2] - width],
            [base_position[0] + width, base_position[1] - length, base_position[2] - width],
            [base_position[0] - width, base_position[1] - length, base_position[2] - width],
        ];

        let normals = [
            [0.0, 0.0, 1.0], [0.0, 0.0, 1.0], [0.0, 0.0, 1.0], [0.0, 0.0, 1.0], // Front
            [0.0, 0.0, -1.0], [0.0, 0.0, -1.0], [0.0, 0.0, -1.0], [0.0, 0.0, -1.0], // Back
        ];

        for (pos, normal) in positions.iter().zip(normals.iter()) {
            mesh.vertices.push(Vertex::new(*pos, color, *normal));
        }

        // Indices for the arm box
        let indices = [
            // Front face
            0, 1, 2, 0, 2, 3,
            // Back face
            4, 6, 5, 4, 7, 6,
            // Left face
            0, 3, 7, 0, 7, 4,
            // Right face
            1, 5, 6, 1, 6, 2,
            // Top face
            0, 4, 5, 0, 5, 1,
            // Bottom face
            3, 2, 6, 3, 6, 7,
        ];

        for &idx in &indices {
            mesh.indices.push(base_idx + idx);
        }
    }

    pub fn render_player_body(
        &self,
        encoder: &metal::RenderCommandEncoderRef,
        renderer: &crate::renderer::MetalRenderer,
        camera_position: Point3<f32>,
        player_position: NaPoint3<f32>,
    ) {
        if !self.is_initialized {
            return;
        }

        // Only render body if camera is far enough from player (third-person view)
        let distance = (camera_position - Point3::new(player_position.x, player_position.y, player_position.z)).magnitude();

        if distance > 2.0 { // Third-person view
            renderer.render_mesh(encoder, &self.body_mesh);
        } else { // First-person view - show arms
            renderer.render_mesh(encoder, &self.arm_mesh);
        }
    }

    pub fn update_arm_animation(&mut self, time: f32, is_building: bool) {
        if !self.is_initialized {
            return;
        }

        // Simple arm animation based on activity
        let arm_color = if is_building {
            [0.8, 0.6, 0.4] // Normal skin tone
        } else {
            [0.7, 0.5, 0.3] // Slightly darker when not active
        };

        // Update arm colors or positions based on animation state
        // This is a simplified animation - in a full implementation,
        // you'd update vertex positions for smooth arm movement
        for vertex in &mut self.arm_mesh.vertices {
            vertex.color = arm_color;
        }
    }

    pub fn get_player_height(&self) -> f32 {
        1.8 // Match the physics capsule height
    }

    pub fn get_player_radius(&self) -> f32 {
        0.3 // Match the physics capsule radius
    }
}

impl Default for PlayerBodyRenderer {
    fn default() -> Self {
        Self::new()
    }
}