// Mesh and vertex types optimized for Metal rendering

use metal::*;
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub color: [f32; 3],
    pub normal: [f32; 3],
    pub tex_coords: [f32; 2],
}

impl Vertex {
    pub fn new(position: [f32; 3], color: [f32; 3], normal: [f32; 3]) -> Self {
        Self {
            position,
            color,
            normal,
            tex_coords: [0.0, 0.0],
        }
    }

    pub fn with_tex_coords(mut self, tex_coords: [f32; 2]) -> Self {
        self.tex_coords = tex_coords;
        self
    }
}

#[derive(Debug)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub vertex_buffer: Option<Buffer>,
    pub index_buffer: Option<Buffer>,
    pub vertex_count: usize,
    pub index_count: usize,
}

impl Mesh {
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            indices: Vec::new(),
            vertex_buffer: None,
            index_buffer: None,
            vertex_count: 0,
            index_count: 0,
        }
    }

    pub fn from_vertices_indices(vertices: Vec<Vertex>, indices: Vec<u32>) -> Self {
        let vertex_count = vertices.len();
        let index_count = indices.len();

        Self {
            vertices,
            indices,
            vertex_buffer: None,
            index_buffer: None,
            vertex_count,
            index_count,
        }
    }

    pub fn create_buffers(&mut self, device: &DeviceRef) {
        if !self.vertices.is_empty() {
            let vertex_data_size = self.vertices.len() * std::mem::size_of::<Vertex>();
            self.vertex_buffer = Some(device.new_buffer(
                vertex_data_size as u64,
                MTLResourceOptions::StorageModeShared,
            ));

            // Copy data to buffer
            if let Some(buffer) = &self.vertex_buffer {
                let ptr = buffer.contents() as *mut Vertex;
                unsafe {
                    std::ptr::copy_nonoverlapping(self.vertices.as_ptr(), ptr, self.vertices.len());
                }
            }
        }

        if !self.indices.is_empty() {
            let index_data_size = self.indices.len() * std::mem::size_of::<u32>();
            self.index_buffer = Some(device.new_buffer(
                index_data_size as u64,
                MTLResourceOptions::StorageModeShared,
            ));

            // Copy data to buffer
            if let Some(buffer) = &self.index_buffer {
                let ptr = buffer.contents() as *mut u32;
                unsafe {
                    std::ptr::copy_nonoverlapping(self.indices.as_ptr(), ptr, self.indices.len());
                }
            }
        }

        // Update counts
        self.vertex_count = self.vertices.len();
        self.index_count = self.indices.len();
    }

    pub fn update_buffers(&mut self, device: &DeviceRef) {
        // For simplicity, just recreate buffers
        self.create_buffers(device);

        self.vertex_count = self.vertices.len();
        self.index_count = self.indices.len();
    }

    pub fn clear(&mut self) {
        self.vertices.clear();
        self.indices.clear();
        self.vertex_count = 0;
        self.index_count = 0;
    }

    pub fn add_quad(&mut self, positions: [[f32; 3]; 4], color: [f32; 3], normal: [f32; 3]) {
        // Use default UV coordinates for backward compatibility
        let uv_coords = [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];
        self.add_quad_with_uv(positions, color, normal, uv_coords);
    }

    pub fn add_quad_with_uv(&mut self, positions: [[f32; 3]; 4], color: [f32; 3], normal: [f32; 3], uv_coords: [[f32; 2]; 4]) {
        let base_index = self.vertices.len() as u32;

        // Add vertices with custom UV coordinates
        for (i, pos) in positions.iter().enumerate() {
            self.vertices.push(Vertex {
                position: *pos,
                color,
                normal,
                tex_coords: uv_coords[i],
            });
        }

        // Add indices for two triangles (quad)
        self.indices.extend_from_slice(&[
            base_index, base_index + 1, base_index + 2,
            base_index, base_index + 2, base_index + 3,
        ]);
    }

    pub fn add_cube(&mut self, center: [f32; 3], size: f32, color: [f32; 3]) {
        let half_size = size * 0.5;
        let [x, y, z] = center;

        // Define the 8 vertices of a cube
        let vertices = [
            [x - half_size, y - half_size, z - half_size], // 0: back-bottom-left
            [x + half_size, y - half_size, z - half_size], // 1: back-bottom-right
            [x + half_size, y + half_size, z - half_size], // 2: back-top-right
            [x - half_size, y + half_size, z - half_size], // 3: back-top-left
            [x - half_size, y - half_size, z + half_size], // 4: front-bottom-left
            [x + half_size, y - half_size, z + half_size], // 5: front-bottom-right
            [x + half_size, y + half_size, z + half_size], // 6: front-top-right
            [x - half_size, y + half_size, z + half_size], // 7: front-top-left
        ];

        // Define the 6 faces with their normals
        let faces = [
            // Front face (z+)
            ([vertices[4], vertices[5], vertices[6], vertices[7]], [0.0, 0.0, 1.0]),
            // Back face (z-)
            ([vertices[1], vertices[0], vertices[3], vertices[2]], [0.0, 0.0, -1.0]),
            // Right face (x+)
            ([vertices[5], vertices[1], vertices[2], vertices[6]], [1.0, 0.0, 0.0]),
            // Left face (x-)
            ([vertices[0], vertices[4], vertices[7], vertices[3]], [-1.0, 0.0, 0.0]),
            // Top face (y+)
            ([vertices[3], vertices[7], vertices[6], vertices[2]], [0.0, 1.0, 0.0]),
            // Bottom face (y-)
            ([vertices[4], vertices[0], vertices[1], vertices[5]], [0.0, -1.0, 0.0]),
        ];

        // Add each face as a quad
        for (face_vertices, normal) in faces.iter() {
            self.add_quad(*face_vertices, color, *normal);
        }
    }
}